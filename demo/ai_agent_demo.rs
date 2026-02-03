use c_mer::*;
use std::io::{self, Write};

#[cfg(feature = "openai")]
use c_mer::llm_integration::openai_provider::OpenAIProvider;

const DEFAULT_MEMORY_FILE: &str = "demo/memory.cmca";

fn main() {
    #[cfg(feature = "openai")]
    {
        
        let api_key = std::env::var("OPENAI_API_KEY")
            .expect("OPENAI_API_KEY not set. Get your key from https:
        let llm_provider = Box::new(OpenAIProvider::new(api_key));
        
        let mut agent = AIAgent::new(llm_provider);
        
        
        agent.set_debug(true);
        
        
        let stats = agent.stats();
        if stats.fragments > 0 {
            println!("📚 Loaded memory: {} fragments, {} edges, {} compiled modules", 
                stats.fragments, stats.edges, stats.compiled_modules);
        }
        
        println!("🤖 AI Agent ready! Type your messages (or 'quit' to exit)\n");
        
        loop {
            print!("You: ");
            io::stdout().flush().unwrap();
            
            let mut input = String::new();
            match io::stdin().read_line(&mut input) {
                Ok(_) => {
                    let input = input.trim();
                    
                    if input.is_empty() {
                        continue;
                    }
                    
                    if input.eq_ignore_ascii_case("quit") || input.eq_ignore_ascii_case("exit") {
                        
                        match agent.save(DEFAULT_MEMORY_FILE) {
                            Ok(_) => {
                                let stats = agent.stats();
                                println!("💾 Saved memory to {} ({} fragments, {} edges)", 
                                    DEFAULT_MEMORY_FILE, stats.fragments, stats.edges);
                            }
                            Err(e) => println!("⚠️  Warning: Failed to save memory: {:?}", e),
                        }
                        println!("Goodbye!");
                        break;
                    }
                    
                    match agent.chat(input) {
                        Ok(response) => {
                            println!("Agent: {}\n", response);
                            
                            if let Err(e) = agent.save(DEFAULT_MEMORY_FILE) {
                                eprintln!("⚠️  Warning: Failed to auto-save memory: {:?}", e);
                            }
                        }
                        Err(e) => println!("Error: {:?}\n", e),
                    }
                }
                Err(e) => {
                    println!("Error reading input: {}\n", e);
                    
                    let _ = agent.save(DEFAULT_MEMORY_FILE);
                    break;
                }
            }
        }
    }
    
    #[cfg(not(feature = "openai"))]
    {
        
        println!("Note: OpenAI feature not enabled. Using mock provider.");
        println!("To use OpenAI: cargo run --example ai_agent_demo --features openai");
        println!("And set OPENAI_API_KEY environment variable.\n");
        
        let llm_provider = Box::new(MockLLMProvider);
        let mut agent = AIAgent::new(llm_provider);
        
        
        agent.set_debug(true);
        
        
        let stats = agent.stats();
        if stats.fragments > 0 {
            println!("📚 Loaded memory: {} fragments, {} edges, {} compiled modules", 
                stats.fragments, stats.edges, stats.compiled_modules);
        }
        
        println!("🤖 AI Agent ready! Type your messages (or 'quit' to exit)\n");
        
        loop {
            print!("You: ");
            io::stdout().flush().unwrap();
            
            let mut input = String::new();
            match io::stdin().read_line(&mut input) {
                Ok(_) => {
                    let input = input.trim();
                    
                    if input.is_empty() {
                        continue;
                    }
                    
                    if input.eq_ignore_ascii_case("quit") || input.eq_ignore_ascii_case("exit") {
                        
                        match agent.save(DEFAULT_MEMORY_FILE) {
                            Ok(_) => {
                                let stats = agent.stats();
                                println!("💾 Saved memory to {} ({} fragments, {} edges)", 
                                    DEFAULT_MEMORY_FILE, stats.fragments, stats.edges);
                            }
                            Err(e) => println!("⚠️  Warning: Failed to save memory: {:?}", e),
                        }
                        println!("Goodbye!");
                        break;
                    }
                    
                    match agent.chat(input) {
                        Ok(response) => {
                            println!("Agent: {}\n", response);
                            
                            if let Err(e) = agent.save(DEFAULT_MEMORY_FILE) {
                                eprintln!("⚠️  Warning: Failed to auto-save memory: {:?}", e);
                            }
                        }
                        Err(e) => println!("Error: {:?}\n", e),
                    }
                }
                Err(e) => {
                    println!("Error reading input: {}\n", e);
                    
                    let _ = agent.save(DEFAULT_MEMORY_FILE);
                    break;
                }
            }
        }
    }
}


struct MockLLMProvider;

impl LLMProvider for MockLLMProvider {
    fn extract_semantics(&self, text: &str) -> std::result::Result<SemanticEvent, LLMError> {
        Ok(ingest_conversation(text))
    }
    
    fn format_response_from_memory(&self, user_query: &str, memory_data: &MemoryData) -> std::result::Result<String, LLMError> {
        if memory_data.fragments.is_empty() {
            return Ok("I don't have that information yet.".to_string());
        }
        
        
        let mut parts = Vec::new();
        for frag in &memory_data.fragments {
            for (key, value) in &frag.content {
                if !value.is_empty() && value != "unknown" && key != "key" {
                    parts.push(format!("{}: {}", key, value));
                }
            }
        }
        
        if parts.is_empty() {
            Ok("I don't have that information yet.".to_string())
        } else {
            Ok(parts.join(", "))
        }
    }
    
    fn format_response(&self, result: &ExecutionResult, _context: &ContextVector) -> std::result::Result<String, LLMError> {
        Ok(result.outcome.result.clone())
    }
    
    fn extract_goal_and_domain(&self, _query: &str) -> std::result::Result<(String, String), LLMError> {
        Ok(("general".to_string(), "general".to_string()))
    }
}
