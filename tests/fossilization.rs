use c_mer::*;
use std::collections::HashMap;
use uuid::Uuid;

#[path = "common.rs"]
mod common;
use common::*;

#[test]
fn test_candidate_selection() {
    let mut memory = create_test_memory();
    let fragment = create_entity_relation_fragment("test", "relates", "target");
    memory.insert_fragment(fragment.clone(), Vec::new());
    
    let context = create_test_context("test", "test", 0.2);
    
    
    let mut report = PatternReport {
        repeated_paths: Vec::new(),
        stable_branches: Vec::new(),
        invariant_subgraphs: Vec::new(),
        high_confidence_outcomes: Vec::new(),
        fossilization_candidates: Vec::new(),
    };
    
    
    report.fossilization_candidates.push(FossilizationCandidate {
        pattern_type: "PathPattern".to_string(),
        pattern_id: Uuid::new_v4(),
        repetition_count: 25,
        average_confidence: 0.9,
        context_variance: 0.2,
        reward_correlation: 0.85,
        estimated_speedup: 3.0,
        priority: 0.95,
    });
    
    let config = FossilizationConfig::default();
    let selected = select_fossilization_candidates(&report, &config);
    
    assert_eq!(selected.len(), 1);
    assert_eq!(selected[0].repetition_count, 25);
}

#[test]
fn test_pattern_extraction() {
    let mut memory = create_test_memory();
    let fragment = create_entity_relation_fragment("test", "relates", "target");
    memory.insert_fragment(fragment.clone(), Vec::new());
    
    let context = create_test_context("test", "test", 0.2);
    let eeg = compile_thought(&context, &mut memory);
    
    let trace = ExecutionTrace {
        eeg_id: Uuid::new_v4(),
        context: context.clone(),
        node_sequence: eeg.nodes.keys().copied().collect(),
        branch_decisions: HashMap::new(),
        execution_time: 0.1,
        timestamp: current_timestamp(),
    };
    
    let mut report = PatternReport {
        repeated_paths: Vec::new(),
        stable_branches: Vec::new(),
        invariant_subgraphs: Vec::new(),
        high_confidence_outcomes: Vec::new(),
        fossilization_candidates: Vec::new(),
    };
    
    report.repeated_paths.push(PathPattern {
        path: trace.node_sequence.clone(),
        occurrence_count: 10,
        contexts: vec![context.clone()],
        average_confidence: 0.9,
        success_rate: 0.95,
    });
    
    let candidate = FossilizationCandidate {
        pattern_type: "PathPattern".to_string(),
        pattern_id: Uuid::new_v4(),
        repetition_count: 10,
        average_confidence: 0.9,
        context_variance: 0.2,
        reward_correlation: 0.95,
        estimated_speedup: 2.5,
        priority: 0.9,
    };
    
    let extracted = extract_pattern(&candidate, &[eeg], &[trace], &report);
    
    assert!(extracted.is_some());
    let pattern = extracted.unwrap();
    assert_eq!(pattern.pattern_type, "PathPattern");
    assert!(!pattern.structure.nodes.is_empty());
}

#[test]
fn test_fsm_compilation() {
    let mut memory = create_test_memory();
    let fragment = create_entity_relation_fragment("test", "relates", "target");
    memory.insert_fragment(fragment.clone(), Vec::new());
    
    let context = create_test_context("test", "test", 0.2);
    let eeg = compile_thought(&context, &mut memory);
    
    let trace = ExecutionTrace {
        eeg_id: Uuid::new_v4(),
        context: context.clone(),
        node_sequence: eeg.nodes.keys().copied().collect(),
        branch_decisions: HashMap::new(),
        execution_time: 0.1,
        timestamp: current_timestamp(),
    };
    
    let mut report = PatternReport {
        repeated_paths: Vec::new(),
        stable_branches: Vec::new(),
        invariant_subgraphs: Vec::new(),
        high_confidence_outcomes: Vec::new(),
        fossilization_candidates: Vec::new(),
    };
    
    report.repeated_paths.push(PathPattern {
        path: trace.node_sequence.clone(),
        occurrence_count: 10,
        contexts: vec![context.clone()],
        average_confidence: 0.9,
        success_rate: 0.95,
    });
    
    let candidate = FossilizationCandidate {
        pattern_type: "PathPattern".to_string(),
        pattern_id: Uuid::new_v4(),
        repetition_count: 10,
        average_confidence: 0.9,
        context_variance: 0.2,
        reward_correlation: 0.95,
        estimated_speedup: 2.5,
        priority: 0.9,
    };
    
    let extracted = extract_pattern(&candidate, &[eeg], &[trace], &report).unwrap();
    let module = compile_to_fsm(&extracted);
    
    assert_eq!(module.module_type, ModuleType::FSM);
    assert!(!module.code.is_empty());
    assert_eq!(module.confidence, 0.9);
}

#[test]
fn test_module_matching() {
    let mut memory = create_test_memory();
    
    
    let module = CompiledModule {
        id: Uuid::new_v4(),
        module_type: ModuleType::FSM,
        code: vec![1, 2, 3],
        input_signature: InputSignature {
            parameters: vec!["http_request".to_string()],
            context_requirements: vec!["web".to_string()],
        },
        output_signature: OutputSignature {
            return_type: "Outcome".to_string(),
            side_effects: Vec::new(),
        },
        activation_condition: ContextPattern {
            goal_patterns: vec!["http".to_string()],
            domain_hints: vec!["web".to_string()],
            confidence_threshold: 0.6,
        },
        confidence: 0.9,
        usage_count: 0,
        success_count: 0,
        failure_count: 0,
        last_used: 0.0,
        created_at: current_timestamp(),
        source_pattern: Uuid::new_v4(),
        version: 1,
    };
    
    memory.add_compiled_module(module.clone());
    
    let context = create_test_context("HTTP 404 error", "web", 0.2);
    let modules = memory.get_compiled_modules();
    
    let found = find_applicable_module(&context, modules);
    
    assert!(found.is_some());
    assert_eq!(found.unwrap().id, module.id);
}

#[test]
fn test_module_execution() {
    let module = CompiledModule {
        id: Uuid::new_v4(),
        module_type: ModuleType::FSM,
        code: vec![1, 2, 3],
        input_signature: InputSignature {
            parameters: Vec::new(),
            context_requirements: Vec::new(),
        },
        output_signature: OutputSignature {
            return_type: "Outcome".to_string(),
            side_effects: Vec::new(),
        },
        activation_condition: ContextPattern {
            goal_patterns: Vec::new(),
            domain_hints: Vec::new(),
            confidence_threshold: 0.5,
        },
        confidence: 0.9,
        usage_count: 0,
        success_count: 0,
        failure_count: 0,
        last_used: 0.0,
        created_at: current_timestamp(),
        source_pattern: Uuid::new_v4(),
        version: 1,
    };
    
    let context = create_test_context("test", "test", 0.2);
    let outcome = execute_compiled_module(&module, &context);
    
    assert_eq!(outcome.outcome_type, OutcomeType::Success);
    assert!(outcome.confidence > 0.8);
}

#[test]
fn test_module_storage() {
    let mut memory = create_test_memory();
    
    let module = CompiledModule {
        id: Uuid::new_v4(),
        module_type: ModuleType::FSM,
        code: vec![1, 2, 3],
        input_signature: InputSignature {
            parameters: Vec::new(),
            context_requirements: Vec::new(),
        },
        output_signature: OutputSignature {
            return_type: "Outcome".to_string(),
            side_effects: Vec::new(),
        },
        activation_condition: ContextPattern {
            goal_patterns: Vec::new(),
            domain_hints: Vec::new(),
            confidence_threshold: 0.5,
        },
        confidence: 0.9,
        usage_count: 0,
        success_count: 0,
        failure_count: 0,
        last_used: 0.0,
        created_at: current_timestamp(),
        source_pattern: Uuid::new_v4(),
        version: 1,
    };
    
    memory.add_compiled_module(module.clone());
    
    let modules = memory.get_compiled_modules();
    assert_eq!(modules.len(), 1);
    assert_eq!(modules[0].id, module.id);
}
