fn main() {
    let addr = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:9999".to_string());

    let socket_addr: std::net::SocketAddr = addr.parse().unwrap();

    tokio_uring::start(async {
        let listener = tokio_uring::net::TcpListener::bind(socket_addr).unwrap();

        println!("listening on {}", listener.local_addr().unwrap());

        loop {
            let (stream, socket_addr) = listener.accept().await.unwrap();
            println!("{socket_addr:?} connected");
            let mut buf = vec![0u8; 4096];

            loop {
                let (result, nbuf) = stream.read(buf).await;
                buf = nbuf;
                let read = result.unwrap();
                println!("read -> {}", read);
                if read == 0 {
                    println!("{socket_addr} closed");
                    break;
                }

                let content = String::from_utf8_lossy(&buf[..read]);
                let request = parse_request(content.to_string()).unwrap();
                match request.method.as_str() {
                    "GET" => {
                        println!("GET");
                    }
                    _ => {
                        unimplemented!("not GET");
                    }
                }

                let (res, _) = stream.write_all("ok".to_string().into_bytes()).await;
                match res {
                    Ok(_) => (),
                    Err(e) => println!("error on stream write: {}", e),
                }
            }
        }
    });
}

#[derive(Default)]
struct Request {
    method: String,
    path: String,
    version: String,
    headers: std::collections::HashMap<String, String>,
}

fn parse_request(request: String) -> Result<Request, String> {
    let request = request.trim();
    println!("{request}");
    let mut lines = request.lines();
    let first_line = lines.next().unwrap();
    let mut parts = first_line.split_whitespace();
    let method = parts.next().unwrap();
    let path = parts.next().unwrap();
    let version = parts.next().unwrap();
    let mut headers = std::collections::HashMap::new();
    for line in lines {
        let (key, value) = line.split_once(':').unwrap();
        headers.insert(key.to_string(), value.to_string());
    }

    Ok(Request {
        method: method.to_string(),
        path: path.to_string(),
        version: version.to_string(),
        headers,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_parse_get() {
        let raw = r#"GET / HTTP/1.1
Host: 127.0.0.1:9999
User-Agent: curl/7.74.0
Accept: */*
"#;

        let output = parse_request(raw.to_string()).unwrap();

        assert_eq!(output.method, "GET");
        assert_eq!(output.path, "/");
        assert_eq!(output.version, "HTTP/1.1");
        assert_eq!(output.headers.len(), 3);
        assert_eq!(output.headers.get("Host").unwrap(), " 127.0.0.1:9999");
        assert_eq!(output.headers.get("User-Agent").unwrap(), " curl/7.74.0");
        assert_eq!(output.headers.get("Accept").unwrap(), " */*");
    }
}
