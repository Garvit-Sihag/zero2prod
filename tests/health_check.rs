use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::net::TcpListener;
use uuid::Uuid;

use zero2prod::configuration::{get_configuration, DatabaseSettings};

struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    let mut connection = PgConnection::connect(&config.connection_string_without_db())
        .await
        .expect("Failed to connect to postgres");

    connection
        .execute(format!(r#"Create database "{}"; "#, config.database_name).as_str())
        .await
        .expect("Failed to create db");

    let connection_pool = PgPool::connect(&config.connection_string())
        .await
        .expect("Falied to connect to postgres");

    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("falied to do migration");

    connection_pool
}

// Launch our application in the background ~somehow~
async fn spawn_app() -> TestApp {
    let listner = TcpListener::bind("127.0.0.1:0").expect("failed to bind a random port");
    let port = listner.local_addr().unwrap().port();

    let mut config = get_configuration().expect("falied to get config");
    config.database.database_name = Uuid::new_v4().to_string();
    let connection = configure_database(&config.database).await;

    let server =
        zero2prod::startup::run(listner, connection.clone()).expect("Falied to launch application");
    let _ = tokio::spawn(server);

    let address = format!("http://127.0.0.1:{}", port);

    TestApp {
        address,
        db_pool: connection,
    }
}

#[tokio::test]
async fn health_check_works() {
    let app = spawn_app().await;

    let client = reqwest::Client::new();

    let response = client
        .get(&format!("{}/health_check", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscribe_returns_200_for_valid_form_data() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = client
        .post(&format!("{}/subscriptions", &app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("Select name, email from subscriptions",)
        .fetch_one(&app.db_pool)
        .await
        .expect("failed to execute the query");

    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
}

#[tokio::test]
async fn subscribe_returns_400_for_missing_form_data() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let test_case = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (body, error_msg) in test_case {
        let response = client
            .post(&format!("{}/subscriptions", &app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request");

        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_msg
        );
    }
}

// DROP DATABASE <database name>
