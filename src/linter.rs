// Copyright (c) 2026 Nolan Taft
use crate::types::*;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

pub fn run_linter(input: LinterInput, config: LinterConfig) -> PatternReport {
    let mut report = PatternReport {
        repeated_paths: Vec::new(),
        stable_branches: Vec::new(),
        invariant_subgraphs: Vec::new(),
        high_confidence_outcomes: Vec::new(),
        fossilization_candidates: Vec::new(),
    };
    
    
    let traces = if let Some(time_window) = input.time_window {
        let cutoff = current_timestamp() - time_window;
        input.execution_traces
            .into_iter()
            .filter(|trace| trace.timestamp >= cutoff)
            .collect()
    } else {
        input.execution_traces
    };
    
    
    report.repeated_paths = detect_repeated_paths(&traces, &input.execution_results, config.min_occurrences, config.min_path_length);
    
    
    report.stable_branches = detect_stable_branches(&traces, &input.compiled_eegs, config.min_occurrences, config.min_branch_ratio);
    
    
    report.invariant_subgraphs = detect_invariant_subgraphs(&traces, &input.compiled_eegs, config.min_occurrences, config.min_context_variance);
    
    
    report.high_confidence_outcomes = detect_high_confidence_outcomes(&input.execution_results, config.min_occurrences, config.min_confidence);
    
    
    report.fossilization_candidates = identify_fossilization_candidates(&report, &config);
    
    report
}

fn detect_repeated_paths(
    traces: &[ExecutionTrace],
    results: &[ExecutionResult],
    min_occurrences: usize,
    min_path_length: usize,
) -> Vec<PathPattern> {
    let mut path_counts: HashMap<Vec<Uuid>, usize> = HashMap::new();
    let mut path_contexts: HashMap<Vec<Uuid>, Vec<ContextVector>> = HashMap::new();
    let mut path_results: HashMap<Vec<Uuid>, Vec<&ExecutionResult>> = HashMap::new();
    
    
    for trace in traces {
        let paths = extract_subpaths(&trace.node_sequence, min_path_length);
        
        for path in paths {
            path_counts.entry(path.clone()).and_modify(|c| *c += 1).or_insert(1);
            path_contexts.entry(path.clone()).or_insert_with(Vec::new).push(trace.context.clone());
            
            
            if let Some(result) = results.iter().find(|r| r.execution_trace == trace.node_sequence) {
                path_results.entry(path.clone()).or_insert_with(Vec::new).push(result);
            }
        }
    }
    
    
    let mut patterns = Vec::new();
    for (path, count) in path_counts {
        if count >= min_occurrences {
            let contexts = path_contexts.get(&path).cloned().unwrap_or_default();
            let results_for_path = path_results.get(&path).cloned().unwrap_or_default();
            
            let average_confidence = calculate_path_confidence(&path, results_for_path.clone());
            let success_rate = calculate_success_rate(&results_for_path);
            
            patterns.push(PathPattern {
                path,
                occurrence_count: count,
                contexts,
                average_confidence,
                success_rate,
            });
        }
    }
    
    
    patterns.sort_by(|a, b| b.occurrence_count.cmp(&a.occurrence_count));
    
    patterns
}

fn extract_subpaths(sequence: &[Uuid], min_length: usize) -> Vec<Vec<Uuid>> {
    let mut paths = Vec::new();
    
    for start in 0..sequence.len() {
        for end in (start + min_length)..=sequence.len() {
            paths.push(sequence[start..end].to_vec());
        }
    }
    
    paths
}

fn calculate_path_confidence(path: &[Uuid], results: Vec<&ExecutionResult>) -> f64 {
    if results.is_empty() {
        return 0.0;
    }
    
    let sum: f64 = results.iter().map(|r| r.confidence).sum();
    sum / results.len() as f64
}

fn calculate_success_rate(results: &[&ExecutionResult]) -> f64 {
    if results.is_empty() {
        return 0.0;
    }
    
    let successes = results.iter()
        .filter(|r| r.outcome.outcome_type == OutcomeType::Success)
        .count();
    
    successes as f64 / results.len() as f64
}

fn detect_stable_branches(
    traces: &[ExecutionTrace],
    eegs: &[EEG],
    min_occurrences: usize,
    min_branch_ratio: f64,
) -> Vec<BranchPattern> {
    let mut decision_stats: HashMap<Uuid, HashMap<Uuid, usize>> = HashMap::new();
    let mut decision_contexts: HashMap<Uuid, Vec<ContextVector>> = HashMap::new();
    
    
    for trace in traces {
        for (decision_node_id, chosen_branch) in &trace.branch_decisions {
            decision_stats
                .entry(*decision_node_id)
                .or_insert_with(HashMap::new)
                .entry(*chosen_branch)
                .and_modify(|c| *c += 1)
                .or_insert(1);
            
            decision_contexts
                .entry(*decision_node_id)
                .or_insert_with(Vec::new)
                .push(trace.context.clone());
        }
    }
    
    
    let mut patterns = Vec::new();
    for (decision_node_id, branch_counts) in decision_stats {
        let total_decisions: usize = branch_counts.values().sum();
        
        if total_decisions < min_occurrences {
            continue;
        }
        
        
        let dominant_branch = branch_counts.iter()
            .max_by_key(|(_, &count)| count)
            .map(|(branch, _)| *branch);
        
        if let Some(dominant) = dominant_branch {
            let dominant_count = branch_counts.get(&dominant).copied().unwrap_or(0);
            let branch_ratio = dominant_count as f64 / total_decisions as f64;
            
            if branch_ratio >= min_branch_ratio {
                let contexts = decision_contexts.get(&decision_node_id).cloned().unwrap_or_default();
                
                
                let mut confidences = Vec::new();
                for trace in traces {
                    if let Some(&chosen) = trace.branch_decisions.get(&decision_node_id) {
                        if chosen == dominant {
                            
                            if let Some(eeg) = eegs.iter().find(|e| e.metadata.compilation_timestamp == trace.timestamp) {
                                confidences.push(eeg.metadata.confidence_score);
                            }
                        }
                    }
                }
                
                let average_confidence = if confidences.is_empty() {
                    0.8 
                } else {
                    confidences.iter().sum::<f64>() / confidences.len() as f64
                };
                
                patterns.push(BranchPattern {
                    decision_node: decision_node_id,
                    dominant_branch: dominant,
                    branch_ratio,
                    contexts,
                    average_confidence,
                });
            }
        }
    }
    
    patterns
}

fn detect_invariant_subgraphs(
    traces: &[ExecutionTrace],
    eegs: &[EEG],
    min_occurrences: usize,
    max_context_variance: f64,
) -> Vec<SubgraphPattern> {
    
    let mut subgraph_counts: HashMap<Vec<Uuid>, usize> = HashMap::new();
    let mut subgraph_contexts: HashMap<Vec<Uuid>, Vec<ContextVector>> = HashMap::new();
    
    for trace in traces {
        
        let subgraph = trace.node_sequence.clone();
        subgraph_counts.entry(subgraph.clone()).and_modify(|c| *c += 1).or_insert(1);
        subgraph_contexts.entry(subgraph).or_insert_with(Vec::new).push(trace.context.clone());
    }
    
    let mut patterns = Vec::new();
    for (subgraph_nodes, count) in subgraph_counts {
        if count >= min_occurrences {
            let contexts = subgraph_contexts.get(&subgraph_nodes).cloned().unwrap_or_default();
            
            
            let context_variance = calculate_context_variance(&contexts);
            
            if context_variance <= max_context_variance {
                
                let mut confidences = Vec::new();
                for trace in traces {
                    if trace.node_sequence == subgraph_nodes {
                        if let Some(eeg) = eegs.iter().find(|e| e.metadata.compilation_timestamp == trace.timestamp) {
                            confidences.push(eeg.metadata.confidence_score);
                        }
                    }
                }
                
                let average_confidence = if confidences.is_empty() {
                    0.7
                } else {
                    confidences.iter().sum::<f64>() / confidences.len() as f64
                };
                
                patterns.push(SubgraphPattern {
                    subgraph_nodes,
                    occurrence_count: count,
                    contexts,
                    average_confidence,
                    context_variance,
                });
            }
        }
    }
    
    patterns
}

fn calculate_context_variance(contexts: &[ContextVector]) -> f64 {
    if contexts.len() < 2 {
        return 0.0;
    }
    
    
    let goals: Vec<&str> = contexts.iter().map(|c| c.goal.description.as_str()).collect();
    let unique_goals: HashSet<&str> = goals.iter().copied().collect();
    
    
    
    1.0 - (unique_goals.len() as f64 / contexts.len() as f64)
}

fn detect_high_confidence_outcomes(
    results: &[ExecutionResult],
    min_occurrences: usize,
    min_confidence: f64,
) -> Vec<OutcomePattern> {
    
    let mut success_results = Vec::new();
    let mut failure_results = Vec::new();
    let mut partial_results = Vec::new();
    
    for result in results {
        match result.outcome.outcome_type {
            OutcomeType::Success => success_results.push(result),
            OutcomeType::Failure => failure_results.push(result),
            OutcomeType::Partial => partial_results.push(result),
            OutcomeType::Uncertain => {}, 
        }
    }
    
    let mut patterns = Vec::new();
    
    
    if success_results.len() >= min_occurrences {
        let average_confidence: f64 = success_results.iter().map(|r| r.confidence).sum::<f64>() / success_results.len() as f64;
        if average_confidence >= min_confidence {
            patterns.push(OutcomePattern {
                outcome_type: OutcomeType::Success,
                occurrence_count: success_results.len(),
                average_confidence,
                success_rate: 1.0,
            });
        }
    }
    
    
    if failure_results.len() >= min_occurrences {
        let average_confidence: f64 = failure_results.iter().map(|r| r.confidence).sum::<f64>() / failure_results.len() as f64;
        if average_confidence >= min_confidence {
            patterns.push(OutcomePattern {
                outcome_type: OutcomeType::Failure,
                occurrence_count: failure_results.len(),
                average_confidence,
                success_rate: 0.0,
            });
        }
    }
    
    
    if partial_results.len() >= min_occurrences {
        let average_confidence: f64 = partial_results.iter().map(|r| r.confidence).sum::<f64>() / partial_results.len() as f64;
        if average_confidence >= min_confidence {
            patterns.push(OutcomePattern {
                outcome_type: OutcomeType::Partial,
                occurrence_count: partial_results.len(),
                average_confidence,
                success_rate: 0.5,
            });
        }
    }
    
    patterns
}

fn identify_fossilization_candidates(
    report: &PatternReport,
    config: &LinterConfig,
) -> Vec<FossilizationCandidate> {
    let mut candidates = Vec::new();
    
    
    for path_pattern in &report.repeated_paths {
        if path_pattern.occurrence_count >= config.min_occurrences
            && path_pattern.average_confidence >= config.min_confidence
        {
            let context_variance = calculate_context_variance(&path_pattern.contexts);
            let reward_correlation = path_pattern.success_rate;
            let estimated_speedup = estimate_speedup(path_pattern.occurrence_count);
            
            if context_variance <= config.min_context_variance
                && reward_correlation >= config.min_reward_correlation
                && estimated_speedup >= config.min_speedup
            {
                let priority = calculate_priority(
                    path_pattern.occurrence_count,
                    path_pattern.average_confidence,
                    context_variance,
                    reward_correlation,
                    estimated_speedup,
                );
                
                candidates.push(FossilizationCandidate {
                    pattern_type: "PathPattern".to_string(),
                    pattern_id: Uuid::new_v4(), 
                    repetition_count: path_pattern.occurrence_count,
                    average_confidence: path_pattern.average_confidence,
                    context_variance,
                    reward_correlation,
                    estimated_speedup,
                    priority,
                });
            }
        }
    }
    
    
    for branch_pattern in &report.stable_branches {
        if branch_pattern.branch_ratio >= config.min_branch_ratio
            && branch_pattern.average_confidence >= config.min_confidence
        {
            let context_variance = calculate_context_variance(&branch_pattern.contexts);
            let reward_correlation = branch_pattern.branch_ratio; 
            let estimated_speedup = 1.5; 
            
            if context_variance <= config.min_context_variance
                && estimated_speedup >= config.min_speedup
            {
                let priority = calculate_priority(
                    branch_pattern.branch_ratio as usize * 10, 
                    branch_pattern.average_confidence,
                    context_variance,
                    reward_correlation,
                    estimated_speedup,
                );
                
                candidates.push(FossilizationCandidate {
                    pattern_type: "BranchPattern".to_string(),
                    pattern_id: Uuid::new_v4(),
                    repetition_count: (branch_pattern.branch_ratio * 10.0) as usize,
                    average_confidence: branch_pattern.average_confidence,
                    context_variance,
                    reward_correlation,
                    estimated_speedup,
                    priority,
                });
            }
        }
    }
    
    
    candidates.sort_by(|a, b| b.priority.partial_cmp(&a.priority).unwrap_or(std::cmp::Ordering::Equal));
    
    candidates
}

fn estimate_speedup(occurrence_count: usize) -> f64 {
    
    
    (occurrence_count as f64 / 10.0).min(10.0)
}

fn calculate_priority(
    repetition_count: usize,
    confidence: f64,
    context_variance: f64,
    reward_correlation: f64,
    speedup: f64,
) -> f64 {
    
    let rep_factor = (repetition_count as f64 / 100.0).min(1.0);
    let conf_factor = confidence;
    let var_factor = 1.0 - context_variance; 
    let reward_factor = reward_correlation;
    let speedup_factor = (speedup / 10.0).min(1.0);
    
    (rep_factor * 0.2 + conf_factor * 0.3 + var_factor * 0.2 + reward_factor * 0.2 + speedup_factor * 0.1).min(1.0)
}
