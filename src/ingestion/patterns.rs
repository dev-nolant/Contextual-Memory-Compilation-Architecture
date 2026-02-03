// Copyright (c) 2026 Nolan Taft
use crate::types::*;
use std::collections::HashMap;





#[derive(Debug, Clone)]
pub struct PatternMatch {
    pub atom_type: AtomType,
    pub content: HashMap<String, String>,
    pub confidence: f64,
    pub pattern_name: String,
}


#[derive(Debug, Clone, PartialEq)]
pub enum PatternCategory {
    Personal,
    Temporal,
    Spatial,
    Quantitative,
    Causal,
    Hierarchical,
    Social,
    Ownership,
    State,
    Technical,
}


pub trait PatternMatcher {
    fn match_pattern(&self, text: &str, context: &HashMap<String, String>) -> Vec<PatternMatch>;
    fn category(&self) -> PatternCategory;
    fn confidence_base(&self) -> f64;
}



pub struct PatternLearner;

impl PatternLearner {
    pub fn new() -> Self {
        PatternLearner
    }

    
    pub fn extract_atoms_structural(text: &str) -> Vec<PatternMatch> {
        let mut matches = Vec::new();
        
        
        
        
        let words: Vec<&str> = text.split_whitespace().collect();
        let text_lower = text.to_lowercase();
        
        
        for (i, word) in words.iter().enumerate() {
            let word_clean = word.trim_matches(|c: char| !c.is_alphanumeric());
            
            if word_clean.len() < 2 {
                continue;
            }
            
            
            if i == 0 && word.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
                let mut content = HashMap::new();
                content.insert("key".to_string(), word_clean.to_lowercase());
                matches.push(PatternMatch {
                    atom_type: AtomType::Person,
                    content,
                    confidence: 0.6,
                    pattern_name: "structural_capitalized_start".to_string(),
                });
            }
            
            
            
            if i == 0 && word_clean.len() <= 2 && word_clean.chars().all(|c| c.is_alphabetic()) {
                let mut content = HashMap::new();
                content.insert("key".to_string(), word_clean.to_string());
                matches.push(PatternMatch {
                    atom_type: AtomType::Person,
                    content,
                    confidence: 0.7,
                    pattern_name: "structural_short_start".to_string(),
                });
            }
            
            
            if word_clean.chars().all(|c| c.is_ascii_digit() || c == '.' || c == ',') {
                let mut content = HashMap::new();
                content.insert("value".to_string(), word_clean.to_string());
                matches.push(PatternMatch {
                    atom_type: AtomType::Quantity,
                    content,
                    confidence: 0.9,
                    pattern_name: "structural_number".to_string(),
                });
            }
            
            
            
            
            let is_verb_form = (word_clean.ends_with("ing") && word_clean.len() > 4) || 
                              (word_clean.ends_with("ed") && word_clean.len() > 4);
            
            
            
            let is_likely_plural_noun = word_clean.ends_with("s") && 
                                       word_clean.len() > 4 &&
                                       !word_clean.ends_with("ss") &&
                                       !is_verb_form;
            
            if is_verb_form && !is_likely_plural_noun {
                let mut content = HashMap::new();
                content.insert("key".to_string(), word_clean.to_string());
                matches.push(PatternMatch {
                    atom_type: AtomType::Action,
                    content,
                    confidence: 0.7,
                    pattern_name: "structural_verb".to_string(),
                });
            }
            
            
            
            if is_likely_plural_noun {
                let mut content = HashMap::new();
                content.insert("key".to_string(), word_clean.to_string());
                matches.push(PatternMatch {
                    atom_type: AtomType::Object,
                    content,
                    confidence: 0.6,
                    pattern_name: "structural_plural_noun".to_string(),
                });
            }
        }
        
        
        
        
        
        for i in 0..words.len().saturating_sub(1) {
            let word = words[i].trim_matches(|c: char| !c.is_alphanumeric());
            let next_word = words[i + 1].trim_matches(|c: char| !c.is_alphanumeric());
            
            
            
            let is_likely_verb = next_word.len() >= 3 && next_word.len() <= 6 &&
                                 (next_word == "like" || next_word == "prefer" || 
                                  next_word == "enjoy" || next_word == "love" ||
                                  next_word == "want" || next_word == "need");
            
            
            
            if word.len() >= 1 && word.len() <= 4 && 
               word.chars().all(|c| c.is_lowercase() || c.is_alphabetic()) &&
               next_word.len() > word.len() &&
               next_word.chars().any(|c| c.is_alphanumeric()) &&
               !is_likely_verb {
                let mut content = HashMap::new();
                content.insert("key".to_string(), next_word.to_lowercase());
                content.insert("ownership_marker".to_string(), word.to_lowercase());
                matches.push(PatternMatch {
                    atom_type: AtomType::Entity,
                    content,
                    confidence: 0.7,
                    pattern_name: "structural_possessive".to_string(),
                });
            }
            
            
            if word == "i" && is_likely_verb {
                
                let mut action_content = HashMap::new();
                action_content.insert("key".to_string(), next_word.to_lowercase());
                matches.push(PatternMatch {
                    atom_type: AtomType::Action,
                    content: action_content,
                    confidence: 0.8,
                    pattern_name: "structural_i_verb".to_string(),
                });
                
                
                let mut person_content = HashMap::new();
                person_content.insert("key".to_string(), word.to_lowercase());
                matches.push(PatternMatch {
                    atom_type: AtomType::Person,
                    content: person_content,
                    confidence: 0.8,
                    pattern_name: "structural_pronoun_i".to_string(),
                });
            }
        }
        
        
        
        
        
        
        let is_question = text.trim().ends_with('?');
        
        if !is_question {
            for i in 0..words.len().saturating_sub(2) {
                let word = words[i + 1].trim_matches(|c: char| !c.is_alphanumeric()).to_lowercase();
                
                
                
                let is_assignment = word == "is" || word == "=";
                
                if is_assignment {
                    
                    
                    let mut key_start = i;
                    let mut ownership_marker: Option<String> = None;
                    
                    if key_start > 0 {
                        let prev_word = words[key_start - 1].trim_matches(|c: char| !c.is_alphanumeric());
                        
                        if prev_word.len() >= 1 && prev_word.len() <= 4 && 
                           prev_word.chars().all(|c| c.is_lowercase() || c.is_alphabetic()) {
                            ownership_marker = Some(prev_word.to_lowercase());
                            key_start = key_start.saturating_sub(1);
                        }
                    }
                    
                    let key_parts: Vec<String> = words.iter()
                        .skip(key_start)
                        .take(i + 1 - key_start)
                        .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()).to_lowercase())
                        .filter(|w| {
                            
                            w.len() > 0 && w.chars().any(|c| c.is_alphabetic()) &&
                            ownership_marker.as_ref().map(|m| w != m).unwrap_or(true)
                        })
                        .collect();
                    
                    
                    let value_parts: Vec<String> = words.iter()
                        .skip(i + 2)
                        .take(5)
                        .map(|w| {
                            w.trim_matches(|c: char| {
                                !c.is_alphanumeric() && c != '-' && c != '_'
                            }).to_string()
                        })
                        .filter(|w| !w.is_empty())
                        .collect();
                    
                    if !key_parts.is_empty() && !value_parts.is_empty() {
                        let entity_key = key_parts.join(" ");
                        let value = value_parts.join(" ");
                        
                        
                        eprintln!("\n🔍 [DEBUG] Assignment Pattern Detected:");
                        eprintln!("   Text: {}", text);
                        eprintln!("   Position i={}, word='{}'", i, word);
                        eprintln!("   Key parts: {:?} -> '{}'", key_parts, entity_key);
                        eprintln!("   Value parts: {:?} -> '{}'", value_parts, value);
                        eprintln!("   Ownership marker: {:?}", ownership_marker);
                        
                        
                        let mut content = HashMap::new();
                        content.insert("key".to_string(), entity_key.clone());
                        
                        content.insert(entity_key.clone(), value.clone());
                        
                        if let Some(marker) = ownership_marker {
                            content.insert("ownership_marker".to_string(), marker.clone());
                            eprintln!("   Added ownership_marker: {}", marker);
                        }
                        
                        eprintln!("   Created Entity atom: {:?}", content);
                        
                        matches.push(PatternMatch {
                            atom_type: AtomType::Entity,
                            content,
                            confidence: 0.9,
                            pattern_name: "structural_assignment".to_string(),
                        });
                    }
                }
            }
        }
        
        
        
        let has_question_mark = text.trim_end().ends_with('?');
        if has_question_mark && !words.is_empty() {
            let first_word = words[0].trim_matches(|c: char| !c.is_alphanumeric());
            
            if first_word.len() >= 3 && first_word.len() <= 5 && 
               first_word.chars().all(|c| c.is_lowercase() || c.is_alphabetic()) {
                let mut content = HashMap::new();
                content.insert("key".to_string(), first_word.to_lowercase());
                matches.push(PatternMatch {
                    atom_type: AtomType::Action,
                    content,
                    confidence: 0.8,
                    pattern_name: "structural_question".to_string(),
                });
            }
        }
        
        
        
        for word in &words {
            let word_lower = word.to_lowercase();
            if word.contains(':') && word.chars().any(|c| c.is_ascii_digit()) {
                
                let mut content = HashMap::new();
                content.insert("time_expression".to_string(), word.to_string());
                matches.push(PatternMatch {
                    atom_type: AtomType::Time,
                    content,
                    confidence: 0.9,
                    pattern_name: "structural_time_format".to_string(),
                });
            }
        }
        
        matches
    }
}


impl Default for PatternLearner {
    fn default() -> Self {
        Self::new()
    }
}


pub fn extract_all_patterns(text: &str) -> Vec<PatternMatch> {
    
    PatternLearner::extract_atoms_structural(text)
}



pub mod personal {
    use super::*;
    
    pub fn extract_personal_atoms(text: &str) -> Vec<PatternMatch> {
        
        extract_all_patterns(text)
            .into_iter()
            .filter(|m| matches!(m.atom_type, AtomType::Person))
            .collect()
    }
}

pub mod temporal {
    use super::*;
    
    pub fn extract_temporal_atoms(text: &str) -> Vec<PatternMatch> {
        extract_all_patterns(text)
            .into_iter()
            .filter(|m| matches!(m.atom_type, AtomType::Time))
            .collect()
    }
}

pub mod spatial {
    use super::*;
    
    pub fn extract_spatial_atoms(text: &str) -> Vec<PatternMatch> {
        extract_all_patterns(text)
            .into_iter()
            .filter(|m| matches!(m.atom_type, AtomType::Location))
            .collect()
    }
}

pub mod quantitative {
    use super::*;
    
    pub fn extract_quantitative_atoms(text: &str) -> Vec<PatternMatch> {
        extract_all_patterns(text)
            .into_iter()
            .filter(|m| matches!(m.atom_type, AtomType::Quantity))
            .collect()
    }
}

pub mod causal {
    use super::*;
    
    pub fn extract_causal_atoms(text: &str) -> Vec<PatternMatch> {
        extract_all_patterns(text)
            .into_iter()
            .filter(|m| matches!(m.atom_type, AtomType::Action | AtomType::Outcome))
            .collect()
    }
}

pub mod hierarchical {
    use super::*;
    
    pub fn extract_hierarchical_atoms(text: &str) -> Vec<PatternMatch> {
        extract_all_patterns(text)
            .into_iter()
            .filter(|m| matches!(m.atom_type, AtomType::Concept))
            .collect()
    }
}

pub mod social {
    use super::*;
    
    pub fn extract_social_atoms(text: &str) -> Vec<PatternMatch> {
        extract_all_patterns(text)
            .into_iter()
            .filter(|m| matches!(m.atom_type, AtomType::Person))
            .collect()
    }
}

pub mod ownership {
    use super::*;
    
    pub fn extract_ownership_atoms(text: &str) -> Vec<PatternMatch> {
        extract_all_patterns(text)
            .into_iter()
            .filter(|m| {
                
                m.content.contains_key("ownership_marker")
            })
            .collect()
    }
}

pub mod state {
    use super::*;
    
    pub fn extract_state_atoms(text: &str) -> Vec<PatternMatch> {
        extract_all_patterns(text)
            .into_iter()
            .filter(|m| matches!(m.atom_type, AtomType::State))
            .collect()
    }
}

pub mod technical {
    use super::*;
    
    pub fn extract_technical_atoms(text: &str) -> Vec<PatternMatch> {
        
        
        
        
        let mut matches = Vec::new();
        let words: Vec<&str> = text.split_whitespace().collect();
        
        for word in words {
            let word_clean = word.trim_matches(|c: char| !c.is_alphanumeric());
            
            
            if word_clean.len() >= 2 && word_clean.chars().all(|c| c.is_uppercase() || c.is_ascii_digit()) {
                let mut content = HashMap::new();
                content.insert("name".to_string(), word_clean.to_string());
                matches.push(PatternMatch {
                    atom_type: AtomType::Entity,
                    content,
                    confidence: 0.8,
                    pattern_name: "structural_acronym".to_string(),
                });
            }
            
            
            if word_clean.chars().any(|c| c.is_uppercase()) && 
               word_clean.chars().any(|c| c.is_lowercase()) &&
               word_clean.len() > 3 {
                let mut content = HashMap::new();
                content.insert("name".to_string(), word_clean.to_string());
                matches.push(PatternMatch {
                    atom_type: AtomType::Entity,
                    content,
                    confidence: 0.7,
                    pattern_name: "structural_camelcase".to_string(),
                });
            }
        }
        
        matches
    }
}
