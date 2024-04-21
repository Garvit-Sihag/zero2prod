use once_cell::sync::Lazy;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::net::TcpListener;
use uuid::Uuid;
use zero2prod::{
    configuration::{get_configuration, DatabaseSettings},
    email_client::EmailClient,
    telemetry::{get_subscriber, init_subscriber},
};

static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    };
});

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    let mut connection = PgConnection::connect_with(&config.without_db())
        .await
        .expect("Failed to connect to postgres");

    connection
        .execute(format!(r#"Create database "{}"; "#, config.database_name).as_str())
        .await
        .expect("Failed to create db");

    let connection_pool = PgPool::connect_with(config.with_db())
        .await
        .expect("Falied to connect to postgres");

    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("falied to do migration");

    connection_pool
}

// Launch our application in the background ~somehow~
pub async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);
    let listner = TcpListener::bind("127.0.0.1:0").expect("failed to bind a random port");
    let port = listner.local_addr().unwrap().port();

    let mut config = get_configuration().expect("falied to get config");
    config.database.database_name = Uuid::new_v4().to_string();
    let connection = configure_database(&config.database).await;

    let sender_email = config.email_client.sender().unwrap();
    let timeout = config.email_client.timeout();
    let email_client = EmailClient::new(
        config.email_client.base_url,
        sender_email,
        config.email_client.authorization_token,
        timeout,
    );

    let server = zero2prod::startup::run(listner, connection.clone(), email_client)
        .expect("Falied to launch application");
    let _ = tokio::spawn(server);

    let address = format!("http://127.0.0.1:{}", port);

    TestApp {
        address,
        db_pool: connection,
    }
}
