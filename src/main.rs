use std::sync::{Arc, Mutex};

use engine::db::{HashMapDb, KeyValueStore};

mod engine;
mod protocol;

use monoio::io::{AsyncReadRent, AsyncWriteRentExt};
use protocol::Protocol;

#[monoio::main]
async fn main() {
    let addr = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:9999".to_string());
    let listener = monoio::net::TcpListener::bind(addr).unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    let db = Arc::new(Mutex::new(HashMapDb::new()));

    loop {
        let incoming = listener.accept().await;
        let db = db.clone();

        monoio::spawn(async move {
            match incoming {
                Ok((mut stream, addr)) => {
                    println!("accepted a connection from {}", addr);
                    let buf: Vec<u8> = Vec::with_capacity(8 * 1024);
                    let (res, b) = stream.read(buf).await;
                    if res.is_ok() {
                        let content = String::from_utf8_lossy(&b[..]);
                        let request = parse_request(content.as_bytes()).unwrap();
                        if request.kind == RequestKind::RedisCLI {
                            let (res, _) = stream.write_all(format!("+OK\r\n").into_bytes()).await;
                            match res {
                                Ok(_) => (),
                                Err(e) => println!("error on stream write: {}", e),
                            }
                            return;
                        }

                        let mut db = db.lock().unwrap();
                        let response = match request.cmd {
                            protocol::Command::Get { key } => {
                                println!("GET detected {key}");

                                db.get(key.as_str()).map_or("not found", |v| v)
                            }
                            protocol::Command::Set { key, value } => {
                                println!("SET detected {key} - {value}");

                                db.set(key.as_str(), value);

                                "ok"
                            }
                            protocol::Command::Del { key } => {
                                println!("DEL detected {key}");

                                db.del(key.as_str());

                                "ok"
                            }
                            protocol::Command::COMMAND => {
                                panic!("invalid command")
                            }
                        };

                        let (res, _) = stream
                            .write_all(
                                format!("HTTP/1.1 200 OK\r\n\r\n{}", response.to_string())
                                    .into_bytes(),
                            )
                            .await;
                        match res {
                            Ok(_) => (),
                            Err(e) => println!("error on stream write: {}", e),
                        }
                    }
                }
                Err(e) => {
                    println!("accepted connection failed: {}", e);
                }
            }
        });
    }
}

#[derive(Default, PartialEq, Eq)]
enum RequestKind {
    /// Http kind is used to handle the HTTP requests using CURL.
    #[default]
    Http,

    /// RedisCLI kind is used to handle the Redis CLI requests, which is a special protocol designed for redis.
    RedisCLI,
}

struct Request {
    kind: RequestKind,
    cmd: protocol::Command,
}

fn parse_request(raw_request: &[u8]) -> Result<Request, String> {
    let http_cmd = protocol::curl::Curl::decode(raw_request);
    let redis_cli_cmd = protocol::resp::Resp::decode(raw_request);

    match (http_cmd, redis_cli_cmd) {
        (Ok(cmd), _) => Ok(Request {
            kind: RequestKind::Http,
            cmd,
        }),
        (_, Ok(cmd)) => Ok(Request {
            kind: RequestKind::RedisCLI,
            cmd,
        }),
        _ => Err("invalid request".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_parse_redis_cli() {
        let raw = "*1\r\n$7\r\nCOMMAND\r\n";
        let output = parse_request(raw.as_bytes()).unwrap();
        assert!(output.kind == RequestKind::RedisCLI);
    }

    #[test]
    fn check_parse_http() {
        let raw = r#"GET /key HTTP/1.1
Host: 127.0.0.1:9999
User-Agent: curl/7.74.0
Accept: */*
"#;

        let output = parse_request(raw.as_bytes()).unwrap();
        assert!(output.kind == RequestKind::Http);
    }
}
