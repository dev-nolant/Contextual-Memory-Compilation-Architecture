use c_mer::ingestion::*;
use c_mer::*;

#[test]
fn test_personal_pattern_extraction() {
    let text = "My name is Nolan and I like coffee";
    let matches = personal::extract_personal_atoms(text);

    assert!(!matches.is_empty(), "Should extract personal information");

    let has_name = matches
        .iter()
        .any(|m| m.content.get("fact_type") == Some(&"name".to_string()));
    assert!(has_name, "Should extract name");

    let has_preference = matches.iter().any(|m| m.content.contains_key("preference"));
    assert!(has_preference, "Should extract preference");
}

#[test]
fn test_temporal_pattern_extraction() {
    let text = "I'll meet you tomorrow at 3pm for 2 hours";
    let matches = temporal::extract_temporal_atoms(text);

    assert!(!matches.is_empty(), "Should extract temporal information");

    let has_temporal = matches
        .iter()
        .any(|m| matches!(m.atom_type, AtomType::Time));
    assert!(has_temporal, "Should extract time atoms");
}

#[test]
fn test_spatial_pattern_extraction() {
    let text = "The office is located in San Francisco near the park";
    let matches = spatial::extract_spatial_atoms(text);

    assert!(!matches.is_empty(), "Should extract spatial information");

    let has_location = matches
        .iter()
        .any(|m| matches!(m.atom_type, AtomType::Location));
    assert!(has_location, "Should extract location atoms");
}

#[test]
fn test_quantitative_pattern_extraction() {
    let text = "The package weighs 5 kg and costs more than 100 dollars";
    let matches = quantitative::extract_quantitative_atoms(text);

    assert!(
        !matches.is_empty(),
        "Should extract quantitative information"
    );

    let has_quantity = matches
        .iter()
        .any(|m| matches!(m.atom_type, AtomType::Quantity));
    assert!(has_quantity, "Should extract quantity atoms");
}

#[test]
fn test_causal_pattern_extraction() {
    let text = "If the server is down, then the API call fails";
    let matches = causal::extract_causal_atoms(text);

    assert!(!matches.is_empty(), "Should extract causal relationships");

    let has_causal = matches.iter().any(|m| {
        m.content.contains_key("causal_marker") || m.content.contains_key("enable_marker")
    });
    assert!(has_causal, "Should extract causal markers");
}

#[test]
fn test_hierarchical_pattern_extraction() {
    let text = "Python is part of the programming languages category";
    let matches = hierarchical::extract_hierarchical_atoms(text);

    assert!(
        !matches.is_empty(),
        "Should extract hierarchical relationships"
    );

    let has_hierarchical = matches
        .iter()
        .any(|m| m.content.contains_key("hierarchical_marker"));
    assert!(has_hierarchical, "Should extract hierarchical markers");
}

#[test]
fn test_social_pattern_extraction() {
    let text = "Alice is a friend of Bob and they work together";
    let matches = social::extract_social_atoms(text);

    assert!(!matches.is_empty(), "Should extract social relationships");

    let has_social = matches
        .iter()
        .any(|m| m.content.contains_key("social_marker"));
    assert!(has_social, "Should extract social markers");
}

#[test]
fn test_ownership_pattern_extraction() {
    let text = "I own a car and it belongs to me";
    let matches = ownership::extract_ownership_atoms(text);

    assert!(
        !matches.is_empty(),
        "Should extract ownership relationships"
    );

    let has_ownership = matches
        .iter()
        .any(|m| m.content.contains_key("ownership_marker"));
    assert!(has_ownership, "Should extract ownership markers");
}

#[test]
fn test_state_pattern_extraction() {
    let text = "The server is running and the service is active";
    let matches = state::extract_state_atoms(text);

    assert!(!matches.is_empty(), "Should extract state information");

    let has_state = matches
        .iter()
        .any(|m| m.content.contains_key("state_marker"));
    assert!(has_state, "Should extract state markers");
}

#[test]
fn test_technical_pattern_extraction() {
    let text = "I'm using Python to call the HTTP API";
    let matches = technical::extract_technical_atoms(text);

    assert!(!matches.is_empty(), "Should extract technical information");

    let has_technical = matches
        .iter()
        .any(|m| matches!(m.atom_type, AtomType::Entity | AtomType::Action));
    assert!(has_technical, "Should extract technical atoms");
}

#[test]
fn test_comprehensive_extraction() {
    let text = "My name is Nolan, I like coffee, and I work in San Francisco. I'll meet you tomorrow at 3pm.";

    let matches = extract_all_patterns(text);

    assert!(
        !matches.is_empty(),
        "Should extract multiple types of information"
    );

    let has_personal = matches
        .iter()
        .any(|m| matches!(m.atom_type, AtomType::Person));
    let has_temporal = matches
        .iter()
        .any(|m| matches!(m.atom_type, AtomType::Time));
    let has_spatial = matches
        .iter()
        .any(|m| matches!(m.atom_type, AtomType::Location));

    assert!(has_personal, "Should extract personal information");
    assert!(has_temporal, "Should extract temporal information");
    assert!(has_spatial, "Should extract spatial information");
}

#[test]
fn test_fragment_extraction_personal_fact() {
    let mut event = SemanticEvent {
        id: Uuid::new_v4(),
        timestamp: current_timestamp(),
        event_type: EventType::Conversation,
        atoms: vec![SemanticAtom {
            atom_type: AtomType::Person,
            content: {
                let mut m = HashMap::new();
                m.insert("name".to_string(), "Nolan".to_string());
                m.insert("fact_type".to_string(), "name".to_string());
                m.insert("value".to_string(), "Nolan".to_string());
                m
            },
        }],
        relationships: Vec::new(),
        salience: 1.0,
        emotional_weight: 0.0,
        source_context: HashMap::new(),
    };

    let fragments = distill_event(&event);

    let has_personal_fact = fragments
        .iter()
        .any(|f| matches!(f.fragment_type, FragmentType::PersonalFact));
    assert!(has_personal_fact, "Should create PersonalFact fragment");
}

#[test]
fn test_fragment_extraction_temporal_event() {
    let mut event = SemanticEvent {
        id: Uuid::new_v4(),
        timestamp: current_timestamp(),
        event_type: EventType::Conversation,
        atoms: vec![SemanticAtom {
            atom_type: AtomType::Time,
            content: {
                let mut m = HashMap::new();
                m.insert("time_expression".to_string(), "tomorrow".to_string());
                m.insert("frequency".to_string(), "once".to_string());
                m
            },
        }],
        relationships: Vec::new(),
        salience: 1.0,
        emotional_weight: 0.0,
        source_context: HashMap::new(),
    };

    let fragments = distill_event(&event);

    let has_temporal_event = fragments
        .iter()
        .any(|f| matches!(f.fragment_type, FragmentType::TemporalEvent));
    assert!(has_temporal_event, "Should create TemporalEvent fragment");
}

#[test]
fn test_fragment_extraction_social_relation() {
    let mut event = SemanticEvent {
        id: Uuid::new_v4(),
        timestamp: current_timestamp(),
        event_type: EventType::Conversation,
        atoms: vec![
            SemanticAtom {
                atom_type: AtomType::Person,
                content: {
                    let mut m = HashMap::new();
                    m.insert("name".to_string(), "Alice".to_string());
                    m
                },
            },
            SemanticAtom {
                atom_type: AtomType::Person,
                content: {
                    let mut m = HashMap::new();
                    m.insert("name".to_string(), "Bob".to_string());
                    m
                },
            },
        ],
        relationships: vec![Relationship {
            from_atom: 0,
            to_atom: 1,
            relation_type: RelationType::Knows,
            strength: 0.8,
        }],
        salience: 1.0,
        emotional_weight: 0.0,
        source_context: HashMap::new(),
    };

    let fragments = distill_event(&event);

    let has_social_relation = fragments
        .iter()
        .any(|f| matches!(f.fragment_type, FragmentType::SocialRelation));
    assert!(has_social_relation, "Should create SocialRelation fragment");
}

#[test]
fn test_enhanced_ingestion() {
    let text = "My name is Nolan, I like coffee, and I work in San Francisco";

    let event = ingest_conversation_enhanced(text);

    assert!(!event.atoms.is_empty(), "Should extract atoms");
    assert!(
        !event.relationships.is_empty(),
        "Should extract relationships"
    );

    let atom_types: Vec<_> = event.atoms.iter().map(|a| &a.atom_type).collect();
    let has_person = atom_types.iter().any(|t| matches!(**t, AtomType::Person));
    let has_location = atom_types.iter().any(|t| matches!(**t, AtomType::Location));

    assert!(has_person, "Should extract Person atoms");
    assert!(has_location, "Should extract Location atoms");
}

#[test]
fn test_memory_indexing_new_fragments() {
    let mut memory = MemoryGraph::new();

    let fragment = MFragment {
        id: Uuid::new_v4(),
        fragment_type: FragmentType::PersonalFact,
        content: FragmentContent::PersonalFact {
            person: "Nolan".to_string(),
            fact_type: "name".to_string(),
            value: "Nolan".to_string(),
            confidence: 0.9,
        },
        confidence: 0.9,
        salience: 1.0,
        emotional_tag: 0.0,
        reinforcement_count: 0,
        last_activated: 0.0,
        activation_history: Vec::new(),
        created_at: current_timestamp(),
        decay_rate: 0.001,
    };

    memory.insert_fragment(fragment, Vec::new());

    assert!(memory.activation_index.by_keyword.contains_key("Nolan"));
    assert!(memory.activation_index.by_keyword.contains_key("nolan"));
    assert!(memory.activation_index.by_keyword.contains_key("name"));
}

#[test]
fn test_extraction_stats() {
    let mut stats = ExtractionStats::new();

    stats.record_pattern_match("name_intro_my_name_is");
    stats.record_atom(&AtomType::Person);
    stats.record_fragment(&FragmentType::PersonalFact);
    stats.record_confidence(0.9);

    assert_eq!(stats.total_extractions, 1);
    assert_eq!(
        stats.pattern_match_counts.get("name_intro_my_name_is"),
        Some(&1)
    );
    assert_eq!(stats.atom_type_counts.get("Person"), Some(&1));
    assert_eq!(stats.fragment_type_counts.get("PersonalFact"), Some(&1));
    assert_eq!(stats.average_confidence(), 0.9);
}
