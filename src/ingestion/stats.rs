// Copyright (c) 2026 Nolan Taft
use crate::types::*;
use std::collections::HashMap;



#[derive(Debug, Clone, Default)]
pub struct ExtractionStats {
    pub total_extractions: usize,
    pub pattern_match_counts: HashMap<String, usize>,
    pub atom_type_counts: HashMap<String, usize>,
    pub fragment_type_counts: HashMap<String, usize>,
    pub relationship_type_counts: HashMap<String, usize>,
    pub confidence_distribution: Vec<f64>,  
    pub extraction_errors: Vec<String>,
}

impl ExtractionStats {
    pub fn new() -> Self {
        ExtractionStats::default()
    }
    
    
    pub fn record_pattern_match(&mut self, pattern_name: &str) {
        *self.pattern_match_counts.entry(pattern_name.to_string()).or_insert(0) += 1;
        self.total_extractions += 1;
    }
    
    
    pub fn record_atom(&mut self, atom_type: &AtomType) {
        let type_name = format!("{:?}", atom_type);
        *self.atom_type_counts.entry(type_name).or_insert(0) += 1;
    }
    
    
    pub fn record_fragment(&mut self, fragment_type: &FragmentType) {
        let type_name = format!("{:?}", fragment_type);
        *self.fragment_type_counts.entry(type_name).or_insert(0) += 1;
    }
    
    
    pub fn record_relationship(&mut self, relation_type: &RelationType) {
        let type_name = format!("{:?}", relation_type);
        *self.relationship_type_counts.entry(type_name).or_insert(0) += 1;
    }
    
    
    pub fn record_confidence(&mut self, confidence: f64) {
        self.confidence_distribution.push(confidence);
        
        if self.confidence_distribution.len() > 1000 {
            self.confidence_distribution.remove(0);
        }
    }
    
    
    pub fn record_error(&mut self, error: String) {
        self.extraction_errors.push(error);
        
        if self.extraction_errors.len() > 100 {
            self.extraction_errors.remove(0);
        }
    }
    
    
    pub fn average_confidence(&self) -> f64 {
        if self.confidence_distribution.is_empty() {
            return 0.0;
        }
        self.confidence_distribution.iter().sum::<f64>() / self.confidence_distribution.len() as f64
    }
    
    
    pub fn pattern_coverage_report(&self) -> HashMap<String, f64> {
        let mut coverage = HashMap::new();
        let total = self.total_extractions as f64;
        
        for (pattern, count) in &self.pattern_match_counts {
            coverage.insert(pattern.clone(), *count as f64 / total);
        }
        
        coverage
    }
}
