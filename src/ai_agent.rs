// Copyright (c) 2026 Nolan Taft
use crate::types::*;
use crate::llm_integration::*;





pub struct AIAgent {
    agent: CMCAgent,
}

impl AIAgent {
    
    
    pub fn new(llm: Box<dyn LLMProvider>) -> Self {
        const DEFAULT_MEMORY_FILE: &str = "memory.cmca";
        let memory = MemoryGraph::load(DEFAULT_MEMORY_FILE)
            .unwrap_or_else(|_| MemoryGraph::new());
        AIAgent {
            agent: CMCAgent::with_memory(llm, memory),
        }
    }
    
    
    pub fn new_empty(llm: Box<dyn LLMProvider>) -> Self {
        AIAgent {
            agent: CMCAgent::new(llm),
        }
    }
    
    
    pub fn new_with_memory_file(llm: Box<dyn LLMProvider>, path: impl AsRef<std::path::Path>) -> crate::storage::Result<Self> {
        let memory = MemoryGraph::load(path)?;
        Ok(AIAgent {
            agent: CMCAgent::with_memory(llm, memory),
        })
    }
    
    
    pub fn with_memory(llm: Box<dyn LLMProvider>, memory: MemoryGraph) -> Self {
        AIAgent {
            agent: CMCAgent::with_memory(llm, memory),
        }
    }
    
    
    
    pub fn chat(&mut self, user_input: &str) -> Result<String, LLMError> {
        self.agent.process(user_input)
    }
    
    
    pub fn stats(&self) -> MemoryStats {
        self.agent.memory_stats()
    }
    
    
    pub fn save(&self, path: impl AsRef<std::path::Path>) -> crate::storage::Result<()> {
        self.agent.save_memory(path)
    }
    
    
    pub fn load(path: impl AsRef<std::path::Path>) -> crate::storage::Result<MemoryGraph> {
        CMCAgent::load_memory(path)
    }
    
    
    pub fn history(&self) -> &[ConversationTurn] {
        self.agent.conversation_history()
    }
    
    
    pub fn set_debug(&mut self, enabled: bool) {
        self.agent.set_debug(enabled);
    }
}
