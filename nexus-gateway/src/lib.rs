
use axum::{
    extract::{Path, Json},
    http::StatusCode,
    response::Json as ResponseJson,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod rest;
pub mod grpc;
pub mod websocket;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTopicRequest {
    pub name: String,
    pub partitions: u32,
    pub replication_factor: u32,
    pub retention_ms: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProduceMessageRequest {
    pub key: Option<String>,
    pub value: serde_json::Value,
    pub headers: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

pub fn create_router() -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/api/v1/topics", post(create_topic))
        .route("/api/v1/topics/:topic/messages", post(produce_message))
}

async fn health_check() -> ResponseJson<ApiResponse<String>> {
    ResponseJson(ApiResponse {
        success: true,
        data: Some("Nexus is healthy".to_string()),
        error: None,
    })
}

async fn create_topic(
    Json(request): Json<CreateTopicRequest>,
) -> Result<ResponseJson<ApiResponse<String>>, StatusCode> {
    // TODO: Integrate with broker to create topic
    tracing::info!("Creating topic: {:?}", request);
    
    Ok(ResponseJson(ApiResponse {
        success: true,
        data: Some(format!("Topic '{}' created successfully", request.name)),
        error: None,
    }))
}

async fn produce_message(
    Path(topic): Path<String>,
    Json(request): Json<ProduceMessageRequest>,
) -> Result<ResponseJson<ApiResponse<u64>>, StatusCode> {
    // TODO: Integrate with broker to produce message
    tracing::info!("Producing message to topic '{}': {:?}", topic, request);
    
    Ok(ResponseJson(ApiResponse {
        success: true,
        data: Some(12345), // Placeholder offset
        error: None,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use axum_test::TestServer;

    #[tokio::test]
    async fn test_health_check() {
        let app = create_router();
        let server = TestServer::new(app).unwrap();
        
        let response = server.get("/health").await;
        assert_eq!(response.status_code(), StatusCode::OK);
    }
}
