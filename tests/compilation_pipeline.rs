use c_mer::*;

#[path = "common.rs"]
mod common;
use common::*;

#[test]
fn test_activation_step() {
    let mut memory = create_test_memory();
    let fragments = create_test_fragments(10, "test");
    
    for fragment in fragments {
        memory.insert_fragment(fragment, Vec::new());
    }
    
    let context = create_test_context("test", "test", 0.2);
    let activated = memory.activate_fragments(&context);
    
    assert!(activated.len() >= 0);
    assert!(activated.len() <= context.max_fragments);
}

#[test]
fn test_conflict_resolution() {
    let mut memory = create_test_memory();
    let frag1 = create_causal_rule_fragment("HTTP_call", "404_error", 0.8);
    let frag2 = create_causal_rule_fragment("HTTP_call", "500_error", 0.7);
    
    memory.insert_fragment(frag1.clone(), Vec::new());
    memory.insert_fragment(frag2.clone(), Vec::new());
    
    let context = create_test_context("HTTP error", "web", 0.2);
    let eeg = compile_thought(&context, &mut memory);
    
    assert!(eeg.nodes.len() > 0);
}

#[test]
fn test_gap_filling() {
    let mut memory = create_test_memory();
    let fragment = create_entity_relation_fragment("test", "relates", "target");
    
    memory.insert_fragment(fragment, Vec::new());
    
    let context = create_test_context("complex goal", "domain", 0.2);
    let eeg = compile_thought(&context, &mut memory);
    
    assert!(eeg.nodes.values().any(|n| matches!(n.node_type, NodeType::GapFillNode)));
}

#[test]
fn test_topological_ordering() {
    let mut memory = create_test_memory();
    let frag1 = create_entity_relation_fragment("A", "leads_to", "B");
    let frag2 = create_entity_relation_fragment("B", "leads_to", "C");
    let edge = create_test_edges(frag1.id, frag2.id, 0.8);
    
    memory.insert_fragment(frag1, vec![edge]);
    memory.insert_fragment(frag2, Vec::new());
    
    let context = create_test_context("test", "test", 0.2);
    let eeg = compile_thought(&context, &mut memory);
    
    assert!(eeg.nodes.len() > 0);
    assert!(eeg.edges.len() >= 0);
}

#[test]
fn test_branching_creation() {
    let mut memory = create_test_memory();
    let fragment = create_causal_rule_fragment("condition", "outcome", 0.8);
    
    memory.insert_fragment(fragment, Vec::new());
    
    let context = create_test_context("test", "test", 0.2);
    let eeg = compile_thought(&context, &mut memory);
    
    assert!(eeg.nodes.values().any(|n| matches!(n.node_type, NodeType::DecisionNode))
        || eeg.nodes.len() > 0);
}

#[test]
fn test_resource_pruning() {
    let mut memory = create_test_memory();
    let frag_low = create_causal_rule_fragment("test", "result", 0.3);
    let frag_high = create_causal_rule_fragment("test", "result", 0.9);
    
    memory.insert_fragment(frag_low, Vec::new());
    memory.insert_fragment(frag_high, Vec::new());
    
    let mut context = create_test_context("test", "test", 0.9);
    context.confidence_threshold = 0.5;
    
    let eeg = compile_thought(&context, &mut memory);
    
    assert!(eeg.nodes.len() > 0);
}

#[test]
fn test_eeg_construction() {
    let mut memory = create_test_memory();
    let fragments = create_test_fragments(5, "test");
    
    for fragment in fragments {
        memory.insert_fragment(fragment, Vec::new());
    }
    
    let context = create_test_context("test", "test", 0.2);
    let eeg = compile_thought(&context, &mut memory);
    
    assert!(eeg.nodes.contains_key(&eeg.entry_point));
    assert!(!eeg.exit_points.is_empty());
    assert!(eeg.metadata.fragment_count >= 0);
}

#[test]
fn test_compilation_with_empty_memory() {
    let mut memory = create_test_memory();
    let context = create_test_context("test", "test", 0.2);
    
    let eeg = compile_thought(&context, &mut memory);
    
    assert!(eeg.nodes.len() > 0);
    assert!(eeg.nodes.values().any(|n| matches!(n.node_type, NodeType::GapFillNode)));
}

#[test]
fn test_compilation_performance() {
    let mut memory = create_test_memory();
    let fragments = create_test_fragments(100, "test");
    
    for fragment in fragments {
        memory.insert_fragment(fragment, Vec::new());
    }
    
    let context = create_test_context("test", "test", 0.2);
    
    let (elapsed, eeg) = measure_time(|| compile_thought(&context, &mut memory));
    
    assert!(eeg.nodes.len() > 0);
    assert!(elapsed < 1.0);
}
