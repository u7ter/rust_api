use actix_web::{web, HttpResponse, Responder};
use sqlx::PgPool;
use tracing::{info, error, instrument};
use crate::models::{User, UserResponse};

#[instrument(skip(db_pool))]
pub async fn get_user(
    user_id: web::Path<i32>,
    db_pool: web::Data<PgPool>,
) -> impl Responder {
    let id = user_id.into_inner();
    info!(user_id = %id, "Fetching user");

    let result = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE id = $1"
    )
        .bind(id)
        .fetch_optional(db_pool.get_ref())
        .await;

    match result {
        Ok(Some(user)) => {
            info!(
                user_id = %user.id,
                username = %user.username,
                "User found"
            );
            HttpResponse::Ok().json(UserResponse {
                id: user.id,
                username: user.username,
                email: user.email,
                role: user.role,
            })
        }
        Ok(None) => {
            info!(user_id = %id, "User not found");
            HttpResponse::NotFound().json(serde_json::json!({
                "status": "error",
                "message": "User not found"
            }))
        }
        Err(e) => {
            error!(
                error = %e,
                user_id = %id,
                "Database error when fetching user"
            );
            HttpResponse::InternalServerError().json(serde_json::json!({
                "status": "error",
                "message": "Failed to retrieve user"
            }))
        }
    }
}