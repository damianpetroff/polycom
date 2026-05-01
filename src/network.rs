// Import required types for UDP networking and threading
use crossbeam::channel::{Receiver, Sender};
use std::net::{SocketAddr, UdpSocket};
use std::thread;

/// Starts the network layer for sending and receiving audio packets over UDP broadcast.
///
/// # Arguments
/// * `tx_audio` - Sender for received audio packets (from network to app)
/// * `rx_audio` - Receiver for outgoing audio packets (from app to network)
pub fn start_network(tx_audio: Sender<(Vec<u8>, SocketAddr)>, rx_audio: Receiver<Vec<u8>>) {
    use std::net::Ipv4Addr;
    // Bind a UDP socket to all interfaces on port 5000
    let socket = UdpSocket::bind("0.0.0.0:5000").expect("bind failed");
    // Enable broadcast
    socket.set_broadcast(true).unwrap();
    // Set the socket to non-blocking mode for async I/O
    socket.set_nonblocking(true).unwrap();

    // Clone the socket for sending (so send and receive can happen in parallel)
    let send_socket = socket.try_clone().unwrap();

    // Broadcast address for local network (255.255.255.255:5000)
    let broadcast_addr = SocketAddr::from((Ipv4Addr::BROADCAST, 5000));

    // Spawn a thread for sending audio packets to broadcast address
    thread::spawn(move || {
        while let Ok(packet) = rx_audio.recv() {
            let _ = send_socket.send_to(&packet, broadcast_addr);
        }
    });

    // Spawn a thread for receiving audio packets from the network
    thread::spawn(move || {
        let mut buf = [0u8; 4096]; // Buffer for incoming packets
        loop {
            // Try to receive a packet (non-blocking)
            if let Ok((size, src)) = socket.recv_from(&mut buf) {
                // Send the received packet (up to 'size' bytes) and sender address to the application
                tx_audio.send((buf[..size].to_vec(), src)).ok();
            }
        }
    });
}
