use c_mer::*;
use std::path::Path;

fn main() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║  CMCA Platform Demonstration                                ║");
    println!("║  Contextual Memory Compilation Architecture                  ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    let memory_file = "demo/demo_memory.cmca";

    let mut memory = match MemoryGraph::load(memory_file) {
        Ok(mem) => {
            println!("Loaded existing memory from {}", memory_file);
            println!("- Fragments: {}", mem.fragments.len());
            println!("- Edges: {}", mem.edges.len());
            println!("- Compiled Modules: {}\n", mem.compiled_modules.len());
            mem
        }
        Err(_) => {
            println!("Starting with fresh memory\n");
            MemoryGraph::new()
        }
    };

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("PHASE 1: INGESTION - Learning from Conversations");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let experiences = vec![
        "I'm getting a 404 error when calling the API endpoint. The HTTP request fails.",
        "When I debug HTTP errors, I check the URL first, then verify the server status.",
        "API calls to /users endpoint return 200 success when the server is running.",
        "If the server is down, HTTP requests timeout and cause 500 errors.",
    ];

    for (i, experience) in experiences.iter().enumerate() {
        println!("[Experience {}] {}", i + 1, experience);

        let event = ingest_conversation(experience);
        println!(
            "→ Created semantic event with {} atoms, {} relationships",
            event.atoms.len(),
            event.relationships.len()
        );

        let fragments = distill_event(&event);
        println!("→ Distilled into {} memory fragments", fragments.len());

        for fragment in &fragments {
            memory.insert_fragment(fragment.clone(), Vec::new());
        }

        for fragment in &fragments {
            match &fragment.content {
                FragmentContent::CausalRule {
                    condition,
                    outcome,
                    confidence,
                } => {
                    println!(
                        "• Causal Rule: {} → {} (conf: {:.2})",
                        condition, outcome, confidence
                    );
                }
                FragmentContent::EntityRelation {
                    entity,
                    relation,
                    target,
                } => {
                    println!(
                        "• Entity Relation: {} {} {} (conf: {:.2})",
                        entity, relation, target, fragment.confidence
                    );
                }
                FragmentContent::GoalStrategy {
                    goal,
                    strategy,
                    success_rate,
                } => {
                    println!(
                        "• Goal Strategy: {} → {} (success: {:.2})",
                        goal, strategy, success_rate
                    );
                }
                _ => {}
            }
        }
        println!();
    }

    println!("Memory after ingestion:");
    println!("- Total fragments: {}", memory.fragments.len());
    println!("- Total edges: {}\n", memory.edges.len());

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("PHASE 2: COMPILATION - Building Execution Graphs (EEGs)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let goals = vec![
        ("debug HTTP 404 error", "web_development"),
        ("fix API endpoint issue", "api_integration"),
    ];

    let mut compiled_eegs = Vec::new();

    for (goal, domain) in &goals {
        println!("[Compiling] Goal: '{}' in domain: '{}'", goal, domain);

        let context = generate_context(goal, domain, 0.3);

        let activated = memory.activate_fragments(&context);
        println!("→ Activated {} relevant fragments", activated.len());

        let eeg = compile_thought(&context, &mut memory);
        compiled_eegs.push(eeg.clone());

        println!("→ Compiled EEG:");
        println!("- Nodes: {}", eeg.nodes.len());
        println!("- Edges: {}", eeg.edges.len());
        println!("- Entry point: {}", eeg.entry_point);
        println!("- Exit points: {}", eeg.exit_points.len());
        println!("- Confidence: {:.2}", eeg.metadata.confidence_score);
        println!(
            "- Estimated execution time: {:.2}s",
            eeg.metadata.estimated_execution_time
        );

        for (id, node) in &eeg.nodes {
            match &node.content {
                NodeContent::Fragment { interpretation, .. } => {
                    println!(
                        "• Node {}: {} (conf: {:.2})",
                        id, interpretation, node.confidence
                    );
                }
                NodeContent::GapFill {
                    gap_description, ..
                } => {
                    println!(
                        "• Node {}: Gap - {} (conf: {:.2})",
                        id, gap_description, node.confidence
                    );
                }
                NodeContent::Decision { condition, .. } => {
                    println!(
                        "• Node {}: Decision - {} (conf: {:.2})",
                        id, condition, node.confidence
                    );
                }
                _ => {}
            }
        }
        println!();
    }

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("PHASE 3: EXECUTION - Running Compiled EEGs");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    for (i, eeg) in compiled_eegs.iter().enumerate() {
        println!("[Executing EEG {}]", i + 1);

        let result = execute_eeg(eeg, &mut memory);

        println!("→ Execution Result:");
        println!("- Outcome: {:?}", result.outcome.outcome_type);
        println!("- Result: {}", result.outcome.result);
        println!("- Confidence: {:.2}", result.confidence);
        println!("- Time taken: {:.3}s", result.time_taken);
        println!("- Execution trace: {} nodes", result.execution_trace.len());

        println!("→ Reinforcement Signals:");
        for signal in &result.reinforcement_signals {
            println!(
                "• Fragment {}: {:?} (strength: {:.2}) - {}",
                signal.fragment_id, signal.signal_type, signal.strength, signal.reason
            );
            memory.reinforce_fragment(signal.fragment_id, &result.outcome);
        }
        println!();
    }

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("PHASE 4: MEMORY PERSISTENCE - Save/Load");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    println!("[Saving] Memory to {}", memory_file);
    match memory.save(memory_file) {
        Ok(()) => {
            println!("Successfully saved memory");
            println!("- Fragments: {}", memory.fragments.len());
            println!("- Edges: {}", memory.edges.len());
            println!("- Compiled modules: {}\n", memory.compiled_modules.len());
        }
        Err(e) => {
            println!("Error saving memory: {}\n", e);
        }
    }

    println!("[Loading] Memory from {}", memory_file);
    match MemoryGraph::load(memory_file) {
        Ok(loaded_memory) => {
            println!("Successfully loaded memory");
            println!("- Fragments: {}", loaded_memory.fragments.len());
            println!("- Edges: {}", loaded_memory.edges.len());
            println!(
                "- Compiled modules: {}",
                loaded_memory.compiled_modules.len()
            );

            println!("\n  Sample fragments:");
            let mut count = 0;
            for (id, fragment) in &loaded_memory.fragments {
                if count >= 3 {
                    break;
                }
                match &fragment.content {
                    FragmentContent::CausalRule {
                        condition, outcome, ..
                    } => {
                        println!(
                            "• {}: {} → {} (conf: {:.2}, reinforced: {}x)",
                            id,
                            condition,
                            outcome,
                            fragment.confidence,
                            fragment.reinforcement_count
                        );
                    }
                    FragmentContent::EntityRelation {
                        entity,
                        relation,
                        target,
                    } => {
                        println!(
                            "• {}: {} {} {} (conf: {:.2})",
                            id, entity, relation, target, fragment.confidence
                        );
                    }
                    _ => {}
                }
                count += 1;
            }
        }
        Err(e) => {
            println!("Error loading memory: {}", e);
        }
    }

    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("PHASE 5: MEMORY IMPROVEMENT - Learning Over Time");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    println!("[Demonstrating] Memory reinforcement and confidence improvement\n");

    if !memory.fragments.is_empty() {
        let sample_id = *memory.fragments.keys().next().unwrap();
        let fragment_before = memory.fragments.get(&sample_id).unwrap().clone();

        println!("Sample fragment before reinforcement:");
        println!("- ID: {}", sample_id);
        println!("- Confidence: {:.2}", fragment_before.confidence);
        println!(
            "- Reinforcement count: {}",
            fragment_before.reinforcement_count
        );

        let success_outcome = Outcome {
            outcome_type: OutcomeType::Success,
            result: "Successfully executed".to_string(),
            explanation: Some("Test reinforcement".to_string()),
            confidence: 0.9,
        };
        memory.reinforce_fragment(sample_id, &success_outcome);

        let fragment_after = memory.fragments.get(&sample_id).unwrap();
        println!("\nAfter reinforcement:");
        println!(
            "- Confidence: {:.2} (improved by {:.2})",
            fragment_after.confidence,
            fragment_after.confidence - fragment_before.confidence
        );
        println!(
            "- Reinforcement count: {}",
            fragment_after.reinforcement_count
        );
    }

    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("SUMMARY");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    println!("Ingested {} experiences", experiences.len());
    println!("Created {} memory fragments", memory.fragments.len());
    println!("Compiled {} execution graphs", compiled_eegs.len());
    println!("Executed all EEGs and reinforced memory");
    println!("Saved memory to persistent storage");
    println!("Demonstrated memory loading and persistence");
    println!("\nMemory file: {}", memory_file);
    println!("You can now use the CLI to interact with this memory:");
    println!("$ cargo run --bin c-mer");
    println!("cmca> load {}", memory_file);
    println!("cmca> memory");
    println!("cmca> fragments");
}
