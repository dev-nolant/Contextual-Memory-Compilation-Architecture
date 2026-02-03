// Copyright (c) 2026 Nolan Taft
use crate::types::*;
use std::collections::{HashMap, HashSet};

pub fn generate_context(goal: &str, domain: &str, time_pressure: f64) -> ContextVector {
    let goal_spec = GoalSpec {
        description: goal.to_string(),
        goal_type: infer_goal_type(goal),
        parameters: extract_goal_parameters(goal),
        priority: 0.8,
    };

    let attention_window = AttentionWindow {
        focus_entities: extract_entities(goal),
        focus_domains: HashSet::from([domain.to_string()]),
        focus_relations: extract_relations(goal),
        exclusion_patterns: HashSet::new(),
    };

    let emotional_bias = EmotionalState {
        frustration: 0.4,
        curiosity: 0.6,
        confidence: 0.5,
        urgency: time_pressure,
        satisfaction: 0.0,
    };

    let domain_hint = DomainPattern {
        domain: domain.to_string(),
        subdomain: None,
        tags: extract_tags(domain),
    };

    ContextVector {
        goal: goal_spec,
        attention_window,
        emotional_bias,
        environmental_constraints: Constraints::default(),
        recent_activations: Vec::new(),
        time_pressure,
        domain_hint,
        confidence_threshold: 0.6,
        max_fragments: 20,
    }
}

fn infer_goal_type(goal: &str) -> GoalType {
    let goal_lower = goal.to_lowercase();
    if goal_lower.contains("debug") || goal_lower.contains("fix") {
        GoalType::Debug
    } else if goal_lower.contains("create") || goal_lower.contains("build") {
        GoalType::Create
    } else if goal_lower.contains("learn") || goal_lower.contains("understand") {
        GoalType::Learn
    } else if goal_lower.contains("explain") {
        GoalType::Explain
    } else if goal_lower.contains("predict") {
        GoalType::Predict
    } else {
        GoalType::Debug
    }
}

fn extract_goal_parameters(goal: &str) -> HashMap<String, String> {
    let mut params = HashMap::new();
    let goal_lower = goal.to_lowercase();

    if goal_lower.contains("404") {
        params.insert("error_code".to_string(), "404".to_string());
    }
    if goal_lower.contains("http") {
        params.insert("protocol".to_string(), "HTTP".to_string());
    }

    params
}

fn extract_entities(goal: &str) -> HashSet<String> {
    let mut entities = HashSet::new();
    let words: Vec<&str> = goal.split_whitespace().collect();

    for word in words {
        let word_clean = word
            .trim_matches(|c: char| !c.is_alphanumeric())
            .to_lowercase();
        if word_clean.len() >= 3 {
            match word_clean.as_str() {
                "http" | "api" | "url" | "server" | "error" | "code" => {
                    entities.insert(word_clean);
                }
                _ => {}
            }
        }
    }

    entities
}

fn extract_relations(goal: &str) -> HashSet<String> {
    let mut relations = HashSet::new();
    let goal_lower = goal.to_lowercase();

    if goal_lower.contains("cause") {
        relations.insert("causes".to_string());
    }
    if goal_lower.contains("fix") {
        relations.insert("fixes".to_string());
    }
    if goal_lower.contains("check") {
        relations.insert("checks".to_string());
    }

    relations
}

fn extract_tags(domain: &str) -> HashSet<String> {
    let mut tags = HashSet::new();
    let domain_lower = domain.to_lowercase();

    if domain_lower.contains("web") {
        tags.insert("web_development".to_string());
    }
    if domain_lower.contains("http") {
        tags.insert("http".to_string());
    }
    if domain_lower.contains("api") {
        tags.insert("api".to_string());
    }

    tags.insert(domain_lower);
    tags
}
