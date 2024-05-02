use protocol::Protocol;
use std::str::FromStr;

mod config;
mod engine;
mod executor;
mod parser;
mod protocol;

use monoio::io::{AsyncReadRent, AsyncWriteRentExt};

#[monoio::main]
async fn main() {
    let config_file = std::fs::read_to_string("configs/local.toml");
    if config_file.is_err() {
        tracing::error!(
            "error on reading config file: {}",
            config_file.err().unwrap()
        );
        return;
    }
    let c = config::Config::new(config_file.unwrap().as_str());

    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::from_str(&c.logger.level).unwrap())
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let addr = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:6379".to_string());
    let listener = monoio::net::TcpListener::bind(addr).unwrap();
    tracing::info!("listening on {}", listener.local_addr().unwrap());

    let db = engine::db::create_db(&c.engine).unwrap();

    let number_of_connections = std::rc::Rc::new(std::cell::RefCell::new(0));

    loop {
        let incoming = listener.accept().await;
        let db = std::rc::Rc::clone(&db);
        let number_of_connections = std::rc::Rc::clone(&number_of_connections);

        monoio::spawn(async move {
            match incoming {
                Ok((mut stream, addr)) => {
                    {
                        let mut n = number_of_connections.borrow_mut();
                        *n += 1;
                        tracing::info!(
                            "accepted a connection from {} (total concurrent {})",
                            addr,
                            *n
                        );
                    }

                    loop {
                        let buf = vec![0; 4096];

                        let (res, buf) = stream.read(buf).await;
                        if res.is_err() {
                            tracing::error!("error on stream read: {}", res.err().unwrap());
                            break;
                        }

                        let content = String::from_utf8_lossy(&buf[..]);
                        if content.is_empty() {
                            tracing::debug!("content is empty, break...");
                            break;
                        }

                        tracing::debug!(content = content.as_ref(), "received");
                        let request = match parser::parse_request(content.as_bytes()) {
                            Ok(r) => {
                                tracing::debug!(?r, "parsed request");
                                r
                            }
                            Err(e) => {
                                tracing::error!("error on parsing request: {}", e);
                                let (r, _) = stream.write_all(b"-ERR unknown command\r\n").await;
                                match r {
                                    Ok(_) => (),
                                    Err(e) => tracing::error!("error on stream write: {}", e),
                                }
                                break;
                            }
                        };

                        let close_stream_after_response = request.kind == parser::RequestKind::Http;
                        let response = {
                            let mut db = db.borrow_mut();
                            let n = std::time::Instant::now();
                            let r = executor::execute_command(request.cmd, &mut db, n);
                            drop(db);
                            r
                        };

                        let answer = match request.kind {
                            parser::RequestKind::Http => protocol::curl::Curl::encode(response),
                            parser::RequestKind::RedisCLI => protocol::resp::Resp::encode(response),
                        };
                        let (res, _) = stream.write_all(answer).await;
                        match res {
                            Ok(_) => (),
                            Err(e) => tracing::error!("error on stream write: {}", e),
                        }

                        if close_stream_after_response {
                            tracing::info!("request close stream");
                            break;
                        }
                    }

                    tracing::info!("close stream connection");
                }
                Err(e) => {
                    tracing::error!("accepted connection failed: {}", e);
                }
            }

            {
                let mut n = number_of_connections.borrow_mut();
                *n -= 1;
            }
        });
    }
}
