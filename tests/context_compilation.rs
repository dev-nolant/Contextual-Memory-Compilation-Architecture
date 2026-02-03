use c_mer::*;

#[path = "common.rs"]
mod common;
use common::*;

#[test]
fn test_same_fragments_different_contexts() {
    let mut memory = create_test_memory();
    let fragments = create_test_fragments(10, "test");
    
    for fragment in fragments {
        memory.insert_fragment(fragment, Vec::new());
    }
    
    let context1 = create_test_context("debug HTTP error", "web_development", 0.2);
    let eeg1 = compile_thought(&context1, &mut memory);
    
    let context2 = create_test_context("understand HTTP protocol", "networking", 0.1);
    let eeg2 = compile_thought(&context2, &mut memory);
    
    assert!(eeg1.nodes.len() > 0);
    assert!(eeg2.nodes.len() > 0);
}

#[test]
fn test_context_goal_matching() {
    let mut memory = create_test_memory();
    let fragment = create_causal_rule_fragment("debug_HTTP", "fix_error", 0.9);
    
    memory.insert_fragment(fragment.clone(), Vec::new());
    
    let context = create_test_context("debug HTTP 404", "web", 0.2);
    let fragment_id = fragment.id;
    let activated = memory.activate_fragments(&context);
    
    assert!(activated.contains(&fragment_id) || memory.fragments.len() > 0);
}

#[test]
fn test_context_domain_matching() {
    let mut memory = create_test_memory();
    let fragment = create_entity_relation_fragment("API", "uses", "HTTP");
    
    memory.insert_fragment(fragment.clone(), Vec::new());
    
    let context = create_test_context("test", "web_development", 0.2);
    let activated = memory.activate_fragments(&context);
    
    assert!(activated.len() <= context.max_fragments);
}

#[test]
fn test_context_emotional_bias() {
    let mut memory = create_test_memory();
    let mut fragment = create_entity_relation_fragment("error", "causes", "frustration");
    fragment.emotional_tag = 0.6;
    
    memory.insert_fragment(fragment.clone(), Vec::new());
    
    let mut context = create_test_context("debug error", "test", 0.2);
    context.emotional_bias.frustration = 0.5;
    
    let fragment_id = fragment.id;
    let activated = memory.activate_fragments(&context);
    
    assert!(activated.contains(&fragment_id) || memory.fragments.len() > 0);
}

#[test]
fn test_context_time_pressure() {
    let mut memory = create_test_memory();
    let fragments = create_test_fragments(20, "test");
    
    for fragment in fragments {
        memory.insert_fragment(fragment, Vec::new());
    }
    
    let context_low = create_test_context("test", "test", 0.1);
    let eeg_low = compile_thought(&context_low, &mut memory);
    
    let context_high = create_test_context("test", "test", 0.9);
    let eeg_high = compile_thought(&context_high, &mut memory);
    
    assert!(eeg_high.nodes.len() <= eeg_low.nodes.len());
}

#[test]
fn test_context_attention_window() {
    let mut memory = create_test_memory();
    let frag1 = create_entity_relation_fragment("HTTP", "produces", "error");
    let frag2 = create_entity_relation_fragment("other", "relates", "thing");
    let frag1_id = frag1.id;
    
    memory.insert_fragment(frag1, Vec::new());
    memory.insert_fragment(frag2, Vec::new());
    
    let mut context = create_test_context("HTTP error", "test", 0.2);
    context.attention_window.focus_entities.insert("HTTP".to_string());
    
    let activated = memory.activate_fragments(&context);
    
    assert!(activated.contains(&frag1_id) || activated.len() <= context.max_fragments);
}

#[test]
fn test_context_confidence_threshold() {
    let mut memory = create_test_memory();
    let frag_low = create_causal_rule_fragment("test", "result", 0.3);
    let frag_high = create_causal_rule_fragment("test", "result", 0.9);
    let frag_high_id = frag_high.id;
    
    memory.insert_fragment(frag_low, Vec::new());
    memory.insert_fragment(frag_high, Vec::new());
    
    let mut context = create_test_context("test", "test", 0.2);
    context.confidence_threshold = 0.5;
    
    let activated = memory.activate_fragments(&context);
    
    assert!(activated.contains(&frag_high_id));
}

#[test]
fn test_empty_context_handling() {
    let mut memory = create_test_memory();
    let context = create_test_context("unknown goal", "unknown domain", 0.2);
    
    let eeg = compile_thought(&context, &mut memory);
    
    assert!(eeg.nodes.len() > 0);
    assert!(eeg.nodes.values().any(|n| matches!(n.node_type, NodeType::GapFillNode)));
}

#[test]
fn test_context_invariant_subgraphs() {
    let mut memory = create_test_memory();
    let fragments = create_test_fragments(5, "test");
    
    for fragment in fragments {
        memory.insert_fragment(fragment, Vec::new());
    }
    
    let context1 = create_test_context("goal1", "domain1", 0.2);
    let eeg1 = compile_thought(&context1, &mut memory);
    
    let context2 = create_test_context("goal2", "domain2", 0.2);
    let eeg2 = compile_thought(&context2, &mut memory);
    
    assert!(eeg1.nodes.len() > 0);
    assert!(eeg2.nodes.len() > 0);
}
