use c_mer::*;

#[test]
fn test_end_to_end_http_404_scenario() {
    let mut memory = MemoryGraph::new();
    
    let conversation = "I'm getting a 404 error when calling an API endpoint. The URL looks correct to me. What could be wrong?";
    
    let event = ingest_conversation(conversation);
    assert!(event.atoms.len() > 0);
    assert_eq!(event.event_type, EventType::Conversation);
    
    let fragments = distill_event(&event);
    assert!(fragments.len() > 0);
    
    for fragment in &fragments {
        memory.insert_fragment(fragment.clone(), Vec::new());
    }
    
    assert_eq!(memory.fragments.len(), fragments.len());
    
    let context = generate_context("debug HTTP 404 error", "web_development", 0.2);
    assert_eq!(context.goal.description, "debug HTTP 404 error");
    assert_eq!(context.domain_hint.domain, "web_development");
    
    let eeg = compile_thought(&context, &mut memory);
    assert!(eeg.nodes.len() > 0);
    
    let result = execute_eeg(&eeg, &mut memory);
    assert!(result.execution_trace.len() > 0);
    assert!(result.confidence > 0.0);
    
    let original_frag_count = memory.fragments.len();
    memory.decay_memory(86400.0);
    
    assert!(memory.fragments.len() <= original_frag_count);
}

#[test]
fn test_context_dependent_compilation() {
    let mut memory = MemoryGraph::new();
    
    let event1 = ingest_conversation("HTTP 404 error in API call");
    let fragments1 = distill_event(&event1);
    for fragment in fragments1 {
        memory.insert_fragment(fragment, Vec::new());
    }
    
    let context1 = generate_context("debug HTTP error", "web_development", 0.2);
    let eeg1 = compile_thought(&context1, &mut memory);
    
    let context2 = generate_context("understand HTTP protocol", "networking", 0.1);
    let eeg2 = compile_thought(&context2, &mut memory);
    
    assert_ne!(eeg1.entry_point, eeg2.entry_point);
}

#[test]
fn test_reinforcement() {
    let mut memory = MemoryGraph::new();
    
    let event = ingest_conversation("HTTP request error 404 in API call");
    let fragments = distill_event(&event);
    
    assert!(fragments.len() > 0, "Should create at least one fragment");
    
    for fragment in &fragments {
        memory.insert_fragment(fragment.clone(), Vec::new());
    }
    
    let fragment_id = fragments[0].id;
    let initial_confidence = memory.fragments.get(&fragment_id).unwrap().confidence;
    let initial_reinforcement = memory.fragments.get(&fragment_id).unwrap().reinforcement_count;
    
    let success_outcome = Outcome {
        outcome_type: OutcomeType::Success,
        result: "Success".to_string(),
        explanation: None,
        confidence: 1.0,
    };
    
    memory.reinforce_fragment(fragment_id, &success_outcome);
    
    let new_confidence = memory.fragments.get(&fragment_id).unwrap().confidence;
    let new_reinforcement = memory.fragments.get(&fragment_id).unwrap().reinforcement_count;
    
    assert!(new_reinforcement > initial_reinforcement, "Reinforcement count should increase");
    if initial_confidence < 1.0 {
        assert!(new_confidence >= initial_confidence, "Confidence should not decrease");
    }
}
