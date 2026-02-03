// Copyright (c) 2026 Nolan Taft
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EventType {
    Conversation,
    Observation,
    Action,
    Outcome,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AtomType {
    Entity,
    Action,
    Condition,
    Outcome,
    Property,

    Person,
    Location,
    Time,
    Quantity,
    Concept,
    Object,
    Event,
    Attribute,
    State,
    Resource,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RelationType {
    Causal,
    Temporal,
    Semantic,
    Spatial,

    Ownership,
    PartOf,
    SimilarTo,
    OppositeOf,
    Causes,
    Prevents,
    Enables,
    Requires,
    LocatedAt,
    OccursAt,
    ParticipatesIn,
    Knows,
    Likes,
    Dislikes,
    RelatedTo,
    Hierarchical,
    Before,
    After,
    During,
    Simultaneous,
    GreaterThan,
    LessThan,
    EqualTo,
    Approximately,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticAtom {
    pub atom_type: AtomType,
    pub content: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    pub from_atom: usize,
    pub to_atom: usize,
    pub relation_type: RelationType,
    pub strength: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticEvent {
    pub id: Uuid,
    pub timestamp: f64,
    pub event_type: EventType,
    pub atoms: Vec<SemanticAtom>,
    pub relationships: Vec<Relationship>,
    pub salience: f64,
    pub emotional_weight: f64,
    pub source_context: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FragmentType {
    EntityRelation,
    CausalRule,
    GoalStrategy,
    Constraint,
    Preference,
    ContextSignature,

    PersonalFact,
    TemporalEvent,
    SpatialRelation,
    QuantitativeFact,
    HierarchicalRelation,
    SocialRelation,
    OwnershipRelation,
    StateTransition,
    Capability,
    Belief,
    SemanticAtom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FragmentContent {
    EntityRelation {
        entity: String,
        relation: String,
        target: String,
    },
    CausalRule {
        condition: String,
        outcome: String,
        confidence: f64,
    },
    GoalStrategy {
        goal: String,
        strategy: String,
        success_rate: f64,
    },
    Constraint {
        constraint: String,
        context: String,
        severity: f64,
    },
    Preference {
        preference: String,
        weight: f64,
        context: String,
    },
    ContextSignature {
        pattern: String,
        typical_activations: Vec<Uuid>,
    },

    PersonalFact {
        person: String,
        fact_type: String,
        value: String,
        confidence: f64,
    },
    TemporalEvent {
        event: String,
        time_expression: String,
        duration: Option<String>,
        frequency: Option<String>,
        confidence: f64,
    },
    SpatialRelation {
        entity: String,
        location: String,
        relation_type: String,
        distance: Option<String>,
        confidence: f64,
    },
    QuantitativeFact {
        entity: String,
        quantity: f64,
        unit: Option<String>,
        comparison: Option<String>,
        reference: Option<String>,
        confidence: f64,
    },
    HierarchicalRelation {
        parent: String,
        child: String,
        relation_type: String,
        level: Option<usize>,
        confidence: f64,
    },
    SocialRelation {
        person1: String,
        person2: String,
        relation_type: String,
        strength: f64,
        context: Option<String>,
        confidence: f64,
    },
    OwnershipRelation {
        owner: String,
        owned: String,
        relation_type: String,
        confidence: f64,
    },
    StateTransition {
        entity: String,
        from_state: String,
        to_state: String,
        condition: Option<String>,
        timestamp: Option<f64>,
        confidence: f64,
    },
    Capability {
        entity: String,
        capability: String,
        level: Option<f64>,
        context: Option<String>,
        confidence: f64,
    },
    Belief {
        entity: String,
        belief: String,
        confidence_level: f64,
        evidence: Option<String>,
        context: Option<String>,
    },
    SemanticAtom {
        atom_type: AtomType,
        content: HashMap<String, String>,
        atom_id: Option<Uuid>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MFragment {
    pub id: Uuid,
    pub fragment_type: FragmentType,
    pub content: FragmentContent,
    pub confidence: f64,
    pub salience: f64,
    pub emotional_tag: f64,
    pub reinforcement_count: u32,
    pub last_activated: f64,
    pub activation_history: Vec<f64>,
    pub created_at: f64,
    pub decay_rate: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EdgeType {
    Causal,
    Temporal,
    Semantic,
    Contextual,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub from_fragment: Uuid,
    pub to_fragment: Uuid,
    pub edge_type: EdgeType,
    pub strength: f64,
    pub last_reinforced: f64,
    pub created_at: f64,
    pub decay_rate: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ActivationIndex {
    pub by_goal: HashMap<String, HashSet<Uuid>>,
    pub by_domain: HashMap<String, HashSet<Uuid>>,
    pub by_keyword: HashMap<String, HashSet<Uuid>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoActivationPattern {
    pub fragment_ids: Vec<Uuid>,
    pub activation_count: usize,
    pub average_confidence: f64,
    pub last_activated: f64,
    pub formatting_pattern: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryGraph {
    pub fragments: HashMap<Uuid, MFragment>,
    pub edges: HashMap<(Uuid, Uuid), Edge>,
    pub activation_index: ActivationIndex,
    pub compiled_modules: Vec<CompiledModule>,
    pub co_activation_patterns: Vec<CoActivationPattern>,
    pub version: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GoalType {
    Debug,
    Create,
    Learn,
    Explain,
    Predict,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalSpec {
    pub description: String,
    pub goal_type: GoalType,
    pub parameters: HashMap<String, String>,
    pub priority: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AttentionWindow {
    pub focus_entities: HashSet<String>,
    pub focus_domains: HashSet<String>,
    pub focus_relations: HashSet<String>,
    pub exclusion_patterns: HashSet<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EmotionalState {
    pub frustration: f64,
    pub curiosity: f64,
    pub confidence: f64,
    pub urgency: f64,
    pub satisfaction: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Constraints {
    pub must_include: HashSet<String>,
    pub must_exclude: HashSet<String>,
    pub resource_limits: HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainPattern {
    pub domain: String,
    pub subdomain: Option<String>,
    pub tags: HashSet<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextVector {
    pub goal: GoalSpec,
    pub attention_window: AttentionWindow,
    pub emotional_bias: EmotionalState,
    pub environmental_constraints: Constraints,
    pub recent_activations: Vec<Uuid>,
    pub time_pressure: f64,
    pub domain_hint: DomainPattern,
    pub confidence_threshold: f64,
    pub max_fragments: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NodeType {
    FragmentNode,
    ConflictNode,
    GapFillNode,
    DecisionNode,
    ActionNode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeContent {
    Fragment {
        fragment_id: Uuid,
        interpretation: String,
    },
    Conflict {
        conflicting_fragments: Vec<Uuid>,
        selected_fragment: Option<Uuid>,
    },
    GapFill {
        gap_description: String,
        estimated_confidence: f64,
    },
    Decision {
        condition: String,
        branches: Vec<Branch>,
    },
    Action {
        action_type: String,
        parameters: HashMap<String, String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Branch {
    pub condition: String,
    pub target_node: Uuid,
    pub weight: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EEGNode {
    pub id: Uuid,
    pub node_type: NodeType,
    pub content: NodeContent,
    pub confidence: f64,
    pub source_fragments: Vec<Uuid>,
    pub execution_cost: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EEGEdge {
    pub from_node: Uuid,
    pub to_node: Uuid,
    pub edge_type: EdgeType,
    pub condition: Option<String>,
    pub weight: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EEGMetadata {
    pub compilation_timestamp: f64,
    pub fragment_count: usize,
    pub estimated_execution_time: f64,
    pub confidence_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EEG {
    pub nodes: HashMap<Uuid, EEGNode>,
    pub edges: Vec<EEGEdge>,
    pub entry_point: Uuid,
    pub exit_points: Vec<Uuid>,
    pub metadata: EEGMetadata,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OutcomeType {
    Success,
    Failure,
    Partial,
    Uncertain,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Outcome {
    pub outcome_type: OutcomeType,
    pub result: String,
    pub explanation: Option<String>,
    pub confidence: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SignalType {
    Positive,
    Negative,
    Neutral,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReinforcementSignal {
    pub fragment_id: Uuid,
    pub signal_type: SignalType,
    pub strength: f64,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub outcome: Outcome,
    pub execution_trace: Vec<Uuid>,
    pub confidence: f64,
    pub time_taken: f64,
    pub reinforcement_signals: Vec<ReinforcementSignal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionTrace {
    pub eeg_id: Uuid,
    pub context: ContextVector,
    pub node_sequence: Vec<Uuid>,
    pub branch_decisions: HashMap<Uuid, Uuid>,
    pub execution_time: f64,
    pub timestamp: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathPattern {
    pub path: Vec<Uuid>,
    pub occurrence_count: usize,
    pub contexts: Vec<ContextVector>,
    pub average_confidence: f64,
    pub success_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchPattern {
    pub decision_node: Uuid,
    pub dominant_branch: Uuid,
    pub branch_ratio: f64,
    pub contexts: Vec<ContextVector>,
    pub average_confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubgraphPattern {
    pub subgraph_nodes: Vec<Uuid>,
    pub occurrence_count: usize,
    pub contexts: Vec<ContextVector>,
    pub average_confidence: f64,
    pub context_variance: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutcomePattern {
    pub outcome_type: OutcomeType,
    pub occurrence_count: usize,
    pub average_confidence: f64,
    pub success_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FossilizationCandidate {
    pub pattern_type: String,
    pub pattern_id: Uuid,
    pub repetition_count: usize,
    pub average_confidence: f64,
    pub context_variance: f64,
    pub reward_correlation: f64,
    pub estimated_speedup: f64,
    pub priority: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternReport {
    pub repeated_paths: Vec<PathPattern>,
    pub stable_branches: Vec<BranchPattern>,
    pub invariant_subgraphs: Vec<SubgraphPattern>,
    pub high_confidence_outcomes: Vec<OutcomePattern>,
    pub fossilization_candidates: Vec<FossilizationCandidate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinterInput {
    pub execution_traces: Vec<ExecutionTrace>,
    pub compiled_eegs: Vec<EEG>,
    pub execution_results: Vec<ExecutionResult>,
    pub time_window: Option<f64>,
    pub min_occurrences: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinterConfig {
    pub min_occurrences: usize,
    pub min_path_length: usize,
    pub min_confidence: f64,
    pub min_branch_ratio: f64,
    pub min_context_variance: f64,
    pub min_reward_correlation: f64,
    pub min_speedup: f64,
}

impl Default for LinterConfig {
    fn default() -> Self {
        LinterConfig {
            min_occurrences: 5,
            min_path_length: 3,
            min_confidence: 0.7,
            min_branch_ratio: 0.8,
            min_context_variance: 0.3,
            min_reward_correlation: 0.7,
            min_speedup: 2.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ModuleType {
    FSM,
    DecisionTable,
    Bytecode,
    MachineCode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompiledModule {
    pub id: Uuid,
    pub module_type: ModuleType,
    pub code: Vec<u8>,
    pub input_signature: InputSignature,
    pub output_signature: OutputSignature,
    pub activation_condition: ContextPattern,
    pub confidence: f64,
    pub usage_count: usize,
    pub success_count: usize,
    pub failure_count: usize,
    pub last_used: f64,
    pub created_at: f64,
    pub source_pattern: Uuid,
    pub version: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedPattern {
    pub pattern_type: String,
    pub structure: PatternStructure,
    pub input_signature: InputSignature,
    pub output_signature: OutputSignature,
    pub activation_condition: ContextPattern,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternStructure {
    pub nodes: Vec<Uuid>,
    pub edges: Vec<(Uuid, Uuid)>,
    pub node_types: HashMap<Uuid, NodeType>,
    pub edge_types: HashMap<(Uuid, Uuid), EdgeType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputSignature {
    pub parameters: Vec<String>,
    pub context_requirements: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputSignature {
    pub return_type: String,
    pub side_effects: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextPattern {
    pub goal_patterns: Vec<String>,
    pub domain_hints: Vec<String>,
    pub confidence_threshold: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FSMState {
    pub id: usize,
    pub name: String,
    pub action: String,
    pub is_accepting: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FSMTransition {
    pub from_state: usize,
    pub to_state: usize,
    pub condition: Option<String>,
    pub action: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FossilizationConfig {
    pub min_repetition: usize,
    pub min_confidence: f64,
    pub max_context_variance: f64,
    pub min_reward_correlation: f64,
    pub min_speedup: f64,
    pub max_candidates_per_run: usize,
    pub preferred_module_type: ModuleType,
}

impl Default for FossilizationConfig {
    fn default() -> Self {
        FossilizationConfig {
            min_repetition: 10,
            min_confidence: 0.8,
            max_context_variance: 0.3,
            min_reward_correlation: 0.7,
            min_speedup: 2.0,
            max_candidates_per_run: 5,
            preferred_module_type: ModuleType::FSM,
        }
    }
}

pub fn current_timestamp() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs_f64()
}
