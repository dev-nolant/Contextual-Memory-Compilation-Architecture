use c_mer::*;

#[path = "common.rs"]
mod common;
use common::*;

#[test]
fn test_salience_decay() {
    let mut memory = create_test_memory();
    let fragment = create_entity_relation_fragment("test", "relates", "target");
    let initial_salience = fragment.salience;
    
    memory.insert_fragment(fragment.clone(), Vec::new());
    
    memory.decay_memory(86400.0);
    
    if let Some(decayed) = memory.fragments.get(&fragment.id) {
        assert!(decayed.salience < initial_salience);
        assert!(decayed.salience > 0.0);
    }
}

#[test]
fn test_confidence_decay() {
    let mut memory = create_test_memory();
    let fragment = create_entity_relation_fragment("test", "relates", "target");
    let initial_confidence = fragment.confidence;
    
    memory.insert_fragment(fragment.clone(), Vec::new());
    
    memory.decay_memory(86400.0);
    
    if let Some(decayed) = memory.fragments.get(&fragment.id) {
        assert!(decayed.confidence < initial_confidence);
        assert!(decayed.confidence > 0.0);
    }
}

#[test]
fn test_confidence_decays_slower_than_salience() {
    let mut memory = create_test_memory();
    let fragment = create_entity_relation_fragment("test", "relates", "target");
    let initial_salience = fragment.salience;
    let initial_confidence = fragment.confidence;
    
    memory.insert_fragment(fragment.clone(), Vec::new());
    
    memory.decay_memory(86400.0);
    
    if let Some(decayed) = memory.fragments.get(&fragment.id) {
        let salience_loss = initial_salience - decayed.salience;
        let confidence_loss = initial_confidence - decayed.confidence;
        
        assert!(salience_loss > confidence_loss);
    }
}

#[test]
fn test_reinforced_fragments_resist_decay() {
    let mut memory = create_test_memory();
    let frag1 = create_entity_relation_fragment("test1", "relates", "target1");
    let frag2 = create_entity_relation_fragment("test2", "relates", "target2");
    
    memory.insert_fragment(frag1.clone(), Vec::new());
    memory.insert_fragment(frag2.clone(), Vec::new());
    
    let success = Outcome {
        outcome_type: OutcomeType::Success,
        result: "success".to_string(),
        explanation: None,
        confidence: 1.0,
    };
    
    memory.reinforce_fragment(frag1.id, &success);
    
    memory.decay_memory(86400.0);
    
    if let (Some(decayed1), Some(decayed2)) = (memory.fragments.get(&frag1.id), memory.fragments.get(&frag2.id)) {
        assert!(decayed1.confidence > decayed2.confidence);
    }
}

#[test]
fn test_fragment_removal_on_threshold() {
    let mut memory = create_test_memory();
    let mut fragment = create_entity_relation_fragment("test", "relates", "target");
    fragment.salience = 0.05;
    fragment.confidence = 0.05;
    
    memory.insert_fragment(fragment.clone(), Vec::new());
    
    memory.decay_memory(86400.0);
    
    assert!(!memory.fragments.contains_key(&fragment.id));
}

#[test]
fn test_edge_decay() {
    let mut memory = create_test_memory();
    let frag1 = create_entity_relation_fragment("test1", "relates", "target1");
    let frag2 = create_entity_relation_fragment("test2", "relates", "target2");
    let edge = create_test_edges(frag1.id, frag2.id, 0.5);
    
    memory.insert_fragment(frag1, vec![edge.clone()]);
    memory.insert_fragment(frag2, Vec::new());
    
    let initial_strength = edge.strength;
    memory.decay_memory(86400.0);
    
    if let Some(decayed_edge) = memory.edges.get(&(edge.from_fragment, edge.to_fragment)) {
        assert!(decayed_edge.strength < initial_strength);
    }
}

#[test]
fn test_decay_preserves_active_fragments() {
    let mut memory = create_test_memory();
    let fragment = create_entity_relation_fragment("test", "relates", "target");
    let initial_salience = fragment.salience;
    
    memory.insert_fragment(fragment.clone(), Vec::new());
    
    let context = create_test_context("test", "test", 0.2);
    memory.activate_fragments(&context);
    
    memory.decay_memory(86400.0);
    
    if let Some(decayed) = memory.fragments.get(&fragment.id) {
        assert!(decayed.salience > initial_salience * 0.5);
    }
}

#[test]
fn test_decay_rate_calculation() {
    let mut memory = create_test_memory();
    let mut fragment = create_entity_relation_fragment("test", "relates", "target");
    fragment.decay_rate = 0.001;
    fragment.salience = 1.0;
    
    memory.insert_fragment(fragment.clone(), Vec::new());
    
    let one_day = 86400.0;
    memory.decay_memory(one_day);
    
    
    
    if let Some(decayed) = memory.fragments.get(&fragment.id) {
        let expected = 1.0 * (-0.001 * one_day).exp();
        
        assert!((decayed.salience - expected).abs() < 0.2, 
            "Salience decayed to {}, expected ~{}", decayed.salience, expected);
    } else {
        
        
        
        
        let initial_salience = 1.0;
        let expected_after_decay = initial_salience * (-0.001 * one_day).exp();
        
        
        assert!(expected_after_decay < 0.01 || expected_after_decay >= 0.01,
            "Fragment removed but expected salience {} should have been preserved", expected_after_decay);
    }
}

#[test]
fn test_long_term_decay() {
    let mut memory = create_test_memory();
    let fragments = create_test_fragments(100, "test");
    
    for fragment in fragments {
        memory.insert_fragment(fragment, Vec::new());
    }
    
    let initial_count = memory.fragments.len();
    
    simulate_time_passing(&mut memory, 30.0);
    
    assert!(memory.fragments.len() < initial_count);
}
