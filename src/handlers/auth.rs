use actix_web::{web, HttpResponse, Responder};
use sqlx::PgPool;
use tracing::{info, error, instrument, Span};

use crate::config::Config;
use crate::models::{UserRegistration, AuthResponse, UserResponse, UserLogin, User};
use crate::utils::password;
use crate::utils::jwt;

#[instrument(skip(user_data, db_pool), fields(username = %user_data.username, email = %user_data.email))]
pub async fn register(
    user_data: web::Json<UserRegistration>,
    db_pool: web::Data<PgPool>,
) -> impl Responder {
    info!("Processing user registration request");

    let existing_user = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE email = ? OR username = ?",
    ).bind(&user_data.username)
        .bind(&user_data.email)
        .fetch_optional(db_pool.get_ref())
        .await;

    if let Ok(Some(_)) = existing_user {
        info!(
            username = %user_data.username,
            email = %user_data.email,
            "Registration failed: user already exists"
        );
        return HttpResponse::Conflict().json(serde_json::json!({
            "status": "error",
            "message": "User with that email or username already exists"
        }));
    }

    let password_hash = password::hash_password(&user_data.password);
    let role = user_data.role.as_deref().unwrap_or("user");

    info!(
        username = %user_data.username,
        email = %user_data.email,
        role = %role,
        "Creating new user"
    );

    let user = sqlx::query_as::<_, User>(
        r#"
    INSERT INTO users (username, email, password_hash, role)
    VALUES ($1, $2, $3, $4)
    RETURNING id, username, email, password_hash, role, created_at
    "#
    )
        .bind(&user_data.username)
        .bind(&user_data.email)
        .bind(&password_hash)
        .bind(&role)
        .fetch_one(db_pool.get_ref())
        .await;

    match user {
        Ok(user) => {
            info!(
                user_id = %user.id,
                username = %user.username,
                "User created successfully"
            );
            HttpResponse::Created().json(UserResponse {
                id: user.id,
                username: user.username,
                email: user.email,
                role: user.role,
            })
        }
        Err(e) => {
            error!(error = %e, "Failed to create user");
            HttpResponse::InternalServerError().json(serde_json::json!({
                "status": "error",
                "message": "Failed to create user"
            }))
        }
    }
}

#[instrument(skip(login_data, db_pool, config), fields(email = %login_data.email))]
pub async fn login(
    login_data: web::Json<UserLogin>,
    db_pool: web::Data<PgPool>,
    config: web::Data<Config>,
) -> impl Responder {
    info!("Processing login attempt");

    let user_result = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE email = $1",

    ).bind(&login_data.email)
        .fetch_optional(db_pool.get_ref())
        .await;

    match user_result {
        Ok(Some(user)) => {
            Span::current().record("user_id", &user.id);
            Span::current().record("username", &user.username);

            if password::verify_password(&user.password_hash, &login_data.password) {
                info!(
                    user_id = %user.id,
                    username = %user.username,
                    "Login successful"
                );

                match jwt::generate_token(user.id, &user.role, &config) {
                    Ok(token) => HttpResponse::Ok().json(AuthResponse {
                        token,
                        user: UserResponse {
                            id: user.id,
                            username: user.username,
                            email: user.email,
                            role: user.role,
                        },
                    }),
                    Err(e) => {
                        error!(error = %e, user_id = %user.id, "Failed to generate token");
                        HttpResponse::InternalServerError().json(serde_json::json!({
                            "status": "error",
                            "message": "Failed to generate authentication token"
                        }))
                    }
                }
            } else {
                info!(user_id = %user.id, username = %user.username, "Login failed: invalid password");
                HttpResponse::Unauthorized().json(serde_json::json!({
                    "status": "error",
                    "message": "Invalid credentials"
                }))
            }
        }
        Ok(None) => {
            info!(email = %login_data.email, "Login failed: user not found");
            HttpResponse::Unauthorized().json(serde_json::json!({
                "status": "error",
                "message": "Invalid credentials"
            }))
        }
        Err(e) => {
            error!(error = %e, "Database error during login");
            HttpResponse::InternalServerError().json(serde_json::json!({
                "status": "error",
                "message": "Authentication failed"
            }))
        }
    }
}
