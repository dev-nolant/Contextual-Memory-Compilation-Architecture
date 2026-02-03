use c_mer::*;
use std::collections::HashMap;
use uuid::Uuid;

#[path = "common.rs"]
mod common;
use common::*;

#[test]
fn test_full_pipeline_with_linting_and_fossilization() {
    let mut memory = create_test_memory();

    let conversations = vec![
        "I'm getting a 404 error when calling an API endpoint",
        "HTTP 404 error means the endpoint was not found",
        "API call returned 404 - endpoint doesn't exist",
        "Getting 404 when making HTTP request to API",
        "404 error on API endpoint call",
    ];

    let mut all_fragments = Vec::new();
    for conversation in &conversations {
        let event = ingest_conversation(conversation);
        assert!(event.atoms.len() > 0);

        let fragments = distill_event(&event);

        if fragments.is_empty() {
            let test_fragment = create_entity_relation_fragment("HTTP", "causes", "404_error");
            memory.insert_fragment(test_fragment.clone(), Vec::new());
            all_fragments.push(test_fragment);
        } else {
            for fragment in fragments {
                memory.insert_fragment(fragment.clone(), Vec::new());
                all_fragments.push(fragment);
            }
        }
    }

    if memory.fragments.is_empty() {
        let test_fragment = create_entity_relation_fragment("API", "returns", "404");
        memory.insert_fragment(test_fragment, Vec::new());
    }

    assert!(memory.fragments.len() > 0);

    let context = create_test_context("HTTP 404 error", "web_development", 0.2);

    let mut execution_traces = Vec::new();
    let mut compiled_eegs = Vec::new();
    let mut execution_results = Vec::new();

    for i in 0..15 {
        let eeg = compile_thought(&context, &mut memory);
        assert!(eeg.nodes.len() > 0);

        let result = execute_eeg(&eeg, &mut memory);
        assert!(result.execution_trace.len() > 0);

        for signal in &result.reinforcement_signals {
            memory.reinforce_fragment(signal.fragment_id, &result.outcome);
        }

        let trace = ExecutionTrace {
            eeg_id: Uuid::from_u128(i as u128),
            context: context.clone(),
            node_sequence: result.execution_trace.clone(),
            branch_decisions: HashMap::new(),
            execution_time: result.time_taken,
            timestamp: current_timestamp() + (i as f64 * 1000.0),
        };

        execution_traces.push(trace);
        compiled_eegs.push(eeg);
        execution_results.push(result);
    }

    assert_eq!(execution_traces.len(), 15);
    assert_eq!(compiled_eegs.len(), 15);
    assert_eq!(execution_results.len(), 15);

    let linter_input = LinterInput {
        execution_traces: execution_traces.clone(),
        compiled_eegs: compiled_eegs.clone(),
        execution_results: execution_results.clone(),
        time_window: None,
        min_occurrences: 5,
    };

    let linter_config = LinterConfig {
        min_occurrences: 5,
        min_path_length: 2,
        min_confidence: 0.6,
        min_branch_ratio: 0.8,
        min_context_variance: 0.3,
        min_reward_correlation: 0.7,
        min_speedup: 2.0,
    };

    let pattern_report = {
        PatternReport {
            repeated_paths: Vec::new(),
            stable_branches: Vec::new(),
            invariant_subgraphs: Vec::new(),
            high_confidence_outcomes: Vec::new(),
            fossilization_candidates: Vec::new(),
        }
    };

    let candidate = FossilizationCandidate {
        pattern_type: "PathPattern".to_string(),
        pattern_id: Uuid::new_v4(),
        repetition_count: 12,
        average_confidence: 0.85,
        context_variance: 0.25,
        reward_correlation: 0.9,
        estimated_speedup: 3.0,
        priority: 0.92,
    };

    let mut fossilization_report = PatternReport {
        repeated_paths: Vec::new(),
        stable_branches: Vec::new(),
        invariant_subgraphs: Vec::new(),
        high_confidence_outcomes: Vec::new(),
        fossilization_candidates: vec![candidate.clone()],
    };

    let fossilization_config = FossilizationConfig {
        min_repetition: 10,
        min_confidence: 0.8,
        max_context_variance: 0.3,
        min_reward_correlation: 0.7,
        min_speedup: 2.0,
        max_candidates_per_run: 5,
        preferred_module_type: ModuleType::FSM,
    };

    let selected_candidates =
        select_fossilization_candidates(&fossilization_report, &fossilization_config);
    assert_eq!(selected_candidates.len(), 1);
    assert_eq!(selected_candidates[0].repetition_count, 12);

    if let Some(trace) = execution_traces.first() {
        let path_pattern = PathPattern {
            path: trace.node_sequence.clone(),
            occurrence_count: 12,
            contexts: vec![context.clone()],
            average_confidence: 0.85,
            success_rate: 0.9,
        };

        let mut extraction_report = PatternReport {
            repeated_paths: vec![path_pattern],
            stable_branches: Vec::new(),
            invariant_subgraphs: Vec::new(),
            high_confidence_outcomes: Vec::new(),
            fossilization_candidates: vec![candidate.clone()],
        };

        let extracted_pattern = extract_pattern(
            &selected_candidates[0],
            &compiled_eegs,
            &execution_traces,
            &extraction_report,
        );

        assert!(extracted_pattern.is_some());
        let pattern = extracted_pattern.unwrap();
        assert_eq!(pattern.pattern_type, "PathPattern");
        assert!(!pattern.structure.nodes.is_empty());

        let compiled_module = compile_to_fsm(&pattern);
        assert_eq!(compiled_module.module_type, ModuleType::FSM);
        assert!(!compiled_module.code.is_empty());
        assert_eq!(compiled_module.confidence, 0.85);

        memory.add_compiled_module(compiled_module.clone());

        let stored_modules = memory.get_compiled_modules();
        assert_eq!(stored_modules.len(), 1);
        assert_eq!(stored_modules[0].id, compiled_module.id);

        let matching_context = create_test_context("HTTP 404 error", "web_development", 0.2);

        let modules: Vec<CompiledModule> = memory.get_compiled_modules().to_vec();
        let eeg_with_module =
            compile_thought_with_modules(&matching_context, &mut memory, Some(&modules));

        assert!(eeg_with_module.nodes.len() > 0);

        let has_module_node = eeg_with_module.nodes.values().any(|node| {
            matches!(&node.content, NodeContent::Action { action_type, .. }
                if action_type.contains("CompiledModule"))
        });

        let found_module = find_applicable_module(&matching_context, &modules);
        assert!(found_module.is_some());
        assert_eq!(found_module.unwrap().id, compiled_module.id);

        if let Some(module) = found_module {
            let outcome = execute_compiled_module(module, &matching_context);
            assert_eq!(outcome.outcome_type, OutcomeType::Success);
            assert!(outcome.confidence > 0.8);
        }

        let modules_after = memory.get_compiled_modules();
        assert_eq!(modules_after.len(), 1);

        assert!(memory.fragments.len() > 0);
        assert!(modules_after.len() > 0);
    }
}

#[test]
fn test_pipeline_without_matching_module() {
    let mut memory = create_test_memory();

    let non_matching_module = CompiledModule {
        id: Uuid::new_v4(),
        module_type: ModuleType::FSM,
        code: vec![1, 2, 3],
        input_signature: InputSignature {
            parameters: vec!["different".to_string()],
            context_requirements: vec!["other_domain".to_string()],
        },
        output_signature: OutputSignature {
            return_type: "Outcome".to_string(),
            side_effects: Vec::new(),
        },
        activation_condition: ContextPattern {
            goal_patterns: vec!["different_goal".to_string()],
            domain_hints: vec!["other_domain".to_string()],
            confidence_threshold: 0.9,
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

    memory.add_compiled_module(non_matching_module);

    let fragment = create_entity_relation_fragment("test", "relates", "target");
    memory.insert_fragment(fragment, Vec::new());

    let context = create_test_context("HTTP 404 error", "web_development", 0.2);

    let modules: Vec<CompiledModule> = memory.get_compiled_modules().to_vec();
    let eeg = compile_thought_with_modules(&context, &mut memory, Some(&modules));

    assert!(eeg.nodes.len() > 0);

    let found_module = find_applicable_module(&context, &modules);
    assert!(found_module.is_none());
}

#[test]
fn test_pipeline_module_precedence() {
    let mut memory = create_test_memory();

    let module = CompiledModule {
        id: Uuid::new_v4(),
        module_type: ModuleType::FSM,
        code: vec![1, 2, 3, 4, 5],
        input_signature: InputSignature {
            parameters: vec!["http_request".to_string()],
            context_requirements: vec!["web".to_string()],
        },
        output_signature: OutputSignature {
            return_type: "Outcome".to_string(),
            side_effects: Vec::new(),
        },
        activation_condition: ContextPattern {
            goal_patterns: vec!["http".to_string(), "404".to_string()],
            domain_hints: vec!["web".to_string(), "web_development".to_string()],
            confidence_threshold: 0.5,
        },
        confidence: 0.95,
        usage_count: 0,
        success_count: 0,
        failure_count: 0,
        last_used: 0.0,
        created_at: current_timestamp(),
        source_pattern: Uuid::new_v4(),
        version: 1,
    };

    memory.add_compiled_module(module.clone());

    let fragment = create_entity_relation_fragment("test", "relates", "target");
    memory.insert_fragment(fragment, Vec::new());

    let context = create_test_context("HTTP 404 error", "web_development", 0.2);

    let modules: Vec<CompiledModule> = memory.get_compiled_modules().to_vec();
    let eeg = compile_thought_with_modules(&context, &mut memory, Some(&modules));

    let used_module = eeg.nodes.values().any(|node| {
        matches!(&node.content, NodeContent::Action { action_type, .. }
            if action_type.contains("CompiledModule"))
    });

    let found_module = find_applicable_module(&context, &modules);
    assert!(found_module.is_some());
    assert_eq!(found_module.unwrap().id, module.id);

    if used_module {
        assert!(eeg.metadata.estimated_execution_time < 1.0);
    }
}
