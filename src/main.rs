use std::net::TcpListener;
use zero2prod::{configuration::get_configuration, startup::run};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // configurationファイルを読めなければパニックさせる
    let configuration = get_configuration().expect("Failed to read configuration");

    let address = format!("127.0.0.1:{}", configuration.application_port);
    let address = TcpListener::bind(address)?;
    run(address)?.await
}
