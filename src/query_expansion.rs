// Copyright (c) 2026 Nolan Taft
use crate::types::*;
use std::collections::HashSet;
use uuid::Uuid;



pub struct QueryExpander;

impl QueryExpander {
    
    
    pub fn expand_query(
        query: &str,
        memory: &MemoryGraph,
        max_expansions: usize,
    ) -> HashSet<String> {
        let mut expanded = HashSet::new();
        
        
        let key_terms = Self::extract_key_terms(query);
        
        
        for term in &key_terms {
            expanded.insert(term.clone());
        }
        
        
        let matching_fragments = Self::find_matching_fragments(&key_terms, memory);
        
        
        let fragment_keywords = Self::extract_keywords_from_fragments(&matching_fragments, memory);
        for keyword in fragment_keywords {
            expanded.insert(keyword);
        }
        
        
        let related_keywords = Self::discover_related_keywords(
            &matching_fragments,
            memory,
            max_expansions,
        );
        for keyword in related_keywords {
            expanded.insert(keyword);
        }
        
        
        let co_activation_keywords = Self::discover_co_activation_keywords(
            &matching_fragments,
            memory,
        );
        for keyword in co_activation_keywords {
            expanded.insert(keyword);
        }
        
        
        Self::generate_compound_variations(&key_terms, &mut expanded);
        
        expanded
    }
    
    
    fn extract_key_terms(query: &str) -> Vec<String> {
        
        let minimal_stop_words: HashSet<&str> = [
            "a", "an", "the", "is", "are", "was", "were",
        ].iter().cloned().collect();
        
        query
            .to_lowercase()
            .split_whitespace()
            .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()).to_string())
            .filter(|w| w.len() >= 2 && !minimal_stop_words.contains(w.as_str()))
            .collect()
    }
    
    
    fn find_matching_fragments(terms: &[String], memory: &MemoryGraph) -> Vec<Uuid> {
        let mut fragment_ids = HashSet::new();
        
        for term in terms {
            
            if let Some(fragments) = memory.activation_index.by_keyword.get(term) {
                fragment_ids.extend(fragments);
            }
            
            
            let term_lower = term.to_lowercase();
            if term_lower != *term {
                if let Some(fragments) = memory.activation_index.by_keyword.get(&term_lower) {
                    fragment_ids.extend(fragments);
                }
            }
            
            
            let variations = Self::generate_term_variations(term);
            for variation in variations {
                if let Some(fragments) = memory.activation_index.by_keyword.get(&variation) {
                    fragment_ids.extend(fragments);
                }
            }
        }
        
        fragment_ids.into_iter().collect()
    }
    
    
    fn extract_keywords_from_fragments(fragment_ids: &[Uuid], memory: &MemoryGraph) -> HashSet<String> {
        let mut keywords = HashSet::new();
        
        for fragment_id in fragment_ids {
            if let Some(fragment) = memory.fragments.get(fragment_id) {
                match &fragment.content {
                    FragmentContent::SemanticAtom { content, .. } => {
                        
                        for (key, value) in content {
                            if !key.is_empty() && key != "key" {
                                keywords.insert(key.clone());
                                keywords.insert(key.to_lowercase());
                            }
                            if !value.is_empty() && value != "unknown" && value.len() >= 2 {
                                keywords.insert(value.clone());
                                keywords.insert(value.to_lowercase());
                            }
                        }
                    }
                    _ => {
                        
                        
                    }
                }
            }
        }
        
        keywords
    }
    
    
    fn discover_related_keywords(
        fragment_ids: &[Uuid],
        memory: &MemoryGraph,
        max_depth: usize,
    ) -> HashSet<String> {
        let mut keywords = HashSet::new();
        let mut explored = HashSet::new();
        let mut to_explore: Vec<(Uuid, usize)> = fragment_ids.iter()
            .map(|&id| (id, 0))
            .collect();
        
        while let Some((fragment_id, depth)) = to_explore.pop() {
            if depth >= max_depth || explored.contains(&fragment_id) {
                continue;
            }
            explored.insert(fragment_id);
            
            
            let related_fragments: Vec<Uuid> = memory.edges.iter()
                .filter_map(|((from, to), edge)| {
                    if *from == fragment_id && edge.strength > 0.3 {
                        Some(*to)
                    } else if *to == fragment_id && edge.strength > 0.3 {
                        Some(*from)
                    } else {
                        None
                    }
                })
                .collect();
            
            
            for related_id in &related_fragments {
                if let Some(fragment) = memory.fragments.get(related_id) {
                    match &fragment.content {
                        FragmentContent::SemanticAtom { content, .. } => {
                            for (key, value) in content {
                                if !key.is_empty() && key != "key" {
                                    keywords.insert(key.clone());
                                }
                                if !value.is_empty() && value != "unknown" && value.len() >= 2 {
                                    keywords.insert(value.clone());
                                }
                            }
                        }
                        _ => {}
                    }
                    
                    
                    if depth < max_depth - 1 {
                        to_explore.push((*related_id, depth + 1));
                    }
                }
            }
        }
        
        keywords
    }
    
    
    fn discover_co_activation_keywords(
        fragment_ids: &[Uuid],
        memory: &MemoryGraph,
    ) -> HashSet<String> {
        let mut keywords = HashSet::new();
        
        
        for pattern in &memory.co_activation_patterns {
            
            let has_match = fragment_ids.iter().any(|id| pattern.fragment_ids.contains(id));
            
            if has_match && pattern.activation_count > 1 {
                
                for pattern_fragment_id in &pattern.fragment_ids {
                    if let Some(fragment) = memory.fragments.get(pattern_fragment_id) {
                        match &fragment.content {
                            FragmentContent::SemanticAtom { content, .. } => {
                                for (key, value) in content {
                                    if !key.is_empty() && key != "key" {
                                        keywords.insert(key.clone());
                                    }
                                    if !value.is_empty() && value != "unknown" && value.len() >= 2 {
                                        keywords.insert(value.clone());
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        
        keywords
    }
    
    
    fn generate_term_variations(term: &str) -> Vec<String> {
        let mut variations = Vec::new();
        
        if term.contains(' ') {
            variations.push(term.replace(' ', "_"));
            variations.push(term.replace(' ', "-"));
            variations.push(term.replace(' ', ""));
        }
        
        variations
    }
    
    
    fn generate_compound_variations(terms: &[String], expanded: &mut HashSet<String>) {
        if terms.len() >= 2 {
            
            expanded.insert(terms.join(""));
            
            
            expanded.insert(terms.join("_"));
            
            
            expanded.insert(terms.join("-"));
            
            
            let reversed: Vec<String> = terms.iter().rev().cloned().collect();
            expanded.insert(reversed.join(""));
            expanded.insert(reversed.join("_"));
            expanded.insert(reversed.join("-"));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;
    
    
    
    fn create_test_memory_with_compound_key(base_term: &str, compound_key: &str, value: &str) -> MemoryGraph {
        let mut memory = MemoryGraph::new();
        
        let fragment_id = Uuid::new_v4();
        let mut content = HashMap::new();
        content.insert(compound_key.to_string(), value.to_string());
        
        content.insert("key".to_string(), base_term.to_string());
        
        let fragment = MFragment {
            id: fragment_id,
            fragment_type: FragmentType::EntityRelation,
            content: FragmentContent::SemanticAtom {
                atom_type: AtomType::Person,
                content,
                atom_id: None,
            },
            confidence: 0.9,
            salience: 0.8,
            emotional_tag: 0.0,
            reinforcement_count: 1,
            last_activated: 0.0,
            activation_history: Vec::new(),
            created_at: 0.0,
            decay_rate: 0.01,
        };
        
        memory.insert_fragment(fragment, Vec::new());
        
        memory
    }
    
    #[test]
    fn test_extract_key_terms() {
        let terms = QueryExpander::extract_key_terms("what is entity?");
        assert!(terms.contains(&"entity".to_string()));
        assert!(terms.contains(&"what".to_string()));
    }
    
    #[test]
    fn test_find_matching_fragments() {
        
        let memory = create_test_memory_with_compound_key("entity", "entity_name", "test_value");
        let terms = vec!["entity".to_string()];
        let fragments = QueryExpander::find_matching_fragments(&terms, &memory);
        
        
        assert!(!fragments.is_empty());
    }
    
    #[test]
    fn test_expand_query_dynamic() {
        
        let memory = create_test_memory_with_compound_key("entity", "entity_name", "test_value");
        let expanded = QueryExpander::expand_query("what is entity?", &memory, 2);
        
        
        assert!(expanded.contains("entity"), "Should contain base term from query");
        
        assert!(
            expanded.contains("entity_name"),
            "Should discover compound key from fragment content. Expanded: {:?}", expanded
        );
    }
    
    #[test]
    fn test_expand_query_discovers_related_keywords() {
        
        let memory = create_test_memory_with_compound_key("base", "base_identifier", "value123");
        let expanded = QueryExpander::expand_query("what is base?", &memory, 2);
        
        
        assert!(
            expanded.contains("base_identifier") || 
            expanded.iter().any(|k| k.contains("base") && k.contains("identifier")),
            "Should discover compound key. Expanded: {:?}", expanded
        );
    }
}
