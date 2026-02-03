use c_mer::*;

#[path = "common.rs"]
mod common;
use common::*;

#[test]
fn test_positive_reinforcement_increases_confidence() {
    let mut memory = create_test_memory();
    let fragment = create_entity_relation_fragment("test", "relates", "target");
    let initial_confidence = fragment.confidence;
    
    memory.insert_fragment(fragment.clone(), Vec::new());
    
    let success = Outcome {
        outcome_type: OutcomeType::Success,
        result: "success".to_string(),
        explanation: None,
        confidence: 1.0,
    };
    
    memory.reinforce_fragment(fragment.id, &success);
    
    let reinforced = memory.fragments.get(&fragment.id).unwrap();
    assert!(reinforced.confidence > initial_confidence);
    assert!(reinforced.confidence <= 1.0);
}

#[test]
fn test_negative_reinforcement_decreases_confidence() {
    let mut memory = create_test_memory();
    let fragment = create_entity_relation_fragment("test", "relates", "target");
    let initial_confidence = fragment.confidence;
    
    memory.insert_fragment(fragment.clone(), Vec::new());
    
    let failure = Outcome {
        outcome_type: OutcomeType::Failure,
        result: "failure".to_string(),
        explanation: None,
        confidence: 0.0,
    };
    
    memory.reinforce_fragment(fragment.id, &failure);
    
    let reinforced = memory.fragments.get(&fragment.id).unwrap();
    assert!(reinforced.confidence < initial_confidence);
    assert!(reinforced.confidence >= 0.0);
}

#[test]
fn test_reinforcement_strengthens_edges() {
    let mut memory = create_test_memory();
    let frag1 = create_entity_relation_fragment("test1", "relates", "target1");
    let frag2 = create_entity_relation_fragment("test2", "relates", "target2");
    let frag1_id = frag1.id;
    let mut edge = create_test_edges(frag1_id, frag2.id, 0.5);
    edge.strength = 0.5;
    
    memory.insert_fragment(frag1, vec![edge.clone()]);
    memory.insert_fragment(frag2, Vec::new());
    
    let success = Outcome {
        outcome_type: OutcomeType::Success,
        result: "success".to_string(),
        explanation: None,
        confidence: 1.0,
    };
    
    memory.reinforce_fragment(frag1_id, &success);
    
    let reinforced_edge = memory.edges.get(&(edge.from_fragment, edge.to_fragment)).unwrap();
    assert!(reinforced_edge.strength > edge.strength);
}

#[test]
fn test_repeated_success_pattern() {
    let mut memory = create_test_memory();
    let fragment = create_entity_relation_fragment("test", "relates", "target");
    
    memory.insert_fragment(fragment.clone(), Vec::new());
    
    let success = Outcome {
        outcome_type: OutcomeType::Success,
        result: "success".to_string(),
        explanation: None,
        confidence: 1.0,
    };
    
    for _ in 0..10 {
        memory.reinforce_fragment(fragment.id, &success);
    }
    
    let reinforced = memory.fragments.get(&fragment.id).unwrap();
    assert!(reinforced.confidence > 0.9);
    assert_eq!(reinforced.reinforcement_count, 10);
}

#[test]
fn test_reinforcement_count_tracking() {
    let mut memory = create_test_memory();
    let fragment = create_entity_relation_fragment("test", "relates", "target");
    
    memory.insert_fragment(fragment.clone(), Vec::new());
    
    let success = Outcome {
        outcome_type: OutcomeType::Success,
        result: "success".to_string(),
        explanation: None,
        confidence: 1.0,
    };
    
    assert_eq!(memory.fragments.get(&fragment.id).unwrap().reinforcement_count, 0);
    
    memory.reinforce_fragment(fragment.id, &success);
    
    assert_eq!(memory.fragments.get(&fragment.id).unwrap().reinforcement_count, 1);
}

#[test]
fn test_high_confidence_fragments_activate_first() {
    let mut memory = create_test_memory();
    let frag_low = create_causal_rule_fragment("test", "result", 0.3);
    let frag_high = create_causal_rule_fragment("test", "result", 0.9);
    let frag_high_id = frag_high.id;
    
    memory.insert_fragment(frag_low, Vec::new());
    memory.insert_fragment(frag_high, Vec::new());
    
    let context = create_test_context("test", "test", 0.2);
    let activated = memory.activate_fragments(&context);
    
    assert!(activated.contains(&frag_high_id));
}

#[test]
fn test_reinforced_patterns_persist() {
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
    
    simulate_time_passing(&mut memory, 7.0);
    
    if let Some(persisted) = memory.fragments.get(&fragment.id) {
        assert!(persisted.confidence > 0.7);
    } else {
        panic!("Fragment was removed but should have persisted");
    }
}

#[test]
fn test_learning_from_failure() {
    let mut memory = create_test_memory();
    let fragment = create_entity_relation_fragment("test", "relates", "target");
    let initial_confidence = fragment.confidence;
    
    memory.insert_fragment(fragment.clone(), Vec::new());
    
    let failure = Outcome {
        outcome_type: OutcomeType::Failure,
        result: "failure".to_string(),
        explanation: None,
        confidence: 0.0,
    };
    
    memory.reinforce_fragment(fragment.id, &failure);
    
    let learned = memory.fragments.get(&fragment.id).unwrap();
    assert!(learned.confidence < initial_confidence);
}

#[test]
fn test_rapid_learning_pathway() {
    let mut memory = create_test_memory();
    let fragment = create_entity_relation_fragment("test", "relates", "target");
    
    memory.insert_fragment(fragment.clone(), Vec::new());
    
    let success = Outcome {
        outcome_type: OutcomeType::Success,
        result: "success".to_string(),
        explanation: None,
        confidence: 1.0,
    };
    
    for _ in 0..20 {
        memory.reinforce_fragment(fragment.id, &success);
    }
    
    let learned = memory.fragments.get(&fragment.id).unwrap();
    assert!(learned.confidence >= 1.0);
    assert_eq!(learned.reinforcement_count, 20);
}

#[test]
fn test_gradual_learning_pathway() {
    let mut memory = create_test_memory();
    let fragment = create_entity_relation_fragment("test", "relates", "target");
    
    memory.insert_fragment(fragment.clone(), Vec::new());
    
    let success = Outcome {
        outcome_type: OutcomeType::Success,
        result: "success".to_string(),
        explanation: None,
        confidence: 1.0,
    };
    
    for _i in 0..5 {
        memory.reinforce_fragment(fragment.id, &success);
        simulate_time_passing(&mut memory, 1.0);
    }
    
    if let Some(learned) = memory.fragments.get(&fragment.id) {
        assert!(learned.confidence > 0.5);
        assert_eq!(learned.reinforcement_count, 5);
    } else {
        panic!("Fragment was removed during gradual learning");
    }
}
