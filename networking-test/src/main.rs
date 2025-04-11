use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::task;
use tokio::time::{sleep, Duration};

async fn connect_with_retry(addr: &str) -> TcpStream {
    loop {
        match TcpStream::connect(addr).await {
            Ok(stream) => {
                println!("Connected to {}", addr);
                return stream;
            }
            Err(e) => {
                println!("Failed to connect to {}: {}. Retrying in 3s...", addr, e);
                sleep(Duration::from_secs(3)).await;
            }
        }
    }
}

async fn handle_incoming(mut socket: TcpStream) {
    let mut buffer = [0u8; 1024];
    loop {
        match socket.read(&mut buffer).await {
            Ok(0) => {
                println!("Connection closed");
                return;
            }
            Ok(n) => {
                println!("Received (incoming): {}", String::from_utf8_lossy(&buffer[..n]));
            }
            Err(e) => {
                println!("Error reading incoming: {}", e);
                return;
            }
        }
    }
}

async fn listen_on_port(port: u16) -> tokio::io::Result<()> {
    let listener = TcpListener::bind(("0.0.0.0", port)).await?;
    println!("Listening on port {}", port);

    loop {
        let (socket, addr) = listener.accept().await?;
        println!("Accepted connection from {}", addr);
        task::spawn(handle_incoming(socket));
    }
}

async fn talk_to_remote(mut stream: TcpStream) {
    loop {
        if let Err(e) = stream.write_all(b"Hello from server!\n").await {
            println!("Error writing to remote: {}", e);
            return;
        }
        println!("Sent message to remote");
        sleep(Duration::from_secs(5)).await; // Send every 5 seconds
    }
}

#[tokio::main]
async fn main() {
    // Define remote IPs and ports
    let remote1 = "127.0.0.1:5000";
    let remote2 = "127.0.0.1:5001";

    // Define local ports to listen on
    let local_port1 = 5000;
    let local_port2 = 5001;

    // Start outbound connections
    let stream1 = connect_with_retry(remote1);
    let stream2 = connect_with_retry(remote2);

    // Start listeners
    let listener1 = listen_on_port(local_port1);
    let listener2 = listen_on_port(local_port2);

    // When all futures are ready, run them
    tokio::join!(
        async {
            let s = stream1.await;
            talk_to_remote(s).await;
        },
        async {
            let s = stream2.await;
            talk_to_remote(s).await;
        },
        listener1,
        listener2,
    );
}
