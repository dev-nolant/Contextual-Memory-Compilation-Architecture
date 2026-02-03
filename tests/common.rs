use c_mer::*;
use std::collections::HashMap;
use uuid::Uuid;

pub fn create_test_memory() -> MemoryGraph {
    MemoryGraph::new()
}

pub fn create_test_context(goal: &str, domain: &str, time_pressure: f64) -> ContextVector {
    generate_context(goal, domain, time_pressure)
}

pub fn create_test_fragments(count: usize, domain: &str) -> Vec<MFragment> {
    let mut fragments = Vec::new();
    let timestamp = current_timestamp();
    
    for i in 0..count {
        let fragment = MFragment {
            id: Uuid::new_v4(),
            fragment_type: FragmentType::EntityRelation,
            content: FragmentContent::EntityRelation {
                entity: format!("entity_{}", i),
                relation: "relates_to".to_string(),
                target: format!("target_{}", i),
            },
            confidence: 0.7 + (i as f64 * 0.01),
            salience: 1.0 - (i as f64 * 0.05),
            emotional_tag: 0.0,
            reinforcement_count: 0,
            last_activated: 0.0,
            activation_history: Vec::new(),
            created_at: timestamp,
            decay_rate: 0.001,
        };
        fragments.push(fragment);
    }
    
    fragments
}

pub fn create_causal_rule_fragment(
    condition: &str,
    outcome: &str,
    confidence: f64,
) -> MFragment {
    MFragment {
        id: Uuid::new_v4(),
        fragment_type: FragmentType::CausalRule,
        content: FragmentContent::CausalRule {
            condition: condition.to_string(),
            outcome: outcome.to_string(),
            confidence,
        },
        confidence,
        salience: 1.0,
        emotional_tag: 0.0,
        reinforcement_count: 0,
        last_activated: 0.0,
        activation_history: Vec::new(),
        created_at: current_timestamp(),
        decay_rate: 0.001,
    }
}

pub fn create_entity_relation_fragment(
    entity: &str,
    relation: &str,
    target: &str,
) -> MFragment {
    MFragment {
        id: Uuid::new_v4(),
        fragment_type: FragmentType::EntityRelation,
        content: FragmentContent::EntityRelation {
            entity: entity.to_string(),
            relation: relation.to_string(),
            target: target.to_string(),
        },
        confidence: 0.8,
        salience: 1.0,
        emotional_tag: 0.0,
        reinforcement_count: 0,
        last_activated: 0.0,
        activation_history: Vec::new(),
        created_at: current_timestamp(),
        decay_rate: 0.001,
    }
}

pub fn assert_fragments_equal(f1: &MFragment, f2: &MFragment) -> bool {
    f1.fragment_type == f2.fragment_type
        && f1.confidence == f2.confidence
        && f1.salience == f2.salience
}

pub fn measure_time<F, R>(f: F) -> (f64, R)
where
    F: FnOnce() -> R,
{
    let start = current_timestamp();
    let result = f();
    let elapsed = current_timestamp() - start;
    (elapsed, result)
}

pub fn simulate_time_passing(memory: &mut MemoryGraph, days: f64) {
    let seconds = days * 86400.0;
    memory.decay_memory(seconds);
}

pub fn create_test_edges(from: Uuid, to: Uuid, strength: f64) -> Edge {
    Edge {
        from_fragment: from,
        to_fragment: to,
        edge_type: EdgeType::Causal,
        strength,
        last_reinforced: current_timestamp(),
        created_at: current_timestamp(),
        decay_rate: 0.001,
    }
}

pub fn populate_memory_with_fragments(memory: &mut MemoryGraph, fragments: Vec<MFragment>) {
    for fragment in fragments {
        memory.insert_fragment(fragment, Vec::new());
    }
}
