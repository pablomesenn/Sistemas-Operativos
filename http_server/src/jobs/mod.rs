//! # Sistema de Jobs
//! src/jobs/mod.rs
//!
//! Implementa el sistema de trabajos asíncronos para tareas largas.
//! 
//! ## Componentes
//! 
//! - **types**: Tipos y estructuras fundamentales
//! - **manager**: Gestor central de jobs
//! - **queue**: Cola de prioridad para jobs pendientes
//! - **storage**: Persistencia efímera de metadatos
//! - **handlers**: Endpoints HTTP para el sistema de jobs

pub mod types;
pub mod manager;
pub mod queue;
pub mod storage;
pub mod handlers;

pub use types::{JobStatus, JobPriority, JobType, JobMetadata};
pub use manager::JobManager;