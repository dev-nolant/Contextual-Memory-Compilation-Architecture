// Copyright (c) 2026 Nolan Taft
use c_mer::*;
use std::io::{self, Write};
use std::path::Path;

const DEFAULT_MEMORY_FILE: &str = "memory.cmca";

fn main() {
    println!("CMCA CLI - Contextual Memory Compilation Architecture");
    println!("Type 'help' for commands\n");

    let mut memory = match MemoryGraph::load(DEFAULT_MEMORY_FILE) {
        Ok(mem) => {
            println!(
                "Loaded memory from {} ({} fragments, {} edges)",
                DEFAULT_MEMORY_FILE,
                mem.fragments.len(),
                mem.edges.len()
            );
            mem
        }
        Err(_) => {
            println!("Starting with new memory (no existing memory file found)");
            MemoryGraph::new()
        }
    };
    let mut last_eeg: Option<EEG> = None;

    loop {
        print!("cmca> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            break;
        }

        let parts: Vec<&str> = input.trim().splitn(3, ' ').collect();
        let command = parts[0];

        match command {
            "ingest" => {
                if parts.len() < 2 {
                    println!("Usage: ingest <text>");
                    continue;
                }
                let text = parts[1..].join("");

                let event = ingest_conversation_enhanced(&text);
                let fragments = distill_event(&event);

                for fragment in &fragments {
                    memory.insert_fragment(fragment.clone(), Vec::new());
                }

                println!(
                    "Ingested: {} atoms, created {} fragments",
                    event.atoms.len(),
                    fragments.len()
                );

                if !event.atoms.is_empty() {
                    println!("\n  Extracted Atoms:");
                    let mut atom_type_counts: std::collections::HashMap<String, usize> =
                        std::collections::HashMap::new();
                    for atom in &event.atoms {
                        let type_name = format!("{:?}", atom.atom_type);
                        *atom_type_counts.entry(type_name).or_insert(0) += 1;
                    }
                    for (atom_type, count) in &atom_type_counts {
                        println!("• {}: {}", atom_type, count);
                    }
                }

                if !fragments.is_empty() {
                    println!("\n  Created Fragments:");
                    let mut fragment_type_counts: std::collections::HashMap<String, usize> =
                        std::collections::HashMap::new();
                    for fragment in &fragments {
                        let type_name = format!("{:?}", fragment.fragment_type);
                        *fragment_type_counts.entry(type_name).or_insert(0) += 1;
                    }
                    for (fragment_type, count) in &fragment_type_counts {
                        println!("• {}: {}", fragment_type, count);
                    }

                    println!("\n  Sample Fragments:");
                    let mut shown = 0;
                    for fragment in &fragments {
                        if shown >= 3 {
                            break;
                        }
                        match &fragment.content {
                            FragmentContent::PersonalFact {
                                person,
                                fact_type,
                                value,
                                ..
                            } => {
                                println!(
                                    "• PersonalFact: {} {} {} (conf: {:.2})",
                                    person, fact_type, value, fragment.confidence
                                );
                            }
                            FragmentContent::EntityRelation {
                                entity,
                                relation,
                                target,
                            } => {
                                println!(
                                    "• EntityRelation: {} {} {} (conf: {:.2})",
                                    entity, relation, target, fragment.confidence
                                );
                            }
                            FragmentContent::CausalRule {
                                condition, outcome, ..
                            } => {
                                println!(
                                    "• CausalRule: {} → {} (conf: {:.2})",
                                    condition, outcome, fragment.confidence
                                );
                            }
                            FragmentContent::TemporalEvent {
                                event,
                                time_expression,
                                ..
                            } => {
                                println!(
                                    "• TemporalEvent: {} at {} (conf: {:.2})",
                                    event, time_expression, fragment.confidence
                                );
                            }
                            FragmentContent::SpatialRelation {
                                entity, location, ..
                            } => {
                                println!(
                                    "• SpatialRelation: {} at {} (conf: {:.2})",
                                    entity, location, fragment.confidence
                                );
                            }
                            FragmentContent::Preference {
                                preference, weight, ..
                            } => {
                                println!(
                                    "• Preference: {} (weight: {:.2}, conf: {:.2})",
                                    preference, weight, fragment.confidence
                                );
                            }
                            _ => {
                                println!(
                                    "• {:?} (conf: {:.2})",
                                    fragment.fragment_type, fragment.confidence
                                );
                            }
                        }
                        shown += 1;
                    }
                }

                if !event.relationships.is_empty() {
                    println!(
                        "\n  Relationships: {} connections",
                        event.relationships.len()
                    );
                }
            }
            "compile" => {
                if parts.len() < 3 {
                    println!("Usage: compile <goal> <domain>");
                    continue;
                }
                let goal = parts[1];
                let domain = parts[2];
                let context = generate_context(goal, domain, 0.2);
                let eeg = compile_thought(&context, &mut memory);
                last_eeg = Some(eeg.clone());

                println!("Compiled EEG:");
                println!("Nodes: {}", eeg.nodes.len());
                println!("Edges: {}", eeg.edges.len());
                println!("Confidence: {:.2}", eeg.metadata.confidence_score);
            }
            "execute" => {
                if let Some(ref eeg) = last_eeg {
                    let result = execute_eeg(eeg, &mut memory);
                    println!("Execution Result:");
                    println!("Outcome: {:?}", result.outcome.outcome_type);
                    println!("Result: {}", result.outcome.result);
                    println!("Trace length: {}", result.execution_trace.len());
                    println!("Time: {:.3}s", result.time_taken);

                    for signal in &result.reinforcement_signals {
                        memory.reinforce_fragment(signal.fragment_id, &result.outcome);
                    }
                } else {
                    println!("No EEG compiled. Use 'compile' first.");
                }
            }
            "memory" => {
                println!("Memory Stats:");
                println!("Fragments: {}", memory.fragments.len());
                println!("Edges: {}", memory.edges.len());
                println!("Version: {}", memory.version);
            }
            "fragments" => {
                println!("Fragments ({}):", memory.fragments.len());
                for (id, fragment) in &memory.fragments {
                    println!(
                        "{}: {:?} (conf: {:.2}, sal: {:.2})",
                        id, fragment.fragment_type, fragment.confidence, fragment.salience
                    );
                }
            }
            "eeg" => {
                if let Some(ref eeg) = last_eeg {
                    println!("EEG Structure:");
                    println!("Entry: {}", eeg.entry_point);
                    println!("Exits: {:?}", eeg.exit_points);
                    println!("Nodes:");
                    for (id, node) in &eeg.nodes {
                        println!(
                            "{}: {:?} (conf: {:.2})",
                            id, node.node_type, node.confidence
                        );
                    }
                    println!("Edges:");
                    for edge in &eeg.edges {
                        println!("{} -> {}", edge.from_node, edge.to_node);
                    }
                } else {
                    println!("No EEG compiled. Use 'compile' first.");
                }
            }
            "save" => {
                let path = if parts.len() >= 2 {
                    parts[1]
                } else {
                    DEFAULT_MEMORY_FILE
                };
                match memory.save(path) {
                    Ok(()) => println!("Memory saved to {}", path),
                    Err(e) => println!("Error saving memory: {}", e),
                }
            }
            "load" => {
                let path = if parts.len() >= 2 {
                    parts[1]
                } else {
                    DEFAULT_MEMORY_FILE
                };
                match MemoryGraph::load(path) {
                    Ok(mem) => {
                        memory = mem;
                        println!(
                            "Memory loaded from {} ({} fragments, {} edges)",
                            path,
                            memory.fragments.len(),
                            memory.edges.len()
                        );
                    }
                    Err(e) => println!("Error loading memory: {}", e),
                }
            }
            "help" => {
                println!("Commands:");
                println!("ingest <text>     - Ingest conversation/experience");
                println!("compile <goal> <domain> - Compile thought for goal");
                println!("execute           - Execute last compiled EEG");
                println!("memory            - Show memory statistics");
                println!("fragments         - List all fragments");
                println!("eeg               - Show last compiled EEG structure");
                println!(
                    "save [path]       - Save memory to file (default: {})",
                    DEFAULT_MEMORY_FILE
                );
                println!(
                    "load [path]       - Load memory from file (default: {})",
                    DEFAULT_MEMORY_FILE
                );
                println!("help              - Show this help");
                println!("quit/exit         - Exit");
            }
            "quit" | "exit" => {
                if let Err(e) = memory.save(DEFAULT_MEMORY_FILE) {
                    eprintln!("Warning: Failed to auto-save memory: {}", e);
                }
                break;
            }
            "" => continue,
            _ => {
                println!("Unknown command: {}. Type 'help' for commands.", command);
            }
        }
    }
}
