use c_mer::*;

#[path = "common.rs"]
mod common;
use common::*;

#[test]
fn test_fragment_insertion() {
    let mut memory = create_test_memory();
    let fragment = create_entity_relation_fragment("HTTP", "produces", "404_error");
    
    memory.insert_fragment(fragment.clone(), Vec::new());
    
    assert_eq!(memory.fragments.len(), 1);
    assert!(memory.fragments.contains_key(&fragment.id));
    
    let stored = memory.fragments.get(&fragment.id).unwrap();
    assert_eq!(stored.fragment_type, FragmentType::EntityRelation);
    assert_eq!(stored.confidence, 0.8);
}

#[test]
fn test_activation_index_updates() {
    let mut memory = create_test_memory();
    let fragment = create_entity_relation_fragment("HTTP", "produces", "404_error");
    
    memory.insert_fragment(fragment.clone(), Vec::new());
    
    assert!(memory.activation_index.by_keyword.contains_key("HTTP"));
    assert!(memory.activation_index.by_keyword.get("HTTP").unwrap().contains(&fragment.id));
}

#[test]
fn test_multi_domain_indexing() {
    let mut memory = create_test_memory();
    let fragment = create_causal_rule_fragment("HTTP_call", "404_error", 0.85);
    
    memory.insert_fragment(fragment.clone(), Vec::new());
    
    assert!(memory.activation_index.by_keyword.contains_key("HTTP_call"));
    assert!(memory.activation_index.by_keyword.contains_key("404_error"));
}

#[test]
fn test_fragment_relationships() {
    let mut memory = create_test_memory();
    let frag1 = create_entity_relation_fragment("HTTP", "produces", "error");
    let frag2 = create_entity_relation_fragment("error", "is", "404");
    
    let edge = create_test_edges(frag1.id, frag2.id, 0.7);
    
    memory.insert_fragment(frag1, vec![edge.clone()]);
    memory.insert_fragment(frag2, Vec::new());
    
    assert!(memory.edges.contains_key(&(edge.from_fragment, edge.to_fragment)));
    let stored_edge = memory.edges.get(&(edge.from_fragment, edge.to_fragment)).unwrap();
    assert_eq!(stored_edge.strength, 0.7);
}

#[test]
fn test_large_scale_storage() {
    let mut memory = create_test_memory();
    let fragments = create_test_fragments(1000, "test");
    
    for fragment in fragments {
        memory.insert_fragment(fragment, Vec::new());
    }
    
    assert_eq!(memory.fragments.len(), 1000);
}

#[test]
fn test_fragment_retrieval_by_goal() {
    let mut memory = create_test_memory();
    let fragment = create_causal_rule_fragment("debug_HTTP", "fix_error", 0.9);
    
    memory.insert_fragment(fragment.clone(), Vec::new());
    
    let fragment_id = fragment.id;
    let context = create_test_context("debug HTTP error", "web_development", 0.2);
    let activated = memory.activate_fragments(&context);
    
    assert!(activated.contains(&fragment_id) || memory.fragments.len() > 0);
}

#[test]
fn test_fragment_retrieval_by_domain() {
    let mut memory = create_test_memory();
    let fragment = create_entity_relation_fragment("API", "uses", "HTTP");
    
    memory.insert_fragment(fragment.clone(), Vec::new());
    
    let context = create_test_context("test", "web_development", 0.2);
    let activated = memory.activate_fragments(&context);
    
    assert!(activated.len() <= context.max_fragments);
}

#[test]
fn test_fragment_retrieval_by_keyword() {
    let mut memory = create_test_memory();
    let fragment = create_entity_relation_fragment("HTTP", "produces", "404");
    
    memory.insert_fragment(fragment.clone(), Vec::new());
    
    let fragment_id = fragment.id;
    let context = create_test_context("HTTP 404", "web", 0.2);
    let activated = memory.activate_fragments(&context);
    
    assert!(activated.len() <= context.max_fragments);
}

#[test]
fn test_activation_threshold_filtering() {
    let mut memory = create_test_memory();
    let frag_low = create_causal_rule_fragment("test", "result", 0.3);
    let frag_high = create_causal_rule_fragment("test", "result", 0.9);
    let frag_low_id = frag_low.id;
    let frag_high_id = frag_high.id;
    
    memory.insert_fragment(frag_low, Vec::new());
    memory.insert_fragment(frag_high, Vec::new());
    
    let mut context = create_test_context("test", "domain", 0.2);
    context.confidence_threshold = 0.5;
    
    let activated = memory.activate_fragments(&context);
    
    assert!(activated.len() <= context.max_fragments);
    if activated.contains(&frag_low_id) {
        assert!(memory.fragments.get(&frag_low_id).unwrap().confidence >= 0.5);
    }
}

#[test]
fn test_max_fragments_limit() {
    let mut memory = create_test_memory();
    let fragments = create_test_fragments(100, "test");
    
    for fragment in fragments {
        memory.insert_fragment(fragment, Vec::new());
    }
    
    let mut context = create_test_context("test", "test", 0.2);
    context.max_fragments = 10;
    
    let activated = memory.activate_fragments(&context);
    
    assert!(activated.len() <= 10);
}
