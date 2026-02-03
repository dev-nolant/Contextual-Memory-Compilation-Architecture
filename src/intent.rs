// Copyright (c) 2026 Nolan Taft
use crate::types::*;
use serde::{Serialize, Deserialize};
use std::collections::HashSet;
use uuid::Uuid;



pub struct IntentClassifier;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Intent {
    
    pub pattern: String,
    
    pub atom_types: Vec<String>,
    
    pub content_patterns: Vec<String>,
    
    pub relationship_patterns: Vec<String>,
    
    pub confidence: f64,
    
    pub occurrence_count: usize,
}

impl IntentClassifier {
    
    
    pub fn classify_intent(semantic_event: &SemanticEvent) -> Intent {
        Self::classify_intent_with_text(semantic_event, "")
    }
    
    
    pub fn classify_intent_with_text(semantic_event: &SemanticEvent, original_text: &str) -> Intent {
        
        let atom_types: Vec<String> = semantic_event.atoms.iter()
            .map(|a| format!("{:?}", a.atom_type))
            .collect();
        
        
        let mut content_keys = HashSet::new();
        for atom in &semantic_event.atoms {
            for key in atom.content.keys() {
                content_keys.insert(key.clone());
            }
        }
        let content_patterns: Vec<String> = content_keys.into_iter().collect();
        
        
        let relationship_patterns: Vec<String> = semantic_event.relationships.iter()
            .map(|r| format!("{:?}", r.relation_type))
            .collect();
        
        
        let pattern = Self::derive_intent_pattern(
            &semantic_event.atoms,
            &semantic_event.relationships,
            original_text
        );
        
        Intent {
            pattern,
            atom_types,
            content_patterns,
            relationship_patterns,
            confidence: semantic_event.salience,
            occurrence_count: 1,
        }
    }
    
    
    
    fn derive_intent_pattern(
        atoms: &[SemanticAtom],
        relationships: &[Relationship],
        original_text: &str,
    ) -> String {
        
        
        let question_starters = [
            "what", "who", "where", "when", "why", "how", "which", "whose",
            "whats", "what's", "whos", "who's", "wheres", "where's",
            "whens", "when's", "whys", "why's", "hows", "how's",
            "is", "are", "was", "were", "do", "does", "did", "can", "could",
            "will", "would", "should", "may", "might", "must", "have", "has", "had"
        ];
        
        let text_lower = original_text.trim().to_lowercase();
        let has_question_starter = question_starters.iter().any(|starter| {
            text_lower.starts_with(starter) || 
            text_lower.starts_with(&format!("{} ", starter))
        });
        
        
        let has_incomplete = atoms.iter().any(|a| {
            a.content.values().any(|v| v.is_empty() || v == "unknown")
        });
        
        
        let has_question = atoms.iter().any(|a| {
            a.content.contains_key("question")
        }) || original_text.trim().ends_with('?');
        
        
        
        let has_interrogative_action = Self::detect_interrogative_patterns(atoms, relationships);
        
        
        let has_person = atoms.iter().any(|a| matches!(a.atom_type, AtomType::Person));
        let has_location = atoms.iter().any(|a| matches!(a.atom_type, AtomType::Location));
        let has_action = atoms.iter().any(|a| matches!(a.atom_type, AtomType::Action));
        
        
        let has_requires = relationships.iter().any(|r| matches!(r.relation_type, RelationType::Requires));
        let has_related = relationships.iter().any(|r| matches!(r.relation_type, RelationType::RelatedTo));
        
        
        
        
        
        
        
        
        let word_count = original_text.trim().split_whitespace().count();
        let has_only_person_atoms = !atoms.is_empty() && atoms.iter().all(|a| matches!(a.atom_type, AtomType::Person));
        let has_generic_keys_only = atoms.iter().all(|a| {
            a.content.len() == 1 && 
            a.content.contains_key("key") &&
            a.content.get("key").map(|k| k.len() <= 10).unwrap_or(false) 
        });
        
        
        
        let has_only_structural_relationships = relationships.is_empty() || 
            relationships.iter().all(|r| {
                
                let is_structural = matches!(r.relation_type, 
                    RelationType::RelatedTo | 
                    RelationType::Semantic |
                    RelationType::SimilarTo
                );
                
                let both_person = r.from_atom < atoms.len() && r.to_atom < atoms.len() &&
                    matches!(atoms[r.from_atom].atom_type, AtomType::Person) &&
                    matches!(atoms[r.to_atom].atom_type, AtomType::Person);
                is_structural && both_person
            });
        let has_no_meaningful_content = !has_action && !has_location && 
                                       atoms.iter().all(|a| {
                                           matches!(a.atom_type, AtomType::Person) || 
                                           matches!(a.atom_type, AtomType::Concept)
                                       });
        
        
        
        
        let is_greeting = word_count <= 3 && 
                         has_only_person_atoms && 
                         has_generic_keys_only && 
                         has_no_meaningful_content &&
                         !has_question_starter && 
                         
                         
                         (word_count <= 2 || has_only_structural_relationships || relationships.is_empty());
        
        
        
        if is_greeting {
            return "greeting".to_string();
        }
        
        
        
        
        
        let has_complete_values = atoms.iter().any(|a| {
            
            
            matches!(a.atom_type, AtomType::Object | AtomType::Entity | AtomType::Concept) &&
            a.content.values().any(|v| {
                !v.is_empty() && 
                v != "unknown" && 
                v != "key" && 
                v.len() > 2 &&
                
                !(a.content.len() == 1 && a.content.contains_key("key") && v == a.content.get("key").unwrap())
            })
        });
        
        
        
        
        
        
        
        
        
        
        let has_person_with_ownership = atoms.iter().any(|a| {
            matches!(a.atom_type, AtomType::Person) &&
            
            relationships.iter().any(|rel| {
                let person_is_involved = (rel.from_atom < atoms.len() && 
                    matches!(atoms[rel.from_atom].atom_type, AtomType::Person)) ||
                    (rel.to_atom < atoms.len() && 
                    matches!(atoms[rel.to_atom].atom_type, AtomType::Person));
                
                if person_is_involved {
                    
                    let other_atom_idx = if rel.from_atom < atoms.len() && 
                        matches!(atoms[rel.from_atom].atom_type, AtomType::Person) {
                        rel.to_atom
                    } else {
                        rel.from_atom
                    };
                    
                    if other_atom_idx < atoms.len() {
                        let other_atom = &atoms[other_atom_idx];
                        return matches!(other_atom.atom_type, AtomType::Entity | AtomType::Object) &&
                               other_atom.content.contains_key("ownership_marker");
                    }
                }
                false
            })
        });
        
        let is_preference_statement = has_action && has_complete_values && !has_question_starter &&
            
            (relationships.iter().any(|rel| {
                let from_is_action = rel.from_atom < atoms.len() && 
                    matches!(atoms[rel.from_atom].atom_type, AtomType::Action);
                let to_is_complete = rel.to_atom < atoms.len() &&
                    matches!(atoms[rel.to_atom].atom_type, AtomType::Object | AtomType::Entity | AtomType::Concept) &&
                    atoms[rel.to_atom].content.values().any(|v| {
                        !v.is_empty() && v != "unknown" && v != "key" && v.len() > 2 &&
                        !(atoms[rel.to_atom].content.len() == 1 && 
                          atoms[rel.to_atom].content.contains_key("key") && 
                          v == atoms[rel.to_atom].content.get("key").unwrap())
                    });
                from_is_action && to_is_complete
            }) || has_person_with_ownership);
        
        
        
        
        
        if !is_preference_statement && (has_question_starter || has_incomplete || has_question || has_requires || has_interrogative_action) {
            
            if has_person {
                "query_person_info".to_string()
            } else if has_location {
                "query_location_info".to_string()
            } else {
                "query_general".to_string()
            }
        } else if has_action {
            
            if has_person {
                "store_person_info".to_string()
            } else if has_location {
                "store_location_info".to_string()
            } else {
                "store_general".to_string()
            }
        } else if has_related {
            
            "store_relationship".to_string()
        } else {
            
            Self::classify_from_content(atoms)
        }
    }
    
    
    
    fn detect_interrogative_patterns(
        atoms: &[SemanticAtom],
        relationships: &[Relationship],
    ) -> bool {
        
        
        
        
        
        
        let incomplete_atom_indices: std::collections::HashSet<usize> = atoms.iter()
            .enumerate()
            .filter(|(_, atom)| {
                
                atom.content.values().any(|v| v.is_empty() || v == "unknown" || v == "?")
            })
            .map(|(i, _)| i)
            .collect();
        
        
        for (i, atom) in atoms.iter().enumerate() {
            if matches!(atom.atom_type, AtomType::Action) {
                
                for rel in relationships {
                    
                    if rel.from_atom == i && incomplete_atom_indices.contains(&rel.to_atom) {
                        
                        if matches!(rel.relation_type, RelationType::Requires) || 
                           matches!(rel.relation_type, RelationType::RelatedTo) {
                            return true;
                        }
                    }
                    
                    if rel.to_atom == i && incomplete_atom_indices.contains(&rel.from_atom) {
                        if matches!(rel.relation_type, RelationType::Requires) {
                            return true;
                        }
                    }
                }
                
                
                let has_incomplete_content = atom.content.values().any(|v| {
                    v.is_empty() || v == "unknown" || v == "?"
                });
                
                
                
                let only_has_generic_key = atom.content.len() == 1 && 
                                          atom.content.contains_key("key");
                
                if has_incomplete_content || only_has_generic_key {
                    
                    
                    let has_complete_targets = atoms.iter().any(|a| {
                        !matches!(a.atom_type, AtomType::Action) &&
                        a.content.values().any(|v| !v.is_empty() && v != "unknown" && v != "?" && v != "key")
                    });
                    
                    if !has_complete_targets {
                        return true;
                    }
                }
            }
        }
        
        
        
        
        let has_reference_entities = atoms.iter().any(|a| {
            matches!(a.atom_type, AtomType::Entity | AtomType::Concept | AtomType::Property) &&
            
            
            (a.content.len() == 1 && a.content.contains_key("key")) ||
            a.content.values().all(|v| v.is_empty() || v == "unknown" || v == "key" || v.len() <= 2)
        });
        
        
        if has_reference_entities {
            let has_actions = atoms.iter().any(|a| matches!(a.atom_type, AtomType::Action));
            if has_actions {
                
                return true;
            }
        }
        
        
        
        let has_interrogative_action_content = atoms.iter().any(|a| {
            if matches!(a.atom_type, AtomType::Action) {
                
                
                let has_key_only = a.content.len() == 1 && a.content.contains_key("key");
                if has_key_only {
                    
                    let has_referenced_entities = atoms.iter().any(|other| {
                        !matches!(other.atom_type, AtomType::Action) &&
                        (matches!(other.atom_type, AtomType::Entity | AtomType::Concept | AtomType::Property) ||
                         matches!(other.atom_type, AtomType::Person))
                    });
                    return has_referenced_entities;
                }
            }
            false
        });
        
        if has_interrogative_action_content {
            return true;
        }
        
        
        
        
        let has_person = atoms.iter().any(|a| matches!(a.atom_type, AtomType::Person));
        
        let has_entity_without_value = atoms.iter().any(|a| {
            matches!(a.atom_type, AtomType::Entity | AtomType::Concept | AtomType::Property) &&
            
            (a.content.len() == 1 && a.content.contains_key("key"))
        });
        
        if has_person && has_entity_without_value {
            
            
            return true;
        }
        
        
        
        
        if has_person {
            let has_action_seeking_preferences = atoms.iter().any(|a| {
                matches!(a.atom_type, AtomType::Action) &&
                
                (a.content.len() == 1 && a.content.contains_key("key"))
            }) && (
                
                atoms.iter().any(|a| {
                    matches!(a.atom_type, AtomType::Object | AtomType::Entity) &&
                    
                    (a.content.len() == 1 && a.content.contains_key("key")) ||
                    a.content.values().all(|v| v.is_empty() || v == "unknown")
                }) ||
                
                atoms.iter().any(|a| {
                    matches!(a.atom_type, AtomType::Concept) &&
                    
                    (a.content.len() == 1 && a.content.contains_key("key"))
                }) ||
                
                
                relationships.iter().any(|rel| {
                    let person_involved = (rel.from_atom < atoms.len() && 
                        matches!(atoms[rel.from_atom].atom_type, AtomType::Person)) ||
                        (rel.to_atom < atoms.len() && 
                        matches!(atoms[rel.to_atom].atom_type, AtomType::Person));
                    let action_involved = (rel.from_atom < atoms.len() && 
                        matches!(atoms[rel.from_atom].atom_type, AtomType::Action)) ||
                        (rel.to_atom < atoms.len() && 
                        matches!(atoms[rel.to_atom].atom_type, AtomType::Action));
                    
                    person_involved && action_involved &&
                    
                    !atoms.iter().any(|a| {
                        matches!(a.atom_type, AtomType::Object | AtomType::Entity) &&
                        a.content.values().any(|v| !v.is_empty() && v != "unknown" && v != "key" && v.len() > 2)
                    })
                })
            );
            
            if has_action_seeking_preferences {
                
                return true;
            }
        }
        
        false
    }
    
    
    fn classify_from_content(atoms: &[SemanticAtom]) -> String {
        
        let mut all_keys = HashSet::new();
        for atom in atoms {
            for key in atom.content.keys() {
                all_keys.insert(key.clone());
            }
        }
        
        
        
        let has_complete_values = atoms.iter().any(|a| {
            a.content.values().any(|v| !v.is_empty() && v != "unknown")
        });
        
        if has_complete_values {
            "store_info".to_string()
        } else {
            "query_info".to_string()
        }
    }
    
    
    
    pub fn match_intent_to_memory(
        intent: &Intent,
        memory: &MemoryGraph,
    ) -> Vec<Uuid> {
        let mut candidates = HashSet::new();
        
        
        for pattern in &intent.content_patterns {
            if let Some(fragments) = memory.activation_index.by_keyword.get(pattern) {
                candidates.extend(fragments);
            }
            
            let pattern_lower = pattern.to_lowercase();
            if let Some(fragments) = memory.activation_index.by_keyword.get(&pattern_lower) {
                candidates.extend(fragments);
            }
        }
        
        
        
        
        candidates.into_iter().collect()
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentPattern {
    pub pattern: String,
    pub atom_type_signature: Vec<AtomType>,
    pub content_signature: Vec<String>,
    pub occurrence_count: usize,
    pub success_rate: f64,
    pub last_seen: f64,
}


