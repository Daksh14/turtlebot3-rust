use tokio::net::{TcpListener, TcpStream};
use tokio::io;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::task;
use tokio::time::{sleep, Duration, timeout};
use serde::{Deserialize, Serialize};
use std::{error::Error, fs::File, io::BufReader};
use std::path::Path;
use std::io::{ErrorKind};
use std::net::SocketAddr;
use hmac::{Hmac, Mac};
use sha2::Sha256;

#[derive(Serialize, Deserialize)]
pub struct IPs {
    // refer to the `data/data.json`
    #[serde(alias = "turtlebot_ips", alias = "ip_addresses")] //Accept either the turtlebot_ips or the ip_addresses in the json. Done to keep my different name. Overcomplicating things be like:
    pub ip_addresses: Vec<String>, //array of ip strings
    #[serde(alias = "my_smelly_key", alias = "secret_key", alias = "my_secret_key")]
    pub secret_key: String, // HMAC Key. Used to securely transmit and decode messages.
}

async fn print_peer_ip(socket: &TcpStream) -> Result<SocketAddr, io::Error> { //Spun off method to retrieve remote IPs, for nice outputs.
    socket.peer_addr()
}

async fn connect_with_retry(addr: &str) -> Result<(), io::Error> { //Tries to connect to the IP/stream. It will then loop.
    let mut tries = 0;
    loop {
        match TcpStream::connect(addr).await {
            Ok(stream) => {
                println!("Connected to {}.", addr);
                tokio::spawn(talk_to_remote(stream));
                return Ok(())
            }
            Err(e) => {
                println!("Failed to connect to {}: {}. Retrying in 3s...", addr, e);
                sleep(Duration::from_secs(3)).await;
                tries += 1;
                
                if tries >= 3 {
                    println!("Attempts to connect to {} have stopped, due to multiple failures.",addr);
                    return Err(io::Error::new(ErrorKind::TimedOut, "Connection failed after 3 attempts"));
                }
            }
        }
    }
}

async fn handle_incoming(mut socket: TcpStream) { //Tries to read the message from a stream, when received.

    let peer_ip = match print_peer_ip(&socket).await { //This is to get and store the IP address of the remote connection.
        Ok(addr) => addr,
        Err(e) => {
            eprintln!("Failed to get peer IP address: {}", e);
            return;
        }
    };

    let mut buffer = [0u8; 1024];
    println!("Opening a Reader.");
    loop {
        match socket.read(&mut buffer).await {
            Ok(0) => {
                println!("Connection closed");
                return;
            }
            Ok(n) => {
                println!("Received (incoming) from {}: {}", peer_ip,String::from_utf8_lossy(&buffer[..n]));
            }
            Err(e) => {
                println!("Error reading incoming: {}", e);
                return;
            }
        }
    }
}

async fn listen_on_port(port: u16) -> Result<(),Box<dyn Error>> { //Listener task.
    let listener = TcpListener::bind(("0.0.0.0", port)).await?;
    println!("Listening on port {}", port);
    
    match listener.accept().await {
        Ok((socket, addr)) => {
            println!("Accepted connection from {}", addr);
            tokio::spawn(handle_incoming(socket));
            Ok(())
        }
        Err(e) => {
            eprintln!("Error accepting connection: {}", e);
            Err(Box::new(e))
        }
    }
}

async fn talk_to_remote(mut stream: TcpStream) { //Talker task.

    let peer_ip = match print_peer_ip(&stream).await { //Get and store the remote IP address of the remote connection.
        Ok(addr) => addr,
        Err(e) => {
            eprintln!("Failed to get peer IP address: {}", e);
            return;
        }
    };

    loop {
        if let Err(e) = stream.write(b"Hello from server!\n").await {
            println!("Error writing to remote: {}", e);
            return;
        }
        println!("Sent message to remote address {}", peer_ip);
        sleep(Duration::from_secs(5)).await; // Send every 5 seconds
    }
}

#[tokio::main]
async fn main() {

    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("./data/data.json");
    let file = File::open(path).expect("The file should be here.");

    let reader = BufReader::new(file);
    let ips: std::result::Result<IPs, serde_json::Error> =
        serde_json::from_reader(reader);
    let ips = match ips {
        Ok(ips) => ips,
        Err(_) => {
            println!("Invalid config json.");
            std::process::exit(0)
        }
    };
    
    for x in &ips.ip_addresses{ //DEBUG, this will print all pulled IPs from the config file.
        println!("{} is the address.",x);
    }

    // Define remote IPs and ports
    let mut current_port = 5000;
    let mut tasks = Vec::new();

    for remote in &ips.ip_addresses {
        let remote_with_port = format!("{}:{}", remote, &current_port);  //Concatenate IP with port
        let addr = remote_with_port.clone();
        
        let listen_task = tokio::spawn(async move {
            if let Err(e) = listen_on_port(current_port).await {
                eprintln!("Failed to listen on port {}: {}", current_port, e);
            }
        });
        
        tasks.push(listen_task);
        
        let connect_task = tokio::spawn(async move { //Try to open a stream
            if let Err(e) = connect_with_retry(&addr).await {
                eprintln!("Failed to connect to: {}, {}", addr, e);
            }
        });
        
        tasks.push(connect_task);
        current_port+=1;
    }
    
    //Debug for localhost. The below should be commented out in the official release.
    
    let listening_task = tokio::spawn(async move { //Test connection to localhost (should work!)
        let listener = listen_on_port(5900).await;
    });
        
    tasks.push(listening_task);
    
    let connecting_task = tokio::spawn(async move { //Try to open a stream
        let addr = "127.0.0.1:5900";
        if let Err(e) = connect_with_retry(addr).await {
            eprintln!("Failed to connect to: {}, {}", addr, e);
        }
    });
    
    tasks.push(connecting_task);
    
    //Note: Everything above this should be commented out in the official release.
    
    for t in tasks{ //Run all the tasks, listeners and streams
        let _ = t.await;
    }
}
