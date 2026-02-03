// Copyright (c) 2026 Nolan Taft
use crate::compiler::*;
use crate::context::*;
use crate::distillation::*;
use crate::execution::*;
use crate::types::*;
use uuid::Uuid;

pub trait LLMProvider {
    fn extract_semantics(&self, text: &str) -> Result<SemanticEvent, LLMError>;

    fn format_response_from_memory(
        &self,
        user_query: &str,
        memory_data: &MemoryData,
    ) -> Result<String, LLMError>;

    fn format_response_from_candidates(
        &self,
        user_query: &str,
        candidates: &[(String, f64)],
        memory_data: &MemoryData,
    ) -> Result<String, LLMError> {
        if candidates.is_empty() {
            return self.format_response_from_memory(user_query, memory_data);
        }

        let candidates_text = candidates
            .iter()
            .enumerate()
            .map(|(i, (value, conf))| format!("{}. {} (confidence: {:.2})", i + 1, value, conf))
            .collect::<Vec<_>>()
            .join("\n");

        let enhanced_query = format!(
            r#"
The user asked: "{}"

I found these top candidates from my memory:
{}

Please select the most appropriate answer based on the user's query. Consider:
- Which candidate best matches what the user is asking about?
- If the user asked a specific question (like "do I like X?"), check if X is in the candidates.
- Respond naturally and conversationally, as if you're choosing the best answer from these options.
"#,
            user_query, candidates_text
        );

        self.format_response_from_memory(&enhanced_query, memory_data)
    }

    fn format_response(
        &self,
        result: &ExecutionResult,
        context: &ContextVector,
    ) -> Result<String, LLMError> {
        let memory_data = MemoryData::from_execution_result(result);
        self.format_response_from_memory("", &memory_data)
    }

    fn extract_goal_and_domain(&self, query: &str) -> Result<(String, String), LLMError>;
}

#[derive(Debug, Clone)]
pub struct MemoryData {
    pub fragments: Vec<FragmentData>,

    pub query: String,

    pub confidence: f64,
}

#[derive(Debug, Clone)]
pub struct FragmentData {
    pub atom_type: String,

    pub content: std::collections::HashMap<String, String>,
}

impl MemoryData {
    pub fn from_execution_and_memory(
        execution_result: &ExecutionResult,
        memory: &MemoryGraph,
        user_query: &str,
    ) -> Self {
        let fragments: Vec<FragmentData> = execution_result
            .execution_trace
            .iter()
            .filter_map(|id| memory.fragments.get(id))
            .filter_map(|frag| match &frag.content {
                FragmentContent::SemanticAtom {
                    atom_type, content, ..
                } => Some(FragmentData {
                    atom_type: format!("{:?}", atom_type),
                    content: content.clone(),
                }),
                _ => None,
            })
            .collect();

        MemoryData {
            fragments,
            query: user_query.to_string(),
            confidence: execution_result.confidence,
        }
    }

    pub fn from_execution_result(result: &ExecutionResult) -> Self {
        MemoryData {
            fragments: Vec::new(),
            query: String::new(),
            confidence: result.confidence,
        }
    }

    pub fn to_compact_string(&self) -> String {
        if self.fragments.is_empty() {
            return "{}".to_string();
        }

        let frags: Vec<String> = self
            .fragments
            .iter()
            .map(|f| {
                let content: Vec<String> = f
                    .content
                    .iter()
                    .filter(|(k, v)| {
                        let has_meaningful_value = !v.is_empty() && v.as_str() != "unknown";

                        if k.as_str() == "key" {
                            has_meaningful_value && v.len() > 2 && v.as_str() != "key"
                        } else {
                            has_meaningful_value
                        }
                    })
                    .map(|(k, v)| format!("{}:{}", k, v))
                    .collect();
                format!("{{type:{},data:{}}}", f.atom_type, content.join(","))
            })
            .collect();

        format!("{{fragments:[{}],query:{}}}", frags.join("|"), self.query)
    }
}

#[derive(Debug)]
pub enum LLMError {
    NetworkError(String),
    ParseError(String),
    InvalidResponse(String),
    ProviderError(String),
}

pub struct CMCAgent {
    memory: MemoryGraph,
    llm: Box<dyn LLMProvider>,
    conversation_history: Vec<ConversationTurn>,
    debug: bool,
}

#[derive(Debug, Clone)]
pub struct ConversationTurn {
    pub user_input: String,
    pub context_used: ContextVector,
    pub eeg_compiled: Option<Uuid>,
    pub execution_result: Option<ExecutionResult>,
    pub response: String,
    pub timestamp: f64,
}

impl CMCAgent {
    pub fn new(llm: Box<dyn LLMProvider>) -> Self {
        CMCAgent {
            memory: MemoryGraph::new(),
            llm,
            conversation_history: Vec::new(),
            debug: false,
        }
    }

    pub fn with_memory(llm: Box<dyn LLMProvider>, memory: MemoryGraph) -> Self {
        CMCAgent {
            memory,
            llm,
            conversation_history: Vec::new(),
            debug: false,
        }
    }

    pub fn set_debug(&mut self, enabled: bool) {
        self.debug = enabled;
    }

    pub fn process(&mut self, user_input: &str) -> Result<String, LLMError> {
        let semantic_event = match self.llm.extract_semantics(user_input) {
            Ok(event) => {
                if self.debug {
                    eprintln!("Using LLM extraction");
                }
                event
            }
            Err(e) => {
                if self.debug {
                    eprintln!(
                        "️  LLM extraction failed: {:?}, falling back to structural extraction",
                        e
                    );
                }

                crate::ingestion::ingest_conversation_enhanced(user_input)
            }
        };

        if self.debug {
            eprintln!("\n [DEBUG] Semantic Event Extraction:");
            eprintln!("Atoms extracted: {}", semantic_event.atoms.len());
            for (i, atom) in semantic_event.atoms.iter().enumerate() {
                eprintln!(
                    "Atom {}: type={:?}, content={:?}",
                    i, atom.atom_type, atom.content
                );
            }
            eprintln!("Relationships: {}", semantic_event.relationships.len());
        }

        let intent =
            crate::intent::IntentClassifier::classify_intent_with_text(&semantic_event, user_input);

        if self.debug {
            eprintln!("\n [DEBUG] Intent Classification:");
            eprintln!("Pattern: {}", intent.pattern);
            eprintln!("Confidence: {:.3}", intent.confidence);
        }

        let is_query = intent.pattern.starts_with("query");
        let is_greeting = intent.pattern == "greeting";

        if self.debug {
            eprintln!("Is query: {}", is_query);
            eprintln!("Is greeting: {}", is_greeting);
        }

        let fragments = distill_event(&semantic_event);

        if self.debug {
            eprintln!("\n [DEBUG] Distilled Fragments:");
            eprintln!("Total fragments: {}", fragments.len());
            for (i, frag) in fragments.iter().enumerate() {
                match &frag.content {
                    FragmentContent::PersonalFact {
                        person,
                        fact_type,
                        value,
                        ..
                    } => {
                        eprintln!(
                            "Fragment {}: PersonalFact - person={}, fact_type={}, value={}",
                            i, person, fact_type, value
                        );
                    }
                    FragmentContent::OwnershipRelation { owner, owned, .. } => {
                        eprintln!(
                            "Fragment {}: OwnershipRelation - owner={}, owned={}",
                            i, owner, owned
                        );
                    }
                    FragmentContent::SemanticAtom {
                        atom_type, content, ..
                    } => {
                        eprintln!(
                            "Fragment {}: SemanticAtom - type={:?}, content={:?}",
                            i, atom_type, content
                        );
                    }
                    _ => {
                        eprintln!("Fragment {}: {:?}", i, frag.fragment_type);
                    }
                }
            }
        }

        if !is_query && !is_greeting {
            let edges =
                crate::distillation::create_edges_from_relationships(&semantic_event, &fragments);

            use std::collections::HashMap;
            let mut edges_by_fragment: HashMap<Uuid, Vec<Edge>> = HashMap::new();
            for edge in edges {
                edges_by_fragment
                    .entry(edge.from_fragment)
                    .or_insert_with(Vec::new)
                    .push(edge);
            }

            if self.debug {
                eprintln!(
                    "\n [DEBUG] Storing {} fragments in memory (statement)",
                    fragments.len()
                );
            }
            for fragment in &fragments {
                let fragment_edges = edges_by_fragment
                    .get(&fragment.id)
                    .cloned()
                    .unwrap_or_default();

                if self.debug {
                    match &fragment.content {
                        FragmentContent::PersonalFact {
                            person,
                            fact_type,
                            value,
                            ..
                        } => {
                            eprintln!(
                                "Storing PersonalFact: person={}, fact_type={}, value={}",
                                person, fact_type, value
                            );
                        }
                        FragmentContent::OwnershipRelation { owner, owned, .. } => {
                            eprintln!(
                                "Storing OwnershipRelation: owner={}, owned={}",
                                owner, owned
                            );
                        }
                        FragmentContent::SemanticAtom {
                            atom_type, content, ..
                        } => {
                            if content.contains_key("ownership_marker") {
                                eprintln!("Storing SemanticAtom with ownership_marker: type={:?}, content={:?}", 
                                    atom_type, content);
                            }
                        }
                        _ => {}
                    }
                }

                self.memory
                    .insert_fragment(fragment.clone(), fragment_edges);
            }
        } else {
            if self.debug {
                if is_greeting {
                    eprintln!(
                        "\n [DEBUG] Skipping fragment storage (greeting - no information to store)"
                    );
                } else {
                    eprintln!("\n [DEBUG] Skipping fragment storage (query - fragments only used for retrieval)");
                }
            }
        }

        if is_greeting {
            return Ok("Hello! How can I help you?".to_string());
        }

        if is_query {
            let mut context = generate_context(&intent.pattern, "general", 0.3);
            Self::extract_keywords_from_atoms(&semantic_event, &mut context);

            self.add_query_keywords(user_input, &mut context);

            let intent_matches =
                crate::intent::IntentClassifier::match_intent_to_memory(&intent, &self.memory);

            for frag_id in intent_matches {
                if let Some(fragment) = self.memory.fragments.get(&frag_id) {
                    match &fragment.content {
                        FragmentContent::SemanticAtom { content, .. } => {
                            for key in content.keys() {
                                context.domain_hint.tags.insert(key.clone());
                            }
                        }
                        _ => {}
                    }
                }
            }

            let eeg = compile_thought(&context, &mut self.memory);

            let execution_result = execute_eeg(&eeg, &mut self.memory);

            if self.debug {
                eprintln!("\n [DEBUG] CMCA Execution Result:");
                eprintln!("Intent pattern: {}", intent.pattern);
                eprintln!("Confidence: {:.3}", execution_result.confidence);
                eprintln!(
                    "Execution trace length: {}",
                    execution_result.execution_trace.len()
                );
                eprintln!("Outcome: {}", execution_result.outcome.result);
                eprintln!("Context tags: {:?}", context.domain_hint.tags);
                if !execution_result.execution_trace.is_empty() {
                    eprintln!(
                        "Activated fragments: {:?}",
                        execution_result.execution_trace
                    );
                }
            }

            let memory_data =
                MemoryData::from_execution_and_memory(&execution_result, &self.memory, user_input);

            if self.debug {
                eprintln!("Memory data fragments: {}", memory_data.fragments.len());
                for (i, frag) in memory_data.fragments.iter().enumerate() {
                    eprintln!(
                        "Fragment {}: type={}, content={:?}",
                        i, frag.atom_type, frag.content
                    );
                }
                eprintln!("Memory data compact: {}", memory_data.to_compact_string());
            }

            let response = if execution_result.confidence > 0.3 && !memory_data.fragments.is_empty()
            {
                let candidates = crate::response_builder::ResponseBuilder::extract_top_candidates(
                    &memory_data,
                    &self.memory,
                    &execution_result,
                );

                if !candidates.is_empty() {
                    self.llm
                        .format_response_from_candidates(user_input, &candidates, &memory_data)
                        .unwrap_or_else(|_| {
                            let (best_answer, _) = &candidates[0];
                            crate::response_builder::ResponseBuilder::format_answer(
                                best_answer,
                                &memory_data,
                                &self.memory,
                                &execution_result,
                            )
                        })
                } else {
                    self.llm
                        .format_response_from_memory(user_input, &memory_data)
                        .unwrap_or_else(|_| "I don't have that information yet.".to_string())
                }
            } else {
                if execution_result.execution_trace.is_empty() {
                    "I don't have that information yet.".to_string()
                } else {
                    self.llm
                        .format_response_from_memory(user_input, &memory_data)
                        .unwrap_or_else(|_| "I'm not sure about that.".to_string())
                }
            };

            if !execution_result.execution_trace.is_empty() {
                let success = execution_result.confidence > 0.5;
                self.memory.learn_intent_pattern(
                    &intent,
                    &execution_result.execution_trace,
                    success,
                );
                self.memory
                    .record_co_activation(&execution_result.execution_trace);
            }

            for signal in &execution_result.reinforcement_signals {
                self.memory
                    .reinforce_fragment(signal.fragment_id, &execution_result.outcome);
            }

            self.conversation_history.push(ConversationTurn {
                user_input: user_input.to_string(),
                context_used: context,
                eeg_compiled: Some(eeg.entry_point),
                execution_result: Some(execution_result),
                response: response.clone(),
                timestamp: current_timestamp(),
            });

            Ok(response)
        } else {
            let mut context = generate_context("statement", "general", 0.3);
            Self::extract_keywords_from_atoms(&semantic_event, &mut context);

            let eeg = compile_thought(&context, &mut self.memory);
            let execution_result = execute_eeg(&eeg, &mut self.memory);

            if self.debug {
                eprintln!("\n [DEBUG] CMCA Execution Result (Statement):");
                eprintln!("Intent pattern: {}", intent.pattern);
                eprintln!("Confidence: {:.3}", execution_result.confidence);
                eprintln!(
                    "Execution trace length: {}",
                    execution_result.execution_trace.len()
                );
                eprintln!("Outcome: {}", execution_result.outcome.result);
                eprintln!("Context tags: {:?}", context.domain_hint.tags);
            }

            let memory_data =
                MemoryData::from_execution_and_memory(&execution_result, &self.memory, user_input);

            if self.debug {
                eprintln!("Memory data fragments: {}", memory_data.fragments.len());
                for (i, frag) in memory_data.fragments.iter().enumerate() {
                    eprintln!(
                        "Fragment {}: type={}, content={:?}",
                        i, frag.atom_type, frag.content
                    );
                }
            }

            let response = if !memory_data.fragments.is_empty() {
                "Got it.".to_string()
            } else {
                "Got it.".to_string()
            };

            self.conversation_history.push(ConversationTurn {
                user_input: user_input.to_string(),
                context_used: context,
                eeg_compiled: Some(eeg.entry_point),
                execution_result: Some(execution_result),
                response: response.clone(),
                timestamp: current_timestamp(),
            });

            Ok(response)
        }
    }

    fn add_query_keywords(&self, user_input: &str, context: &mut ContextVector) {
        let expanded_keywords =
            crate::query_expansion::QueryExpander::expand_query(user_input, &self.memory, 2);

        for keyword in expanded_keywords {
            context.domain_hint.tags.insert(keyword);
        }

        use std::collections::HashSet;
        let keywords: HashSet<String> = user_input
            .to_lowercase()
            .split_whitespace()
            .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()).to_string())
            .filter(|w| w.len() >= 2)
            .collect();

        for keyword in keywords {
            context.domain_hint.tags.insert(keyword.clone());

            if keyword.contains(' ') {
                context.domain_hint.tags.insert(keyword.replace(' ', "_"));
                context.domain_hint.tags.insert(keyword.replace(' ', "-"));
            }
        }

        let phrase = user_input
            .to_lowercase()
            .split_whitespace()
            .filter(|w| w.len() >= 2)
            .collect::<Vec<_>>()
            .join("");
        if phrase.len() >= 4 {
            context.domain_hint.tags.insert(phrase);
        }
    }

    fn extract_keywords_from_atoms(semantic_event: &SemanticEvent, context: &mut ContextVector) {
        use std::collections::HashSet;
        let mut keywords = HashSet::new();

        for atom in &semantic_event.atoms {
            for (key, value) in &atom.content {
                if !key.is_empty() {
                    keywords.insert(key.clone());
                    keywords.insert(key.to_lowercase());
                }

                if !value.is_empty() && value != "unknown" {
                    keywords.insert(value.clone());
                    keywords.insert(value.to_lowercase());
                }
            }
        }

        for keyword in keywords {
            context.domain_hint.tags.insert(keyword);
        }
    }

    pub fn query_memory(&mut self, goal: &str, domain: &str) -> ExecutionResult {
        let context = generate_context(goal, domain, 0.3);
        let eeg = compile_thought(&context, &mut self.memory);
        execute_eeg(&eeg, &mut self.memory)
    }

    pub fn memory_stats(&self) -> MemoryStats {
        MemoryStats {
            fragments: self.memory.fragments.len(),
            edges: self.memory.edges.len(),
            compiled_modules: self.memory.compiled_modules.len(),
            conversation_turns: self.conversation_history.len(),
        }
    }

    pub fn save_memory(&self, path: impl AsRef<std::path::Path>) -> crate::storage::Result<()> {
        self.memory.save(path)
    }

    pub fn load_memory(path: impl AsRef<std::path::Path>) -> crate::storage::Result<MemoryGraph> {
        MemoryGraph::load(path)
    }

    pub fn conversation_history(&self) -> &[ConversationTurn] {
        &self.conversation_history
    }
}

#[derive(Debug, Clone)]
pub struct MemoryStats {
    pub fragments: usize,
    pub edges: usize,
    pub compiled_modules: usize,
    pub conversation_turns: usize,
}

#[cfg(feature = "openai")]
pub mod openai_provider {
    use super::*;
    use serde_json::json;
    use std::collections::HashMap;
    use std::time::{SystemTime, UNIX_EPOCH};

    pub struct OpenAIProvider {
        api_key: String,
        model: String,
        client: reqwest::blocking::Client,
    }

    impl OpenAIProvider {
        pub fn new(api_key: String) -> Self {
            OpenAIProvider {
                api_key,
                model: "gpt-4o-mini".to_string(),
                client: reqwest::blocking::Client::new(),
            }
        }

        pub fn with_model(api_key: String, model: String) -> Self {
            OpenAIProvider {
                api_key,
                model,
                client: reqwest::blocking::Client::new(),
            }
        }

        fn call_openai(
            &self,
            prompt: &str,
            system_prompt: Option<&str>,
            require_json: bool,
        ) -> Result<String, LLMError> {
            let messages = vec![
                if let Some(sys) = system_prompt {
                    json!({"role": "system", "content": sys})
                } else {
                    json!({"role": "system", "content": "You are a helpful assistant that extracts structured information from text."})
                },
                json!({"role": "user", "content": prompt}),
            ];

            let mut body = json!({
                "model": self.model,
                "messages": messages,
                "temperature": 0.3,
            });

            if require_json {
                body["response_format"] = json!({"type": "json_object"});
            }

            let response = self
                .client
                .post("https://api.openai.com/v1/chat/completions")
                .header("Authorization", format!("Bearer {}", self.api_key))
                .header("Content-Type", "application/json")
                .json(&body)
                .send()
                .map_err(|e| LLMError::NetworkError(format!("Request failed: {}", e)))?;

            if !response.status().is_success() {
                let status = response.status();
                let text = response.text().unwrap_or_default();
                return Err(LLMError::ProviderError(format!(
                    "OpenAI API error {}: {}",
                    status, text
                )));
            }

            let json_response: serde_json::Value = response
                .json()
                .map_err(|e| LLMError::ParseError(format!("Failed to parse response: {}", e)))?;

            let content = json_response["choices"][0]["message"]["content"]
                .as_str()
                .ok_or_else(|| LLMError::ParseError("Missing content in response".to_string()))?;

            Ok(content.to_string())
        }
    }

    impl LLMProvider for OpenAIProvider {
        fn extract_semantics(&self, text: &str) -> Result<SemanticEvent, LLMError> {
            let prompt = format!(
                r#"
Extract semantic atoms and relationships from this text using grammatical analysis. 
Analyze the dependency parse structure and semantic roles (agent, patient, theme, etc.) to correctly identify:
- Subject-verb-object relationships
- Possessive relationships (e.g., "my X", "I like X")
- Action-object relationships (e.g., "like cheese"where "like"is the action and "cheese"is the object)
- Question structures vs statements

Text: "{}"

IMPORTANT GUIDELINES:
1. Use grammatical roles, not word patterns. "do"in "do I like"is an auxiliary verb, NOT a Person.
2. Nouns/objects should be Object or Entity atoms, not Action atoms. "banjos"is an Object, not an Action.
3. Verbs should be Action atoms. "like"in "I like cheese"is an Action.
4. Pronouns like "I", "my"should be Person atoms with ownership_marker relationships.
5. For "I like X"patterns: Person("I") -> Action("like") -> Object/Entity(X) with Likes relationship.
6. For questions like "what do I like": Action("like") with incomplete Object (seeking information).

Return a JSON object with this exact structure:
{{
    "event_type": "Conversation"| "Observation"| "Action"| "Outcome",
    "atoms": [
        {{
            "atom_type": "Entity"| "Action"| "Condition"| "Outcome"| "Property"| "Person"| "Location"| "Time"| "Quantity"| "Concept"| "Object"| "Event"| "Attribute"| "State"| "Resource",
            "content": {{
                "key": "value",
                "semantic_role": "agent"| "patient"| "theme"| "experiencer"| "possessor"| "owned"| null,
                "grammatical_role": "subject"| "verb"| "object"| "complement"| "auxiliary"| null
            }}
        }}
    ],
    "relationships": [
        {{
            "from_atom": 0,
            "to_atom": 1,
            "relation_type": "Causal"| "Temporal"| "Semantic"| "Spatial"| "Ownership"| "PartOf"| "SimilarTo"| "OppositeOf"| "Causes"| "Prevents"| "Enables"| "Requires"| "LocatedAt"| "OccursAt"| "ParticipatesIn"| "Knows"| "Likes"| "Dislikes"| "RelatedTo"| "Hierarchical"| "Before"| "After"| "During"| "Simultaneous"| "GreaterThan"| "LessThan"| "EqualTo"| "Approximately",
            "strength": 0.0-1.0
        }}
    ],
    "salience": 1.0,
    "emotional_weight": 0.0
}}

Return ONLY valid JSON, no explanations.
"#,
                text
            );

            let system_prompt = "You are a semantic parser that extracts structured information from text using grammatical analysis and dependency parsing. You understand semantic roles (agent, patient, theme, experiencer) and grammatical roles (subject, verb, object). Use this understanding to correctly classify atoms: verbs are Actions, nouns are Objects/Entities, pronouns are Persons. Return only valid JSON matching the exact structure requested. Do not add explanations or markdown formatting. You must return JSON format.";

            let json_str = self.call_openai(&prompt, Some(system_prompt), true)?;

            let cleaned_json = json_str
                .trim()
                .trim_start_matches("```json")
                .trim_start_matches("```")
                .trim_end_matches("```")
                .trim();

            let json_value: serde_json::Value =
                serde_json::from_str(cleaned_json).map_err(|e| {
                    LLMError::ParseError(format!("Failed to parse JSON: {} (raw: {})", e, json_str))
                })?;

            let event_type_str = json_value["event_type"]
                .as_str()
                .ok_or_else(|| LLMError::ParseError("Missing event_type".to_string()))?;
            let event_type = match event_type_str {
                "Conversation" => EventType::Conversation,
                "Observation" => EventType::Observation,
                "Action" => EventType::Action,
                "Outcome" => EventType::Outcome,
                _ => EventType::Conversation,
            };

            let atoms: Vec<SemanticAtom> = json_value["atoms"]
                .as_array()
                .ok_or_else(|| LLMError::ParseError("Missing atoms array".to_string()))?
                .iter()
                .map(|atom_json| {
                    let atom_type_str = atom_json["atom_type"].as_str().unwrap_or("Entity");
                    let atom_type = match atom_type_str {
                        "Entity" => AtomType::Entity,
                        "Action" => AtomType::Action,
                        "Condition" => AtomType::Condition,
                        "Outcome" => AtomType::Outcome,
                        "Property" => AtomType::Property,
                        "Person" => AtomType::Person,
                        "Location" => AtomType::Location,
                        "Time" => AtomType::Time,
                        "Quantity" => AtomType::Quantity,
                        "Concept" => AtomType::Concept,
                        "Object" => AtomType::Object,
                        "Event" => AtomType::Event,
                        "Attribute" => AtomType::Attribute,
                        "State" => AtomType::State,
                        "Resource" => AtomType::Resource,
                        _ => AtomType::Entity,
                    };
                    let content: HashMap<String, String> = atom_json["content"]
                        .as_object()
                        .unwrap_or(&serde_json::Map::new())
                        .iter()
                        .map(|(k, v)| (k.clone(), v.as_str().unwrap_or("").to_string()))
                        .collect();
                    SemanticAtom { atom_type, content }
                })
                .collect();

            let relationships: Vec<Relationship> = json_value["relationships"]
                .as_array()
                .unwrap_or(&vec![])
                .iter()
                .map(|rel_json| {
                    let from_atom = rel_json["from_atom"].as_u64().unwrap_or(0) as usize;
                    let to_atom = rel_json["to_atom"].as_u64().unwrap_or(0) as usize;
                    let relation_type_str =
                        rel_json["relation_type"].as_str().unwrap_or("Semantic");
                    let relation_type = match relation_type_str {
                        "Causal" => RelationType::Causal,
                        "Temporal" => RelationType::Temporal,
                        "Semantic" => RelationType::Semantic,
                        "Spatial" => RelationType::Spatial,
                        "Ownership" => RelationType::Ownership,
                        "PartOf" => RelationType::PartOf,
                        "SimilarTo" => RelationType::SimilarTo,
                        "OppositeOf" => RelationType::OppositeOf,
                        "Causes" => RelationType::Causes,
                        "Prevents" => RelationType::Prevents,
                        "Enables" => RelationType::Enables,
                        "Requires" => RelationType::Requires,
                        "LocatedAt" => RelationType::LocatedAt,
                        "OccursAt" => RelationType::OccursAt,
                        "ParticipatesIn" => RelationType::ParticipatesIn,
                        "Knows" => RelationType::Knows,
                        "Likes" => RelationType::Likes,
                        "Dislikes" => RelationType::Dislikes,
                        "RelatedTo" => RelationType::RelatedTo,
                        "Hierarchical" => RelationType::Hierarchical,
                        "Before" => RelationType::Before,
                        "After" => RelationType::After,
                        "During" => RelationType::During,
                        "Simultaneous" => RelationType::Simultaneous,
                        "GreaterThan" => RelationType::GreaterThan,
                        "LessThan" => RelationType::LessThan,
                        "EqualTo" => RelationType::EqualTo,
                        "Approximately" => RelationType::Approximately,
                        _ => RelationType::Semantic,
                    };
                    let strength = rel_json["strength"].as_f64().unwrap_or(0.5);
                    Relationship {
                        from_atom,
                        to_atom,
                        relation_type,
                        strength,
                    }
                })
                .collect();

            let salience = json_value["salience"].as_f64().unwrap_or(1.0);
            let emotional_weight = json_value["emotional_weight"].as_f64().unwrap_or(0.0);

            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs_f64();

            Ok(SemanticEvent {
                id: Uuid::new_v4(),
                timestamp,
                event_type,
                atoms,
                relationships,
                salience,
                emotional_weight,
                source_context: HashMap::new(),
            })
        }

        fn format_response_from_memory(
            &self,
            user_query: &str,
            memory_data: &MemoryData,
        ) -> Result<String, LLMError> {
            if memory_data.fragments.is_empty() {
                let prompt = format!(
                    r#"
The user asked: "{}"

I don't have that information in my memory yet. Respond naturally and conversationally.
"#,
                    user_query
                );

                let system_prompt = "You are a helpful AI assistant with a memory system. When you don't have information, respond naturally and conversationally. Be friendly and concise.";

                return self.call_openai(&prompt, Some(system_prompt), false);
            }

            let memory_str = memory_data.to_compact_string();

            let prompt = format!(
                r#"
The user asked: "{}"

Here's what I found in my memory (compact format):
{}

Create a natural, conversational response based on this memory data. Answer their question directly and naturally.
If the memory contains their information (like name, preferences, etc.), present it naturally.
Be friendly, concise, and conversational.
"#,
                user_query, memory_str
            );

            let system_prompt = "You are a helpful AI assistant with a memory system. You receive memory data in compact format and create natural conversational responses. Answer questions based on the memory data provided. Be direct, friendly, and concise.";

            self.call_openai(&prompt, Some(system_prompt), false)
        }

        fn format_response(
            &self,
            result: &ExecutionResult,
            _context: &ContextVector,
        ) -> Result<String, LLMError> {
            let memory_data = MemoryData::from_execution_result(result);
            self.format_response_from_memory("", &memory_data)
        }

        fn extract_goal_and_domain(&self, query: &str) -> Result<(String, String), LLMError> {
            let prompt = format!(
                r#"
Extract the goal and domain from this query. Return ONLY valid JSON with "goal"and "domain"fields.

Query: "{}"

Return JSON: {{"goal": "debug"| "create"| "explain"| "general", "domain": "api_integration"| "programming"| "debugging"| "general"}}
"#,
                query
            );

            let system_prompt = "You extract goal and domain from queries. Return only valid JSON with goal and domain fields. You must return JSON format.";

            let json_str = self.call_openai(&prompt, Some(system_prompt), true)?;

            let cleaned_json = json_str
                .trim()
                .trim_start_matches("```json")
                .trim_start_matches("```")
                .trim_end_matches("```")
                .trim();

            let json_value: serde_json::Value =
                serde_json::from_str(cleaned_json).map_err(|e| {
                    LLMError::ParseError(format!("Failed to parse JSON: {} (raw: {})", e, json_str))
                })?;

            let goal = json_value["goal"].as_str().unwrap_or("general").to_string();
            let domain = json_value["domain"]
                .as_str()
                .unwrap_or("general")
                .to_string();

            Ok((goal, domain))
        }
    }
}

#[cfg(feature = "anthropic")]
pub mod anthropic_provider {
    use super::*;
    use serde_json::json;
    use std::collections::HashMap;
    use std::time::{SystemTime, UNIX_EPOCH};

    pub struct AnthropicProvider {
        api_key: String,
        model: String,
        client: reqwest::blocking::Client,
    }

    impl AnthropicProvider {
        pub fn new(api_key: String) -> Self {
            AnthropicProvider {
                api_key,
                model: "claude-3-opus-20240229".to_string(),
                client: reqwest::blocking::Client::new(),
            }
        }

        pub fn with_model(api_key: String, model: String) -> Self {
            AnthropicProvider {
                api_key,
                model,
                client: reqwest::blocking::Client::new(),
            }
        }

        fn call_anthropic(
            &self,
            prompt: &str,
            system_prompt: Option<&str>,
            _require_json: bool,
        ) -> Result<String, LLMError> {
            let messages = vec![json!({"role": "user", "content": prompt})];

            let mut body = json!({
                "model": self.model,
                "max_tokens": 4096,
                "messages": messages,
                "temperature": 0.3,
            });

            if let Some(sys) = system_prompt {
                body["system"] = json!(sys);
            }

            let response = self
                .client
                .post("https://api.anthropic.com/v1/messages")
                .header("x-api-key", &self.api_key)
                .header("anthropic-version", "2023-06-01")
                .header("Content-Type", "application/json")
                .json(&body)
                .send()
                .map_err(|e| LLMError::NetworkError(format!("Request failed: {}", e)))?;

            if !response.status().is_success() {
                let status = response.status();
                let text = response.text().unwrap_or_default();
                return Err(LLMError::ProviderError(format!(
                    "Anthropic API error {}: {}",
                    status, text
                )));
            }

            let json_response: serde_json::Value = response
                .json()
                .map_err(|e| LLMError::ParseError(format!("Failed to parse response: {}", e)))?;

            let content = json_response["content"]
                .as_array()
                .and_then(|arr| arr.get(0))
                .and_then(|item| item["text"].as_str())
                .ok_or_else(|| LLMError::ParseError("Missing content in response".to_string()))?;

            Ok(content.to_string())
        }
    }

    impl LLMProvider for AnthropicProvider {
        fn extract_semantics(&self, text: &str) -> Result<SemanticEvent, LLMError> {
            let prompt = format!(
                r#"
Extract semantic atoms and relationships from this text using grammatical analysis. 
Analyze the dependency parse structure and semantic roles (agent, patient, theme, etc.) to correctly identify:
- Subject-verb-object relationships
- Possessive relationships (e.g., "my X", "I like X")
- Action-object relationships (e.g., "like cheese"where "like"is the action and "cheese"is the object)
- Question structures vs statements

Text: "{}"

IMPORTANT GUIDELINES:
1. Use grammatical roles, not word patterns. "do"in "do I like"is an auxiliary verb, NOT a Person.
2. Nouns/objects should be Object or Entity atoms, not Action atoms. "banjos"is an Object, not an Action.
3. Verbs should be Action atoms. "like"in "I like cheese"is an Action.
4. Pronouns like "I", "my"should be Person atoms with ownership_marker relationships.
5. For "I like X"patterns: Person("I") -> Action("like") -> Object/Entity(X) with Likes relationship.
6. For questions like "what do I like": Action("like") with incomplete Object (seeking information).

Return a JSON object with this exact structure:
{{
    "event_type": "Conversation"| "Observation"| "Action"| "Outcome",
    "atoms": [
        {{
            "atom_type": "Entity"| "Action"| "Condition"| "Outcome"| "Property"| "Person"| "Location"| "Time"| "Quantity"| "Concept"| "Object"| "Event"| "Attribute"| "State"| "Resource",
            "content": {{
                "key": "value",
                "semantic_role": "agent"| "patient"| "theme"| "experiencer"| "possessor"| "owned"| null,
                "grammatical_role": "subject"| "verb"| "object"| "complement"| "auxiliary"| null
            }}
        }}
    ],
    "relationships": [
        {{
            "from_atom": 0,
            "to_atom": 1,
            "relation_type": "Causal"| "Temporal"| "Semantic"| "Spatial"| "Ownership"| "PartOf"| "SimilarTo"| "OppositeOf"| "Causes"| "Prevents"| "Enables"| "Requires"| "LocatedAt"| "OccursAt"| "ParticipatesIn"| "Knows"| "Likes"| "Dislikes"| "RelatedTo"| "Hierarchical"| "Before"| "After"| "During"| "Simultaneous"| "GreaterThan"| "LessThan"| "EqualTo"| "Approximately",
            "strength": 0.0-1.0
        }}
    ],
    "salience": 1.0,
    "emotional_weight": 0.0
}}

Return ONLY valid JSON, no explanations.
"#,
                text
            );

            let system_prompt = "You are a semantic parser that extracts structured information from text using grammatical analysis and dependency parsing. You understand semantic roles (agent, patient, theme, experiencer) and grammatical roles (subject, verb, object). Use this understanding to correctly classify atoms: verbs are Actions, nouns are Objects/Entities, pronouns are Persons. Return only valid JSON matching the exact structure requested. Do not add explanations or markdown formatting. You must return JSON format.";

            let json_str = self.call_anthropic(&prompt, Some(system_prompt), true)?;

            let cleaned_json = json_str
                .trim()
                .trim_start_matches("```json")
                .trim_start_matches("```")
                .trim_end_matches("```")
                .trim();

            let json_value: serde_json::Value =
                serde_json::from_str(cleaned_json).map_err(|e| {
                    LLMError::ParseError(format!("Failed to parse JSON: {} (raw: {})", e, json_str))
                })?;

            let event_type_str = json_value["event_type"]
                .as_str()
                .ok_or_else(|| LLMError::ParseError("Missing event_type".to_string()))?;
            let event_type = match event_type_str {
                "Conversation" => EventType::Conversation,
                "Observation" => EventType::Observation,
                "Action" => EventType::Action,
                "Outcome" => EventType::Outcome,
                _ => EventType::Conversation,
            };

            let atoms: Vec<SemanticAtom> = json_value["atoms"]
                .as_array()
                .ok_or_else(|| LLMError::ParseError("Missing atoms array".to_string()))?
                .iter()
                .map(|atom_json| {
                    let atom_type_str = atom_json["atom_type"].as_str().unwrap_or("Entity");
                    let atom_type = match atom_type_str {
                        "Entity" => AtomType::Entity,
                        "Action" => AtomType::Action,
                        "Condition" => AtomType::Condition,
                        "Outcome" => AtomType::Outcome,
                        "Property" => AtomType::Property,
                        "Person" => AtomType::Person,
                        "Location" => AtomType::Location,
                        "Time" => AtomType::Time,
                        "Quantity" => AtomType::Quantity,
                        "Concept" => AtomType::Concept,
                        "Object" => AtomType::Object,
                        "Event" => AtomType::Event,
                        "Attribute" => AtomType::Attribute,
                        "State" => AtomType::State,
                        "Resource" => AtomType::Resource,
                        _ => AtomType::Entity,
                    };
                    let content: HashMap<String, String> = atom_json["content"]
                        .as_object()
                        .unwrap_or(&serde_json::Map::new())
                        .iter()
                        .map(|(k, v)| (k.clone(), v.as_str().unwrap_or("").to_string()))
                        .collect();
                    SemanticAtom { atom_type, content }
                })
                .collect();

            let relationships: Vec<Relationship> = json_value["relationships"]
                .as_array()
                .unwrap_or(&vec![])
                .iter()
                .map(|rel_json| {
                    let from_atom = rel_json["from_atom"].as_u64().unwrap_or(0) as usize;
                    let to_atom = rel_json["to_atom"].as_u64().unwrap_or(0) as usize;
                    let relation_type_str =
                        rel_json["relation_type"].as_str().unwrap_or("Semantic");
                    let relation_type = match relation_type_str {
                        "Causal" => RelationType::Causal,
                        "Temporal" => RelationType::Temporal,
                        "Semantic" => RelationType::Semantic,
                        "Spatial" => RelationType::Spatial,
                        "Ownership" => RelationType::Ownership,
                        "PartOf" => RelationType::PartOf,
                        "SimilarTo" => RelationType::SimilarTo,
                        "OppositeOf" => RelationType::OppositeOf,
                        "Causes" => RelationType::Causes,
                        "Prevents" => RelationType::Prevents,
                        "Enables" => RelationType::Enables,
                        "Requires" => RelationType::Requires,
                        "LocatedAt" => RelationType::LocatedAt,
                        "OccursAt" => RelationType::OccursAt,
                        "ParticipatesIn" => RelationType::ParticipatesIn,
                        "Knows" => RelationType::Knows,
                        "Likes" => RelationType::Likes,
                        "Dislikes" => RelationType::Dislikes,
                        "RelatedTo" => RelationType::RelatedTo,
                        "Hierarchical" => RelationType::Hierarchical,
                        "Before" => RelationType::Before,
                        "After" => RelationType::After,
                        "During" => RelationType::During,
                        "Simultaneous" => RelationType::Simultaneous,
                        "GreaterThan" => RelationType::GreaterThan,
                        "LessThan" => RelationType::LessThan,
                        "EqualTo" => RelationType::EqualTo,
                        "Approximately" => RelationType::Approximately,
                        _ => RelationType::Semantic,
                    };
                    let strength = rel_json["strength"].as_f64().unwrap_or(0.5);
                    Relationship {
                        from_atom,
                        to_atom,
                        relation_type,
                        strength,
                    }
                })
                .collect();

            let salience = json_value["salience"].as_f64().unwrap_or(1.0);
            let emotional_weight = json_value["emotional_weight"].as_f64().unwrap_or(0.0);

            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs_f64();

            Ok(SemanticEvent {
                id: Uuid::new_v4(),
                timestamp,
                event_type,
                atoms,
                relationships,
                salience,
                emotional_weight,
                source_context: HashMap::new(),
            })
        }

        fn format_response_from_memory(
            &self,
            user_query: &str,
            memory_data: &MemoryData,
        ) -> Result<String, LLMError> {
            if memory_data.fragments.is_empty() {
                let prompt = format!(
                    r#"
The user asked: "{}"

I don't have that information in my memory yet. Respond naturally and conversationally.
"#,
                    user_query
                );

                let system_prompt = "You are a helpful AI assistant with a memory system. When you don't have information, respond naturally and conversationally. Be friendly and concise.";

                return self.call_anthropic(&prompt, Some(system_prompt), false);
            }

            let memory_str = memory_data.to_compact_string();

            let prompt = format!(
                r#"
The user asked: "{}"

Here's what I found in my memory (compact format):
{}

Create a natural, conversational response based on this memory data. Answer their question directly and naturally.
If the memory contains their information (like name, preferences, etc.), present it naturally.
Be friendly, concise, and conversational.
"#,
                user_query, memory_str
            );

            let system_prompt = "You are a helpful AI assistant with a memory system. You receive memory data in compact format and create natural conversational responses. Answer questions based on the memory data provided. Be direct, friendly, and concise.";

            self.call_anthropic(&prompt, Some(system_prompt), false)
        }

        fn format_response(
            &self,
            result: &ExecutionResult,
            _context: &ContextVector,
        ) -> Result<String, LLMError> {
            let memory_data = MemoryData::from_execution_result(result);
            self.format_response_from_memory("", &memory_data)
        }

        fn extract_goal_and_domain(&self, query: &str) -> Result<(String, String), LLMError> {
            let prompt = format!(
                r#"
Extract the goal and domain from this query. Return ONLY valid JSON with "goal"and "domain"fields.

Query: "{}"

Return JSON: {{"goal": "debug"| "create"| "explain"| "general", "domain": "api_integration"| "programming"| "debugging"| "general"}}
"#,
                query
            );

            let system_prompt = "You extract goal and domain from queries. Return only valid JSON with goal and domain fields. You must return JSON format.";

            let json_str = self.call_anthropic(&prompt, Some(system_prompt), true)?;

            let cleaned_json = json_str
                .trim()
                .trim_start_matches("```json")
                .trim_start_matches("```")
                .trim_end_matches("```")
                .trim();

            let json_value: serde_json::Value =
                serde_json::from_str(cleaned_json).map_err(|e| {
                    LLMError::ParseError(format!("Failed to parse JSON: {} (raw: {})", e, json_str))
                })?;

            let goal = json_value["goal"].as_str().unwrap_or("general").to_string();
            let domain = json_value["domain"]
                .as_str()
                .unwrap_or("general")
                .to_string();

            Ok((goal, domain))
        }
    }
}

#[cfg(feature = "local-llm")]
pub mod local_provider {
    use super::*;

    pub struct LocalLLMProvider {
        model_path: String,
    }

    impl LocalLLMProvider {
        pub fn new(model_path: String) -> Self {
            LocalLLMProvider { model_path }
        }
    }

    impl LLMProvider for LocalLLMProvider {
        fn extract_semantics(&self, text: &str) -> Result<SemanticEvent, LLMError> {
            Ok(ingest_conversation(text))
        }

        fn format_response_from_memory(
            &self,
            user_query: &str,
            memory_data: &MemoryData,
        ) -> Result<String, LLMError> {
            if memory_data.fragments.is_empty() {
                return Ok("I don't have that information yet.".to_string());
            }

            let memory_str = memory_data.to_compact_string();
            Ok(format!("Query: {}\nMemory: {}", user_query, memory_str))
        }

        fn format_response(
            &self,
            result: &ExecutionResult,
            _context: &ContextVector,
        ) -> Result<String, LLMError> {
            Ok(format!(
                "Execution completed: {} (confidence: {:.2})",
                result.outcome.result, result.confidence
            ))
        }

        fn extract_goal_and_domain(&self, query: &str) -> Result<(String, String), LLMError> {
            let goal = "general".to_string();
            let domain = "general".to_string();
            Ok((goal, domain))
        }
    }
}
