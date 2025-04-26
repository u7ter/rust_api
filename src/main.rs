mod logging;
mod models;
mod db;
mod config;
mod handlers;
mod routes;
mod utils;
mod middleware;

use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use dotenv::dotenv;
use tracing::{info, error, instrument};
use tracing_actix_web::TracingLogger;

#[actix_web::main]
#[instrument]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    // Ініціалізуємо логування
    logging::init_logging();

    info!("Starting application");

    let config = config::Config::from_env();
    info!("Configuration loaded");

    // Підключення до бази даних
    let pool = match db::create_pool(&config.database_url).await {
        Ok(pool) => {
            info!("Successfully connected to database");
            pool
        },
        Err(e) => {
            error!("Failed to create database pool: {:?}", e);
            panic!("Database connection failed");
        }
    };

    // Запускаємо міграції
    if let Err(e) = db::run_migrations(&pool).await {
        error!("Failed to run migrations: {:?}", e);
        panic!("Database migrations failed");
    }
    info!("Database migrations completed successfully");

    info!(
        server_address = format!("{}:{}", config.host, config.port),
        "Starting server"
    );
    let config = config::Config::from_env();
    let config_clone = config.clone();

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .wrap(TracingLogger::default())
            .wrap(cors)
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(config_clone.clone()))
            .configure(routes::config)
    })
        .bind(format!("{}:{}", config.host, config.port))?
        .run()
        .await
}
