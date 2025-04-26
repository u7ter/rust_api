use std::path::Path;
use tracing_subscriber::{
    fmt::{self},
    prelude::*,
    EnvFilter,
    Registry,
};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use time::format_description::well_known::Rfc3339;
use time::{OffsetDateTime, UtcOffset};
use std::fmt::Result;
use tracing_subscriber::fmt::time::FormatTime;
use tracing_subscriber::fmt::format::Writer;
#[allow(dead_code)]
struct OffsetTime(UtcOffset, Rfc3339);

struct MyLocalTimer;

impl FormatTime for MyLocalTimer {
    fn format_time(&self, w: &mut Writer<'_>) -> Result {
        let now = OffsetDateTime::now_utc();
        if let Ok(offset) = UtcOffset::current_local_offset() {
            let local = now.to_offset(offset);
            let fmt_time = local.format(&Rfc3339).unwrap();
            write!(w, "{}", fmt_time)
        } else {
            write!(w, "UNKNOWN_TIME")
        }
    }
}

pub fn init_logging() {
    // Створюємо директорію logs якщо вона не існує
    let logs_dir = Path::new("logs");
    if !logs_dir.exists() {
        std::fs::create_dir_all(logs_dir).expect("Failed to create logs directory");
    }

    // Логування у файл з ротацією щодня
    let file_appender = RollingFileAppender::new(
        Rotation::DAILY,
        logs_dir,
        "application.log",
    );
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    // Встановлюємо фільтр рівня логування з ENV або за замовчуванням "info"
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    // Логування в консоль
    let stdout_layer = fmt::layer()
        .with_target(true)
        .with_timer(MyLocalTimer);

    // Логування у файл (JSON формат для легшого парсингу)
    let file_layer = fmt::layer()
        .with_target(true)
        .with_timer(MyLocalTimer)
        .json()
        .with_writer(non_blocking);

    // Ініціалізуємо підписника логування
    Registry::default()
        .with(env_filter)
        .with(stdout_layer)
        .with(file_layer)
        .init();

    tracing::info!("Logging initialized");
}