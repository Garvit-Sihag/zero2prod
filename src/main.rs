use std::net::TcpListener;

use zero2prod::run;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let listner = TcpListener::bind("127.0.0.1:0").expect("Falied to bind the port");
    let server = run(listner).expect("failed to start the application");

    server.await
}
