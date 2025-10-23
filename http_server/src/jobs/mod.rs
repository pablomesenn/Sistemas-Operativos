//! # Sistema de Jobs
//!
//! Implementa un sistema asíncrono para ejecutar tareas largas
//! sin bloquear las conexiones HTTP.
//!
//! ## Endpoints
//!
//! - `/jobs/submit?task=TASK&params...` - Encolar job
//! - `/jobs/status?id=JOBID` - Consultar estado
//! - `/jobs/result?id=JOBID` - Obtener resultado
//! - `/jobs/cancel?id=JOBID` - Cancelar job

pub mod job;
pub mod manager;

pub use job::{Job, JobTask, JobPriority, JobStatus};
pub use manager::JobManager;