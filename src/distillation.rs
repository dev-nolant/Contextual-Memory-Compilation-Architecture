// Copyright (c) 2026 Nolan Taft
use crate::types::*;
use std::collections::HashMap;
use uuid::Uuid;

pub fn distill_event(event: &SemanticEvent) -> Vec<MFragment> {
    let mut fragments = Vec::new();
    let timestamp = current_timestamp();
    
    
    for atom in &event.atoms {
                fragments.push(MFragment {
                    id: Uuid::new_v4(),
            fragment_type: FragmentType::SemanticAtom,
            content: FragmentContent::SemanticAtom {
                atom_type: atom.atom_type.clone(),
                content: atom.content.clone(),  
                atom_id: None,  
                    },
                    confidence: event.salience,
                    salience: event.salience,
                    emotional_tag: event.emotional_weight,
                    reinforcement_count: 0,
                    last_activated: 0.0,
                    activation_history: Vec::new(),
                    created_at: timestamp,
                    decay_rate: 0.001,
                });
            }
    
    fragments
}



pub fn create_edges_from_relationships(
    event: &SemanticEvent,
    fragments: &[MFragment],
) -> Vec<Edge> {
    let mut edges = Vec::new();
    let timestamp = current_timestamp();
    
    
    
    let atom_to_fragment: HashMap<usize, Uuid> = fragments
        .iter()
        .enumerate()
        .filter_map(|(idx, frag)| {
            if matches!(frag.fragment_type, FragmentType::SemanticAtom) {
                Some((idx, frag.id))
            } else {
                None
            }
        })
        .collect();
    
    
    for relationship in &event.relationships {
        if let (Some(&from_frag_id), Some(&to_frag_id)) = (
            atom_to_fragment.get(&relationship.from_atom),
            atom_to_fragment.get(&relationship.to_atom),
        ) {
            
            let edge_type = match relationship.relation_type {
                RelationType::Causal | RelationType::Causes => EdgeType::Causal,
                RelationType::Temporal | RelationType::Before | RelationType::After | RelationType::During | RelationType::Simultaneous => EdgeType::Temporal,
                RelationType::Spatial | RelationType::LocatedAt | RelationType::OccursAt => EdgeType::Semantic,
                _ => EdgeType::Semantic,  
            };
            
            edges.push(Edge {
                from_fragment: from_frag_id,
                to_fragment: to_frag_id,
                edge_type,
                strength: relationship.strength,
                last_reinforced: timestamp,
            created_at: timestamp,
            decay_rate: 0.001,
        });
    }
    }
    
    edges
}



fn extract_personal_facts(event: &SemanticEvent) -> Vec<MFragment> {
    let mut fragments = Vec::new();
    let timestamp = current_timestamp();
    
    
    let person_atoms: Vec<(usize, &SemanticAtom)> = event.atoms.iter()
        .enumerate()
        .filter(|(_, a)| matches!(a.atom_type, AtomType::Person))
        .collect();
    
    
    let default_person = person_atoms.first()
        .and_then(|(_, a)| a.content.get("name").or_else(|| a.content.get("key")))
        .cloned()
        .unwrap_or_else(|| "user".to_string());
    
    for (atom_idx, atom) in event.atoms.iter().enumerate() {
        
        if let AtomType::Person = atom.atom_type {
            if let Some(name) = atom.content.get("name") {
                
                let fact_type = atom.content.get("fact_type")
                    .cloned()
                    .unwrap_or_else(|| "name".to_string());
                
                
                let value = atom.content.get("value")
                    .or_else(|| atom.content.get("fact"))
                    .or_else(|| atom.content.get("preference"))
                    .or_else(|| atom.content.get("name")) 
                    .cloned()
                    .unwrap_or_else(|| "unknown".to_string());
                
                fragments.push(MFragment {
                    id: Uuid::new_v4(),
                    fragment_type: FragmentType::PersonalFact,
                    content: FragmentContent::PersonalFact {
                        person: name.clone(),
                        fact_type: fact_type.clone(),
                        value: value.clone(),
                        confidence: event.salience,
                    },
                    confidence: event.salience,
                    salience: event.salience,
                    emotional_tag: event.emotional_weight,
                    reinforcement_count: 0,
                    last_activated: 0.0,
                    activation_history: Vec::new(),
                    created_at: timestamp,
                    decay_rate: 0.001,
                });
            }
            
            
            for (key, value) in &atom.content {
                if key != "name" && key != "key" && !value.is_empty() {
                    let person_name = atom.content.get("name")
                        .or_else(|| atom.content.get("key"))
                        .cloned()
                        .unwrap_or_else(|| default_person.clone());
                    
                    fragments.push(MFragment {
                        id: Uuid::new_v4(),
                        fragment_type: FragmentType::PersonalFact,
                        content: FragmentContent::PersonalFact {
                            person: person_name,
                            fact_type: key.clone(),
                            value: value.clone(),
                            confidence: event.salience,
                        },
                        confidence: event.salience,
                        salience: event.salience,
                        emotional_tag: event.emotional_weight,
                        reinforcement_count: 0,
                        last_activated: 0.0,
                        activation_history: Vec::new(),
                        created_at: timestamp,
                        decay_rate: 0.001,
                    });
                }
            }
        }
        
        
        if let AtomType::Location = atom.atom_type {
            
            let person_name = event.relationships.iter()
                .find_map(|rel| {
                    
                    if let Some(person_atom) = person_atoms.first() {
                        if (rel.from_atom == person_atom.0 && rel.to_atom == atom_idx) ||
                           (rel.from_atom == atom_idx && rel.to_atom == person_atom.0) {
                            return person_atom.1.content.get("name")
                                .or_else(|| person_atom.1.content.get("key"))
                                .cloned();
                        }
                    }
                    None
                })
                
                .or_else(|| {
                    
                    let has_possessive = atom.content.values().any(|v| {
                        let v_lower = v.to_lowercase();
                        v_lower.contains("my ") || v_lower.contains("my_") || 
                        v_lower.contains("i ") || v_lower.contains("user")
                    });
                    if has_possessive {
                        Some(default_person.clone())
                    } else {
                        None
                    }
                })
                
                .or_else(|| {
                    if !person_atoms.is_empty() {
                        Some(default_person.clone())
                    } else {
                        None
                    }
                })
                .unwrap_or_else(|| default_person.clone());
            
            
            let location_value = atom.content.get("name")
                .or_else(|| atom.content.get("address"))
                .or_else(|| atom.content.get("location"))
                .or_else(|| atom.content.get("value"))
                .or_else(|| atom.content.values().next())
                .cloned()
                .unwrap_or_else(|| "unknown".to_string());
            
            
            let fact_type = atom.content.get("fact_type")
                .cloned()
                .or_else(|| {
                    
                    if atom.content.contains_key("address") { Some("address".to_string()) }
                    else if atom.content.contains_key("location") { Some("location".to_string()) }
                    else if location_value.contains("street") || location_value.contains("avenue") || 
                            location_value.contains("road") || location_value.contains("drive") ||
                            location_value.matches(char::is_numeric).count() > 0 {
                        Some("address".to_string())
                    }
                    else { Some("location".to_string()) }
                })
                .unwrap_or_else(|| "location".to_string());
            
            fragments.push(MFragment {
                id: Uuid::new_v4(),
                fragment_type: FragmentType::PersonalFact,
                content: FragmentContent::PersonalFact {
                    person: person_name,
                    fact_type,
                    value: location_value,
                    confidence: event.salience,
                },
                confidence: event.salience,
                salience: event.salience,
                emotional_tag: event.emotional_weight,
                reinforcement_count: 0,
                last_activated: 0.0,
                activation_history: Vec::new(),
                created_at: timestamp,
                decay_rate: 0.001,
            });
        }
        
        
        if matches!(atom.atom_type, AtomType::Property | AtomType::Attribute) {
            
            let person_name = event.relationships.iter()
                .find_map(|rel| {
                    if let Some(person_atom) = person_atoms.first() {
                        if (rel.from_atom == person_atom.0 && rel.to_atom == atom_idx) ||
                           (rel.from_atom == atom_idx && rel.to_atom == person_atom.0) {
                            return person_atom.1.content.get("name")
                                .or_else(|| person_atom.1.content.get("key"))
                                .cloned();
                        }
                    }
                    None
                })
                
                .or_else(|| {
                    if !person_atoms.is_empty() {
                        Some(default_person.clone())
                    } else {
                        None
                    }
                })
                .unwrap_or_else(|| default_person.clone());
            
            
            for (key, value) in &atom.content {
                if !value.is_empty() {
                    fragments.push(MFragment {
                        id: Uuid::new_v4(),
                        fragment_type: FragmentType::PersonalFact,
                        content: FragmentContent::PersonalFact {
                            person: person_name.clone(),
                            fact_type: key.clone(),
                            value: value.clone(),
                            confidence: event.salience,
                        },
                        confidence: event.salience,
                        salience: event.salience,
                        emotional_tag: event.emotional_weight,
                        reinforcement_count: 0,
                        last_activated: 0.0,
                        activation_history: Vec::new(),
                        created_at: timestamp,
                        decay_rate: 0.001,
                    });
                }
            }
        }
        
        
        if let AtomType::Action = atom.atom_type {
            
            
            let action_text = atom.content.get("key")
                .or_else(|| atom.content.get("action"))
                .map(|s| s.to_lowercase());
            
            
            
            let is_preference_action = atom.content.values().any(|v| {
                let v_lower = v.to_lowercase();
                
                v_lower.contains("like") || v_lower.contains("love") || v_lower.contains("prefer") ||
                v_lower.contains("enjoy") || v_lower.contains("adore") || v_lower.contains("favorite") ||
                v_lower.contains("fond") || v_lower.contains("into")
            }) || action_text.as_ref().map(|s| {
                s.contains("like") || s.contains("love") || s.contains("prefer") ||
                s.contains("enjoy") || s.contains("adore") || s.contains("favorite")
            }).unwrap_or(false);
            
            if is_preference_action {
                
                let value = atom.content.get("value")
                    .or_else(|| atom.content.get("object"))
                    .or_else(|| atom.content.get("target"))
                    .or_else(|| {
                        
                        event.atoms.iter()
                            .find_map(|a| {
                                if let AtomType::Object = a.atom_type {
                                    a.content.get("key").or_else(|| a.content.get("name"))
                                } else {
                                    None
                                }
                            })
                    })
                    .cloned();
                
                if let Some(preference_value) = value {
                    
                    let person_name = event.atoms.iter()
                        .find_map(|a| {
                            if let AtomType::Person = a.atom_type {
                                a.content.get("name")
                                    .or_else(|| a.content.get("key"))
                                    .cloned()
                            } else {
                                None
                            }
                        })
                        .unwrap_or_else(|| "user".to_string());
                    
                    fragments.push(MFragment {
                        id: Uuid::new_v4(),
                        fragment_type: FragmentType::PersonalFact,
                        content: FragmentContent::PersonalFact {
                            person: person_name,
                            fact_type: "preference".to_string(),
                            value: preference_value,
                            confidence: event.salience,
                        },
                        confidence: event.salience,
                        salience: event.salience,
                        emotional_tag: event.emotional_weight,
                        reinforcement_count: 0,
                        last_activated: 0.0,
                        activation_history: Vec::new(),
                        created_at: timestamp,
                        decay_rate: 0.001,
                    });
                }
            }
        }
        
        
        if let AtomType::Person = atom.atom_type {
            
            if let Some(preference) = atom.content.get("preference") {
                let person_name = atom.content.get("name")
                    .or_else(|| atom.content.get("key"))
                    .cloned()
                    .unwrap_or_else(|| "user".to_string());
                
                fragments.push(MFragment {
                    id: Uuid::new_v4(),
                    fragment_type: FragmentType::PersonalFact,
                    content: FragmentContent::PersonalFact {
                        person: person_name,
                        fact_type: "preference".to_string(),
                        value: preference.clone(),
                        confidence: event.salience,
                    },
                    confidence: event.salience,
                    salience: event.salience,
                    emotional_tag: event.emotional_weight,
                    reinforcement_count: 0,
                    last_activated: 0.0,
                    activation_history: Vec::new(),
                    created_at: timestamp,
                    decay_rate: 0.001,
                });
            }
        }
        
        
        
        if let AtomType::Entity = atom.atom_type {
            
            if atom.content.contains_key("ownership_marker") {
                eprintln!("\n🔍 [DEBUG] Distillation: Entity with ownership_marker found");
                eprintln!("   Atom content: {:?}", atom.content);
                
                
                let person_name = person_atoms.first()
                    .and_then(|(_, a)| a.content.get("name").or_else(|| a.content.get("key")))
                    .cloned()
                    .unwrap_or_else(|| default_person.clone());
                eprintln!("   Person name: {}", person_name);
                
                
                let fact_type = atom.content.get("key")
                    .cloned()
                    .unwrap_or_else(|| "unknown".to_string());
                eprintln!("   Fact type: {}", fact_type);
                
                
                
                let value = atom.content.get(&fact_type)
                    .or_else(|| atom.content.get("value"))
                    .or_else(|| {
                        atom.content.values().find(|v| {
                            **v != fact_type && **v != "key" && !v.is_empty()
                        })
                    })
                    .cloned();
                
                eprintln!("   Value found: {:?}", value);
                eprintln!("   All content keys: {:?}", atom.content.keys().collect::<Vec<_>>());
                eprintln!("   All content values: {:?}", atom.content.values().collect::<Vec<_>>());
                
                if let Some(value_str) = value {
                    if !value_str.is_empty() && value_str != "unknown" && value_str != fact_type {
                        eprintln!("   ✅ Creating PersonalFact: person={}, fact_type={}, value={}", 
                            person_name, fact_type, value_str);
                        fragments.push(MFragment {
                            id: Uuid::new_v4(),
                            fragment_type: FragmentType::PersonalFact,
                            content: FragmentContent::PersonalFact {
                                person: person_name.clone(),
                                fact_type: fact_type.clone(),
                                value: value_str.clone(),
                                confidence: event.salience,
                            },
                            confidence: event.salience,
                            salience: event.salience,
                            emotional_tag: event.emotional_weight,
                            reinforcement_count: 0,
                            last_activated: 0.0,
                            activation_history: Vec::new(),
                            created_at: timestamp,
                            decay_rate: 0.001,
                        });
                    } else {
                        eprintln!("   ❌ Value filtered out: empty={}, unknown={}, equals_fact_type={}", 
                            value_str.is_empty(), value_str == "unknown", value_str == fact_type);
                    }
                } else {
                    eprintln!("   ❌ No value found in entity content");
                }
            }
        }
    }
    
    fragments
}


fn extract_temporal_events(event: &SemanticEvent) -> Vec<MFragment> {
    let mut fragments = Vec::new();
    let timestamp = current_timestamp();
    
    for atom in &event.atoms {
        if let AtomType::Time = atom.atom_type {
            if let Some(time_expr) = atom.content.get("time_expression")
                .or_else(|| atom.content.get("duration"))
                .or_else(|| atom.content.get("frequency"))
            {
                fragments.push(MFragment {
                    id: Uuid::new_v4(),
                    fragment_type: FragmentType::TemporalEvent,
                    content: FragmentContent::TemporalEvent {
                        event: "temporal_event".to_string(),
                        time_expression: time_expr.clone(),
                        duration: atom.content.get("duration").cloned(),
                        frequency: atom.content.get("frequency").cloned(),
                        confidence: event.salience,
                    },
                    confidence: event.salience,
                    salience: event.salience,
                    emotional_tag: event.emotional_weight,
                    reinforcement_count: 0,
                    last_activated: 0.0,
                    activation_history: Vec::new(),
                    created_at: timestamp,
                    decay_rate: 0.001,
                });
            }
        }
    }
    
    fragments
}


fn extract_spatial_relations(event: &SemanticEvent) -> Vec<MFragment> {
    let mut fragments = Vec::new();
    let timestamp = current_timestamp();
    
    for atom in &event.atoms {
        if let AtomType::Location = atom.atom_type {
            if let Some(location_keyword) = atom.content.get("location_keyword")
                .or_else(|| atom.content.get("direction"))
            {
                
                let entity = event.atoms.iter()
                    .find(|a| matches!(a.atom_type, AtomType::Entity | AtomType::Person | AtomType::Object))
                    .and_then(|a| a.content.get("name"))
                    .cloned()
                    .unwrap_or_else(|| "unknown".to_string());
                
                fragments.push(MFragment {
                    id: Uuid::new_v4(),
                    fragment_type: FragmentType::SpatialRelation,
                    content: FragmentContent::SpatialRelation {
                        entity,
                        location: location_keyword.clone(),
                        relation_type: "located_at".to_string(),
                        distance: None,
                        confidence: event.salience,
                    },
                    confidence: event.salience,
                    salience: event.salience,
                    emotional_tag: event.emotional_weight,
                    reinforcement_count: 0,
                    last_activated: 0.0,
                    activation_history: Vec::new(),
                    created_at: timestamp,
                    decay_rate: 0.001,
                });
            }
        }
    }
    
    fragments
}


fn extract_quantitative_facts(event: &SemanticEvent) -> Vec<MFragment> {
    let mut fragments = Vec::new();
    let timestamp = current_timestamp();
    
    for atom in &event.atoms {
        if let AtomType::Quantity = atom.atom_type {
            if let Some(comparison) = atom.content.get("comparison") {
                let quantity = atom.content.get("quantity")
                    .and_then(|q| q.parse::<f64>().ok())
                    .unwrap_or(0.0);
                
                let entity = event.atoms.iter()
                    .find(|a| matches!(a.atom_type, AtomType::Entity | AtomType::Object))
                    .and_then(|a| a.content.get("name"))
                    .cloned()
                    .unwrap_or_else(|| "unknown".to_string());
                
                fragments.push(MFragment {
                    id: Uuid::new_v4(),
                    fragment_type: FragmentType::QuantitativeFact,
                    content: FragmentContent::QuantitativeFact {
                        entity,
                        quantity,
                        unit: atom.content.get("unit").cloned(),
                        comparison: Some(comparison.clone()),
                        reference: None,
                        confidence: event.salience,
                    },
                    confidence: event.salience,
                    salience: event.salience,
                    emotional_tag: event.emotional_weight,
                    reinforcement_count: 0,
                    last_activated: 0.0,
                    activation_history: Vec::new(),
                    created_at: timestamp,
                    decay_rate: 0.001,
                });
            }
        }
    }
    
    fragments
}


fn extract_hierarchical_relations(event: &SemanticEvent) -> Vec<MFragment> {
    let mut fragments = Vec::new();
    let timestamp = current_timestamp();
    
    for relationship in &event.relationships {
        if matches!(relationship.relation_type, RelationType::PartOf | RelationType::Hierarchical) {
            if relationship.from_atom < event.atoms.len() && relationship.to_atom < event.atoms.len() {
                let parent_atom = &event.atoms[relationship.from_atom];
                let child_atom = &event.atoms[relationship.to_atom];
                
                let parent = parent_atom.content.get("name")
                    .or_else(|| parent_atom.content.get("hierarchical_marker"))
                    .cloned()
                    .unwrap_or_else(|| "unknown".to_string());
                
                let child = child_atom.content.get("name")
                    .or_else(|| child_atom.content.get("hierarchical_marker"))
                    .cloned()
                    .unwrap_or_else(|| "unknown".to_string());
                
                fragments.push(MFragment {
                    id: Uuid::new_v4(),
                    fragment_type: FragmentType::HierarchicalRelation,
                    content: FragmentContent::HierarchicalRelation {
                        parent,
                        child,
                        relation_type: format!("{:?}", relationship.relation_type),
                        level: None,
                        confidence: relationship.strength * event.salience,
                    },
                    confidence: relationship.strength * event.salience,
                    salience: event.salience,
                    emotional_tag: event.emotional_weight,
                    reinforcement_count: 0,
                    last_activated: 0.0,
                    activation_history: Vec::new(),
                    created_at: timestamp,
                    decay_rate: 0.001,
                });
            }
        }
    }
    
    fragments
}


fn extract_social_relations(event: &SemanticEvent) -> Vec<MFragment> {
    let mut fragments = Vec::new();
    let timestamp = current_timestamp();
    
    for relationship in &event.relationships {
        if matches!(relationship.relation_type, RelationType::Knows | RelationType::ParticipatesIn | RelationType::RelatedTo) {
            if relationship.from_atom < event.atoms.len() && relationship.to_atom < event.atoms.len() {
                let person1_atom = &event.atoms[relationship.from_atom];
                let person2_atom = &event.atoms[relationship.to_atom];
                
                if matches!(person1_atom.atom_type, AtomType::Person) && matches!(person2_atom.atom_type, AtomType::Person) {
                    let person1 = person1_atom.content.get("name")
                        .or_else(|| person1_atom.content.get("social_marker"))
                        .cloned()
                        .unwrap_or_else(|| "person1".to_string());
                    
                    let person2 = person2_atom.content.get("name")
                        .or_else(|| person2_atom.content.get("social_marker"))
                        .cloned()
                        .unwrap_or_else(|| "person2".to_string());
                    
                    fragments.push(MFragment {
                        id: Uuid::new_v4(),
                        fragment_type: FragmentType::SocialRelation,
                        content: FragmentContent::SocialRelation {
                            person1,
                            person2,
                            relation_type: format!("{:?}", relationship.relation_type),
                            strength: relationship.strength,
                            context: None,
                            confidence: relationship.strength * event.salience,
                        },
                        confidence: relationship.strength * event.salience,
                        salience: event.salience,
                        emotional_tag: event.emotional_weight,
                        reinforcement_count: 0,
                        last_activated: 0.0,
                        activation_history: Vec::new(),
                        created_at: timestamp,
                        decay_rate: 0.001,
                    });
                }
            }
        }
    }
    
    fragments
}


fn extract_ownership_relations(event: &SemanticEvent) -> Vec<MFragment> {
    let mut fragments = Vec::new();
    let timestamp = current_timestamp();
    
    for relationship in &event.relationships {
        if matches!(relationship.relation_type, RelationType::Ownership) {
            if relationship.from_atom < event.atoms.len() && relationship.to_atom < event.atoms.len() {
                let owner_atom = &event.atoms[relationship.from_atom];
                let owned_atom = &event.atoms[relationship.to_atom];
                
                let owner = owner_atom.content.get("name")
                    .or_else(|| owner_atom.content.get("ownership_marker"))
                    .cloned()
                    .unwrap_or_else(|| "unknown".to_string());
                
                let owned = owned_atom.content.get("name")
                    .or_else(|| owned_atom.content.get("ownership_marker"))
                    .cloned()
                    .unwrap_or_else(|| "unknown".to_string());
                
                fragments.push(MFragment {
                    id: Uuid::new_v4(),
                    fragment_type: FragmentType::OwnershipRelation,
                    content: FragmentContent::OwnershipRelation {
                        owner,
                        owned,
                        relation_type: "owns".to_string(),
                        confidence: relationship.strength * event.salience,
                    },
                    confidence: relationship.strength * event.salience,
                    salience: event.salience,
                    emotional_tag: event.emotional_weight,
                    reinforcement_count: 0,
                    last_activated: 0.0,
                    activation_history: Vec::new(),
                    created_at: timestamp,
                    decay_rate: 0.001,
                });
            }
        }
    }
    
    fragments
}


fn extract_state_transitions(event: &SemanticEvent) -> Vec<MFragment> {
    let mut fragments = Vec::new();
    let timestamp = current_timestamp();
    
    for atom in &event.atoms {
        if let AtomType::State = atom.atom_type {
            if let Some(state_marker) = atom.content.get("state_marker") {
                let entity = event.atoms.iter()
                    .find(|a| matches!(a.atom_type, AtomType::Entity | AtomType::Object))
                    .and_then(|a| a.content.get("name"))
                    .cloned()
                    .unwrap_or_else(|| "unknown".to_string());
                
                fragments.push(MFragment {
                    id: Uuid::new_v4(),
                    fragment_type: FragmentType::StateTransition,
                    content: FragmentContent::StateTransition {
                        entity,
                        from_state: "unknown".to_string(),
                        to_state: state_marker.clone(),
                        condition: None,
                        timestamp: Some(event.timestamp),
                        confidence: event.salience,
                    },
                    confidence: event.salience,
                    salience: event.salience,
                    emotional_tag: event.emotional_weight,
                    reinforcement_count: 0,
                    last_activated: 0.0,
                    activation_history: Vec::new(),
                    created_at: timestamp,
                    decay_rate: 0.001,
                });
            }
        }
    }
    
    fragments
}


fn extract_capabilities(event: &SemanticEvent) -> Vec<MFragment> {
    let mut fragments = Vec::new();
    let timestamp = current_timestamp();
    
    for atom in &event.atoms {
        if let AtomType::Resource = atom.atom_type {
            if let Some(capability) = atom.content.get("capability")
                .or_else(|| atom.content.get("name"))
            {
                let entity = event.atoms.iter()
                    .find(|a| matches!(a.atom_type, AtomType::Person | AtomType::Entity))
                    .and_then(|a| a.content.get("name"))
                    .cloned()
                    .unwrap_or_else(|| "unknown".to_string());
                
                fragments.push(MFragment {
                    id: Uuid::new_v4(),
                    fragment_type: FragmentType::Capability,
                    content: FragmentContent::Capability {
                        entity,
                        capability: capability.clone(),
                        level: None,
                        context: None,
                        confidence: event.salience,
                    },
                    confidence: event.salience,
                    salience: event.salience,
                    emotional_tag: event.emotional_weight,
                    reinforcement_count: 0,
                    last_activated: 0.0,
                    activation_history: Vec::new(),
                    created_at: timestamp,
                    decay_rate: 0.001,
                });
            }
        }
    }
    
    fragments
}


fn extract_beliefs(event: &SemanticEvent) -> Vec<MFragment> {
    let mut fragments = Vec::new();
    let timestamp = current_timestamp();
    
    for atom in &event.atoms {
        if let AtomType::Concept = atom.atom_type {
            if let Some(belief) = atom.content.get("belief")
                .or_else(|| atom.content.get("name"))
            {
                let entity = event.atoms.iter()
                    .find(|a| matches!(a.atom_type, AtomType::Person))
                    .and_then(|a| a.content.get("name"))
                    .cloned()
                    .unwrap_or_else(|| "unknown".to_string());
                
                fragments.push(MFragment {
                    id: Uuid::new_v4(),
                    fragment_type: FragmentType::Belief,
                    content: FragmentContent::Belief {
                        entity,
                        belief: belief.clone(),
                        confidence_level: event.salience,
                        evidence: None,
                        context: None,
                    },
                    confidence: event.salience,
                    salience: event.salience,
                    emotional_tag: event.emotional_weight,
                    reinforcement_count: 0,
                    last_activated: 0.0,
                    activation_history: Vec::new(),
                    created_at: timestamp,
                    decay_rate: 0.001,
                });
            }
        }
    }
    
    fragments
}

fn find_related_entity(entity: &str, property: &str, atoms: &[SemanticAtom]) -> Option<String> {
    for atom in atoms {
        if let AtomType::Entity = atom.atom_type {
            if let Some(name) = atom.content.get("name") {
                if name != entity && name.contains(property) {
                    return Some(name.clone());
                }
            }
        }
    }
    None
}

fn extract_goal_strategy(event: &SemanticEvent) -> Option<(String, String)> {
    let text = format!("{:?}", event.atoms);
    if text.contains("debug") || text.contains("fix") {
        Some((
            "debug_error".to_string(),
            "check_cause_then_fix".to_string(),
        ))
    } else {
        None
    }
}

fn extract_constraint(event: &SemanticEvent) -> Option<(String, String)> {
    let text = format!("{:?}", event.atoms);
    if text.contains("must") || text.contains("require") {
        Some((
            "must_be_valid".to_string(),
            "general".to_string(),
        ))
    } else {
        None
    }
}
