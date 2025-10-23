//! # Sistema de Métricas
//! src/metrics/mod.rs
//!
//! Este módulo implementa la recolección y agregación de métricas del servidor:
//! - Contadores de requests
//! - Latencias (p50, p95, p99)
//! - Workers activos/ocupados
//! - Tamaño de colas

pub mod collector;

pub use collector::MetricsCollector;