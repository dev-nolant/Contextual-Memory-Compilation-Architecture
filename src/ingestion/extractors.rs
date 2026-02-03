// Copyright (c) 2026 Nolan Taft
use crate::ingestion::patterns::*;
use crate::types::*;

pub fn extract_personal_atoms(text: &str) -> Vec<SemanticAtom> {
    let matches = personal::extract_personal_atoms(text);
    matches
        .into_iter()
        .map(|m| SemanticAtom {
            atom_type: m.atom_type,
            content: m.content,
        })
        .collect()
}

pub fn extract_temporal_atoms(text: &str) -> Vec<SemanticAtom> {
    let matches = temporal::extract_temporal_atoms(text);
    matches
        .into_iter()
        .map(|m| SemanticAtom {
            atom_type: m.atom_type,
            content: m.content,
        })
        .collect()
}

pub fn extract_spatial_atoms(text: &str) -> Vec<SemanticAtom> {
    let matches = spatial::extract_spatial_atoms(text);
    matches
        .into_iter()
        .map(|m| SemanticAtom {
            atom_type: m.atom_type,
            content: m.content,
        })
        .collect()
}

pub fn extract_quantitative_atoms(text: &str) -> Vec<SemanticAtom> {
    let matches = quantitative::extract_quantitative_atoms(text);
    matches
        .into_iter()
        .map(|m| SemanticAtom {
            atom_type: m.atom_type,
            content: m.content,
        })
        .collect()
}

pub fn extract_causal_atoms(text: &str) -> Vec<SemanticAtom> {
    let matches = causal::extract_causal_atoms(text);
    matches
        .into_iter()
        .map(|m| SemanticAtom {
            atom_type: m.atom_type,
            content: m.content,
        })
        .collect()
}

pub fn extract_hierarchical_atoms(text: &str) -> Vec<SemanticAtom> {
    let matches = hierarchical::extract_hierarchical_atoms(text);
    matches
        .into_iter()
        .map(|m| SemanticAtom {
            atom_type: m.atom_type,
            content: m.content,
        })
        .collect()
}

pub fn extract_social_atoms(text: &str) -> Vec<SemanticAtom> {
    let matches = social::extract_social_atoms(text);
    matches
        .into_iter()
        .map(|m| SemanticAtom {
            atom_type: m.atom_type,
            content: m.content,
        })
        .collect()
}

pub fn extract_ownership_atoms(text: &str) -> Vec<SemanticAtom> {
    let matches = ownership::extract_ownership_atoms(text);
    matches
        .into_iter()
        .map(|m| SemanticAtom {
            atom_type: m.atom_type,
            content: m.content,
        })
        .collect()
}

pub fn extract_state_atoms(text: &str) -> Vec<SemanticAtom> {
    let matches = state::extract_state_atoms(text);
    matches
        .into_iter()
        .map(|m| SemanticAtom {
            atom_type: m.atom_type,
            content: m.content,
        })
        .collect()
}

pub fn extract_technical_atoms(text: &str) -> Vec<SemanticAtom> {
    let matches = technical::extract_technical_atoms(text);
    matches
        .into_iter()
        .map(|m| SemanticAtom {
            atom_type: m.atom_type,
            content: m.content,
        })
        .collect()
}

pub fn extract_all_atoms(text: &str) -> Vec<SemanticAtom> {
    let mut all_atoms = Vec::new();

    all_atoms.extend(extract_personal_atoms(text));
    all_atoms.extend(extract_temporal_atoms(text));
    all_atoms.extend(extract_spatial_atoms(text));
    all_atoms.extend(extract_quantitative_atoms(text));
    all_atoms.extend(extract_causal_atoms(text));
    all_atoms.extend(extract_hierarchical_atoms(text));
    all_atoms.extend(extract_social_atoms(text));
    all_atoms.extend(extract_ownership_atoms(text));
    all_atoms.extend(extract_state_atoms(text));
    all_atoms.extend(extract_technical_atoms(text));

    all_atoms
}

pub fn extract_relationships(atoms: &[SemanticAtom], text: &str) -> Vec<Relationship> {
    let mut relationships = Vec::new();
    let text_lower = text.to_lowercase();

    for (i, atom1) in atoms.iter().enumerate() {
        for (j, atom2) in atoms.iter().enumerate().skip(i + 1) {
            let relation_type = determine_relationship_type(atom1, atom2, &text_lower);

            if relation_type.is_some() {
                relationships.push(Relationship {
                    from_atom: i,
                    to_atom: j,
                    relation_type: relation_type.unwrap(),
                    strength: 0.7,
                });
            }
        }
    }

    relationships
}

fn determine_relationship_type(
    atom1: &SemanticAtom,
    atom2: &SemanticAtom,
    context: &str,
) -> Option<RelationType> {
    match (&atom1.atom_type, &atom2.atom_type) {
        (AtomType::Person, AtomType::Person) => {
            if context.contains("friend") || context.contains("knows") {
                Some(RelationType::Knows)
            } else if context.contains("works with") || context.contains("colleague") {
                Some(RelationType::ParticipatesIn)
            } else {
                Some(RelationType::RelatedTo)
            }
        }

        (AtomType::Person, AtomType::Resource) | (AtomType::Resource, AtomType::Person) => {
            Some(RelationType::Ownership)
        }

        (AtomType::Entity, AtomType::Location) | (AtomType::Location, AtomType::Entity) => {
            Some(RelationType::LocatedAt)
        }

        (AtomType::Event, AtomType::Time) | (AtomType::Time, AtomType::Event) => {
            Some(RelationType::OccursAt)
        }

        (AtomType::Action, AtomType::Outcome) => {
            if context.contains("causes") || context.contains("leads to") {
                Some(RelationType::Causes)
            } else {
                Some(RelationType::Causal)
            }
        }

        (AtomType::Concept, AtomType::Concept) => {
            if context.contains("part of") || context.contains("contains") {
                Some(RelationType::PartOf)
            } else {
                Some(RelationType::Hierarchical)
            }
        }

        _ => Some(RelationType::Semantic),
    }
}
