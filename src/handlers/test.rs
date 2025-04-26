use actix_web::{web, HttpResponse, Responder};
use tracing::{info, instrument};
use crate::models::TestPayload;

#[instrument(skip(payload))]
pub async fn test_route(
    payload: web::Json<TestPayload>,
) -> impl Responder {
    info!(
        message = %payload.message,
        has_data = payload.data.is_some(),
        "Processing test route"
    );

    HttpResponse::Ok().json(serde_json::json!({
        "status": "success",
        "message": format!("Received message: {}", payload.message),
        "data": payload.data,
        "received_at": chrono::Utc::now().to_rfc3339()
    }))
}