use std::net::TcpListener;

use zero2prod::run;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // ポートの0番を指定すると、OSから使用可能なポート番号が指定される
    let address = TcpListener::bind("127.0.0.1:8000")?;
    run(address)?.await
}
