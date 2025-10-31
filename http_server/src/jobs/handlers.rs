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
    use crate::http::request::Method;
    
    let (task, priority, params_json) = match req.method() {
        Method::GET => {
            // GET: usar query parameters
            let task = match req.query_param("task") {
                Some(t) => t.to_string(),
                None => {
                    return Response::error(
                        StatusCode::BadRequest,
                        "Missing required parameter: task"
                    );
                }
            };
            
            let priority = req.query_param("prio")
                .and_then(|p| JobPriority::from_str(p))
                .unwrap_or(JobPriority::Normal);
            
            let mut params_map = std::collections::HashMap::new();
            for (key, value) in req.query_params() {
                if key != "task" && key != "prio" {
                    params_map.insert(key.clone(), value.clone());
                }
            }
            
            let params_json = serde_json::to_string(&params_map)
                .unwrap_or_else(|_| "{}".to_string());
            
            (task, priority, params_json)
        }
        Method::POST => {
            // POST: parsear JSON del body
            let body_str = match req.body_string() {
                Some(s) => s,
                None => {
                    return Response::error(
                        StatusCode::BadRequest,
                        "Invalid UTF-8 in request body"
                    );
                }
            };
            
            let json: serde_json::Value = match serde_json::from_str(&body_str) {
                Ok(v) => v,
                Err(_) => {
                    return Response::error(
                        StatusCode::BadRequest,
                        "Invalid JSON in request body"
                    );
                }
            };
            
            let task = match json.get("command").or_else(|| json.get("task")) {
                Some(serde_json::Value::String(t)) => t.clone(),
                _ => {
                    return Response::error(
                        StatusCode::BadRequest,
                        "Missing required field: command or task"
                    );
                }
            };
            
            let priority = json.get("priority").or_else(|| json.get("prio"))
                .and_then(|v| v.as_str())
                .and_then(|s| JobPriority::from_str(s))
                .unwrap_or(JobPriority::Normal);
            
            let params_json = match json.get("params") {
                Some(params) => serde_json::to_string(params)
                    .unwrap_or_else(|_| "{}".to_string()),
                None => "{}".to_string(),
            };
            
            (task, priority, params_json)
        }
        _ => {
            return Response::error(
                StatusCode::BadRequest,
                "Method not allowed. Use GET or POST"
            );
        }
    };
    
    // Resto igual...
    let job_type = match JobType::from_task_name(&task) {
        Some(jt) => jt,
        None => {
            return Response::error(
                StatusCode::BadRequest,
                &format!("Unknown task type: {}", task)
            );
        }
    };
    
    match job_manager.submit_job(job_type, params_json, priority) {
        Ok(job_id) => {
            let body = format!(
                r#"{{"job_id": "{}", "status": "queued"}}"#,
                job_id
            );
            Response::json(&body)
        }
        Err(error) => {
            if error.contains("full") {
                let mut response = Response::error(
                    StatusCode::ServiceUnavailable,
                    &error
                );
                response.add_header("Retry-After", "5");
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


        #[test]
        fn test_submit_handler_unknown_task() {
            let raw = b"GET /jobs/submit?task=unknown HTTP/1.0\r\n\r\n";
            let request = Request::parse(raw).unwrap();
    
            let config = JobManagerConfig::default();
            let manager = JobManager::new(config);
    
            let response = submit_handler(&request, &manager);
            assert_eq!(response.status(), StatusCode::BadRequest);
    
            let body = String::from_utf8_lossy(response.body());
            assert!(body.contains("Unknown task type"));
        }
    
        #[test]
        fn test_submit_handler_empty_task_value() {
            // task presente pero vacío → debe fallar como "Unknown task type"
            let raw = b"GET /jobs/submit?task=&prio=high HTTP/1.0\r\n\r\n";
            let request = Request::parse(raw).unwrap();
    
            let config = JobManagerConfig::default();
            let manager = JobManager::new(config);
    
            let response = submit_handler(&request, &manager);
            assert_eq!(response.status(), StatusCode::BadRequest);
    
            let body = String::from_utf8_lossy(response.body());
            // Dependiendo de tu implementación, el mensaje puede incluir el task vacío
            assert!(body.contains("Unknown task type"));
        }
    
        #[test]
        fn test_status_handler_empty_id_value() {
            // id presente pero vacío → el manager no lo encuentra → 404
            let raw = b"GET /jobs/status?id= HTTP/1.0\r\n\r\n";
            let request = Request::parse(raw).unwrap();
    
            let config = JobManagerConfig::default();
            let manager = JobManager::new(config);
    
            let response = status_handler(&request, &manager);
            assert_eq!(response.status(), StatusCode::NotFound);
    
            let body = String::from_utf8_lossy(response.body());
            assert!(body.contains("Job not found"));
        }
    
        #[test]
        fn test_result_handler_missing_id() {
            let raw = b"GET /jobs/result HTTP/1.0\r\n\r\n";
            let request = Request::parse(raw).unwrap();
    
            let config = JobManagerConfig::default();
            let manager = JobManager::new(config);
    
            let response = result_handler(&request, &manager);
            assert_eq!(response.status(), StatusCode::BadRequest);
    
            let body = String::from_utf8_lossy(response.body());
            assert!(body.contains("Missing required parameter: id"));
        }
    
        #[test]
        fn test_result_handler_not_found() {
            let raw = b"GET /jobs/result?id=no_such_job HTTP/1.0\r\n\r\n";
            let request = Request::parse(raw).unwrap();
    
            let config = JobManagerConfig::default();
            let manager = JobManager::new(config);
    
            let response = result_handler(&request, &manager);
            assert_eq!(response.status(), StatusCode::NotFound);
    
            let body = String::from_utf8_lossy(response.body());
            assert!(body.contains("Job not found"));
        }
    
        #[test]
        fn test_cancel_handler_missing_id() {
            let raw = b"GET /jobs/cancel HTTP/1.0\r\n\r\n";
            let request = Request::parse(raw).unwrap();
    
            let config = JobManagerConfig::default();
            let manager = JobManager::new(config);
    
            let response = cancel_handler(&request, &manager);
            assert_eq!(response.status(), StatusCode::BadRequest);
    
            let body = String::from_utf8_lossy(response.body());
            assert!(body.contains("Missing required parameter: id"));
        }
    
        #[test]
        fn test_cancel_handler_not_found() {
            let raw = b"GET /jobs/cancel?id=does_not_exist HTTP/1.0\r\n\r\n";
            let request = Request::parse(raw).unwrap();
    
            let config = JobManagerConfig::default();
            let manager = JobManager::new(config);
    
            let response = cancel_handler(&request, &manager);
            assert_eq!(response.status(), StatusCode::NotFound);
    
            let body = String::from_utf8_lossy(response.body());
            assert!(body.contains("not found"));
        }
    
        #[test]
        fn test_submit_handler_ignores_prio_when_task_missing() {
            // Aunque venga prio, si falta task debe ser 400 por parámetro requerido faltante
            let raw = b"GET /jobs/submit?prio=high HTTP/1.0\r\n\r\n";
            let request = Request::parse(raw).unwrap();
    
            let config = JobManagerConfig::default();
            let manager = JobManager::new(config);
    
            let response = submit_handler(&request, &manager);
            assert_eq!(response.status(), StatusCode::BadRequest);
    
            let body = String::from_utf8_lossy(response.body());
            assert!(body.contains("Missing required parameter: task"));
        }
    
        #[test]
        fn test_submit_handler_ignores_unrelated_params_when_task_missing() {
            // Si faltó task, da igual que vengan otros params: debe ser 400
            let raw = b"GET /jobs/submit?foo=bar&n=123 HTTP/1.0\r\n\r\n";
            let request = Request::parse(raw).unwrap();
    
            let config = JobManagerConfig::default();
            let manager = JobManager::new(config);
    
            let response = submit_handler(&request, &manager);
            assert_eq!(response.status(), StatusCode::BadRequest);
    
            let body = String::from_utf8_lossy(response.body());
            assert!(body.contains("Missing required parameter: task"));
        }
    
}