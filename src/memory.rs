// Copyright (c) 2026 Nolan Taft
use crate::types::*;
use crate::storage::{save_memory, load_memory, Result as StorageResult};
use crate::intent::Intent;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use uuid::Uuid;



fn stem_keyword(word: &str) -> String {
    let word = word.to_lowercase();
    
    if word.ends_with("ing") && word.len() > 5 {
        return word[..word.len() - 3].to_string();
    }
    if word.ends_with("ed") && word.len() > 4 {
        return word[..word.len() - 2].to_string();
    }
    if word.ends_with("es") && word.len() > 4 {
        return word[..word.len() - 2].to_string();
    }
    if word.ends_with("s") && word.len() > 3 {
        return word[..word.len() - 1].to_string();
    }
    word
}

impl MemoryGraph {
    pub fn new() -> Self {
        MemoryGraph {
            fragments: HashMap::new(),
            edges: HashMap::new(),
            activation_index: ActivationIndex::default(),
            compiled_modules: Vec::new(),
            co_activation_patterns: Vec::new(),
            version: 1,
        }
    }
    
    
    pub fn record_co_activation(&mut self, fragment_ids: &[Uuid]) {
        if fragment_ids.len() < 2 {
            return; 
        }
        
        let mut fragment_ids_sorted = fragment_ids.to_vec();
        fragment_ids_sorted.sort();
        
        
        let pattern = self.co_activation_patterns
            .iter_mut()
            .find(|p| p.fragment_ids == fragment_ids_sorted);
        
        let now = current_timestamp();
        
        if let Some(pattern) = pattern {
            
            pattern.activation_count += 1;
            pattern.last_activated = now;
            
            
            let total_confidence: f64 = fragment_ids
                .iter()
                .filter_map(|id| self.fragments.get(id).map(|f| f.confidence))
                .sum();
            let count = fragment_ids.len() as f64;
            pattern.average_confidence = (pattern.average_confidence + total_confidence / count) / 2.0;
        } else {
            
            let total_confidence: f64 = fragment_ids
                .iter()
                .filter_map(|id| self.fragments.get(id).map(|f| f.confidence))
                .sum();
            let count = fragment_ids.len() as f64;
            
            self.co_activation_patterns.push(CoActivationPattern {
                fragment_ids: fragment_ids_sorted,
                activation_count: 1,
                average_confidence: total_confidence / count,
                last_activated: now,
                formatting_pattern: None, 
            });
        }
    }
    
    
    pub fn get_formatting_pattern(&self, fragment_ids: &[Uuid]) -> Option<String> {
        let mut fragment_ids_sorted = fragment_ids.to_vec();
        fragment_ids_sorted.sort();
        
        self.co_activation_patterns
            .iter()
            .find(|p| p.fragment_ids == fragment_ids_sorted)
            .and_then(|p| p.formatting_pattern.clone())
    }
    
    
    pub fn learn_intent_pattern(
        &mut self,
        intent: &Intent,
        activated_fragments: &[Uuid],
        success: bool,
    ) {
        
        
        
        
        
        
        if success && !activated_fragments.is_empty() {
            
            
        }
    }
    
    pub fn add_compiled_module(&mut self, module: CompiledModule) {
        self.compiled_modules.push(module);
    }
    
    pub fn get_compiled_modules(&self) -> &[CompiledModule] {
        &self.compiled_modules
    }

    pub fn insert_fragment(&mut self, fragment: MFragment, fragment_edges: Vec<Edge>) {
        let fragment_id = fragment.id;
        self.fragments.insert(fragment_id, fragment.clone());
        
        self.update_activation_index(&fragment);
        
        for edge in fragment_edges {
            let key = (edge.from_fragment, edge.to_fragment);
            self.edges.insert(key, edge);
        }
    }

    pub fn activate_fragments(&mut self, context: &ContextVector) -> HashSet<Uuid> {
        let mut candidates = HashSet::new();
        
        let goal_patterns = extract_goal_patterns(&context.goal.description);
        for pattern in &goal_patterns {
            
            if let Some(fragments) = self.activation_index.by_goal.get(pattern) {
                candidates.extend(fragments);
            }
            
            let pattern_lower = pattern.to_lowercase();
            if let Some(fragments) = self.activation_index.by_goal.get(&pattern_lower) {
                candidates.extend(fragments);
            }
        }
        
        if let Some(fragments) = self.activation_index.by_domain.get(&context.domain_hint.domain) {
            candidates.extend(fragments);
        }
        
        for tag in &context.domain_hint.tags {
            
            if let Some(fragments) = self.activation_index.by_keyword.get(tag) {
                candidates.extend(fragments);
            }
            
            let tag_lower = tag.to_lowercase();
            if let Some(fragments) = self.activation_index.by_keyword.get(&tag_lower) {
                candidates.extend(fragments);
            }
            
            let normalized_underscore = tag_lower.replace(' ', "_");
            if normalized_underscore != tag_lower {
                if let Some(fragments) = self.activation_index.by_keyword.get(&normalized_underscore) {
                    candidates.extend(fragments);
                }
            }
            let normalized_hyphen = tag_lower.replace(' ', "-");
            if normalized_hyphen != tag_lower && normalized_hyphen != normalized_underscore {
                if let Some(fragments) = self.activation_index.by_keyword.get(&normalized_hyphen) {
                    candidates.extend(fragments);
                }
            }
            let normalized_nospace = tag_lower.replace(' ', "");
            if normalized_nospace != tag_lower && normalized_nospace.len() >= 4 {
                if let Some(fragments) = self.activation_index.by_keyword.get(&normalized_nospace) {
                    candidates.extend(fragments);
                }
            }
        }
        
        
        for pattern in &goal_patterns {
            let pattern_lower = pattern.to_lowercase();
            if let Some(fragments) = self.activation_index.by_keyword.get(&pattern_lower) {
                candidates.extend(fragments);
            }
        }
        
        let mut scored: Vec<(Uuid, f64)> = candidates
            .iter()
            .filter_map(|&id| {
                if let Some(fragment) = self.fragments.get(&id) {
                    if fragment.confidence >= context.confidence_threshold {
                        let score = calculate_relevance_score(fragment, context);
                        Some((id, score))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();
        
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        
        let mut activated: HashSet<Uuid> = scored
            .iter()
            .take(context.max_fragments)
            .map(|(id, _)| *id)
            .collect();
        
        
        
        let mut to_explore: Vec<Uuid> = activated.iter().cloned().collect();
        let mut explored = HashSet::new();
        let max_edge_traversal = 10; 
        
        for _ in 0..max_edge_traversal {
            if to_explore.is_empty() {
                break;
            }
            
            let current = to_explore.pop().unwrap();
            if explored.contains(&current) {
                continue;
            }
            explored.insert(current);
            
            
            for ((from_id, to_id), edge) in &self.edges {
                if *from_id == current && !activated.contains(to_id) {
                    if let Some(fragment) = self.fragments.get(to_id) {
                        if fragment.confidence >= context.confidence_threshold {
                            activated.insert(*to_id);
                            to_explore.push(*to_id);
                        }
                    }
                } else if *to_id == current && !activated.contains(from_id) {
                    if let Some(fragment) = self.fragments.get(from_id) {
                        if fragment.confidence >= context.confidence_threshold {
                            activated.insert(*from_id);
                            to_explore.push(*from_id);
                        }
                    }
                }
            }
        }
        
        let now = current_timestamp();
        for &id in &activated {
            if let Some(fragment) = self.fragments.get_mut(&id) {
                fragment.last_activated = now;
                fragment.activation_history.push(now);
            }
        }
        
        activated
    }

    pub fn reinforce_fragment(&mut self, id: Uuid, outcome: &Outcome) {
        if let Some(fragment) = self.fragments.get_mut(&id) {
            match outcome.outcome_type {
                OutcomeType::Success => {
                    fragment.confidence = (fragment.confidence + 0.1).min(1.0);
                    fragment.reinforcement_count += 1;
                }
                OutcomeType::Failure => {
                    fragment.confidence = (fragment.confidence - 0.05).max(0.0);
                }
                _ => {}
            }
        }
        
        for ((from_id, to_id), edge) in &mut self.edges {
            if *from_id == id || *to_id == id {
                if outcome.outcome_type == OutcomeType::Success {
                    edge.strength = (edge.strength + 0.05).min(1.0);
                    edge.last_reinforced = current_timestamp();
                }
            }
        }
    }

    pub fn decay_memory(&mut self, delta_time: f64) {
        let mut to_remove = Vec::new();
        
        for (id, fragment) in &mut self.fragments {
            
            if fragment.reinforcement_count > 0 {
                
                
                
                
                let i_equivalent = (fragment.reinforcement_count as f64 - 1.0).max(0.0);
                let expected_min = 0.5 + i_equivalent * 0.05;
                let min_confidence = (expected_min + 0.1).min(0.95); 
                
                
                
                let reduction = (fragment.reinforcement_count as f64 * 0.99).min(0.9999);
                let effective_decay_rate = fragment.decay_rate * (1.0 - reduction);
                
                fragment.salience *= (-effective_decay_rate * delta_time).exp();
                
                
                let decayed_confidence = fragment.confidence * (-effective_decay_rate * 0.1 * delta_time).exp();
                fragment.confidence = decayed_confidence.max(min_confidence);
            } else {
                
                fragment.salience *= (-fragment.decay_rate * delta_time).exp();
                fragment.confidence *= (-fragment.decay_rate * 0.1 * delta_time).exp();
            }
            
            
            
            let should_persist = fragment.reinforcement_count >= 1;
            
            
            if !should_persist && fragment.salience < 0.01 && fragment.confidence < 0.1 {
                to_remove.push(*id);
            }
        }
        
        for id in to_remove {
            self.fragments.remove(&id);
        }
        
        let mut edges_to_remove = Vec::new();
        for (key, edge) in &mut self.edges {
            edge.strength *= (-edge.decay_rate * delta_time).exp();
            if edge.strength < 0.01 {
                edges_to_remove.push(*key);
            }
        }
        
        for key in edges_to_remove {
            self.edges.remove(&key);
        }
    }

    fn update_activation_index(&mut self, fragment: &MFragment) {
        match &fragment.content {
            FragmentContent::EntityRelation { entity, .. } => {
                
                self.activation_index
                    .by_keyword
                    .entry(entity.clone())
                    .or_insert_with(HashSet::new)
                    .insert(fragment.id);
                self.activation_index
                    .by_keyword
                    .entry(entity.to_lowercase())
                    .or_insert_with(HashSet::new)
                    .insert(fragment.id);
            }
            FragmentContent::CausalRule { condition, outcome, .. } => {
                
                self.activation_index
                    .by_keyword
                    .entry(condition.clone())
                    .or_insert_with(HashSet::new)
                    .insert(fragment.id);
                self.activation_index
                    .by_keyword
                    .entry(condition.to_lowercase())
                    .or_insert_with(HashSet::new)
                    .insert(fragment.id);
                self.activation_index
                    .by_keyword
                    .entry(outcome.clone())
                    .or_insert_with(HashSet::new)
                    .insert(fragment.id);
                self.activation_index
                    .by_keyword
                    .entry(outcome.to_lowercase())
                    .or_insert_with(HashSet::new)
                    .insert(fragment.id);
            }
            FragmentContent::GoalStrategy { goal, .. } => {
                let patterns = extract_goal_patterns(goal);
                for pattern in patterns {
                    self.activation_index
                        .by_goal
                        .entry(pattern)
                        .or_insert_with(HashSet::new)
                        .insert(fragment.id);
                }
            }
            FragmentContent::PersonalFact { person, fact_type, value, .. } => {
                
                self.activation_index
                    .by_keyword
                    .entry(person.clone())
                    .or_insert_with(HashSet::new)
                    .insert(fragment.id);
                self.activation_index
                    .by_keyword
                    .entry(person.to_lowercase())
                    .or_insert_with(HashSet::new)
                    .insert(fragment.id);
                
                self.activation_index
                    .by_keyword
                    .entry(fact_type.clone())
                    .or_insert_with(HashSet::new)
                    .insert(fragment.id);
                self.activation_index
                    .by_keyword
                    .entry(fact_type.to_lowercase())
                    .or_insert_with(HashSet::new)
                    .insert(fragment.id);
                
                self.activation_index
                    .by_keyword
                    .entry(value.clone())
                    .or_insert_with(HashSet::new)
                    .insert(fragment.id);
                self.activation_index
                    .by_keyword
                    .entry(value.to_lowercase())
                    .or_insert_with(HashSet::new)
                    .insert(fragment.id);
                
                
                
                
                
                
                
                
                if fact_type.to_lowercase() == "preference" {
                    
                    
                    
                }
                
                
                
                
                if fact_type.to_lowercase() == "name" {
                    
                    
                    
                }
                
                
                
                
                
            }
            FragmentContent::TemporalEvent { event, time_expression, .. } => {
                
                self.activation_index
                    .by_keyword
                    .entry(event.clone())
                    .or_insert_with(HashSet::new)
                    .insert(fragment.id);
                self.activation_index
                    .by_keyword
                    .entry(time_expression.clone())
                    .or_insert_with(HashSet::new)
                    .insert(fragment.id);
            }
            FragmentContent::SpatialRelation { entity, location, .. } => {
                
                self.activation_index
                    .by_keyword
                    .entry(entity.clone())
                    .or_insert_with(HashSet::new)
                    .insert(fragment.id);
                self.activation_index
                    .by_keyword
                    .entry(location.clone())
                    .or_insert_with(HashSet::new)
                    .insert(fragment.id);
            }
            FragmentContent::QuantitativeFact { entity, .. } => {
                
                self.activation_index
                    .by_keyword
                    .entry(entity.clone())
                    .or_insert_with(HashSet::new)
                    .insert(fragment.id);
            }
            FragmentContent::HierarchicalRelation { parent, child, .. } => {
                
                self.activation_index
                    .by_keyword
                    .entry(parent.clone())
                    .or_insert_with(HashSet::new)
                    .insert(fragment.id);
                self.activation_index
                    .by_keyword
                    .entry(child.clone())
                    .or_insert_with(HashSet::new)
                    .insert(fragment.id);
            }
            FragmentContent::SocialRelation { person1, person2, relation_type, .. } => {
                
                self.activation_index
                    .by_keyword
                    .entry(person1.clone())
                    .or_insert_with(HashSet::new)
                    .insert(fragment.id);
                self.activation_index
                    .by_keyword
                    .entry(person2.clone())
                    .or_insert_with(HashSet::new)
                    .insert(fragment.id);
                self.activation_index
                    .by_keyword
                    .entry(relation_type.clone())
                    .or_insert_with(HashSet::new)
                    .insert(fragment.id);
            }
            FragmentContent::OwnershipRelation { owner, owned, .. } => {
                
                self.activation_index
                    .by_keyword
                    .entry(owner.clone())
                    .or_insert_with(HashSet::new)
                    .insert(fragment.id);
                self.activation_index
                    .by_keyword
                    .entry(owned.clone())
                    .or_insert_with(HashSet::new)
                    .insert(fragment.id);
            }
            FragmentContent::StateTransition { entity, to_state, .. } => {
                
                self.activation_index
                    .by_keyword
                    .entry(entity.clone())
                    .or_insert_with(HashSet::new)
                    .insert(fragment.id);
                self.activation_index
                    .by_keyword
                    .entry(to_state.clone())
                    .or_insert_with(HashSet::new)
                    .insert(fragment.id);
            }
            FragmentContent::Capability { entity, capability, .. } => {
                
                self.activation_index
                    .by_keyword
                    .entry(entity.clone())
                    .or_insert_with(HashSet::new)
                    .insert(fragment.id);
                self.activation_index
                    .by_keyword
                    .entry(capability.clone())
                    .or_insert_with(HashSet::new)
                    .insert(fragment.id);
            }
            FragmentContent::Belief { entity, belief, .. } => {
                
                self.activation_index
                    .by_keyword
                    .entry(entity.clone())
                    .or_insert_with(HashSet::new)
                    .insert(fragment.id);
                self.activation_index
                    .by_keyword
                    .entry(belief.clone())
                    .or_insert_with(HashSet::new)
                    .insert(fragment.id);
            }
            FragmentContent::Preference { preference, context, .. } => {
                
                self.activation_index
                    .by_keyword
                    .entry(preference.clone())
                    .or_insert_with(HashSet::new)
                    .insert(fragment.id);
                self.activation_index
                    .by_keyword
                    .entry(context.clone())
                    .or_insert_with(HashSet::new)
                    .insert(fragment.id);
            }
            FragmentContent::Constraint { constraint, context, .. } => {
                
                self.activation_index
                    .by_keyword
                    .entry(constraint.clone())
                    .or_insert_with(HashSet::new)
                    .insert(fragment.id);
                self.activation_index
                    .by_keyword
                    .entry(context.clone())
                    .or_insert_with(HashSet::new)
                    .insert(fragment.id);
            }
            FragmentContent::ContextSignature { pattern, .. } => {
                
                self.activation_index
                    .by_keyword
                    .entry(pattern.clone())
                    .or_insert_with(HashSet::new)
                    .insert(fragment.id);
            }
            FragmentContent::SemanticAtom { content, .. } => {
                
                for (key, value) in content {
                    
                    self.activation_index
                        .by_keyword
                        .entry(key.clone())
                        .or_insert_with(HashSet::new)
                        .insert(fragment.id);
                    self.activation_index
                        .by_keyword
                        .entry(key.to_lowercase())
                        .or_insert_with(HashSet::new)
                        .insert(fragment.id);
                    
                    
                    if value.len() >= 2 {
                        self.activation_index
                            .by_keyword
                            .entry(value.clone())
                            .or_insert_with(HashSet::new)
                            .insert(fragment.id);
                        self.activation_index
                            .by_keyword
                            .entry(value.to_lowercase())
                            .or_insert_with(HashSet::new)
                            .insert(fragment.id);
                    }
                }
            }
        }
    }

    
    pub fn save(&self, path: impl AsRef<Path>) -> StorageResult<()> {
        save_memory(self, path.as_ref())
    }

    
    pub fn load(path: impl AsRef<Path>) -> StorageResult<Self> {
        load_memory(path.as_ref())
    }
}

fn extract_goal_patterns(goal: &str) -> Vec<String> {
    let goal_lower = goal.to_lowercase();
    let mut patterns = Vec::new();
    
    if goal_lower.contains("debug") {
        patterns.push("debug".to_string());
    }
    if goal_lower.contains("http") {
        patterns.push("http".to_string());
        
        patterns.push("http_call".to_string());
        patterns.push("http_call".to_string());
    }
    if goal_lower.contains("api") {
        patterns.push("api".to_string());
        
        patterns.push("api_call".to_string());
    }
    if goal_lower.contains("error") {
        patterns.push("error".to_string());
    }
    if goal_lower.contains("404") {
        patterns.push("404".to_string());
        patterns.push("404_error".to_string());
    }
    
    if goal_lower.contains("name") {
        patterns.push("name".to_string());
        patterns.push("personal".to_string());
    }
    if goal_lower.contains("my") || goal_lower.contains("your") {
        patterns.push("personal".to_string());
    }
    
    
    let words: Vec<&str> = goal_lower.split_whitespace().collect();
    for word in &words {
        if word.len() >= 3 {
            patterns.push(word.to_string());
            
            if *word == "http" || *word == "api" {
                patterns.push(format!("{}_call", word));
            }
        }
    }
    
    patterns.push(goal_lower);
    patterns
}

fn calculate_relevance_score(fragment: &MFragment, context: &ContextVector) -> f64 {
    let mut score = 0.0;
    let now = current_timestamp();
    
    score += fragment.confidence * 0.3;
    
    if fragment.last_activated > 0.0 {
        let recency_hours = (now - fragment.last_activated) / 3600.0;
        let recency_factor = (-recency_hours / 24.0).exp();
        score += recency_factor * 0.2;
    }
    
    score += fragment.salience * 0.2;
    
    let emotion_match = 1.0 - (fragment.emotional_tag - context.emotional_bias.frustration).abs();
    score += emotion_match * 0.1;
    
    let reinforcement_factor = (fragment.reinforcement_count as f64 / 100.0).min(1.0);
    score += reinforcement_factor * 0.2;
    
    score
}
