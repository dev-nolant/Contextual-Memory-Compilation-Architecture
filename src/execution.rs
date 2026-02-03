// Copyright (c) 2026 Nolan Taft
use crate::types::*;
use std::collections::{HashMap, HashSet};

pub fn execute_eeg(eeg: &EEG, memory: &mut crate::types::MemoryGraph) -> ExecutionResult {
    execute_eeg_with_context(eeg, memory, None)
}

pub fn execute_eeg_with_context(
    eeg: &EEG,
    memory: &mut crate::types::MemoryGraph,
    _context: Option<&ContextVector>,
) -> ExecutionResult {
    let start_time = current_timestamp();
    let mut execution_trace = Vec::new();
    let mut reinforcement_signals = Vec::new();
    let mut branch_decisions = HashMap::new();
    let mut fragment_outcomes = Vec::new();

    let mut current_node_id = eeg.entry_point;
    let mut visited = HashSet::new();

    while let Some(node) = eeg.nodes.get(&current_node_id) {
        if visited.contains(&current_node_id) {
            break;
        }
        visited.insert(current_node_id);
        execution_trace.push(current_node_id);

        match &node.content {
            NodeContent::Fragment { fragment_id, .. } => {
                if let Some(fragment) = memory.fragments.get(fragment_id) {
                    let outcome = interpret_fragment(fragment);
                    fragment_outcomes.push(outcome.clone());
                    if outcome.outcome_type == OutcomeType::Success {
                        reinforcement_signals.push(ReinforcementSignal {
                            fragment_id: *fragment_id,
                            signal_type: SignalType::Positive,
                            strength: 0.8,
                            reason: "Successful execution".to_string(),
                        });
                    }
                }
            }
            NodeContent::GapFill { .. } => {}
            NodeContent::Decision { branches, .. } => {
                if let Some(branch) = branches.first() {
                    branch_decisions.insert(current_node_id, branch.target_node);
                    current_node_id = branch.target_node;
                    continue;
                }
            }
            NodeContent::Conflict {
                selected_fragment, ..
            } => {
                if let Some(frag_id) = selected_fragment {
                    current_node_id = *frag_id;
                    continue;
                }
            }
            NodeContent::Action { .. } => {}
        }

        let next_edge = eeg.edges.iter().find(|e| e.from_node == current_node_id);
        if let Some(edge) = next_edge {
            current_node_id = edge.to_node;
        } else {
            break;
        }
    }

    let time_taken = current_timestamp() - start_time;

    let outcome = if !fragment_outcomes.is_empty() {
        let results: Vec<String> = fragment_outcomes.iter().map(|o| o.result.clone()).collect();
        let combined_result = results.join("; ");

        Outcome {
            outcome_type: if execution_trace.len() > 1 {
                OutcomeType::Success
            } else {
                OutcomeType::Partial
            },
            result: combined_result,
            explanation: Some(format!("Executed {} fragments", fragment_outcomes.len())),
            confidence: eeg.metadata.confidence_score,
        }
    } else if execution_trace.len() > 1 {
        Outcome {
            outcome_type: OutcomeType::Success,
            result: format!("Executed {} nodes", execution_trace.len()),
            explanation: Some("Execution completed successfully".to_string()),
            confidence: eeg.metadata.confidence_score,
        }
    } else {
        Outcome {
            outcome_type: OutcomeType::Partial,
            result: "Minimal execution".to_string(),
            explanation: Some("Limited execution path".to_string()),
            confidence: 0.5,
        }
    };

    ExecutionResult {
        outcome,
        execution_trace,
        confidence: eeg.metadata.confidence_score,
        time_taken,
        reinforcement_signals,
    }
}

fn interpret_fragment(fragment: &MFragment) -> Outcome {
    match &fragment.content {
        FragmentContent::EntityRelation {
            entity,
            relation,
            target,
        } => Outcome {
            outcome_type: OutcomeType::Success,
            result: format!("{} {} {}", entity, relation, target),
            explanation: Some("Entity relation identified".to_string()),
            confidence: fragment.confidence,
        },
        FragmentContent::CausalRule {
            condition, outcome, ..
        } => Outcome {
            outcome_type: OutcomeType::Success,
            result: format!("{} -> {}", condition, outcome),
            explanation: Some("Causal rule applied".to_string()),
            confidence: fragment.confidence,
        },
        FragmentContent::GoalStrategy { goal, strategy, .. } => Outcome {
            outcome_type: OutcomeType::Success,
            result: format!("Goal: {}, Strategy: {}", goal, strategy),
            explanation: Some("Goal-strategy pair identified".to_string()),
            confidence: fragment.confidence,
        },
        FragmentContent::Constraint { constraint, .. } => Outcome {
            outcome_type: OutcomeType::Success,
            result: constraint.clone(),
            explanation: Some("Constraint identified".to_string()),
            confidence: fragment.confidence,
        },
        FragmentContent::Preference { preference, .. } => Outcome {
            outcome_type: OutcomeType::Success,
            result: preference.clone(),
            explanation: Some("Preference identified".to_string()),
            confidence: fragment.confidence,
        },
        FragmentContent::ContextSignature { pattern, .. } => Outcome {
            outcome_type: OutcomeType::Success,
            result: pattern.clone(),
            explanation: Some("Context signature identified".to_string()),
            confidence: fragment.confidence,
        },
        FragmentContent::PersonalFact {
            person,
            fact_type,
            value,
            ..
        } => Outcome {
            outcome_type: OutcomeType::Success,
            result: format!("{} {} {}", person, fact_type, value),
            explanation: Some("Personal fact identified".to_string()),
            confidence: fragment.confidence,
        },
        FragmentContent::TemporalEvent {
            event,
            time_expression,
            ..
        } => Outcome {
            outcome_type: OutcomeType::Success,
            result: format!("{} at {}", event, time_expression),
            explanation: Some("Temporal event identified".to_string()),
            confidence: fragment.confidence,
        },
        FragmentContent::SpatialRelation {
            entity, location, ..
        } => Outcome {
            outcome_type: OutcomeType::Success,
            result: format!("{} at {}", entity, location),
            explanation: Some("Spatial relation identified".to_string()),
            confidence: fragment.confidence,
        },
        FragmentContent::QuantitativeFact {
            entity,
            quantity,
            unit,
            ..
        } => Outcome {
            outcome_type: OutcomeType::Success,
            result: format!(
                "{}: {} {}",
                entity,
                quantity,
                unit.as_ref().unwrap_or(&"".to_string())
            ),
            explanation: Some("Quantitative fact identified".to_string()),
            confidence: fragment.confidence,
        },
        FragmentContent::HierarchicalRelation { parent, child, .. } => Outcome {
            outcome_type: OutcomeType::Success,
            result: format!("{} contains {}", parent, child),
            explanation: Some("Hierarchical relation identified".to_string()),
            confidence: fragment.confidence,
        },
        FragmentContent::SocialRelation {
            person1,
            person2,
            relation_type,
            ..
        } => Outcome {
            outcome_type: OutcomeType::Success,
            result: format!("{} {} {}", person1, relation_type, person2),
            explanation: Some("Social relation identified".to_string()),
            confidence: fragment.confidence,
        },
        FragmentContent::OwnershipRelation { owner, owned, .. } => Outcome {
            outcome_type: OutcomeType::Success,
            result: format!("{} owns {}", owner, owned),
            explanation: Some("Ownership relation identified".to_string()),
            confidence: fragment.confidence,
        },
        FragmentContent::StateTransition {
            entity,
            from_state,
            to_state,
            ..
        } => Outcome {
            outcome_type: OutcomeType::Success,
            result: format!("{}: {} -> {}", entity, from_state, to_state),
            explanation: Some("State transition identified".to_string()),
            confidence: fragment.confidence,
        },
        FragmentContent::Capability {
            entity, capability, ..
        } => Outcome {
            outcome_type: OutcomeType::Success,
            result: format!("{} can {}", entity, capability),
            explanation: Some("Capability identified".to_string()),
            confidence: fragment.confidence,
        },
        FragmentContent::Belief { entity, belief, .. } => Outcome {
            outcome_type: OutcomeType::Success,
            result: format!("{} believes {}", entity, belief),
            explanation: Some("Belief identified".to_string()),
            confidence: fragment.confidence,
        },
        FragmentContent::SemanticAtom {
            atom_type, content, ..
        } => {
            let content_str: Vec<String> = content
                .iter()
                .map(|(k, v)| format!("{}: {}", k, v))
                .collect();
            let content_display = content_str.join(", ");
            Outcome {
                outcome_type: OutcomeType::Success,
                result: format!("Atom type: {:?}, Content: {}", atom_type, content_display),
                explanation: Some("Semantic atom identified".to_string()),
                confidence: fragment.confidence,
            }
        }
    }
}
