// Copyright (c) 2026 Nolan Taft
use crate::types::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternConfig {
    pub name: String,
    pub category: String,
    pub pattern: String,
    pub atom_type: AtomType,
    pub priority: usize,
    pub confidence_weight: f64,
    pub activation_conditions: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternConfigSet {
    pub patterns: Vec<PatternConfig>,
    pub default_confidence: f64,
    pub max_patterns_per_category: Option<usize>,
}

impl Default for PatternConfigSet {
    fn default() -> Self {
        PatternConfigSet {
            patterns: Vec::new(),
            default_confidence: 0.7,
            max_patterns_per_category: Some(10),
        }
    }
}

impl PatternConfigSet {
    pub fn load_defaults() -> Self {
        let mut config = PatternConfigSet::default();

        config
    }

    pub fn add_pattern(&mut self, pattern: PatternConfig) {
        self.patterns.push(pattern);

        self.patterns.sort_by(|a, b| b.priority.cmp(&a.priority));
    }

    pub fn get_patterns_for_category(&self, category: &str) -> Vec<&PatternConfig> {
        self.patterns
            .iter()
            .filter(|p| p.category == category)
            .collect()
    }
}
