
// Import required types for UDP networking and threading
use std::net::{UdpSocket, SocketAddr};
use std::thread;
use crossbeam::channel::{Sender, Receiver};

/// Starts the network layer for sending and receiving audio packets over UDP.
///
/// # Arguments
/// * `tx_audio` - Sender for received audio packets (from network to app)
/// * `rx_audio` - Receiver for outgoing audio packets (from app to network)
/// * `peers` - List of peer socket addresses to send packets to
pub fn start_network(
    tx_audio: Sender<Vec<u8>>,
    rx_audio: Receiver<Vec<u8>>,
    peers: Vec<SocketAddr>,
) {
    // Bind a UDP socket to all interfaces on port 5000
    let socket = UdpSocket::bind("0.0.0.0:5000").expect("bind failed");
    // Set the socket to non-blocking mode for async I/O
    socket.set_nonblocking(true).unwrap();

    // Clone the socket for sending (so send and receive can happen in parallel)
    let send_socket = socket.try_clone().unwrap();

    // Spawn a thread for sending audio packets to all peers
    thread::spawn(move || {
        while let Ok(packet) = rx_audio.recv() {
            // Send the packet to each peer in the list
            for peer in &peers {
                let _ = send_socket.send_to(&packet, peer);
            }
        }
    });

    // Spawn a thread for receiving audio packets from the network
    thread::spawn(move || {
        let mut buf = [0u8; 4096]; // Buffer for incoming packets
        loop {
            // Try to receive a packet (non-blocking)
            if let Ok((size, _)) = socket.recv_from(&mut buf) {
                // Send the received packet (up to 'size' bytes) to the application
                tx_audio.send(buf[..size].to_vec()).ok();
            }
        }
    });
}