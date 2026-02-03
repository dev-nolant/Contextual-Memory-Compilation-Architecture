use c_mer::*;

#[path = "common.rs"]
mod common;
use common::*;

#[test]
fn test_eeg_traversal() {
    let mut memory = create_test_memory();
    let fragments = create_test_fragments(5, "test");

    for fragment in fragments {
        memory.insert_fragment(fragment, Vec::new());
    }

    let context = create_test_context("test", "test", 0.2);
    let eeg = compile_thought(&context, &mut memory);

    let result = execute_eeg(&eeg, &mut memory);

    assert!(result.execution_trace.len() > 0);
    assert!(result.execution_trace.contains(&eeg.entry_point));
}

#[test]
fn test_fragment_node_execution() {
    let mut memory = create_test_memory();
    let fragment = create_entity_relation_fragment("test", "relates", "target");

    memory.insert_fragment(fragment.clone(), Vec::new());

    let context = create_test_context("test", "test", 0.2);
    let eeg = compile_thought(&context, &mut memory);

    let result = execute_eeg(&eeg, &mut memory);

    assert!(result.outcome.confidence > 0.0);
    assert!(result.execution_trace.len() > 0);
}

#[test]
fn test_decision_node_execution() {
    let mut memory = create_test_memory();
    let fragment = create_causal_rule_fragment("condition", "outcome", 0.8);

    memory.insert_fragment(fragment, Vec::new());

    let context = create_test_context("test", "test", 0.2);
    let eeg = compile_thought(&context, &mut memory);

    let result = execute_eeg(&eeg, &mut memory);

    assert!(result.execution_trace.len() > 0);
}

#[test]
fn test_gap_fill_node_execution() {
    let mut memory = create_test_memory();
    let context = create_test_context("unknown", "unknown", 0.2);

    let eeg = compile_thought(&context, &mut memory);

    let result = execute_eeg(&eeg, &mut memory);

    assert!(result.execution_trace.len() > 0);
    assert!(result.outcome.confidence >= 0.0);
}

#[test]
fn test_execution_trace_collection() {
    let mut memory = create_test_memory();
    let fragments = create_test_fragments(3, "test");

    for fragment in fragments {
        memory.insert_fragment(fragment, Vec::new());
    }

    let context = create_test_context("test", "test", 0.2);
    let eeg = compile_thought(&context, &mut memory);

    let result = execute_eeg(&eeg, &mut memory);

    assert_eq!(
        result.execution_trace.len(),
        result
            .execution_trace
            .iter()
            .collect::<std::collections::HashSet<_>>()
            .len()
    );
    assert!(result.execution_trace.contains(&eeg.entry_point));
}

#[test]
fn test_reinforcement_signal_generation() {
    let mut memory = create_test_memory();
    let fragment = create_entity_relation_fragment("test", "relates", "target");

    memory.insert_fragment(fragment.clone(), Vec::new());

    let context = create_test_context("test", "test", 0.2);
    let eeg = compile_thought(&context, &mut memory);

    let result = execute_eeg(&eeg, &mut memory);

    if result.outcome.outcome_type == OutcomeType::Success {
        assert!(result.reinforcement_signals.len() > 0);
    }
}

#[test]
fn test_execution_with_cycles() {
    let mut memory = create_test_memory();
    let frag1 = create_entity_relation_fragment("A", "leads_to", "B");
    let frag2 = create_entity_relation_fragment("B", "leads_to", "A");

    memory.insert_fragment(frag1, Vec::new());
    memory.insert_fragment(frag2, Vec::new());

    let context = create_test_context("test", "test", 0.2);
    let eeg = compile_thought(&context, &mut memory);

    let result = execute_eeg(&eeg, &mut memory);

    assert!(result.execution_trace.len() > 0);
    assert!(result.execution_trace.len() < 100);
}

#[test]
fn test_execution_performance() {
    let mut memory = create_test_memory();
    let fragments = create_test_fragments(50, "test");

    for fragment in fragments {
        memory.insert_fragment(fragment, Vec::new());
    }

    let context = create_test_context("test", "test", 0.2);
    let eeg = compile_thought(&context, &mut memory);

    let (elapsed, result) = measure_time(|| execute_eeg(&eeg, &mut memory));

    assert!(result.execution_trace.len() > 0);
    assert!(elapsed < 1.0);
}
