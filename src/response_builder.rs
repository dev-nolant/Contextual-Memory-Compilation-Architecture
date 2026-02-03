// Copyright (c) 2026 Nolan Taft
use crate::llm_integration::MemoryData;
use crate::types::*;
use std::collections::HashSet;
use uuid::Uuid;

pub struct ResponseBuilder;

#[derive(Debug, Clone)]
enum FormatType {
    Possessive { entity_key: String },
    Direct,
}

impl ResponseBuilder {
    pub fn extract_top_candidates(
        memory_data: &MemoryData,
        memory: &MemoryGraph,
        execution_result: &ExecutionResult,
    ) -> Vec<(String, f64)> {
        eprintln!("\n [DEBUG] Answer Extraction:");
        eprintln!("Memory data fragments: {}", memory_data.fragments.len());
        eprintln!(
            "Execution trace length: {}",
            execution_result.execution_trace.len()
        );

        if memory_data.fragments.is_empty() {
            eprintln!("No fragments in memory_data");
            return Vec::new();
        }

        let query_fragments: Vec<usize> = memory_data
            .fragments
            .iter()
            .enumerate()
            .filter(|(_, frag)| Self::is_query_fragment(frag))
            .map(|(i, _)| i)
            .collect();

        eprintln!("Query fragments (indices): {:?}", query_fragments);
        if query_fragments.is_empty() {
            eprintln!("No query fragments found");
            return Vec::new();
        }

        let mut answer_fragments =
            Self::find_answer_fragments(&execution_result.execution_trace, memory);

        if !execution_result.execution_trace.is_empty() {
            let graph_traversal_answers = Self::find_answers_via_graph_traversal(
                &execution_result.execution_trace,
                memory,
                memory_data,
            );

            for (frag_id, confidence) in graph_traversal_answers {
                if !answer_fragments.contains(&frag_id) {
                    answer_fragments.push(frag_id);
                    eprintln!(
                        "Added answer fragment from graph traversal: {} (confidence={:.2})",
                        frag_id, confidence
                    );
                }
            }
        }

        eprintln!(
            "Answer fragment IDs: {:?} ({} fragments)",
            &answer_fragments[..answer_fragments.len().min(10)],
            answer_fragments.len()
        );

        let candidates = Self::extract_values_from_fragments(
            &answer_fragments,
            memory,
            &query_fragments,
            memory_data,
        );

        eprintln!("Candidates found: {}", candidates.len());
        for (i, (value, conf)) in candidates.iter().take(5).enumerate() {
            eprintln!("Candidate {}: value='{}', confidence={:.3}", i, value, conf);
        }

        let scored_candidates: Vec<(String, f64)> = candidates
            .into_iter()
            .map(|(value, base_confidence)| {
                let structural_confidence =
                    if execution_result
                        .execution_trace
                        .iter()
                        .any(|query_frag_id| {
                            memory.edges.iter().any(|((from_id, to_id), edge)| {
                                answer_fragments.iter().any(|answer_frag_id| {
                                    let connects_query_to_answer = (*from_id == *query_frag_id
                                        && *to_id == *answer_frag_id)
                                        || (*to_id == *query_frag_id
                                            && *from_id == *answer_frag_id);
                                    connects_query_to_answer && edge.strength > 0.5
                                })
                            })
                        })
                    {
                        base_confidence * 1.1
                    } else {
                        base_confidence
                    };

                (value, structural_confidence.min(1.0))
            })
            .filter(|(_, confidence)| {
                let keep = *confidence > 0.3;
                if !keep {
                    eprintln!(
                        "Filtered out candidate (confidence {:.3} <= 0.3)",
                        confidence
                    );
                }
                keep
            })
            .collect();

        eprintln!(
            "Scored candidates after filtering: {}",
            scored_candidates.len()
        );

        let mut top_candidates: Vec<(String, f64)> = scored_candidates.into_iter().collect();
        top_candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        top_candidates.truncate(3);

        if !top_candidates.is_empty() {
            eprintln!("Top {} candidates:", top_candidates.len());
            for (i, (value, conf)) in top_candidates.iter().enumerate() {
                eprintln!("Rank {}: '{}' with confidence {:.3}", i + 1, value, conf);
            }
            return top_candidates;
        }

        {
            eprintln!("No answer found in activated fragments, searching all fragments...");

            let query_references_fallback: HashSet<String> = memory_data
                .fragments
                .iter()
                .flat_map(|frag| {
                    let mut refs = HashSet::new();
                    for key in frag.content.keys() {
                        refs.insert(key.clone());
                        refs.insert(key.to_lowercase());
                    }
                    for value in frag.content.values() {
                        refs.insert(value.clone());
                        refs.insert(value.to_lowercase());
                    }
                    refs
                })
                .collect();

            let query_keywords: HashSet<String> = memory_data
                .query
                .split_whitespace()
                .map(|w| {
                    w.trim_matches(|c: char| !c.is_alphanumeric())
                        .to_lowercase()
                })
                .filter(|w| w.len() > 2)
                .collect();

            let query_lower = memory_data.query.to_lowercase();
            let mut exclusion_terms = HashSet::new();

            for word in query_lower.split_whitespace() {
                if word.starts_with("non-") {
                    let excluded = word.strip_prefix("non-").unwrap_or("");
                    if excluded.len() > 2 {
                        exclusion_terms.insert(excluded.to_string());

                        exclusion_terms.insert(format!("{} name", excluded));
                        exclusion_terms.insert(format!("{}_name", excluded));
                    }
                }
            }

            let words: Vec<&str> = query_lower.split_whitespace().collect();
            for i in 0..words.len().saturating_sub(1) {
                if words[i] == "not" && i + 1 < words.len() {
                    let excluded = words[i + 1].trim_matches(|c: char| !c.is_alphanumeric());
                    if excluded.len() > 2 {
                        exclusion_terms.insert(excluded.to_string());
                    }
                }
            }

            eprintln!("Query keywords: {:?}", query_keywords);
            if !exclusion_terms.is_empty() {
                eprintln!("Exclusion terms: {:?}", exclusion_terms);
            }

            let mut all_candidates: Vec<(String, f64, String)> = Vec::new();

            for (_fragment_id, frag) in memory.fragments.iter() {
                match &frag.content {
                    FragmentContent::PersonalFact {
                        fact_type, value, ..
                    } => {
                        let fact_type_lower = fact_type.to_lowercase();

                        if exclusion_terms
                            .iter()
                            .any(|excl| fact_type_lower.contains(excl))
                        {
                            eprintln!(
                                "Skipping PersonalFact (matches exclusion): fact_type={}",
                                fact_type
                            );
                            continue;
                        }

                        if query_keywords.iter().any(|kw| fact_type_lower.contains(kw))
                            && !value.is_empty()
                            && value != "unknown"
                        {
                            let mut score = 0.8;

                            if query_keywords.iter().any(|kw| fact_type_lower == *kw) {
                                score += 0.1;
                            }

                            if query_references_fallback.contains(&value.to_lowercase()) {
                                score -= 0.3;
                            }
                            eprintln!(
                                "Candidate PersonalFact: fact_type={}, value={}, score={:.3}",
                                fact_type, value, score
                            );
                            all_candidates.push((value.clone(), score, "PersonalFact".to_string()));
                        }
                    }
                    FragmentContent::SemanticAtom {
                        atom_type, content, ..
                    } => {
                        let name_value_opt = if matches!(atom_type, AtomType::Person) {
                            content.get("name")
                        } else if matches!(atom_type, AtomType::Entity) {
                            if content.get("key").map(|k| k == "name").unwrap_or(false) {
                                content.get("name")
                            } else {
                                None
                            }
                        } else {
                            None
                        };

                        if let Some(name_value) = name_value_opt {
                            if !name_value.is_empty() && name_value != "unknown" {
                                let name_lower = name_value.to_lowercase();

                                let matches_exclusion =
                                    exclusion_terms.iter().any(|excl| name_lower.contains(excl));

                                if matches_exclusion {
                                    eprintln!(
                                        "Skipping Person name (matches exclusion): name={}",
                                        name_value
                                    );
                                    continue;
                                }

                                if query_keywords.contains("name") || query_lower.contains("name") {
                                    let mut score = 0.8;

                                    if query_keywords.contains("name") {
                                        score += 0.15;
                                    }

                                    if name_value.chars().all(|c| c.is_alphabetic())
                                        && name_value.len() <= 20
                                    {
                                        score += 0.1;
                                    }

                                    if matches!(atom_type, AtomType::Entity)
                                        && content.contains_key("ownership_marker")
                                    {
                                        score += 0.05;
                                    }

                                    let matches_query_ref = query_references_fallback
                                        .contains(name_value)
                                        || query_references_fallback.contains(&name_lower);

                                    if matches_query_ref {
                                        score -= 0.1;
                                    }
                                    eprintln!(
                                        "Candidate Person name (from {:?}): name={}, score={:.3}",
                                        atom_type, name_value, score
                                    );
                                    all_candidates.push((
                                        name_value.clone(),
                                        score,
                                        "PersonName".to_string(),
                                    ));
                                }
                            }
                        }

                        if matches!(atom_type, AtomType::Object) {
                            if let Some(value) = content.get("key") {
                                if !value.is_empty() && value != "unknown" {
                                    let is_query_entity = query_references_fallback.contains(value)
                                        || query_references_fallback
                                            .contains(&value.to_lowercase());

                                    let query_has_action =
                                        memory_data.fragments.iter().any(|frag| {
                                            frag.atom_type == "Action"
                                                || (frag.atom_type == "Entity"
                                                    && frag
                                                        .content
                                                        .get("key")
                                                        .map(|k| k.len() > 2 && k.len() < 10)
                                                        .unwrap_or(false)
                                                    && frag
                                                        .content
                                                        .contains_key("ownership_marker"))
                                        });

                                    if query_has_action {
                                        let mut score = 0.6;

                                        if content.contains_key("ownership_marker") {
                                            score += 0.1;
                                        }

                                        if is_query_entity && Self::is_meaningful_entity(value) {
                                            score += 0.4;
                                            eprintln!("Candidate Object (query entity): value={}, score={:.3}", value, score);
                                        } else {
                                            eprintln!(
                                                "Candidate Object: value={}, score={:.3}",
                                                value, score
                                            );
                                        }
                                        all_candidates.push((
                                            value.clone(),
                                            score,
                                            "Object".to_string(),
                                        ));
                                    }
                                }
                            }
                        }

                        if matches!(atom_type, AtomType::Action) {
                            if let Some(value) = content.get("key") {
                                if !value.is_empty() && value != "unknown" {
                                    let is_query_entity = query_references_fallback.contains(value)
                                        || query_references_fallback
                                            .contains(&value.to_lowercase());

                                    let query_has_action =
                                        memory_data.fragments.iter().any(|frag| {
                                            frag.atom_type == "Action"
                                                || (frag.atom_type == "Entity"
                                                    && frag
                                                        .content
                                                        .get("key")
                                                        .map(|k| k.len() > 2 && k.len() < 10)
                                                        .unwrap_or(false)
                                                    && frag
                                                        .content
                                                        .contains_key("ownership_marker"))
                                        });

                                    if query_has_action {
                                        let mut score = 0.6;

                                        if is_query_entity && Self::is_meaningful_entity(value) {
                                            score += 0.4;
                                            eprintln!("Candidate Action (query entity): value={}, score={:.3}", value, score);
                                        } else {
                                            eprintln!(
                                                "Candidate Action: value={}, score={:.3}",
                                                value, score
                                            );
                                        }
                                        all_candidates.push((
                                            value.clone(),
                                            score,
                                            "Action".to_string(),
                                        ));
                                    }
                                }
                            }
                        }

                        if matches!(atom_type, AtomType::Entity) {
                            if let Some(value) = content.get("key") {
                                if !value.is_empty() && value != "unknown" && value != "name" {
                                    let is_query_entity = query_references_fallback.contains(value)
                                        || query_references_fallback
                                            .contains(&value.to_lowercase());

                                    if exclusion_terms
                                        .iter()
                                        .any(|excl| value.to_lowercase().contains(excl))
                                    {
                                        continue;
                                    }

                                    let query_has_action =
                                        memory_data.fragments.iter().any(|frag| {
                                            frag.atom_type == "Action"
                                                || (frag.atom_type == "Entity"
                                                    && frag
                                                        .content
                                                        .get("key")
                                                        .map(|k| k.len() > 2 && k.len() < 10)
                                                        .unwrap_or(false)
                                                    && frag
                                                        .content
                                                        .contains_key("ownership_marker"))
                                        });

                                    if query_has_action && content.contains_key("ownership_marker")
                                    {
                                        let mut score = 0.5;

                                        if value.chars().all(|c| c.is_alphabetic())
                                            && value.len() <= 20
                                        {
                                            score += 0.1;
                                        }

                                        if is_query_entity && Self::is_meaningful_entity(value) {
                                            score += 0.4;
                                            eprintln!("Candidate Entity preference (query entity): value={}, score={:.3}", value, score);
                                        } else {
                                            eprintln!("Candidate Entity preference: value={}, score={:.3}", value, score);
                                        }
                                        all_candidates.push((
                                            value.clone(),
                                            score,
                                            "EntityPreference".to_string(),
                                        ));
                                    }
                                }
                            }
                        }

                        for (key, value) in content {
                            let key_lower = key.to_lowercase();
                            let value_lower = value.to_lowercase();

                            if value.is_empty()
                                || value == "unknown"
                                || value == "key"
                                || query_references_fallback.contains(value)
                                || query_references_fallback.contains(&value_lower)
                            {
                                continue;
                            }

                            if exclusion_terms.iter().any(|excl| key_lower.contains(excl)) {
                                eprintln!(
                                    "Skipping SemanticAtom (key matches exclusion): {}={}",
                                    key, value
                                );
                                continue;
                            }

                            if query_keywords.iter().any(|kw| key_lower.contains(kw)) {
                                let looks_like_answer = value.contains('-')
                                    || value.contains('_')
                                    || value.len() > 5
                                    || !value.chars().all(|c| c.is_alphabetic());

                                if looks_like_answer {
                                    let mut score = 0.4;

                                    if query_keywords.iter().any(|kw| key_lower == *kw) {
                                        score += 0.2;
                                    }

                                    if exclusion_terms.iter().any(|excl| key_lower.contains(excl)) {
                                        score -= 0.5;
                                    }

                                    if value.contains('-') || value.contains('_') {
                                        score += 0.1;
                                    }
                                    eprintln!(
                                        "Candidate SemanticAtom: {}={}, score={:.3}",
                                        key, value, score
                                    );
                                    all_candidates.push((
                                        value.clone(),
                                        score,
                                        "SemanticAtom".to_string(),
                                    ));
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }

            if !all_candidates.is_empty() {
                eprintln!(
                    "Found {} candidates, ranking statistically...",
                    all_candidates.len()
                );

                all_candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

                for (i, (value, score, source)) in all_candidates.iter().take(5).enumerate() {
                    eprintln!(
                        "Rank {}: value='{}', score={:.3}, source={}",
                        i + 1,
                        value,
                        score,
                        source
                    );
                }

                let top_candidates: Vec<(String, f64)> = all_candidates
                    .into_iter()
                    .filter(|(_, score, _)| *score > 0.3)
                    .take(3)
                    .map(|(value, score, _)| (value, score))
                    .collect();

                if !top_candidates.is_empty() {
                    eprintln!(
                        "Returning top {} candidates from fallback search",
                        top_candidates.len()
                    );
                    return top_candidates;
                }
            }

            eprintln!("No answer found after searching all fragments");
        }

        Vec::new()
    }

    pub fn extract_answer(
        memory_data: &MemoryData,
        memory: &MemoryGraph,
        execution_result: &ExecutionResult,
    ) -> Option<(String, f64)> {
        let candidates = Self::extract_top_candidates(memory_data, memory, execution_result);
        candidates.into_iter().next()
    }

    fn is_query_fragment(frag: &crate::llm_integration::FragmentData) -> bool {
        let mut has_complete_value = false;
        let mut has_only_key = true;

        for (key, value) in &frag.content {
            if key != "key" {
                has_only_key = false;
            }

            if !value.is_empty() && value != "unknown" && value != "key" && value.len() > 2 {
                if !Self::is_reference_value(value) {
                    has_complete_value = true;
                }
            }
        }

        !has_complete_value || (has_only_key && frag.content.len() == 1)
    }

    fn is_reference_value(value: &str) -> bool {
        value.len() <= 15
            && value
                .chars()
                .all(|c| c.is_lowercase() || c == '_' || c == '-')
            && !value.contains(' ')
            && value
                .chars()
                .all(|c| c.is_alphabetic() || c == '_' || c == '-')
    }

    fn is_meaningful_entity(value: &str) -> bool {
        let value_lower = value.to_lowercase();

        let common_words: &[&str] = &[
            "like", "i", "do", "is", "are", "was", "were", "and", "or", "the", "a", "an", "my",
            "your", "his", "her", "their", "our", "what", "who", "where", "when", "why", "how",
            "which", "that", "this", "these", "those", "have", "has", "had",
        ];

        if common_words.contains(&value_lower.as_str()) {
            return false;
        }

        if value.len() < 3 {
            return false;
        }

        if !value.chars().all(|c| c.is_alphabetic()) {
            return false;
        }

        true
    }

    fn answer_matches_query_intent(
        answer_value: &str,
        query_indices: &[usize],
        memory_data: &MemoryData,
        memory: &MemoryGraph,
        answer_fragment_ids: &[Uuid],
    ) -> bool {
        let mut answer_key: Option<String> = None;
        for fragment_id in answer_fragment_ids {
            if let Some(frag) = memory.fragments.get(fragment_id) {
                match &frag.content {
                    FragmentContent::SemanticAtom { content, .. } => {
                        for (key, value) in content {
                            if value == answer_value && key != "key" {
                                answer_key = Some(key.clone());
                                break;
                            }
                        }
                    }
                    FragmentContent::PersonalFact {
                        value, fact_type, ..
                    } => {
                        if value == answer_value {
                            answer_key = Some(fact_type.clone());
                            break;
                        }
                    }
                    _ => {}
                }
                if answer_key.is_some() {
                    break;
                }
            }
        }

        let query_keys: HashSet<String> = query_indices
            .iter()
            .flat_map(|&i| {
                let frag = &memory_data.fragments[i];
                let mut keys = HashSet::new();

                for key in frag.content.keys() {
                    if key != "key" {
                        keys.insert(key.clone());
                        keys.insert(key.to_lowercase());
                    }
                }

                for value in frag.content.values() {
                    if value.contains(' ') {
                        for word in value.split_whitespace() {
                            if word.len() > 2 {
                                keys.insert(word.to_lowercase());
                            }
                        }
                    } else if value.len() > 2 {
                        keys.insert(value.to_lowercase());
                    }
                }
                keys
            })
            .collect();

        if let Some(ref key) = answer_key {
            if query_keys.contains(key) || query_keys.contains(&key.to_lowercase()) {
                return true;
            }

            for qk in &query_keys {
                if qk.contains(key) || key.contains(qk) {
                    return true;
                }
            }
        }

        answer_key.is_some()
    }

    fn find_answer_fragments(activated_fragment_ids: &[Uuid], memory: &MemoryGraph) -> Vec<Uuid> {
        let mut answer_fragments = HashSet::new();

        eprintln!(
            "\n [DEBUG] Finding answer fragments from {} activated fragments",
            activated_fragment_ids.len()
        );

        let all_personal_facts: Vec<_> = memory
            .fragments
            .iter()
            .filter(|(_, frag)| matches!(frag.fragment_type, FragmentType::PersonalFact))
            .collect();
        eprintln!(
            "Total PersonalFact fragments in memory: {}",
            all_personal_facts.len()
        );
        for (id, frag) in &all_personal_facts {
            if let FragmentContent::PersonalFact {
                person,
                fact_type,
                value,
                ..
            } = &frag.content
            {
                let is_activated = activated_fragment_ids.contains(id);
                eprintln!(
                    "PersonalFact {}: person={}, fact_type={}, value={}, activated={}",
                    id, person, fact_type, value, is_activated
                );
            }
        }

        eprintln!("Using structural pattern matching to find answer fragments:");

        let mut connected_answer_fragments = HashSet::new();
        for fragment_id in activated_fragment_ids {
            for ((from_id, to_id), edge) in &memory.edges {
                let connected_id = if *from_id == *fragment_id {
                    Some(*to_id)
                } else if *to_id == *fragment_id {
                    Some(*from_id)
                } else {
                    None
                };

                if let Some(connected_id) = connected_id {
                    let is_answer_edge =
                        matches!(edge.edge_type, EdgeType::Semantic | EdgeType::Contextual);

                    if is_answer_edge && edge.strength > 0.3 {
                        if let Some(frag) = memory.fragments.get(&connected_id) {
                            let is_answer_type = matches!(
                                frag.fragment_type,
                                FragmentType::PersonalFact | FragmentType::OwnershipRelation
                            ) || Self::is_answer_fragment(frag);

                            if is_answer_type {
                                connected_answer_fragments.insert(connected_id);
                                eprintln!("Found answer fragment via edge traversal: {} (edge_type={:?}, strength={:.2})", 
                                    connected_id, edge.edge_type, edge.strength);
                            }
                        }
                    }
                }
            }
        }

        for fragment_id in activated_fragment_ids {
            if let Some(frag) = memory.fragments.get(fragment_id) {
                if Self::is_answer_fragment(frag) {
                    eprintln!(
                        "Found answer fragment in activated set: {} (has complete values)",
                        fragment_id
                    );
                }
            }
        }

        for fragment_id in activated_fragment_ids {
            if let Some(frag) = memory.fragments.get(fragment_id) {
                match &frag.fragment_type {
                    FragmentType::PersonalFact => {
                        if let FragmentContent::PersonalFact {
                            person,
                            fact_type,
                            value,
                            ..
                        } = &frag.content
                        {
                            eprintln!("Found PersonalFact in activated: person={}, fact_type={}, value={}", person, fact_type, value);
                        }
                        answer_fragments.insert(*fragment_id);
                    }
                    FragmentType::OwnershipRelation => {
                        if let FragmentContent::OwnershipRelation { owner, owned, .. } =
                            &frag.content
                        {
                            eprintln!("Found OwnershipRelation: owner={}, owned={}", owner, owned);
                        }
                        answer_fragments.insert(*fragment_id);
                    }
                    FragmentType::SemanticAtom => {
                        if Self::is_answer_fragment(frag) {
                            eprintln!("Found answer SemanticAtom: {:?}", frag.content);
                            answer_fragments.insert(*fragment_id);
                        }
                    }
                    FragmentType::SemanticAtom => {
                        if Self::is_answer_fragment(frag) {
                            eprintln!("Found answer SemanticAtom: {:?}", frag.content);
                            answer_fragments.insert(*fragment_id);
                        }

                        if let FragmentContent::SemanticAtom { atom_type, .. } = &frag.content {
                            if matches!(atom_type, AtomType::Object) {
                                answer_fragments.insert(*fragment_id);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        for fragment_id in activated_fragment_ids {
            for ((from_id, to_id), _edge) in &memory.edges {
                let connected_id = if *from_id == *fragment_id {
                    Some(*to_id)
                } else if *to_id == *fragment_id {
                    Some(*from_id)
                } else {
                    None
                };

                if let Some(connected_id) = connected_id {
                    if let Some(frag) = memory.fragments.get(&connected_id) {
                        match &frag.fragment_type {
                            FragmentType::PersonalFact | FragmentType::OwnershipRelation => {
                                answer_fragments.insert(connected_id);
                            }
                            FragmentType::SemanticAtom => {
                                if Self::is_answer_fragment(frag) {
                                    answer_fragments.insert(connected_id);
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        answer_fragments.into_iter().collect()
    }

    fn is_answer_fragment(frag: &MFragment) -> bool {
        match &frag.content {
            FragmentContent::SemanticAtom { content, .. } => {
                for (key, value) in content {
                    if value.is_empty() || value == "unknown" || value == "key" {
                        continue;
                    }
                    if !Self::is_reference_value(value) {
                        return true;
                    }
                }
            }
            FragmentContent::PersonalFact { value, .. } => {
                if !value.is_empty() && value != "unknown" {
                    return true;
                }
            }
            FragmentContent::OwnershipRelation { owned, .. } => {
                if !owned.is_empty() && owned != "unknown" {
                    return true;
                }
            }
            _ => {}
        }
        false
    }

    fn find_answers_via_graph_traversal(
        query_fragment_ids: &[Uuid],
        memory: &MemoryGraph,
        memory_data: &MemoryData,
    ) -> Vec<(Uuid, f64)> {
        let mut answer_fragments = Vec::new();
        let mut visited = HashSet::new();

        let query_has_action = memory_data
            .fragments
            .iter()
            .any(|frag| frag.atom_type == "Action");
        let query_has_person = memory_data
            .fragments
            .iter()
            .any(|frag| frag.atom_type == "Person");

        for query_id in query_fragment_ids {
            if visited.contains(query_id) {
                continue;
            }
            visited.insert(*query_id);

            for ((from_id, to_id), edge) in &memory.edges {
                let connected_id = if *from_id == *query_id {
                    Some(*to_id)
                } else if *to_id == *query_id {
                    Some(*from_id)
                } else {
                    None
                };

                if let Some(connected_id) = connected_id {
                    if visited.contains(&connected_id) {
                        continue;
                    }

                    let is_answer_edge =
                        matches!(edge.edge_type, EdgeType::Semantic | EdgeType::Contextual);

                    if is_answer_edge && edge.strength > 0.3 {
                        if let Some(frag) = memory.fragments.get(&connected_id) {
                            let matches_query_structure = match &frag.content {
                                FragmentContent::SemanticAtom {
                                    atom_type, content, ..
                                } => {
                                    if query_has_action {
                                        matches!(atom_type, AtomType::Object | AtomType::Entity)
                                            && content.contains_key("key")
                                            && content
                                                .get("key")
                                                .map(|k| !k.is_empty() && k != "unknown")
                                                .unwrap_or(false)
                                    } else {
                                        Self::is_answer_fragment(frag)
                                    }
                                }
                                FragmentContent::PersonalFact { .. } => true,
                                FragmentContent::OwnershipRelation { .. } => true,
                                _ => false,
                            };

                            if matches_query_structure {
                                visited.insert(connected_id);

                                answer_fragments.push((connected_id, edge.strength));
                                eprintln!("Found answer via graph traversal: {} (edge_type={:?}, strength={:.2})",
                                    connected_id, edge.edge_type, edge.strength);
                            }
                        }
                    }
                }
            }
        }

        answer_fragments
    }

    fn extract_values_from_fragments(
        answer_fragment_ids: &[Uuid],
        memory: &MemoryGraph,
        query_indices: &[usize],
        memory_data: &MemoryData,
    ) -> Vec<(String, f64)> {
        let mut candidates = Vec::new();

        eprintln!(
            "\n [DEBUG] Extracting values from {} answer fragments",
            answer_fragment_ids.len()
        );

        let query_references: HashSet<String> = {
            let mut refs = HashSet::new();

            let query_text_lower = memory_data.query.to_lowercase();
            for word in query_text_lower.split_whitespace() {
                let word_clean = word.trim_matches(|c: char| !c.is_alphanumeric());
                if word_clean.len() > 1 {
                    refs.insert(word_clean.to_string());
                }
            }

            for &i in query_indices {
                let frag = &memory_data.fragments[i];
                for (key, value) in &frag.content {
                    let key_lower = key.to_lowercase();

                    let is_metadata_key = key_lower.ends_with("_role")
                        || key_lower.ends_with("_marker")
                        || key_lower == "key";
                    if is_metadata_key {
                        refs.insert(key_lower.clone());

                        let value_lower = value.to_lowercase();
                        if value_lower.len() < 15 && !value_lower.contains(' ') {
                            refs.insert(value_lower);
                        }
                    }
                }
            }

            refs.insert("unknown".to_string());
            refs
        };

        eprintln!(
            "Query references to exclude: {:?}",
            &query_references.iter().take(15).collect::<Vec<_>>()
        );

        for fragment_id in answer_fragment_ids {
            if let Some(frag) = memory.fragments.get(fragment_id) {
                match &frag.content {
                    FragmentContent::PersonalFact {
                        value,
                        fact_type,
                        person,
                        ..
                    } => {
                        eprintln!(
                            "Checking PersonalFact: person={}, fact_type={}, value={}",
                            person, fact_type, value
                        );
                        if !value.is_empty() && value != "unknown" {
                            if query_references.contains(value)
                                || query_references.contains(&value.to_lowercase())
                            {
                                eprintln!("Filtered out (matches query reference)");
                                continue;
                            }
                            let confidence = Self::calculate_personal_fact_confidence(
                                fact_type,
                                value,
                                query_indices,
                                memory_data,
                            );
                            eprintln!("Confidence: {:.3}", confidence);
                            if confidence > 0.3 {
                                eprintln!("Added candidate: '{}'", value);
                                candidates.push((value.clone(), confidence));
                            } else {
                                eprintln!("Confidence too low: {:.3}", confidence);
                            }
                        } else {
                            eprintln!("Empty or unknown value");
                        }
                    }
                    FragmentContent::OwnershipRelation { owned, .. } => {
                        if !owned.is_empty() && owned != "unknown" {
                            if query_references.contains(owned)
                                || query_references.contains(&owned.to_lowercase())
                            {
                                continue;
                            }
                            let confidence = Self::calculate_ownership_confidence(
                                owned,
                                query_indices,
                                memory_data,
                            );
                            if confidence > 0.3 {
                                candidates.push((owned.clone(), confidence));
                            }
                        }
                    }
                    FragmentContent::SemanticAtom {
                        atom_type, content, ..
                    } => {
                        if matches!(atom_type, AtomType::Object) {
                            if let Some(value) = content.get("key") {
                                if !value.is_empty() && value != "unknown" {
                                    if query_references.contains(value)
                                        || query_references.contains(&value.to_lowercase())
                                    {
                                        continue;
                                    }

                                    let query_action_key = memory_data
                                        .fragments
                                        .iter()
                                        .find(|frag| frag.atom_type == "Action")
                                        .and_then(|frag| frag.content.get("key"));

                                    let value_in_query = memory_data
                                        .query
                                        .to_lowercase()
                                        .contains(&value.to_lowercase());

                                    let is_preference_object =
                                        if let Some(action_key) = query_action_key {
                                            memory.edges.iter().any(|((from_id, to_id), edge)| {
                                                let connected_to_this = *from_id == *fragment_id
                                                    || *to_id == *fragment_id;
                                                if connected_to_this {
                                                    let other_id = if *from_id == *fragment_id {
                                                        *to_id
                                                    } else {
                                                        *from_id
                                                    };
                                                    if let Some(other_frag) =
                                                        memory.fragments.get(&other_id)
                                                    {
                                                        if let FragmentContent::SemanticAtom {
                                                            atom_type: other_type,
                                                            content: other_content,
                                                            ..
                                                        } = &other_frag.content
                                                        {
                                                            return matches!(
                                                                other_type,
                                                                AtomType::Action
                                                            ) && other_content
                                                                .get("key")
                                                                .map(|k| k == action_key)
                                                                .unwrap_or(false);
                                                        }
                                                    }
                                                }
                                                false
                                            })
                                        } else {
                                            false
                                        };

                                    let is_direct_match = value_in_query && is_preference_object;

                                    let query_has_action =
                                        memory_data.fragments.iter().any(|frag| {
                                            frag.atom_type == "Action"
                                                || (frag.atom_type == "Entity"
                                                    && frag
                                                        .content
                                                        .get("key")
                                                        .map(|k| k.len() > 2 && k.len() < 10)
                                                        .unwrap_or(false)
                                                    && frag
                                                        .content
                                                        .contains_key("ownership_marker"))
                                        });

                                    if query_has_action || is_preference_object {
                                        let confidence = if is_direct_match {
                                            0.95
                                        } else if is_preference_object {
                                            0.85
                                        } else {
                                            0.7
                                        };
                                        eprintln!("Checking Object SemanticAtom: value={}, confidence={:.3}, is_preference_object={}, is_direct_match={}", 
                                            value, confidence, is_preference_object, is_direct_match);
                                        eprintln!("Added candidate: '{}'", value);
                                        candidates.push((value.clone(), confidence));
                                        continue;
                                    }
                                }
                            }
                        }

                        if matches!(atom_type, AtomType::Entity) {
                            if let Some(value) = content.get("key") {
                                if value.is_empty() || value == "unknown" {
                                    continue;
                                }

                                let is_query_reference = query_references.contains(value)
                                    || query_references.contains(&value.to_lowercase());

                                let connected_to_answer =
                                    answer_fragment_ids.iter().any(|other_frag_id| {
                                        if *other_frag_id == *fragment_id {
                                            return false;
                                        }
                                        memory.edges.iter().any(|((from_id, to_id), edge)| {
                                            ((*from_id == *fragment_id && *to_id == *other_frag_id)
                                                || (*to_id == *fragment_id
                                                    && *from_id == *other_frag_id))
                                                && edge.strength > 0.3
                                        })
                                    });

                                if is_query_reference && !connected_to_answer {
                                    continue;
                                }

                                let query_asks_for_name_check =
                                    memory_data.fragments.iter().any(|frag| {
                                        frag.content
                                            .get("key")
                                            .map(|k| k == "name")
                                            .unwrap_or(false)
                                    });

                                let query_action_key = if !query_asks_for_name_check {
                                    memory_data
                                        .fragments
                                        .iter()
                                        .find(|frag| frag.atom_type == "Action")
                                        .and_then(|frag| frag.content.get("key"))
                                } else {
                                    None
                                };

                                let is_preference_object = if let Some(action_key) =
                                    query_action_key
                                {
                                    memory.edges.iter().any(|((from_id, to_id), edge)| {
                                        let connected_to_this =
                                            *from_id == *fragment_id || *to_id == *fragment_id;
                                        if connected_to_this && edge.strength > 0.3 {
                                            let other_id = if *from_id == *fragment_id {
                                                *to_id
                                            } else {
                                                *from_id
                                            };
                                            if let Some(other_frag) =
                                                memory.fragments.get(&other_id)
                                            {
                                                if let FragmentContent::SemanticAtom {
                                                    atom_type: other_type,
                                                    content: other_content,
                                                    ..
                                                } = &other_frag.content
                                                {
                                                    return matches!(other_type, AtomType::Action)
                                                        && other_content
                                                            .get("key")
                                                            .map(|k| k == action_key)
                                                            .unwrap_or(false);
                                                }
                                            }
                                        }
                                        false
                                    })
                                } else {
                                    false
                                };

                                let query_asks_for_name =
                                    memory_data.fragments.iter().any(|frag| {
                                        frag.content
                                            .get("key")
                                            .map(|k| k == "name")
                                            .unwrap_or(false)
                                    });

                                let is_name_value = if query_asks_for_name {
                                    let connected_via_edge =
                                        memory.edges.iter().any(|((from_id, to_id), edge)| {
                                            let connected_to_this =
                                                *from_id == *fragment_id || *to_id == *fragment_id;
                                            if connected_to_this && edge.strength > 0.3 {
                                                let other_id = if *from_id == *fragment_id {
                                                    *to_id
                                                } else {
                                                    *from_id
                                                };
                                                if let Some(other_frag) =
                                                    memory.fragments.get(&other_id)
                                                {
                                                    if let FragmentContent::SemanticAtom {
                                                        content: other_content,
                                                        ..
                                                    } = &other_frag.content
                                                    {
                                                        return other_content
                                                            .get("key")
                                                            .map(|k| k == "name")
                                                            .unwrap_or(false);
                                                    }
                                                }
                                            }
                                            false
                                        });

                                    let connected_via_activation =
                                        answer_fragment_ids.iter().any(|other_frag_id| {
                                            if *other_frag_id == *fragment_id {
                                                return false;
                                            }
                                            if let Some(other_frag) =
                                                memory.fragments.get(other_frag_id)
                                            {
                                                if let FragmentContent::SemanticAtom {
                                                    content: other_content,
                                                    ..
                                                } = &other_frag.content
                                                {
                                                    return other_content
                                                        .get("key")
                                                        .map(|k| k == "name")
                                                        .unwrap_or(false);
                                                }
                                            }
                                            false
                                        });

                                    connected_via_edge || connected_via_activation
                                } else {
                                    false
                                };

                                let query_has_action = memory_data.fragments.iter().any(|frag| {
                                    frag.atom_type == "Action"
                                        || (frag.atom_type == "Entity"
                                            && frag
                                                .content
                                                .get("key")
                                                .map(|k| k.len() > 2 && k.len() < 10)
                                                .unwrap_or(false)
                                            && frag.content.contains_key("ownership_marker"))
                                });

                                let is_general_preference =
                                    query_has_action && content.contains_key("ownership_marker");

                                if is_preference_object || is_name_value || is_general_preference {
                                    let confidence = if is_name_value {
                                        0.95
                                    } else if is_preference_object {
                                        0.85
                                    } else {
                                        0.65
                                    };
                                    eprintln!("Checking Entity SemanticAtom: value={}, confidence={:.3}, is_preference_object={}, is_name_value={}", 
                                        value, confidence, is_preference_object, is_name_value);
                                    eprintln!("Added candidate: '{}'", value);
                                    candidates.push((value.clone(), confidence));
                                    continue;
                                }
                            }
                        }

                        for (key, value) in content {
                            if value.is_empty() || value == "unknown" || value == "key" {
                                continue;
                            }

                            let is_query_reference = query_references.contains(value)
                                || query_references.contains(&value.to_lowercase());

                            let connected_to_answer_fragments =
                                answer_fragment_ids.iter().any(|other_frag_id| {
                                    if *other_frag_id == *fragment_id {
                                        return false;
                                    }
                                    memory.edges.iter().any(|((from_id, to_id), edge)| {
                                        ((*from_id == *fragment_id && *to_id == *other_frag_id)
                                            || (*to_id == *fragment_id
                                                && *from_id == *other_frag_id))
                                            && edge.strength > 0.3
                                    })
                                });

                            if is_query_reference && !connected_to_answer_fragments {
                                continue;
                            }

                            let query_text_lower = memory_data.query.to_lowercase();
                            let key_matches_query = query_text_lower.contains(&key.to_lowercase())
                                || query_text_lower
                                    .split_whitespace()
                                    .any(|w| key.to_lowercase().contains(w));

                            let is_name_value = if query_text_lower.contains("name") {
                                memory.edges.iter().any(|((from_id, to_id), edge)| {
                                    let connected_to_this =
                                        *from_id == *fragment_id || *to_id == *fragment_id;
                                    if connected_to_this && edge.strength > 0.3 {
                                        let other_id = if *from_id == *fragment_id {
                                            *to_id
                                        } else {
                                            *from_id
                                        };
                                        if let Some(other_frag) = memory.fragments.get(&other_id) {
                                            if let FragmentContent::SemanticAtom {
                                                content: other_content,
                                                ..
                                            } = &other_frag.content
                                            {
                                                return other_content
                                                    .get("key")
                                                    .map(|k| k == "name")
                                                    .unwrap_or(false);
                                            }
                                        }
                                    }
                                    false
                                })
                            } else {
                                false
                            };

                            let mut confidence = 0.5;

                            if connected_to_answer_fragments {
                                confidence = 0.85;
                            } else if is_name_value {
                                confidence = 0.9;
                            } else if key_matches_query {
                                confidence = 0.7;
                            } else if value.len() > 3 && !Self::is_reference_value(value) {
                                confidence = 0.6;
                            }

                            eprintln!("Checking SemanticAtom value: key={}, value={}, confidence={:.3}, connected_to_answer_fragments={}, is_name_value={}", 
                                key, value, confidence, connected_to_answer_fragments, is_name_value);

                            if confidence > 0.3 {
                                eprintln!("Added candidate from SemanticAtom: '{}'", value);
                                candidates.push((value.clone(), confidence));
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        candidates
    }

    fn calculate_personal_fact_confidence(
        fact_type: &str,
        value: &str,
        query_indices: &[usize],
        memory_data: &MemoryData,
    ) -> f64 {
        let mut confidence: f64 = 0.7;

        let query_keys: HashSet<String> = query_indices
            .iter()
            .flat_map(|&i| memory_data.fragments[i].content.keys())
            .cloned()
            .collect();

        if query_keys.contains(fact_type) {
            confidence += 0.2;
        }

        if value.len() > 3 {
            confidence += 0.1;
        }

        confidence.min(1.0_f64)
    }

    fn calculate_ownership_confidence(
        owned: &str,
        query_indices: &[usize],
        memory_data: &MemoryData,
    ) -> f64 {
        let mut confidence: f64 = 0.6;

        let query_values: HashSet<String> = query_indices
            .iter()
            .flat_map(|&i| memory_data.fragments[i].content.values())
            .cloned()
            .collect();

        if query_values.contains(owned) {
            confidence += 0.2;
        }

        if owned.len() > 3 {
            confidence += 0.1;
        }

        confidence.min(1.0_f64)
    }

    fn calculate_value_confidence(
        atom_type: &AtomType,
        key: &str,
        value: &str,
        query_indices: &[usize],
        memory_data: &MemoryData,
        fragment_id: &Uuid,
        memory: &MemoryGraph,
    ) -> f64 {
        let mut confidence: f64 = 0.5;

        if value.len() > 3 {
            confidence += 0.1;
        }
        if value.len() > 5 {
            confidence += 0.1;
        }

        if key != "key" {
            confidence += 0.2;
        }

        let query_keys: HashSet<String> = query_indices
            .iter()
            .flat_map(|&i| memory_data.fragments[i].content.keys())
            .cloned()
            .collect();

        let query_values: HashSet<String> = query_indices
            .iter()
            .flat_map(|&i| memory_data.fragments[i].content.values())
            .cloned()
            .collect();

        if query_keys.contains(key) {
            confidence += 0.3;
        } else if query_values
            .iter()
            .any(|qv| qv.contains(key) || key.contains(qv))
        {
            confidence += 0.2;
        }

        let has_semantic_edge = memory.edges.iter().any(|((from, to), edge)| {
            (*from == *fragment_id || *to == *fragment_id)
                && matches!(edge.edge_type, EdgeType::Semantic)
                && edge.strength > 0.3
        });

        if has_semantic_edge {
            confidence += 0.1;
        }

        if matches!(atom_type, AtomType::Person) && key != "key" {
            confidence += 0.2;
        }

        confidence.min(1.0_f64)
    }

    pub fn format_candidates_for_llm(
        candidates: &[(String, f64)],
        memory_data: &MemoryData,
        memory: &MemoryGraph,
        execution_result: &ExecutionResult,
    ) -> String {
        if candidates.is_empty() {
            return String::new();
        }

        let is_preference = candidates.iter().any(|(answer, _)| {
            Self::is_preference_query(answer, memory_data, memory, execution_result)
        });

        let mut candidate_list = Vec::new();
        for (i, (value, confidence)) in candidates.iter().enumerate() {
            if is_preference {
                candidate_list.push(format!(
                    "{}. {} (confidence: {:.2})",
                    i + 1,
                    value,
                    confidence
                ));
            } else {
                candidate_list.push(format!(
                    "{}. {} (confidence: {:.2})",
                    i + 1,
                    value,
                    confidence
                ));
            }
        }

        format!("Candidates found:\n{}\n\nPlease select the most appropriate answer based on the user's query.", candidate_list.join("\n"))
    }

    pub fn format_answer(
        answer: &str,
        memory_data: &MemoryData,
        memory: &MemoryGraph,
        execution_result: &ExecutionResult,
    ) -> String {
        let format_type =
            Self::detect_format_type_from_structure(memory_data, memory, execution_result);

        match format_type {
            FormatType::Possessive { entity_key } => {
                let is_preference_query =
                    Self::is_preference_query(answer, memory_data, memory, execution_result);
                if is_preference_query {
                    format!("You like {}.", answer)
                } else {
                    format!("Your {} is {}.", entity_key, answer)
                }
            }
            FormatType::Direct => {
                let is_preference_query =
                    Self::is_preference_query(answer, memory_data, memory, execution_result);
                if is_preference_query {
                    format!("You like {}.", answer)
                } else {
                    format!("{}", answer)
                }
            }
        }
    }

    fn is_preference_query(
        answer: &str,
        memory_data: &MemoryData,
        memory: &MemoryGraph,
        execution_result: &ExecutionResult,
    ) -> bool {
        let query_lower = memory_data.query.to_lowercase();

        let query_has_preference_action = memory_data.fragments.iter().any(|frag| {
            frag.atom_type == "Action"
                || (frag.atom_type == "Entity"
                    && frag
                        .content
                        .get("key")
                        .map(|k| {
                            let k_lower = k.to_lowercase();

                            k_lower.len() >= 3
                                && k_lower.len() <= 6
                                && query_lower.contains(&k_lower)
                        })
                        .unwrap_or(false))
        });

        let answer_from_preference_fragment =
            execution_result.execution_trace.iter().any(|frag_id| {
                if let Some(frag) = memory.fragments.get(frag_id) {
                    match &frag.content {
                        FragmentContent::SemanticAtom {
                            atom_type, content, ..
                        } => {
                            (matches!(atom_type, AtomType::Object)
                                || matches!(atom_type, AtomType::Entity))
                                && content.contains_key("ownership_marker")
                        }
                        _ => false,
                    }
                } else {
                    false
                }
            });

        let memory_has_preference_pattern = memory_data.fragments.iter().any(|frag| {
            (frag.atom_type == "Object" || frag.atom_type == "Entity")
                && frag.content.contains_key("ownership_marker")
                && frag
                    .content
                    .get("key")
                    .map(|k| k == answer || k.to_lowercase() == answer.to_lowercase())
                    .unwrap_or(false)
        });

        query_has_preference_action
            && (answer_from_preference_fragment || memory_has_preference_pattern)
    }

    fn detect_format_type_from_structure(
        memory_data: &MemoryData,
        memory: &MemoryGraph,
        execution_result: &ExecutionResult,
    ) -> FormatType {
        let mut has_personal_fact = false;
        let mut has_ownership_relation = false;
        let mut has_person_atom = false;
        let mut entity_key = String::new();

        for fragment_id in &execution_result.execution_trace {
            if let Some(frag) = memory.fragments.get(fragment_id) {
                match &frag.fragment_type {
                    FragmentType::PersonalFact => {
                        has_personal_fact = true;

                        if let FragmentContent::PersonalFact { fact_type, .. } = &frag.content {
                            if !fact_type.is_empty() {
                                entity_key = fact_type.clone();
                            }
                        }
                    }
                    FragmentType::OwnershipRelation => {
                        has_ownership_relation = true;

                        if let FragmentContent::OwnershipRelation { owned, .. } = &frag.content {
                            if !owned.is_empty() && owned != "unknown" {
                                entity_key = owned.clone();
                            }
                        }
                    }
                    FragmentType::SemanticAtom => {
                        if let FragmentContent::SemanticAtom { atom_type, .. } = &frag.content {
                            if matches!(atom_type, AtomType::Person) {
                                has_person_atom = true;
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        if entity_key.is_empty() {
            for frag in &memory_data.fragments {
                for key in frag.content.keys() {
                    if key != "key" && key != "ownership_marker" && !Self::is_reference_value(key) {
                        let normalized = key.replace("_", "").replace("-", "");
                        if normalized.len() > 3 {
                            entity_key = normalized;
                            break;
                        }
                    }
                }
                if !entity_key.is_empty() {
                    break;
                }
            }
        }

        if entity_key.is_empty() {
            for fragment_id in &execution_result.execution_trace {
                if let Some(frag) = memory.fragments.get(fragment_id) {
                    if let FragmentContent::SemanticAtom { content, .. } = &frag.content {
                        for key in content.keys() {
                            if key != "key"
                                && key != "ownership_marker"
                                && !Self::is_reference_value(key)
                                && key.len() > 3
                            {
                                let normalized = key.replace("_", "").replace("-", "");
                                entity_key = normalized;
                                break;
                            }
                        }
                        if !entity_key.is_empty() {
                            break;
                        }
                    }
                }
            }
        }

        if has_personal_fact || has_ownership_relation || has_person_atom {
            FormatType::Possessive {
                entity_key: if entity_key.is_empty() {
                    "information".to_string()
                } else {
                    entity_key
                },
            }
        } else {
            FormatType::Direct
        }
    }
}

impl ResponseBuilder {
    pub fn build_response(
        execution_result: &ExecutionResult,
        memory: &MemoryGraph,
        memory_data: &MemoryData,
    ) -> String {
        if execution_result.outcome.result == "Minimal execution"
            || execution_result.outcome.result.is_empty()
        {
            return "I don't have that information yet.".to_string();
        }

        if let Some((answer, confidence)) =
            Self::extract_answer(memory_data, memory, execution_result)
        {
            if confidence > 0.5 {
                Self::format_answer(&answer, memory_data, memory, execution_result)
            } else {
                answer
            }
        } else {
            "I don't have that information yet.".to_string()
        }
    }
}
