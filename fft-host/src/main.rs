use std::time::Instant;

use futures_lite::future::block_on;
use indicatif::ProgressBar;
use nusb::transfer::RequestBuffer;

fn main() {
    let di = nusb::list_devices()
        .unwrap()
        .find(|d| d.vendor_id() == 0xb16b && d.product_id() == 0x00ba)
        .expect("device should be connected");

    println!("Device info: {:?}", di);

    let device = di.open().expect("error opening device");
    let interface = device.claim_interface(0).expect("error claiming interface");

    let bulk_endpoint_address = 0x81;
    let mut total_data_received = 0usize;
    let data_to_receive = 1024 * 1024;
    let mut data_received = Vec::new();

    let start_time = Instant::now();

    let pb = ProgressBar::new(data_to_receive as u64);

    let mut queue = interface.bulk_in_queue(bulk_endpoint_address);

    while total_data_received < data_to_receive {
        while queue.pending() < 8 {
            queue.submit(RequestBuffer::new(64));
        }

        let result = block_on(queue.next_complete());
        match result.status {
            Ok(()) => {
                total_data_received += result.data.len();
                data_received.extend_from_slice(&result.data);
                pb.set_position(total_data_received as u64);
            }
            _ => break,
        }
    }

    let duration = start_time.elapsed();
    let speed_mbps = total_data_received as f64 / duration.as_secs_f64() / (1024.0 * 1024.0);
    println!("Total data received: {} bytes", total_data_received);
    println!("Speed: {:.2} MB/s", speed_mbps);

    // Validate the received data
    let valid_data = validate_received_data(&data_received);
    if valid_data {
        println!("Data validation: SUCCESS");
    } else {
        println!("Data validation: FAILURE");
    }
}

fn validate_received_data(data: &[u8]) -> bool {
    if data.len() % 4 != 0 {
        return false;
    }

    for chunk in data.chunks(4) {
        let value = u32::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
        if value != 0xDEADBEEF {
            return false;
        }
    }

    true
}
