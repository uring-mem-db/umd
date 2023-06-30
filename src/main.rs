use std::env;
use std::error::Error;
use std::net::SocketAddr;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:9999".to_string());

    let listener = TcpListener::bind(&addr).await?;
    println!("umd listening on: {}", addr);

    loop {
        tokio::select! {
            res = handle_connection(&listener) => {
                match res {
                    Ok(addr) => {
                        println!("new connection from: {}", addr);
                    }
                    Err(e) => {
                        println!("accept error: {}", e);
                    }
                }
            }
            _ = wait_for_shutdown() => {
                println!();
                println!("shutting down...");
                break;
            }
        }
    }

    Ok(())
}

async fn handle_connection(listener: &TcpListener) -> Result<SocketAddr, Box<dyn Error>> {
    match listener.accept().await {
        Ok((_, addr)) => Ok(addr),
        Err(e) => Err(Box::new(e)),
    }
}

async fn wait_for_shutdown() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install CTRL+C signal handler");
}
