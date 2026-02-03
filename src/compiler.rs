// Copyright (c) 2026 Nolan Taft
use crate::types::*;
use crate::fossilization::*;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

pub fn compile_thought(context: &ContextVector, memory: &mut crate::types::MemoryGraph) -> EEG {
    compile_thought_with_modules(context, memory, None)
}

pub fn compile_thought_with_modules(
    context: &ContextVector,
    memory: &mut crate::types::MemoryGraph,
    compiled_modules: Option<&[CompiledModule]>,
) -> EEG {
    
    if let Some(modules) = compiled_modules {
        if let Some(module) = find_applicable_module(context, modules) {
            
            return create_eeg_from_module(module, context);
        }
    }
    
    
    let activated = memory.activate_fragments(context);
    
    if activated.is_empty() {
        return create_empty_eeg(&context);
    }
    
    let resolved = resolve_conflicts(&activated, memory);
    let filled = fill_gaps(&resolved, memory, context);
    let ordered = order_fragments(&filled, memory);
    let branched = add_branching(&ordered, memory, context);
    let pruned = prune_resources(&branched, context);
    construct_eeg(&pruned, context, memory)
}

fn create_eeg_from_module(module: &CompiledModule, _context: &ContextVector) -> EEG {
    
    
    let node_id = Uuid::new_v4();
    let mut nodes = HashMap::new();
    
    nodes.insert(
        node_id,
        EEGNode {
            id: node_id,
            node_type: NodeType::FragmentNode,
            content: NodeContent::Action {
                action_type: format!("CompiledModule_{:?}", module.module_type),
                parameters: HashMap::new(),
            },
            confidence: module.confidence,
            source_fragments: vec![module.source_pattern],
            execution_cost: 0.1, 
        },
    );
    
    EEG {
        nodes,
        edges: Vec::new(),
        entry_point: node_id,
        exit_points: vec![node_id],
        metadata: EEGMetadata {
            compilation_timestamp: current_timestamp(),
            fragment_count: 1,
            estimated_execution_time: 0.1,
            confidence_score: module.confidence,
        },
    }
}

fn resolve_conflicts(activated: &HashSet<Uuid>, memory: &crate::types::MemoryGraph) -> Vec<Uuid> {
    let mut resolved = Vec::new();
    let mut conflicts = Vec::new();
    
    let fragment_list: Vec<Uuid> = activated.iter().copied().collect();
    
    for i in 0..fragment_list.len() {
        let frag1_id = fragment_list[i];
        if let Some(frag1) = memory.fragments.get(&frag1_id) {
            for j in (i + 1)..fragment_list.len() {
                let frag2_id = fragment_list[j];
                if let Some(frag2) = memory.fragments.get(&frag2_id) {
                    if check_conflict(frag1, frag2) {
                        conflicts.push((frag1_id, frag2_id));
                    }
                }
            }
            
            resolved.push(frag1_id);
        }
    }
    
    resolved
}

fn check_conflict(frag1: &MFragment, frag2: &MFragment) -> bool {
    match (&frag1.content, &frag2.content) {
        (
            FragmentContent::EntityRelation { entity: e1, relation: r1, target: t1 },
            FragmentContent::EntityRelation { entity: e2, relation: r2, target: t2 },
        ) => {
            e1 == e2 && r1 == r2 && t1 != t2
        }
        (
            FragmentContent::CausalRule { condition: c1, outcome: o1, .. },
            FragmentContent::CausalRule { condition: c2, outcome: o2, .. },
        ) => {
            c1 == c2 && o1 != o2
        }
        _ => false,
    }
}

fn fill_gaps(
    resolved: &[Uuid],
    memory: &crate::types::MemoryGraph,
    _context: &ContextVector,
) -> Vec<OrderedNode> {
    let mut nodes = Vec::new();
    
    for &frag_id in resolved {
        if let Some(fragment) = memory.fragments.get(&frag_id) {
            nodes.push(OrderedNode {
                id: frag_id,
                node_type: NodeType::FragmentNode,
                order: nodes.len(),
                confidence: fragment.confidence,
            });
        }
    }
    
    if nodes.len() < 3 {
        let gap_id = Uuid::new_v4();
        nodes.push(OrderedNode {
            id: gap_id,
            node_type: NodeType::GapFillNode,
            order: nodes.len(),
            confidence: 0.5,
        });
    }
    
    nodes
}

#[derive(Debug, Clone)]
struct OrderedNode {
    id: Uuid,
    node_type: NodeType,
    order: usize,
    confidence: f64,
}

fn order_fragments(ordered_nodes: &[OrderedNode], memory: &crate::types::MemoryGraph) -> Vec<OrderedNode> {
    let mut result = ordered_nodes.to_vec();
    
    result.sort_by(|a, b| {
        let a_conf = memory.fragments.get(&a.id).map(|f| f.confidence).unwrap_or(a.confidence);
        let b_conf = memory.fragments.get(&b.id).map(|f| f.confidence).unwrap_or(b.confidence);
        b_conf.partial_cmp(&a_conf).unwrap_or(std::cmp::Ordering::Equal)
    });
    
    for (i, node) in result.iter_mut().enumerate() {
        node.order = i;
    }
    
    result
}

fn add_branching(
    ordered: &[OrderedNode],
    memory: &crate::types::MemoryGraph,
    _context: &ContextVector,
) -> Vec<OrderedNode> {
    let mut branched = Vec::new();
    
    for node in ordered {
        branched.push(node.clone());
        
        if let NodeType::FragmentNode = node.node_type {
            if let Some(fragment) = memory.fragments.get(&node.id) {
                if let FragmentContent::CausalRule { .. } = &fragment.content {
                    let decision_id = Uuid::new_v4();
                    branched.push(OrderedNode {
                        id: decision_id,
                        node_type: NodeType::DecisionNode,
                        order: branched.len(),
                        confidence: 0.8,
                    });
                }
            }
        }
    }
    
    branched
}

fn prune_resources(branched: &[OrderedNode], context: &ContextVector) -> Vec<OrderedNode> {
    let threshold = context.confidence_threshold + (context.time_pressure * 0.2);
    
    branched
        .iter()
        .filter(|node| node.confidence >= threshold)
        .cloned()
        .collect()
}

fn construct_eeg(
    pruned: &[OrderedNode],
    context: &ContextVector,
    memory: &crate::types::MemoryGraph,
) -> EEG {
    let mut nodes = HashMap::new();
    let mut edges = Vec::new();
    
    if pruned.is_empty() {
        return create_empty_eeg(&context);
    }
    
    let entry_point = pruned[0].id;
    let mut exit_points = Vec::new();
    
    for node in pruned {
        let eeg_node = match node.node_type {
            NodeType::FragmentNode => {
                if let Some(fragment) = memory.fragments.get(&node.id) {
                    EEGNode {
                        id: node.id,
                        node_type: NodeType::FragmentNode,
                        content: NodeContent::Fragment {
                            fragment_id: node.id,
                            interpretation: format!("{:?}", fragment.fragment_type),
                        },
                        confidence: fragment.confidence,
                        source_fragments: vec![node.id],
                        execution_cost: 1.0,
                    }
                } else {
                    continue;
                }
            }
            NodeType::GapFillNode => EEGNode {
                id: node.id,
                node_type: NodeType::GapFillNode,
                content: NodeContent::GapFill {
                    gap_description: "Missing link".to_string(),
                    estimated_confidence: 0.5,
                },
                confidence: 0.5,
                source_fragments: Vec::new(),
                execution_cost: 1.0,
            },
            NodeType::DecisionNode => EEGNode {
                id: node.id,
                node_type: NodeType::DecisionNode,
                content: NodeContent::Decision {
                    condition: "check_condition".to_string(),
                    branches: Vec::new(),
                },
                confidence: 0.8,
                source_fragments: Vec::new(),
                execution_cost: 1.0,
            },
            _ => continue,
        };
        
        nodes.insert(node.id, eeg_node);
        
        if node.order == pruned.len() - 1 {
            exit_points.push(node.id);
        }
    }
    
    for i in 0..pruned.len().saturating_sub(1) {
        edges.push(EEGEdge {
            from_node: pruned[i].id,
            to_node: pruned[i + 1].id,
            edge_type: EdgeType::Causal,
            condition: None,
            weight: 1.0,
        });
    }
    
    if exit_points.is_empty() && !pruned.is_empty() {
        exit_points.push(pruned[pruned.len() - 1].id);
    }
    
    EEG {
        nodes,
        edges,
        entry_point,
        exit_points,
        metadata: EEGMetadata {
            compilation_timestamp: current_timestamp(),
            fragment_count: pruned.len(),
            estimated_execution_time: pruned.len() as f64,
            confidence_score: pruned.iter().map(|n| n.confidence).sum::<f64>() / pruned.len() as f64,
        },
    }
}

fn create_empty_eeg(_context: &ContextVector) -> EEG {
    let gap_id = uuid::Uuid::new_v4();
    let mut nodes = HashMap::new();
    nodes.insert(
        gap_id,
        EEGNode {
            id: gap_id,
            node_type: NodeType::GapFillNode,
            content: NodeContent::GapFill {
                gap_description: "No relevant memory".to_string(),
                estimated_confidence: 0.3,
            },
            confidence: 0.3,
            source_fragments: Vec::new(),
            execution_cost: 1.0,
        },
    );
    
    EEG {
        nodes,
        edges: Vec::new(),
        entry_point: gap_id,
        exit_points: vec![gap_id],
        metadata: EEGMetadata {
            compilation_timestamp: current_timestamp(),
            fragment_count: 0,
            estimated_execution_time: 1.0,
            confidence_score: 0.3,
        },
    }
}
