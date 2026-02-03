// Copyright (c) 2026 Nolan Taft
use crate::types::*;
use std::collections::HashMap;
use uuid::Uuid;

pub fn select_fossilization_candidates(
    pattern_report: &PatternReport,
    config: &FossilizationConfig,
) -> Vec<FossilizationCandidate> {
    let mut candidates = Vec::new();

    for candidate in &pattern_report.fossilization_candidates {
        if candidate.repetition_count >= config.min_repetition
            && candidate.average_confidence >= config.min_confidence
            && candidate.context_variance <= config.max_context_variance
            && candidate.reward_correlation >= config.min_reward_correlation
            && candidate.estimated_speedup >= config.min_speedup
        {
            candidates.push(candidate.clone());
        }
    }

    candidates.sort_by(|a, b| {
        b.priority
            .partial_cmp(&a.priority)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    candidates.truncate(config.max_candidates_per_run);

    candidates
}

pub fn extract_pattern(
    candidate: &FossilizationCandidate,
    eegs: &[EEG],
    execution_traces: &[ExecutionTrace],
    pattern_report: &PatternReport,
) -> Option<ExtractedPattern> {
    match candidate.pattern_type.as_str() {
        "PathPattern" => {
            if let Some(path_pattern) = pattern_report.repeated_paths.iter().find(|p| {
                p.occurrence_count == candidate.repetition_count
                    && (p.average_confidence - candidate.average_confidence).abs() < 0.01
            }) {
                Some(extract_path_pattern(path_pattern, eegs, execution_traces))
            } else {
                None
            }
        }
        "BranchPattern" => {
            if let Some(branch_pattern) = pattern_report
                .stable_branches
                .iter()
                .find(|b| (b.branch_ratio - candidate.reward_correlation).abs() < 0.1)
            {
                Some(extract_branch_pattern(
                    branch_pattern,
                    eegs,
                    execution_traces,
                ))
            } else {
                None
            }
        }
        "SubgraphPattern" => {
            if let Some(subgraph_pattern) = pattern_report
                .invariant_subgraphs
                .iter()
                .find(|s| s.occurrence_count == candidate.repetition_count)
            {
                Some(extract_subgraph_pattern(
                    subgraph_pattern,
                    eegs,
                    execution_traces,
                ))
            } else {
                None
            }
        }
        _ => None,
    }
}

fn extract_path_pattern(
    path_pattern: &PathPattern,
    eegs: &[EEG],
    traces: &[ExecutionTrace],
) -> ExtractedPattern {
    let mut instances = Vec::new();
    for trace in traces {
        if path_matches_trace(&path_pattern.path, trace) {
            if let Some(eeg) = eegs
                .iter()
                .find(|e| e.metadata.compilation_timestamp == trace.timestamp)
            {
                instances.push((eeg, trace));
            }
        }
    }

    let canonical_path = path_pattern.path.clone();

    let mut node_types = HashMap::new();
    let mut edge_types = HashMap::new();

    if let Some((eeg, _)) = instances.first() {
        for node_id in &canonical_path {
            if let Some(node) = eeg.nodes.get(node_id) {
                node_types.insert(*node_id, node.node_type.clone());
            }
        }

        for i in 0..canonical_path.len().saturating_sub(1) {
            let from = canonical_path[i];
            let to = canonical_path[i + 1];
            if let Some(edge) = eeg
                .edges
                .iter()
                .find(|e| e.from_node == from && e.to_node == to)
            {
                edge_types.insert((from, to), edge.edge_type.clone());
            }
        }
    }

    let structure = PatternStructure {
        nodes: canonical_path.clone(),
        edges: canonical_path.windows(2).map(|w| (w[0], w[1])).collect(),
        node_types,
        edge_types,
    };

    let input_signature = infer_input_signature(&instances);
    let output_signature = infer_output_signature(&instances);
    let activation_condition = infer_activation_condition(&path_pattern.contexts);

    ExtractedPattern {
        pattern_type: "PathPattern".to_string(),
        structure,
        input_signature,
        output_signature,
        activation_condition,
        confidence: path_pattern.average_confidence,
    }
}

fn extract_branch_pattern(
    branch_pattern: &BranchPattern,
    eegs: &[EEG],
    traces: &[ExecutionTrace],
) -> ExtractedPattern {
    let decision_node = branch_pattern.decision_node;
    let dominant_branch = branch_pattern.dominant_branch;

    let mut instances = Vec::new();
    for trace in traces {
        if let Some(&chosen_branch) = trace.branch_decisions.get(&decision_node) {
            if chosen_branch == dominant_branch {
                if let Some(eeg) = eegs
                    .iter()
                    .find(|e| e.metadata.compilation_timestamp == trace.timestamp)
                {
                    instances.push((eeg, trace));
                }
            }
        }
    }

    let structure = PatternStructure {
        nodes: vec![decision_node, dominant_branch],
        edges: vec![(decision_node, dominant_branch)],
        node_types: HashMap::new(),
        edge_types: HashMap::new(),
    };

    let input_signature = infer_input_signature(&instances);
    let output_signature = infer_output_signature(&instances);
    let activation_condition = infer_activation_condition(&branch_pattern.contexts);

    ExtractedPattern {
        pattern_type: "BranchPattern".to_string(),
        structure,
        input_signature,
        output_signature,
        activation_condition,
        confidence: branch_pattern.average_confidence,
    }
}

fn extract_subgraph_pattern(
    subgraph_pattern: &SubgraphPattern,
    eegs: &[EEG],
    traces: &[ExecutionTrace],
) -> ExtractedPattern {
    let nodes = subgraph_pattern.subgraph_nodes.clone();

    let mut edges = Vec::new();
    let mut node_types = HashMap::new();
    let mut edge_types = HashMap::new();

    if let Some(trace) = traces.iter().find(|t| t.node_sequence == nodes) {
        if let Some(eeg) = eegs
            .iter()
            .find(|e| e.metadata.compilation_timestamp == trace.timestamp)
        {
            for node_id in &nodes {
                if let Some(node) = eeg.nodes.get(node_id) {
                    node_types.insert(*node_id, node.node_type.clone());
                }
            }

            for i in 0..nodes.len().saturating_sub(1) {
                let from = nodes[i];
                let to = nodes[i + 1];
                if let Some(edge) = eeg
                    .edges
                    .iter()
                    .find(|e| e.from_node == from && e.to_node == to)
                {
                    edges.push((from, to));
                    edge_types.insert((from, to), edge.edge_type.clone());
                }
            }
        }
    }

    let structure = PatternStructure {
        nodes,
        edges,
        node_types,
        edge_types,
    };

    let mut instances = Vec::new();
    for trace in traces {
        if trace.node_sequence == structure.nodes {
            if let Some(eeg) = eegs
                .iter()
                .find(|e| e.metadata.compilation_timestamp == trace.timestamp)
            {
                instances.push((eeg, trace));
            }
        }
    }

    let input_signature = infer_input_signature(&instances);
    let output_signature = infer_output_signature(&instances);
    let activation_condition = infer_activation_condition(&subgraph_pattern.contexts);

    ExtractedPattern {
        pattern_type: "SubgraphPattern".to_string(),
        structure,
        input_signature,
        output_signature,
        activation_condition,
        confidence: subgraph_pattern.average_confidence,
    }
}

fn path_matches_trace(path: &[Uuid], trace: &ExecutionTrace) -> bool {
    let mut path_idx = 0;
    for &node_id in &trace.node_sequence {
        if path_idx < path.len() && path[path_idx] == node_id {
            path_idx += 1;
        }
    }
    path_idx == path.len()
}

fn infer_input_signature(instances: &[(&EEG, &ExecutionTrace)]) -> InputSignature {
    let mut parameters = Vec::new();
    let mut context_requirements = Vec::new();

    if let Some((_, trace)) = instances.first() {
        let goal_desc = trace.context.goal.description.to_lowercase();
        if goal_desc.contains("http") {
            parameters.push("http_request".to_string());
        }
        if goal_desc.contains("api") {
            parameters.push("api_call".to_string());
        }
        if goal_desc.contains("error") {
            parameters.push("error_code".to_string());
        }

        context_requirements.push(trace.context.domain_hint.domain.clone());
        for tag in &trace.context.domain_hint.tags {
            context_requirements.push(tag.clone());
        }
    }

    InputSignature {
        parameters,
        context_requirements,
    }
}

fn infer_output_signature(instances: &[(&EEG, &ExecutionTrace)]) -> OutputSignature {
    OutputSignature {
        return_type: "Outcome".to_string(),
        side_effects: vec!["memory_update".to_string()],
    }
}

fn infer_activation_condition(contexts: &[ContextVector]) -> ContextPattern {
    let mut goal_patterns = Vec::new();
    let mut domain_hints = Vec::new();

    for context in contexts {
        let goal_lower = context.goal.description.to_lowercase();
        if goal_lower.contains("http") && !goal_patterns.contains(&"http".to_string()) {
            goal_patterns.push("http".to_string());
        }
        if goal_lower.contains("api") && !goal_patterns.contains(&"api".to_string()) {
            goal_patterns.push("api".to_string());
        }
        if goal_lower.contains("error") && !goal_patterns.contains(&"error".to_string()) {
            goal_patterns.push("error".to_string());
        }

        if !domain_hints.contains(&context.domain_hint.domain) {
            domain_hints.push(context.domain_hint.domain.clone());
        }
    }

    let avg_confidence: f64 =
        contexts.iter().map(|c| c.confidence_threshold).sum::<f64>() / contexts.len() as f64;

    ContextPattern {
        goal_patterns,
        domain_hints,
        confidence_threshold: avg_confidence,
    }
}

pub fn compile_to_fsm(extracted_pattern: &ExtractedPattern) -> CompiledModule {
    let mut states = Vec::new();
    let mut transitions = Vec::new();

    for (i, &node_id) in extracted_pattern.structure.nodes.iter().enumerate() {
        let action = format!("node_{}", i);
        let is_accepting = i == extracted_pattern.structure.nodes.len() - 1;

        states.push(FSMState {
            id: i,
            name: format!("state_{}", i),
            action,
            is_accepting,
        });
    }

    for (from_node, to_node) in &extracted_pattern.structure.edges {
        let from_idx = extracted_pattern
            .structure
            .nodes
            .iter()
            .position(|&n| n == *from_node);
        let to_idx = extracted_pattern
            .structure
            .nodes
            .iter()
            .position(|&n| n == *to_node);

        if let (Some(from_state), Some(to_state)) = (from_idx, to_idx) {
            transitions.push(FSMTransition {
                from_state,
                to_state,
                condition: None,
                action: None,
            });
        }
    }

    let mut fsm_bytes = Vec::new();
    fsm_bytes.extend_from_slice(&(states.len() as u32).to_le_bytes());
    fsm_bytes.extend_from_slice(&(transitions.len() as u32).to_le_bytes());

    for state in &states {
        fsm_bytes.push(state.id as u8);
        fsm_bytes.push(if state.is_accepting { 1 } else { 0 });
    }

    for transition in &transitions {
        fsm_bytes.push(transition.from_state as u8);
        fsm_bytes.push(transition.to_state as u8);
    }

    CompiledModule {
        id: Uuid::new_v4(),
        module_type: ModuleType::FSM,
        code: fsm_bytes,
        input_signature: extracted_pattern.input_signature.clone(),
        output_signature: extracted_pattern.output_signature.clone(),
        activation_condition: extracted_pattern.activation_condition.clone(),
        confidence: extracted_pattern.confidence,
        usage_count: 0,
        success_count: 0,
        failure_count: 0,
        last_used: 0.0,
        created_at: current_timestamp(),
        source_pattern: Uuid::new_v4(),
        version: 1,
    }
}

pub fn compile_to_decision_table(extracted_pattern: &ExtractedPattern) -> CompiledModule {
    let mut table_bytes = Vec::new();
    table_bytes.extend_from_slice(&(extracted_pattern.structure.nodes.len() as u32).to_le_bytes());

    CompiledModule {
        id: Uuid::new_v4(),
        module_type: ModuleType::DecisionTable,
        code: table_bytes,
        input_signature: extracted_pattern.input_signature.clone(),
        output_signature: extracted_pattern.output_signature.clone(),
        activation_condition: extracted_pattern.activation_condition.clone(),
        confidence: extracted_pattern.confidence,
        usage_count: 0,
        success_count: 0,
        failure_count: 0,
        last_used: 0.0,
        created_at: current_timestamp(),
        source_pattern: Uuid::new_v4(),
        version: 1,
    }
}

pub fn store_compiled_module(module: CompiledModule, storage: &mut Vec<CompiledModule>) {
    storage.push(module);
}

pub fn find_applicable_module<'a>(
    context: &ContextVector,
    modules: &'a [CompiledModule],
) -> Option<&'a CompiledModule> {
    for module in modules {
        if module_matches_context(module, context) {
            return Some(module);
        }
    }
    None
}

fn module_matches_context(module: &CompiledModule, context: &ContextVector) -> bool {
    let goal_lower = context.goal.description.to_lowercase();

    let matches_goal = module
        .activation_condition
        .goal_patterns
        .iter()
        .any(|pattern| goal_lower.contains(pattern))
        || module.activation_condition.goal_patterns.is_empty();

    let matches_domain = module
        .activation_condition
        .domain_hints
        .contains(&context.domain_hint.domain)
        || module.activation_condition.domain_hints.is_empty();

    let matches_confidence =
        context.confidence_threshold >= module.activation_condition.confidence_threshold;

    matches_goal && matches_domain && matches_confidence
}

pub fn execute_compiled_module(module: &CompiledModule, context: &ContextVector) -> Outcome {
    match module.module_type {
        ModuleType::FSM => Outcome {
            outcome_type: OutcomeType::Success,
            result: format!("Executed FSM module {}", module.id),
            explanation: Some("Compiled module execution".to_string()),
            confidence: module.confidence,
        },
        ModuleType::DecisionTable => Outcome {
            outcome_type: OutcomeType::Success,
            result: format!("Executed decision table module {}", module.id),
            explanation: Some("Compiled module execution".to_string()),
            confidence: module.confidence,
        },
        _ => Outcome {
            outcome_type: OutcomeType::Partial,
            result: "Module type not implemented".to_string(),
            explanation: None,
            confidence: 0.5,
        },
    }
}
