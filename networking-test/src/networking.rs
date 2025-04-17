use tokio::net::{TcpListener, TcpStream};
use tokio::io;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::task;
use tokio::time::{sleep, Duration};
use serde::{Deserialize, Serialize};
use std::{error::Error, fs::File, io::BufReader};
use std::path::Path;
use std::io::{ErrorKind};

#[derive(Serialize, Deserialize)]
pub struct IPs {
    // refer to the `data/data.json`
    #[serde(alias = "turtlebot_ips", alias = "ip_addresses")] //Accept either the turtlebot_ips or the ip_addresses in the json. Done to keep my different name. Overcomplicating things be like:
    pub ip_addresses: Vec<String>, //array of ip strings
    #[serde(alias = "my_smelly_key", alias = "secret_key", alias = "my_secret_key")]
    pub secret_key: String, // HMAC Key. Used to securely transmit and decode messages.
}

async fn connect_with_retry(addr: &str) -> Result<TcpStream, io::Error> { //Tries to connect to the IP/stream.
    let mut tries = 0;
    loop {
        match TcpStream::connect(addr).await {
            Ok(stream) => {
                println!("Connected to {}", addr);
                return Ok(stream);
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

async fn listen_on_port(port: u16) -> tokio::io::Result<()> { //Listener task.
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
        if let Err(e) = stream.write(b"Hello from server!\n").await {
            println!("Error writing to remote: {}", e);
            return;
        }
        println!("Sent message to remote");
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
        println!("{} be like",x);
    }

    // Define remote IPs and ports
    let mut current_port = 5000;
    let mut tasks = Vec::new();
    
    //SEPARATOR TO DISTINGUISH OLD CODE
    //Start outbound connections (use Vec to store the streams)
    //let mut streams = Vec::new();
    //let mut listeners = Vec::new();
    println!("Notice: First fail may happen, this is expected. Subsequent attempts should work.");

    for remote in &ips.ip_addresses {
        let remote_with_port = format!("{}:{}", remote, current_port);  //Concatenate IP with port
        let addr = remote_with_port.clone();
        /*let stream = connect_with_retry(&remote_with_port);
        let listener = listen_on_port(current_port);
        streams.push(stream);
        listeners.push(listener);*/
        
        let listen_task = tokio::spawn(async move {
            if let Err(e) = listen_on_port(current_port).await {
                eprintln!("Failed to listen on port {}: {}", current_port, e);
            }
        });
        tasks.push(listen_task);
        
        let connect_task = tokio::spawn(async move { //Try to open a stream
            match connect_with_retry(&addr).await{
                Ok(stream) => talk_to_remote(stream).await, //If succeeded, we try to talk in a loop.
                Err(e) => eprintln!("Failed to connect to: {}, error {}",addr,e), //If failed, we print this error.
            }
            //let s = stream.await;
            //talk_to_remote(s).await;
            
        });
        
        tasks.push(connect_task);
        current_port+=1;
    }
    
    //streams.push(connect_with_retry("127.0.0.1:5900"));
    //listeners.push(listen_on_port(5900));
    
    let task = tokio::spawn(async move { //Test connection to localhost (should work!)
        let addr = "127.0.0.1:5900";
        let listener = listen_on_port(5900).await;
        match connect_with_retry(addr).await{
            Ok(stream) => talk_to_remote(stream).await,
            Err(e) => eprintln!("Failed to connect to localhost somehow: {}, error {}",addr,e),
        }
    });
        
    tasks.push(task);

    // Use tokio::join! with a dynamic number of futures
    //let mut tasks = Vec::new();

    // Handle connections
    /*for stream in streams {
        let task = tokio::spawn(async {
            let s = stream.await;
            talk_to_remote(s).await;
        });
        tasks.push(task);
    }

    // Handle listeners
    for listener in listeners {
        let task = tokio::spawn(async {
            listener.await;
        });
        tasks.push(task);
    }*/

    // Await all tasks
    //futures::future::join_all(tasks).await;
    
    for t in tasks{ //Run all the tasks, listeners and streams
        let _ = t.await;
    }
   
    /*let remote1 = "127.0.0.1:5000";
    let remote2 = "127.0.0.1:5001";
    let remote3 = "127.0.0.1:5002";

    // Define local ports to listen on
    let local_port1 = 5000;
    let local_port2 = 5001;

    // Start outbound connections
    let stream1 = connect_with_retry(remote1);
    let stream2 = connect_with_retry(remote2);
    let stream3 = connect_with_retry(remote3);

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
    );*/
  
    //SEPARATOR END
}
