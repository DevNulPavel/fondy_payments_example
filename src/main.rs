mod error;
mod http;
mod database;
mod application;


use std::{
    sync::{
        Arc
    }
};
use tracing_subscriber::{
    prelude::{
        *
    }
};
use url::{
    Url
};
use crate::{
    http::{
        start_server
    },
    database::{
        Database
    },
    application::{
        Application,
        AppConfig
    },
    error::{
        FondyError
    }
};

////////////////////////////////////////////////////////////////////////////////////////////////////////////////

fn initialize_logs() {
    // Логи в stdout
    let stdoud_sub = tracing_subscriber::fmt::layer()
        // .pretty()
        // .json()
        // .with_span_events(FmtSpan::NONE)
        // .compact()
        // .with_target(false)
        .with_writer(std::io::stdout);

    // Суммарный обработчик
    let full_subscriber = tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env()
                .and_then(stdoud_sub));

    // Установка по-умолчанию
    tracing::subscriber::set_global_default(full_subscriber)
        .unwrap();
}

#[tokio::main]
async fn main() -> Result<(), FondyError> {
    // Настраиваем удобное чтение паники
    human_panic::setup_panic!();

    // Подтягиваем окружение из файлика .env
    dotenv::dotenv().ok();

    // Инициализируем менеджер логирования
    initialize_logs();

    // База данных
    let db = Arc::new(Database::open_database()
        .await);

    // Шаблоны HTML
    let mut templates = handlebars::Handlebars::new();
    {
        templates.register_template_file("index", "templates/index.hbs")
            .expect("Index template read failed");
    }

    // Адрес нашего сайта
    let site_url = Url::parse(std::env::var("SITE_URL")
                                .expect("SITE_URL variable is missing")
                                .as_str())
        .expect("SITE_URL is invalid url");

    // Идентификаторы продавца
    let merchant_id = std::env::var("MERCHANT_ID")
        .expect("MERCHANT_ID env variable is missing")
        .parse::<u64>()
        .expect("MERCHANT_ID must be u64");
    let merchant_password = std::env::var("MERCHANT_PASSWORD")
        .expect("MERCHANT_PASSWORD env variable is missing");

    // Приложение со всеми нужными нам менеджерами
    let app = Arc::new(Application{
        db,
        templates: Arc::new(templates),
        http_client: reqwest::Client::new(),
        config: Arc::new(AppConfig{
            site_url,
            merchant_id,
            merchant_password
        })
    });

    // Стартуем сервер
    start_server(app)
        .await;
    
    Ok(())
}