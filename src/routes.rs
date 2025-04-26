use actix_web::{web};
use crate::handlers::{auth, users, test};
use crate::middleware::auth::Auth;

pub fn config(cfg: &mut web::ServiceConfig) {
    // Публічні маршрути (без аутентифікації)
    cfg.service(
        web::scope("/api/v1")
            .service(
                web::resource("/auth/register")
                    .route(web::post().to(auth::register))
            )
            .service(
                web::resource("/auth/login")
                    .route(web::post().to(auth::login))
            )
            // Тестовий маршрут, що приймає JSON
            .service(
                web::resource("/test")
                    .route(web::post().to(test::test_route))
            )
    );

    // Захищені маршрути (потребують аутентифікації)
    cfg.service(
        web::scope("/api/v1")
            .wrap(Auth)
            .service(
                web::resource("/users/{id}")
                    .route(web::get().to(users::get_user))
            )
    );
}