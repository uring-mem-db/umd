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

                println!("content -> {}", String::from_utf8_lossy(&buf[..read]));

                let (res, _) = stream.write_all("ok".to_string().into_bytes()).await;
                match res {
                    Ok(_) => (),
                    Err(e) => println!("error on stream write: {}", e),
                }
            }
        }
    });
}
