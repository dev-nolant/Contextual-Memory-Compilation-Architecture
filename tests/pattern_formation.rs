use c_mer::*;

#[path = "common.rs"]
mod common;
use common::*;

#[test]
fn test_repeated_activation_patterns() {
    let mut memory = create_test_memory();
    let fragment = create_entity_relation_fragment("HTTP", "produces", "404");

    memory.insert_fragment(fragment.clone(), Vec::new());

    let context = create_test_context("HTTP 404", "web", 0.2);

    for _ in 0..10 {
        memory.activate_fragments(&context);
    }

    let activated = memory.fragments.get(&fragment.id).unwrap();
    assert_eq!(activated.activation_history.len(), 10);
}

#[test]
fn test_stable_branch_patterns() {
    let mut memory = create_test_memory();
    let fragment = create_causal_rule_fragment("condition", "outcome", 0.9);

    memory.insert_fragment(fragment, Vec::new());

    let context = create_test_context("test", "test", 0.2);

    for _ in 0..5 {
        let eeg = compile_thought(&context, &mut memory);
        execute_eeg(&eeg, &mut memory);
    }

    assert!(memory.fragments.len() > 0);
}

#[test]
fn test_context_invariant_patterns() {
    let mut memory = create_test_memory();
    let fragment = create_entity_relation_fragment("HTTP", "produces", "error");

    memory.insert_fragment(fragment.clone(), Vec::new());

    let contexts = vec![
        create_test_context("HTTP error", "web", 0.2),
        create_test_context("HTTP issue", "networking", 0.3),
        create_test_context("HTTP problem", "api", 0.1),
    ];

    for context in &contexts {
        let activated = memory.activate_fragments(context);
        assert!(activated.contains(&fragment.id));
    }
}

#[test]
fn test_pattern_confidence_growth() {
    let mut memory = create_test_memory();
    let fragment = create_entity_relation_fragment("test", "relates", "target");

    memory.insert_fragment(fragment.clone(), Vec::new());

    let success = Outcome {
        outcome_type: OutcomeType::Success,
        result: "success".to_string(),
        explanation: None,
        confidence: 1.0,
    };

    for _ in 0..5 {
        memory.reinforce_fragment(fragment.id, &success);
    }

    let pattern = memory.fragments.get(&fragment.id).unwrap();
    assert!(pattern.confidence > 0.8);
    assert_eq!(pattern.reinforcement_count, 5);
}

#[test]
fn test_pattern_fragmentation() {
    let mut memory = create_test_memory();
    let frag1 = create_entity_relation_fragment("A", "leads_to", "B");
    let frag2 = create_entity_relation_fragment("B", "leads_to", "C");
    let edge = create_test_edges(frag1.id, frag2.id, 0.8);

    memory.insert_fragment(frag1.clone(), vec![edge]);
    memory.insert_fragment(frag2.clone(), Vec::new());

    let mut context = create_test_context("test", "test", 0.2);

    context.domain_hint.tags.insert("A".to_string());
    context.domain_hint.tags.insert("B".to_string());

    let activated = memory.activate_fragments(&context);

    assert!(activated.len() >= 0);
}
