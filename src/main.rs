use engine::db::{HashMapDb, KeyValueStore};
use protocol::Protocol;
use std::str::FromStr;

mod config;
mod engine;
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
        .unwrap_or_else(|| "127.0.0.1:9999".to_string());
    let listener = monoio::net::TcpListener::bind(addr).unwrap();
    tracing::info!("listening on {}", listener.local_addr().unwrap());
    let db = std::rc::Rc::new(std::cell::RefCell::new(HashMapDb::new(c.engine)));

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
                        let buf: Vec<u8> = Vec::with_capacity(8 * 1024);
                        let (res, b) = stream.read(buf).await;
                        if res.is_err() {
                            tracing::error!("error on stream read: {}", res.err().unwrap());
                            break;
                        }

                        let content = String::from_utf8_lossy(&b[..]);
                        if content.is_empty() {
                            tracing::debug!("content is empty, break...");
                            break;
                        }

                        tracing::debug!(content = content.as_ref(), "received");
                        let request = parse_request(content.as_bytes()).unwrap();
                        let close_stream_after_response = request.kind == RequestKind::Http;
                        let mut db = db.borrow_mut();
                        let response =
                            execute_command(request.cmd, &mut db, std::time::Instant::now());
                        drop(db);
                        let answer: Vec<u8> = create_answer(response, request.kind);
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

fn execute_command(
    cmd: protocol::commands::Command,
    db: &mut HashMapDb,
    now: std::time::Instant,
) -> protocol::commands::CommandResponse {
    match cmd {
        protocol::commands::Command::Get { key } => protocol::commands::CommandResponse::String {
            value: db
                .get(key.as_str(), std::time::Instant::now())
                .map_or("not found", |v| v)
                .to_owned(),
        },
        protocol::commands::Command::Set { key, value, ttl } => {
            db.set(key.as_str(), value, ttl.map(|ttl| now + ttl));
            protocol::commands::CommandResponse::String {
                value: "OK".to_owned(),
            }
        }
        protocol::commands::Command::Del { key } => {
            db.del(key.as_str());
            protocol::commands::CommandResponse::String {
                value: "OK".to_owned(),
            }
        }
        protocol::commands::Command::Exists { key } => {
            let exists = db.exists(key.as_str(), now);
            protocol::commands::CommandResponse::Integer {
                value: if exists { 1 } else { 0 },
            }
        }
        protocol::commands::Command::CommandDocs => {
            protocol::commands::CommandResponse::Array { value: Vec::new() }
        }
        protocol::commands::Command::Config => protocol::commands::CommandResponse::String {
            value: "OK".to_owned(),
        },
        protocol::commands::Command::Ping => protocol::commands::CommandResponse::String {
            value: "PONG".to_owned(),
        },
        protocol::commands::Command::Incr { key } => {
            match db.get(&key, std::time::Instant::now()) {
                Some(k) => {
                    let k = k.parse::<u64>().unwrap();
                    db.set(&key, (k + 1).to_string(), None);
                }
                None => db.set(&key, 1.to_string(), None),
            }

            protocol::commands::CommandResponse::String {
                value: "OK".to_owned(),
            }
        }
        protocol::commands::Command::FlushDb => {
            db.flush();
            protocol::commands::CommandResponse::String {
                value: "OK".to_owned(),
            }
        }
    }
}

fn create_answer(response: protocol::commands::CommandResponse, kind: RequestKind) -> Vec<u8> {
    match kind {
        RequestKind::Http => protocol::curl::Curl::encode(response),
        RequestKind::RedisCLI => protocol::resp::Resp::encode(response),
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
enum RequestKind {
    /// Http kind is used to handle the HTTP requests using CURL.
    #[default]
    Http,

    /// RedisCLI kind is used to handle the Redis CLI requests, which is a special protocol designed for redis.
    RedisCLI,
}

#[derive(Debug)]
struct Request {
    kind: RequestKind,
    cmd: protocol::commands::Command,
}

fn parse_request(raw_request: &[u8]) -> Result<Request, String> {
    // TODO: This should be optimize because in this way we try to do 2 decoding instead of stopping at first, bonus,
    // http can be under feature flag in order to skip when we are stable.
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
    fn exec_get() {
        let mut db = HashMapDb::new(config::Engine::default());
        db.set("key", "value".to_string(), None);

        let cmd = protocol::commands::Command::Get {
            key: "key".to_string(),
        };
        let res = execute_command(cmd, &mut db, std::time::Instant::now());
        assert_eq!(
            res,
            protocol::commands::CommandResponse::String {
                value: "value".to_owned()
            }
        );
    }

    #[test]
    fn exec_exists() {
        let mut db = HashMapDb::new(config::Engine::default());
        db.set("key", "value".to_string(), None);

        let cmd = protocol::commands::Command::Exists {
            key: "key".to_string(),
        };
        let res = execute_command(cmd, &mut db, std::time::Instant::now());
        assert_eq!(
            res,
            protocol::commands::CommandResponse::Integer { value: 1 }
        );
    }

    #[test]
    fn exec_incr() {
        let mut db = HashMapDb::new(config::Engine::default());

        // incr with no key
        let cmd = protocol::commands::Command::Incr {
            key: "key".to_string(),
        };

        let res = execute_command(cmd, &mut db, std::time::Instant::now());
        assert_eq!(
            res,
            protocol::commands::CommandResponse::String {
                value: "OK".to_owned()
            }
        );

        let v = db
            .get("key", std::time::Instant::now())
            .unwrap()
            .parse::<u64>()
            .unwrap();
        assert_eq!(v, 1);

        // incr with key
        let cmd = protocol::commands::Command::Incr {
            key: "key".to_string(),
        };
        let res = execute_command(cmd, &mut db, std::time::Instant::now());
        assert_eq!(
            res,
            protocol::commands::CommandResponse::String {
                value: "OK".to_owned()
            }
        );

        let v = db
            .get("key", std::time::Instant::now())
            .unwrap()
            .parse::<u64>()
            .unwrap();
        assert_eq!(v, 2);
    }

    #[test]
    fn parse_redis_cli() {
        // NOTE: This is the first command redis-cli sends.
        let raw = "*2\r\n$7\r\nCOMMAND\r\n$4\r\nDOCS\r\n";
        let output = parse_request(raw.as_bytes()).unwrap();
        assert_eq!(output.kind, RequestKind::RedisCLI);
    }

    #[test]
    fn parse_http() {
        let raw = r#"GET /key HTTP/1.1
Host: 127.0.0.1:9999
User-Agent: curl/7.74.0
Accept: */*
"#;

        let output = parse_request(raw.as_bytes()).unwrap();
        assert_eq!(output.kind, RequestKind::Http);
    }
}
