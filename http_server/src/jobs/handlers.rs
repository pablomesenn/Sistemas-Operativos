//! # Handlers HTTP para Jobs
//! src/jobs/handlers.rs
//!
//! Implementa los endpoints del sistema de jobs:
//! - /jobs/submit
//! - /jobs/status
//! - /jobs/result
//! - /jobs/cancel

use crate::http::{Request, Response, StatusCode};
use crate::jobs::manager::JobManager;
use crate::jobs::types::{JobType, JobPriority};

/// Handler para /jobs/submit?task=TASK&<params>&prio=low|normal|high
/// 
/// Encola un nuevo job y retorna su ID.
/// 
/// # Query parameters
/// - `task`: Tipo de tarea (isprime, factor, etc.) (requerido)
/// - `prio`: Prioridad (low, normal, high) (opcional, default: normal)
/// - Resto de parámetros: dependen del task
/// 
/// # Ejemplo de response
/// ```json
/// {"job_id": "job-abc123", "status": "queued"}
/// ```
pub fn submit_handler(req: &Request, job_manager: &JobManager) -> Response {
    // Obtener el task
    let task = match req.query_param("task") {
        Some(t) => t,
        None => {
            return Response::error(
                StatusCode::BadRequest,
                "Missing required parameter: task"
            );
        }
    };
    
    // Parsear JobType
    let job_type = match JobType::from_task_name(task) {
        Some(jt) => jt,
        None => {
            return Response::error(
                StatusCode::BadRequest,
                &format!("Unknown task type: {}", task)
            );
        }
    };
    
    // Obtener prioridad (opcional)
    let priority = req.query_param("prio")
        .and_then(|p| JobPriority::from_str(p))
        .unwrap_or(JobPriority::Normal);
    
    // Construir JSON con los parámetros (excepto task y prio)
    let mut params_map = std::collections::HashMap::new();
    for (key, value) in req.query_params() {
        if key != "task" && key != "prio" {
            params_map.insert(key.clone(), value.clone());
        }
    }
    
    let params_json = serde_json::to_string(&params_map)
        .unwrap_or_else(|_| "{}".to_string());
    
    // Encolar el job
    match job_manager.submit_job(job_type, params_json, priority) {
        Ok(job_id) => {
            let body = format!(
                r#"{{"job_id": "{}", "status": "queued"}}"#,
                job_id
            );
            Response::json(&body)
        }
        Err(error) => {
            // Si la cola está llena, retornar 503
            if error.contains("full") {
                let mut response = Response::error(
                    StatusCode::ServiceUnavailable,
                    &error
                );
                response.add_header("Retry-After", "5"); // Reintentar en 5 segundos
                response
            } else {
                Response::error(StatusCode::InternalServerError, &error)
            }
        }
    }
}

/// Handler para /jobs/status?id=JOBID
/// 
/// Obtiene el estado actual de un job.
/// 
/// # Query parameters
/// - `id`: ID del job (requerido)
/// 
/// # Ejemplo de response
/// ```json
/// {
///   "status": "running",
///   "progress": 42,
///   "eta_ms": 3800
/// }
/// ```
pub fn status_handler(req: &Request, job_manager: &JobManager) -> Response {
    let job_id = match req.query_param("id") {
        Some(id) => id,
        None => {
            return Response::error(
                StatusCode::BadRequest,
                "Missing required parameter: id"
            );
        }
    };
    
    match job_manager.get_job_status(job_id) {
        Some(metadata) => {
            // Construir response JSON
            let progress_field = if metadata.progress > 0 {
                format!(r#","progress":{}"#, metadata.progress)
            } else {
                String::new()
            };
            
            let eta_field = if let Some(eta) = metadata.eta_ms {
                format!(r#","eta_ms":{}"#, eta)
            } else {
                String::new()
            };
            
            let body = format!(
                r#"{{"status":"{}"{}{}}}"#,
                serde_json::to_string(&metadata.status).unwrap().trim_matches('"'),
                progress_field,
                eta_field
            );
            
            Response::json(&body)
        }
        None => {
            Response::error(
                StatusCode::NotFound,
                &format!("Job not found: {}", job_id)
            )
        }
    }
}

/// Handler para /jobs/result?id=JOBID
/// 
/// Obtiene el resultado de un job completado.
/// 
/// # Query parameters
/// - `id`: ID del job (requerido)
/// 
/// # Ejemplo de response
/// Si el job está done:
/// ```json
/// {"n": 97, "is_prime": true, ...}
/// ```
/// 
/// Si el job falló:
/// ```json
/// {"error": "Job failed: timeout"}
/// ```
pub fn result_handler(req: &Request, job_manager: &JobManager) -> Response {
    let job_id = match req.query_param("id") {
        Some(id) => id,
        None => {
            return Response::error(
                StatusCode::BadRequest,
                "Missing required parameter: id"
            );
        }
    };
    
    match job_manager.get_job_status(job_id) {
        Some(metadata) => {
            // Verificar estado
            match metadata.status {
                crate::jobs::types::JobStatus::Done => {
                    // Retornar el resultado
                    if let Some(result) = metadata.result {
                        Response::new(StatusCode::Ok)
                            .with_header("Content-Type", "application/json")
                            .with_body(&result)
                    } else {
                        Response::error(
                            StatusCode::InternalServerError,
                            "Job marked as done but no result available"
                        )
                    }
                }
                crate::jobs::types::JobStatus::Error | crate::jobs::types::JobStatus::Timeout => {
                    // Retornar el error
                    let error_msg = metadata.error.unwrap_or_else(|| "Unknown error".to_string());
                    Response::error(StatusCode::InternalServerError, &error_msg)
                }
                crate::jobs::types::JobStatus::Canceled => {
                    Response::error(StatusCode::Conflict, "Job was canceled")
                }
                _ => {
                    // Job aún no está listo
                    Response::error(
                        StatusCode::Conflict,
                        &format!("Job not ready yet (status: {:?})", metadata.status)
                    )
                }
            }
        }
        None => {
            Response::error(
                StatusCode::NotFound,
                &format!("Job not found: {}", job_id)
            )
        }
    }
}

/// Handler para /jobs/cancel?id=JOBID
/// 
/// Intenta cancelar un job.
/// 
/// # Query parameters
/// - `id`: ID del job (requerido)
/// 
/// # Ejemplo de response
/// ```json
/// {"status": "canceled"}
/// ```
pub fn cancel_handler(req: &Request, job_manager: &JobManager) -> Response {
    let job_id = match req.query_param("id") {
        Some(id) => id,
        None => {
            return Response::error(
                StatusCode::BadRequest,
                "Missing required parameter: id"
            );
        }
    };
    
    match job_manager.cancel_job(job_id) {
        Ok(()) => {
            let body = r#"{"status": "canceled"}"#;
            Response::json(body)
        }
        Err(error) => {
            if error.contains("not found") {
                Response::error(StatusCode::NotFound, &error)
            } else if error.contains("cannot be canceled") || error.contains("already finished") {
                Response::error(StatusCode::Conflict, &error)
            } else {
                Response::error(StatusCode::InternalServerError, &error)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jobs::manager::{JobManager, JobManagerConfig};
    
    #[test]
    fn test_submit_handler_missing_task() {
        let raw = b"GET /jobs/submit HTTP/1.0\r\n\r\n";
        let request = Request::parse(raw).unwrap();
        
        let config = JobManagerConfig::default();
        let manager = JobManager::new(config);
        
        let response = submit_handler(&request, &manager);
        assert_eq!(response.status(), StatusCode::BadRequest);
    }
    
    #[test]
    fn test_status_handler_missing_id() {
        let raw = b"GET /jobs/status HTTP/1.0\r\n\r\n";
        let request = Request::parse(raw).unwrap();
        
        let config = JobManagerConfig::default();
        let manager = JobManager::new(config);
        
        let response = status_handler(&request, &manager);
        assert_eq!(response.status(), StatusCode::BadRequest);
    }
    
    #[test]
    fn test_status_handler_not_found() {
        let raw = b"GET /jobs/status?id=nonexistent HTTP/1.0\r\n\r\n";
        let request = Request::parse(raw).unwrap();
        
        let config = JobManagerConfig::default();
        let manager = JobManager::new(config);
        
        let response = status_handler(&request, &manager);
        assert_eq!(response.status(), StatusCode::NotFound);
    }
}