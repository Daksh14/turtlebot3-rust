use tokio::net::{TcpListener, TcpStream};
use tokio::io;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::task;
use tokio::time::{sleep, Duration, timeout};
use tokio::signal;
use serde::{Deserialize, Serialize};
use std::{error::Error, fs::File, io::BufReader};
use std::path::Path;
use std::io::{ErrorKind};
use std::net::SocketAddr;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::sync::Arc;
use hex;

#[derive(Serialize, Deserialize)]
pub struct IPs {
    // refer to the `data/data.json`
    #[serde(alias = "turtlebot_ips", alias = "ip_addresses")] //Accept either the turtlebot_ips or the ip_addresses in the json. Done to keep my different name. Overcomplicating things be like:
    pub ip_addresses: Vec<String>, //array of ip strings
    #[serde(alias = "my_smelly_key", alias = "secret_key", alias = "my_secret_key")]
    pub secret_key: String, //HMAC Key. Used to securely transmit and decode messages.
}

async fn print_peer_ip(socket: &TcpStream) -> Result<SocketAddr, io::Error> { //Spun off method to retrieve remote IPs, for nice outputs.
    socket.peer_addr()
}

async fn connect_with_retry(addr: &str, key: Arc<String>) -> Result<(), io::Error> { //Tries to connect to the IP/stream. It will then loop.
    let mut tries = 0;
    loop {
        match TcpStream::connect(addr).await {
            
            Ok(stream) => {
                println!("Connected to {}.", addr);
                tokio::spawn(talk_to_remote(stream, key));
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

async fn handle_incoming(mut socket: TcpStream, key: Arc<String>) { //Tries to read the message from a stream, when received.

    let peer_ip = match print_peer_ip(&socket).await { //This is to get and store the IP address of the remote connection.
        Ok(addr) => addr,
        Err(e) => {
            eprintln!("Failed to get peer IP address: {}", e);
            return;
        }
    };
    const maximum_size: usize = 1024;
    const hmac_size: usize = 64;
    let mut buffer = [0u8; maximum_size + hmac_size];
    type HmacSha256 = Hmac<Sha256>;
    println!("Opening a Reader.");
    //Notes: Add a way to block too long messages. Dropped this as a priority.
    //Implement HMAC
    loop {
        match socket.read(&mut buffer).await {
            Ok(0) => {
                println!("A connection from {} closed", peer_ip);
                return;
            }
            Ok(n) => {
                
                if n < hmac_size {
                    println!("Message too short ({} bytes) from {}", n, peer_ip);
                    continue;
                }

                let (msg, received_hmac) = buffer[..n].split_at(n - hmac_size);
                let message = String::from_utf8_lossy(msg);
                let message2 = String::from_utf8_lossy(received_hmac);
                println!("Massage1: {}", message);
                println!("Massage2: {}", message2);

                let mut mac = HmacSha256::new_from_slice(&*key.as_bytes()).expect("HMAC accepts our key");
                mac.update(msg);
                
                match hex::decode(received_hmac.as_ref()) {
                    Ok(decoded_hmac) => {
                        match mac.verify_slice(&decoded_hmac) {
                            Ok(_) => println!("Wowzers!"),
                            Err(_) => {
                                println!("Invalid HMAC from {}", peer_ip);
                                continue;
                            }
                        }
                    },
                    Err(e) => {
                        println!("Invalid hex format from {}: {}", peer_ip, e);
                        continue;
                    }
                }
                
                //let message = String::from_utf8_lossy(msg);
                println!("Received (valid) from {}: {}", peer_ip, message);
            }
            Err(e) => {
                println!("Error reading incoming: {}", e);
                return;
            }
        }
    }
}

async fn listen_on_port(port: u16, key: Arc<String>) {
    let listener = match TcpListener::bind(("0.0.0.0", port)).await {
        Ok(listener) => listener,
        Err(e) => {
            eprintln!("Failed to bind to port {}: {}", port, e);
            return;  //Exit the method early on error
        }
    };

    println!("Listening on port {}", port);

    loop {
        match listener.accept().await {
            Ok((socket, addr)) => {
                println!("Accepted connection from {}", addr);
                let key_clone = Arc::clone(&key);
                tokio::spawn(handle_incoming(socket, key_clone));
            }
            Err(e) => {
                eprintln!("Error accepting connection: {}", e);
            }
        }
    }
}

async fn talk_to_remote(mut stream: TcpStream, key: Arc<String>) { //Talker task.

    type HmacSha256 = Hmac<Sha256>;
    
    let mut mac = HmacSha256::new_from_slice(&*key.as_bytes()).expect("HMAC accepts our key");
    //mac.update(b"hello"); //Look these two up online to verify
    //let result2 = mac.finalize();
    
    let peer_ip = match print_peer_ip(&stream).await { //Get and store the remote IP address of the remote connection.
        Ok(addr) => addr,
        Err(e) => {
            eprintln!("Failed to get peer IP address: {}", e);
            return;
        }
    };

    loop {
        let test_formatting = format!("Hello from server! Your smelly key is: {}\n", key);
        let mut temp = mac.clone();
        temp.update(test_formatting.as_bytes());
        let result = temp.finalize().into_bytes();
        let hex = hex::encode(result);
        let printed_hex = hex.clone();
        
        if let Err(e) = stream.write((test_formatting + &printed_hex).as_bytes()).await { //what is this?
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
    let ips: std::result::Result<IPs, serde_json::Error> = //This will try to read the json file's ip_addresses field and put it into a copy of our struct, named ips.
        serde_json::from_reader(reader);
    let ips = match ips {
        Ok(ips) => ips,
        Err(_) => {
            println!("Invalid config json.");
            std::process::exit(0)
        }
    };
    
    let arc_key = Arc::new(ips.secret_key); //Wrap it in an Arc to make Rust happy (this makes it a longer living reference).
    
    for x in &ips.ip_addresses{ //DEBUG, this will print all pulled IPs from the config file.
        println!("{} is the address.", x);
    }

    //Define remote IPs and ports.
    let mut current_port = 5000;
    let mut tries = 0;
    let mut tasks = Vec::new();
    
    //Open and try ten ports.

    let mut successful = false;

    while tries <= 5 {
        
        let result = timeout(Duration::from_secs(2), TcpListener::bind(("0.0.0.0", current_port))).await;
        match result {
            Ok(Ok(listener)) => {
                // Successfully bound to the port, spawn listener task
                let key_clone = Arc::clone(&arc_key);
                tokio::spawn(listen_on_port(current_port, key_clone));
                successful = true;
                break;
            }
            Ok(Err(e)) => {
                // Failed to bind to the port
                eprintln!("Failed to bind to port {}: {}", current_port, e);
                tries += 1;
                sleep(Duration::from_millis(1000)).await; // Wait before retrying
            }
            Err(_) => {
                // Timeout occurred
                eprintln!("Timeout occurred while trying to bind to port {}", current_port);
                tries += 1;
                sleep(Duration::from_millis(1000)).await; // Wait before retrying
            }
        }
    }

    if !successful {
        eprintln!("Failed to bind port {} after many tries. Program is exiting early.", current_port);
        std::process::exit(1);
    }

    for remote in &ips.ip_addresses {
        let remote_with_port = format!("{}:{}", remote, &current_port);  //Concatenate IP with port
        let addr = remote_with_port.clone();
        let key_copy_listen = Arc::clone(&arc_key);
        let key_copy_write = Arc::clone(&arc_key);
        
        let connect_task = tokio::spawn(async move { //Try to open a stream
            if let Err(e) = connect_with_retry(&addr, key_copy_write).await {
                eprintln!("Failed to connect to: {}, {}", addr, e);
            }
        });
        
        tasks.push(connect_task);
    }
    
    //Debug for localhost. The below should be commented out in the official release.
    
    let key_copy_listen_test = Arc::clone(&arc_key);
    let key_copy_write_test = Arc::clone(&arc_key);
    
    let connecting_task = tokio::spawn(async move { //Try to open a stream
        let addr = "127.0.0.1:5000";
        if let Err(e) = connect_with_retry(addr, key_copy_write_test).await {
            eprintln!("Failed to connect to: {}, {}", addr, e);
        }
    });
    
    tasks.push(connecting_task);
    
    //Note: Everything above this should be commented out in the official release.
    
    for t in tasks{ //Run all the tasks, listeners and streams
        let _ = t.await;
    }
    
    println!("\n<<All connections have been attempted or initialized. We will keep listening and writing! Press Ctrl+C to terminate this program.>>\n");

    //Match the ctrl_c signal, which will kill our program.
    match signal::ctrl_c().await {
        Ok(()) => {
            println!("\nExiting program.");
        }
        Err(err) => {
            eprintln!("Failed to listen for shutdown signal: {}", err);
        }
    }
    
}
