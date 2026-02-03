use c_mer::*;

#[path = "common.rs"]
mod common;
use common::*;

#[test]
fn test_recall_without_retrieval() {
    let mut memory = create_test_memory();
    let conversation = "HTTP 404 error in API call";

    let event = ingest_conversation(conversation);
    let fragments = distill_event(&event);

    for fragment in fragments {
        memory.insert_fragment(fragment, Vec::new());
    }

    let context = create_test_context("HTTP 404", "web", 0.2);
    let eeg = compile_thought(&context, &mut memory);
    let result = execute_eeg(&eeg, &mut memory);

    assert!(result.outcome.confidence > 0.0);
    assert!(!result.outcome.result.is_empty());
}

#[test]
fn test_context_adaptation() {
    let mut memory = create_test_memory();

    let frag1 = create_entity_relation_fragment("error", "requires", "debugging");
    let frag2 = create_entity_relation_fragment("protocol", "requires", "understanding");

    memory.insert_fragment(frag1, Vec::new());
    memory.insert_fragment(frag2, Vec::new());

    let context1 = create_test_context("debug error", "web", 0.2);
    let eeg1 = compile_thought(&context1, &mut memory);

    let context2 = create_test_context("understand protocol", "networking", 0.1);
    let eeg2 = compile_thought(&context2, &mut memory);

    assert_ne!(eeg1.entry_point, eeg2.entry_point);

    assert!(eeg1.metadata.confidence_score >= 0.0);
    assert!(eeg2.metadata.confidence_score >= 0.0);
}

#[test]
fn test_natural_forgetting() {
    let mut memory = create_test_memory();
    let fragments = create_test_fragments(100, "test");

    for fragment in fragments {
        memory.insert_fragment(fragment, Vec::new());
    }

    let initial_count = memory.fragments.len();

    simulate_time_passing(&mut memory, 30.0);

    assert!(memory.fragments.len() < initial_count);
}

#[test]
fn test_compact_storage() {
    let conversation = "HTTP 404 error when calling API endpoint with incorrect URL";
    let event = ingest_conversation(conversation);
    let fragments = distill_event(&event);

    let raw_size = conversation.len();
    let fragment_count = fragments.len();

    assert!(fragment_count > 0);
    assert!(fragment_count < 20);
}

#[test]
fn test_explainability() {
    let mut memory = create_test_memory();
    let fragment = create_entity_relation_fragment("HTTP", "produces", "404");

    memory.insert_fragment(fragment.clone(), Vec::new());

    let context = create_test_context("HTTP 404", "web", 0.2);
    let eeg = compile_thought(&context, &mut memory);

    let node = eeg.nodes.get(&eeg.entry_point).unwrap();

    match &node.content {
        NodeContent::Fragment { fragment_id, .. } => {
            assert_eq!(*fragment_id, fragment.id);
        }
        _ => {}
    }

    let result = execute_eeg(&eeg, &mut memory);
    assert!(result.execution_trace.len() > 0);
}
