// Copyright (c) 2026 Nolan Taft

pub mod config;
pub mod extractors;
pub mod patterns;
pub mod stats;
pub mod utils;

pub use extractors::*;
pub use patterns::*;
pub use stats::*;
pub use utils::*;

use crate::types::*;
use std::collections::HashMap;
use uuid::Uuid;

pub fn ingest_conversation(text: &str) -> SemanticEvent {
    let mut atoms = Vec::new();
    let mut relationships = Vec::new();
    let mut emotional_weight = 0.0;

    let text_lower = text.to_lowercase();

    if text_lower.contains("error")
        || text_lower.contains("problem")
        || text_lower.contains("issue")
    {
        emotional_weight = 0.4;
    }
    if text_lower.contains("frustrated") || text_lower.contains("stuck") {
        emotional_weight = 0.6;
    }
    if text_lower.contains("success") || text_lower.contains("solved") {
        emotional_weight = -0.3;
    }

    let words: Vec<&str> = text.split_whitespace().collect();

    let mut entity_atoms = Vec::new();
    let mut action_atoms = Vec::new();
    let mut outcome_atoms = Vec::new();

    for (i, word) in words.iter().enumerate() {
        let word_clean = word.trim_matches(|c: char| !c.is_alphanumeric());

        if word_clean.len() < 2 {
            continue;
        }

        if is_entity(word_clean) {
            let mut content = HashMap::new();
            content.insert("name".to_string(), word_clean.to_string());
            if i + 1 < words.len() {
                content.insert("property".to_string(), words[i + 1].to_string());
            }
            atoms.push(SemanticAtom {
                atom_type: AtomType::Entity,
                content,
            });
            entity_atoms.push(atoms.len() - 1);
        }

        if is_action(word_clean) {
            let mut content = HashMap::new();
            content.insert("action".to_string(), word_clean.to_string());
            atoms.push(SemanticAtom {
                atom_type: AtomType::Action,
                content,
            });
            action_atoms.push(atoms.len() - 1);
        }

        if is_outcome(word_clean) {
            let mut content = HashMap::new();
            content.insert("outcome".to_string(), word_clean.to_string());
            atoms.push(SemanticAtom {
                atom_type: AtomType::Outcome,
                content,
            });
            outcome_atoms.push(atoms.len() - 1);
        }
    }

    for &action_idx in &action_atoms {
        for &outcome_idx in &outcome_atoms {
            relationships.push(Relationship {
                from_atom: action_idx,
                to_atom: outcome_idx,
                relation_type: RelationType::Causal,
                strength: 0.7,
            });
        }
    }

    for i in 0..atoms.len().saturating_sub(1) {
        relationships.push(Relationship {
            from_atom: i,
            to_atom: i + 1,
            relation_type: RelationType::Temporal,
            strength: 0.5,
        });
    }

    SemanticEvent {
        id: Uuid::new_v4(),
        timestamp: current_timestamp(),
        event_type: EventType::Conversation,
        atoms,
        relationships,
        salience: 1.0,
        emotional_weight,
        source_context: HashMap::new(),
    }
}

fn is_entity(word: &str) -> bool {
    let entities = [
        "http", "api", "url", "server", "client", "request", "response", "python", "code",
        "library", "function", "variable", "error", "endpoint", "method", "status", "code",
    ];
    entities
        .iter()
        .any(|&e| word.contains(e) || e.contains(word))
}

fn is_action(word: &str) -> bool {
    let actions = [
        "call", "get", "post", "request", "check", "verify", "test", "debug", "fix", "solve",
        "create", "update", "delete",
    ];
    actions
        .iter()
        .any(|&a| word.contains(a) || a.contains(word))
}

fn is_outcome(word: &str) -> bool {
    let outcomes = [
        "error",
        "success",
        "failure",
        "404",
        "500",
        "200",
        "timeout",
        "exception",
        "crash",
        "bug",
        "issue",
        "problem",
    ];
    outcomes
        .iter()
        .any(|&o| word.contains(o) || o.contains(word))
}

pub fn ingest_conversation_enhanced(text: &str) -> SemanticEvent {
    let mut atoms = Vec::new();
    let mut relationships = Vec::new();
    let mut emotional_weight = 0.0;

    let text_lower = text.to_lowercase();

    if text_lower.contains("error")
        || text_lower.contains("problem")
        || text_lower.contains("issue")
    {
        emotional_weight = 0.4;
    }
    if text_lower.contains("frustrated") || text_lower.contains("stuck") {
        emotional_weight = 0.6;
    }
    if text_lower.contains("success") || text_lower.contains("solved") {
        emotional_weight = -0.3;
    }

    let extracted_atoms = extract_all_atoms(text);

    for pattern_match in extracted_atoms {
        atoms.push(SemanticAtom {
            atom_type: pattern_match.atom_type,
            content: pattern_match.content,
        });
    }

    relationships = extract_relationships(&atoms, text);

    for i in 0..atoms.len().saturating_sub(1) {
        relationships.push(Relationship {
            from_atom: i,
            to_atom: i + 1,
            relation_type: RelationType::Temporal,
            strength: 0.5,
        });
    }

    SemanticEvent {
        id: Uuid::new_v4(),
        timestamp: current_timestamp(),
        event_type: EventType::Conversation,
        atoms,
        relationships,
        salience: 1.0,
        emotional_weight,
        source_context: HashMap::new(),
    }
}
