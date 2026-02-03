// Copyright (c) 2026 Nolan Taft
use crate::types::MemoryGraph;
use std::fs;
use std::path::Path;
use rmp_serde::{to_vec, from_slice};

pub type Result<T> = std::result::Result<T, StorageError>;

#[derive(Debug)]
pub enum StorageError {
    IoError(std::io::Error),
    SerializationError(rmp_serde::encode::Error),
    DeserializationError(rmp_serde::decode::Error),
}

impl std::fmt::Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StorageError::IoError(e) => write!(f, "IO error: {}", e),
            StorageError::SerializationError(e) => write!(f, "Serialization error: {}", e),
            StorageError::DeserializationError(e) => write!(f, "Deserialization error: {}", e),
        }
    }
}

impl std::error::Error for StorageError {}

impl From<std::io::Error> for StorageError {
    fn from(err: std::io::Error) -> Self {
        StorageError::IoError(err)
    }
}

impl From<rmp_serde::encode::Error> for StorageError {
    fn from(err: rmp_serde::encode::Error) -> Self {
        StorageError::SerializationError(err)
    }
}

impl From<rmp_serde::decode::Error> for StorageError {
    fn from(err: rmp_serde::decode::Error) -> Self {
        StorageError::DeserializationError(err)
    }
}


pub fn save_memory(memory: &MemoryGraph, path: &Path) -> Result<()> {
    
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    
    
    let data = to_vec(memory)?;
    
    
    fs::write(path, data)?;
    
    Ok(())
}


pub fn load_memory(path: &Path) -> Result<MemoryGraph> {
    
    if !path.exists() {
        return Err(StorageError::IoError(
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Memory file not found: {}", path.display())
            )
        ));
    }
    
    
    let data = fs::read(path)?;
    
    
    let memory: MemoryGraph = from_slice(&data)?;
    
    Ok(memory)
}
