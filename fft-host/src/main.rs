use tokio::io::{self, AsyncReadExt};
use tokio::net::TcpListener;
use tokio::time::Instant;

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:8000").await?;

    println!("Server listening on port 8000");

    loop {
        let (mut socket, addr) = listener.accept().await?;
        println!("Accepted connection from {}", addr);

        // This task handles the client connection.
        tokio::spawn(async move {
            let mut buffer = [0; 1460];
            let start_time = Instant::now();
            let mut total_bytes_read = 0usize;
            let mut valid_data = true;

            loop {
                match socket.read(&mut buffer).await {
                    Ok(0) => {
                        // Connection was closed
                        println!("Connection closed");
                        break;
                    }
                    Ok(n) => {
                        total_bytes_read += n;

                        for chunk in buffer[..n].chunks(4) {
                            if chunk.len() == 4 {
                                let value = u32::from_le_bytes(chunk.try_into().unwrap());
                                if value != 0xDEADBEEF {
                                    println!("{:#08x}", value);
                                    valid_data = false;
                                    println!("Invalid data detected");
                                    break;
                                }
                            }
                        }
                        if !valid_data {
                            break;
                        }
                    }
                    Err(e) => {
                        println!("Failed to read from socket; err = {:?}", e);
                        break;
                    }
                }
            }

            let duration = start_time.elapsed();
            println!("Total MB received: {}", total_bytes_read / 1_000_000);
            println!("Duration: {:.2?}", duration);
            let speed_mbps = (total_bytes_read / 1_000_000) as f64 / duration.as_secs_f64(); // Converts bytes to megabytes and calculates per second
            println!("Speed: {:.2} MBps", speed_mbps);
            if valid_data {
                println!("All received data is valid.");
            } else {
                println!("Received invalid data.");
            }
        });
    }
}
