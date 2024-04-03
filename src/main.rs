use env_logger::Env;
use sqlx::PgPool;
use std::net::TcpListener;
use zero2prod::configuration::get_configuration;
use zero2prod::startup::run;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let configuration = get_configuration().expect("failed to get configuration");

    let connection = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("falied to create connection pool");

    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listner = TcpListener::bind(address).expect("Falied to bind the port");
    let server = run(listner, connection).expect("failed to start the application");

    env_logger::Builder::from_env(Env::default().default_filter_or("trace")).init();

    server.await
}
