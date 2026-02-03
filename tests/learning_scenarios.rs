use c_mer::*;

#[path = "common.rs"]
mod common;
use common::*;

#[test]
fn test_http_404_learning_scenario() {
    let mut memory = create_test_memory();
    
    let conversation = "I'm getting a 404 error when calling an API endpoint. The URL looks correct to me. What could be wrong?";
    
    let event = ingest_conversation(conversation);
    assert!(event.atoms.len() > 0);
    
    let fragments = distill_event(&event);
    assert!(fragments.len() > 0);
    
    for fragment in &fragments {
        memory.insert_fragment(fragment.clone(), Vec::new());
    }
    
    let context = create_test_context("debug HTTP 404 error", "web_development", 0.2);
    let eeg = compile_thought(&context, &mut memory);
    
    assert!(eeg.nodes.len() > 0);
    
    let result = execute_eeg(&eeg, &mut memory);
    assert!(result.execution_trace.len() > 0);
    
    for signal in &result.reinforcement_signals {
        memory.reinforce_fragment(signal.fragment_id, &result.outcome);
    }
    
    assert!(memory.fragments.len() > 0);
}

#[test]
fn test_multi_turn_learning() {
    let mut memory = create_test_memory();
    
    let conversations = vec![
        "HTTP 404 error in API call",
        "HTTP 404 error when URL is wrong",
        "HTTP 404 error means endpoint not found",
    ];
    
    for conversation in conversations {
        let event = ingest_conversation(conversation);
        let fragments = distill_event(&event);
        
        for fragment in fragments {
            memory.insert_fragment(fragment, Vec::new());
        }
    }
    
    let context = create_test_context("HTTP 404", "web", 0.2);
    let eeg = compile_thought(&context, &mut memory);
    let result = execute_eeg(&eeg, &mut memory);
    
    assert!(result.outcome.confidence > 0.0);
    assert!(memory.fragments.len() > 0);
}

#[test]
fn test_error_correction_learning() {
    let mut memory = create_test_memory();
    let fragment = create_causal_rule_fragment("wrong_condition", "wrong_outcome", 0.8);
    
    memory.insert_fragment(fragment.clone(), Vec::new());
    
    let failure = Outcome {
        outcome_type: OutcomeType::Failure,
        result: "incorrect".to_string(),
        explanation: None,
        confidence: 0.0,
    };
    
    memory.reinforce_fragment(fragment.id, &failure);
    
    let corrected = memory.fragments.get(&fragment.id).unwrap();
    assert!(corrected.confidence < 0.8);
}

#[test]
fn test_transfer_learning() {
    let mut memory = create_test_memory();
    
    let fragment1 = create_causal_rule_fragment("HTTP_call", "404_error", 0.9);
    let fragment2 = create_causal_rule_fragment("API_call", "404_error", 0.8);
    
    memory.insert_fragment(fragment1.clone(), Vec::new());
    memory.insert_fragment(fragment2.clone(), Vec::new());
    
    let context1 = create_test_context("HTTP 404", "web", 0.2);
    let activated1 = memory.activate_fragments(&context1);
    
    let context2 = create_test_context("API 404", "api", 0.2);
    let activated2 = memory.activate_fragments(&context2);
    
    assert!(activated1.contains(&fragment1.id) || activated2.contains(&fragment2.id));
}

#[test]
fn test_forgetting_unused_knowledge() {
    let mut memory = create_test_memory();
    let fragments = create_test_fragments(50, "test");
    
    for fragment in fragments {
        memory.insert_fragment(fragment, Vec::new());
    }
    
    let initial_count = memory.fragments.len();
    
    let context = create_test_context("specific_goal", "specific_domain", 0.2);
    memory.activate_fragments(&context);
    
    simulate_time_passing(&mut memory, 60.0);
    
    assert!(memory.fragments.len() < initial_count);
}

#[test]
fn test_rapid_skill_acquisition() {
    let mut memory = create_test_memory();
    let fragment = create_entity_relation_fragment("skill", "enables", "action");
    
    memory.insert_fragment(fragment.clone(), Vec::new());
    
    let success = Outcome {
        outcome_type: OutcomeType::Success,
        result: "success".to_string(),
        explanation: None,
        confidence: 1.0,
    };
    
    for _ in 0..50 {
        memory.reinforce_fragment(fragment.id, &success);
    }
    
    let acquired = memory.fragments.get(&fragment.id).unwrap();
    assert!(acquired.confidence >= 1.0);
    assert_eq!(acquired.reinforcement_count, 50);
}

#[test]
fn test_gradual_expertise_building() {
    let mut memory = create_test_memory();
    let fragment = create_entity_relation_fragment("expertise", "improves", "performance");
    
    memory.insert_fragment(fragment.clone(), Vec::new());
    
    let success = Outcome {
        outcome_type: OutcomeType::Success,
        result: "success".to_string(),
        explanation: None,
        confidence: 1.0,
    };
    
    for i in 0..10 {
        memory.reinforce_fragment(fragment.id, &success);
        simulate_time_passing(&mut memory, 7.0);
        
        let building = memory.fragments.get(&fragment.id).unwrap();
        assert!(building.confidence > 0.5 + (i as f64 * 0.05));
    }
    
    let expert = memory.fragments.get(&fragment.id).unwrap();
    assert!(expert.confidence > 0.9);
    assert_eq!(expert.reinforcement_count, 10);
}
