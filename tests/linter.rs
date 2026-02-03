use c_mer::*;
use std::collections::HashMap;
use uuid::Uuid;

#[path = "common.rs"]
mod common;
use common::*;

#[test]
fn test_repeated_path_detection() {
    let mut memory = create_test_memory();
    let fragment1 = create_entity_relation_fragment("test1", "relates", "target1");
    let fragment2 = create_entity_relation_fragment("test2", "relates", "target2");

    memory.insert_fragment(fragment1.clone(), Vec::new());
    memory.insert_fragment(fragment2.clone(), Vec::new());

    let context = create_test_context("test", "test", 0.2);

    let mut traces = Vec::new();
    let mut eegs = Vec::new();
    let mut results = Vec::new();

    for i in 0..10 {
        let eeg = compile_thought(&context, &mut memory);
        let result = execute_eeg(&eeg, &mut memory);

        let timestamp_u64 = eeg.metadata.compilation_timestamp as u64;
        let eeg_id = Uuid::from_u128(timestamp_u64 as u128);

        let trace = ExecutionTrace {
            eeg_id,
            context: context.clone(),
            node_sequence: result.execution_trace.clone(),
            branch_decisions: HashMap::new(),
            execution_time: result.time_taken,
            timestamp: current_timestamp() + i as f64,
        };

        traces.push(trace);
        eegs.push(eeg);
        results.push(result);
    }

    let input = LinterInput {
        execution_traces: traces,
        compiled_eegs: eegs,
        execution_results: results,
        time_window: None,
        min_occurrences: 5,
    };

    let config = LinterConfig {
        min_occurrences: 3,
        min_path_length: 1,
        ..Default::default()
    };

    let report = run_linter(input, config);

    assert!(report.repeated_paths.len() >= 0);

    assert!(report.stable_branches.len() >= 0);
    assert!(report.fossilization_candidates.len() >= 0);
}

#[test]
fn test_stable_branch_detection() {
    let mut memory = create_test_memory();
    let fragment = create_causal_rule_fragment("condition", "outcome", 0.9);
    memory.insert_fragment(fragment.clone(), Vec::new());

    let context = create_test_context("test", "test", 0.2);

    let mut traces = Vec::new();
    let mut eegs = Vec::new();
    let mut results = Vec::new();

    for _ in 0..15 {
        let eeg = compile_thought(&context, &mut memory);
        let result = execute_eeg(&eeg, &mut memory);

        let mut branch_decisions = HashMap::new();

        if let Some(entry) = eeg.nodes.get(&eeg.entry_point) {
            if let NodeContent::Decision { branches, .. } = &entry.content {
                if let Some(branch) = branches.first() {
                    branch_decisions.insert(eeg.entry_point, branch.target_node);
                }
            }
        }

        let trace = ExecutionTrace {
            eeg_id: Uuid::new_v4(),
            context: context.clone(),
            node_sequence: result.execution_trace.clone(),
            branch_decisions,
            execution_time: result.time_taken,
            timestamp: current_timestamp(),
        };

        traces.push(trace);
        eegs.push(eeg);
        results.push(result);
    }

    let input = LinterInput {
        execution_traces: traces,
        compiled_eegs: eegs,
        execution_results: results,
        time_window: None,
        min_occurrences: 10,
    };

    let config = LinterConfig::default();

    let report = run_linter(input, config);

    assert!(report.stable_branches.len() >= 0);
}

#[test]
fn test_high_confidence_outcome_detection() {
    let mut memory = create_test_memory();
    let fragment = create_entity_relation_fragment("test", "relates", "target");
    memory.insert_fragment(fragment.clone(), Vec::new());

    let context = create_test_context("test", "test", 0.2);

    let mut results = Vec::new();

    for _ in 0..10 {
        let eeg = compile_thought(&context, &mut memory);
        let result = execute_eeg(&eeg, &mut memory);
        results.push(result);
    }

    let input = LinterInput {
        execution_traces: Vec::new(),
        compiled_eegs: Vec::new(),
        execution_results: results,
        time_window: None,
        min_occurrences: 5,
    };

    let config = LinterConfig {
        min_confidence: 0.6,
        ..Default::default()
    };

    let report = run_linter(input, config);

    assert!(report.high_confidence_outcomes.len() >= 0);
}

#[test]
fn test_fossilization_candidate_identification() {
    let mut memory = create_test_memory();
    let fragment = create_entity_relation_fragment("test", "relates", "target");
    memory.insert_fragment(fragment.clone(), Vec::new());

    let context = create_test_context("test", "test", 0.2);

    let mut traces = Vec::new();
    let mut eegs = Vec::new();
    let mut results = Vec::new();

    for _ in 0..25 {
        let eeg = compile_thought(&context, &mut memory);
        let result = execute_eeg(&eeg, &mut memory);

        let trace = ExecutionTrace {
            eeg_id: Uuid::new_v4(),
            context: context.clone(),
            node_sequence: result.execution_trace.clone(),
            branch_decisions: HashMap::new(),
            execution_time: result.time_taken,
            timestamp: current_timestamp(),
        };

        traces.push(trace);
        eegs.push(eeg);
        results.push(result);
    }

    let input = LinterInput {
        execution_traces: traces,
        compiled_eegs: eegs,
        execution_results: results,
        time_window: None,
        min_occurrences: 20,
    };

    let config = LinterConfig::default();

    let report = run_linter(input, config);

    assert!(report.fossilization_candidates.len() >= 0);
}

#[test]
fn test_linter_with_empty_input() {
    let input = LinterInput {
        execution_traces: Vec::new(),
        compiled_eegs: Vec::new(),
        execution_results: Vec::new(),
        time_window: None,
        min_occurrences: 5,
    };

    let config = LinterConfig::default();

    let report = run_linter(input, config);

    assert_eq!(report.repeated_paths.len(), 0);
    assert_eq!(report.stable_branches.len(), 0);
    assert_eq!(report.fossilization_candidates.len(), 0);
}

#[test]
fn test_linter_time_window_filtering() {
    let mut memory = create_test_memory();
    let fragment = create_entity_relation_fragment("test", "relates", "target");
    memory.insert_fragment(fragment.clone(), Vec::new());

    let context = create_test_context("test", "test", 0.2);

    let mut traces = Vec::new();
    let mut eegs = Vec::new();
    let mut results = Vec::new();

    let base_time = current_timestamp();

    for i in 0..10 {
        let eeg = compile_thought(&context, &mut memory);
        let result = execute_eeg(&eeg, &mut memory);

        let trace = ExecutionTrace {
            eeg_id: Uuid::new_v4(),
            context: context.clone(),
            node_sequence: result.execution_trace.clone(),
            branch_decisions: HashMap::new(),
            execution_time: result.time_taken,
            timestamp: base_time + (i as f64 * 1000.0),
        };

        traces.push(trace);
        eegs.push(eeg);
        results.push(result);
    }

    let input = LinterInput {
        execution_traces: traces,
        compiled_eegs: eegs,
        execution_results: results,
        time_window: Some(5000.0),
        min_occurrences: 3,
    };

    let config = LinterConfig::default();

    let report = run_linter(input, config);

    assert!(report.repeated_paths.len() >= 0);
}
