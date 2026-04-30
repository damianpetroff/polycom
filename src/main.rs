
// Import project modules
mod audio;
mod network;
mod codec;

// Import required crates and types
use crossbeam::channel;
use std::thread;
use std::net::SocketAddr;
use codec::OpusCodec;

/// Entry point for the Polycom audio chat application
fn main() {
    // Prompt user for a nickname (not used further in this code, but could be used for identification)
    println!("Enter your nickname:");
    let mut nickname = String::new();
    std::io::stdin().read_line(&mut nickname).unwrap();

    // Prompt user for the peer's IP address and port
    println!("Enter peer IP (example: 192.168.1.42:5000):");
    let mut peer = String::new();
    std::io::stdin().read_line(&mut peer).unwrap();

    // Parse the peer address string into a SocketAddr
    let peer: SocketAddr = peer.trim().parse().unwrap();

    // Create channels for communication between threads:
    // mic_tx/mic_rx: raw audio frames from microphone
    // encoded_tx/encoded_rx: Opus-encoded audio packets
    // network_tx/network_rx: received network packets
    // play_tx/play_rx: decoded audio frames for playback
    let (mic_tx, mic_rx) = channel::unbounded::<Vec<f32>>();
    let (encoded_tx, encoded_rx) = channel::unbounded::<Vec<u8>>();
    let (network_tx, network_rx) = channel::unbounded::<Vec<u8>>();
    let (play_tx, play_rx) = channel::unbounded::<Vec<f32>>();

    // Spawn thread to capture audio from microphone and send to mic_tx
    thread::spawn(move || {
        audio::start_input(mic_tx);
    });

    // Spawn thread to play audio received from play_rx
    thread::spawn(move || {
        audio::start_output(play_rx);
    });

    // Spawn thread to encode raw audio frames from mic_rx to Opus and send to encoded_tx
    let mut buffer = Vec::<f32>::new();
    let encoded_tx_clone = encoded_tx.clone();
    thread::spawn(move || {
        let mut codec = OpusCodec::new();

        while let Ok(frame) = mic_rx.recv() {
            buffer.extend(frame);

            while buffer.len() >= 960 {
                let chunk: Vec<f32> = buffer.drain(..960).collect();

                let encoded = codec.encode(&chunk);
                encoded_tx_clone.send(encoded).ok();
            }
        }
    });

    // Start network thread: send encoded audio to peer and receive packets from network
    network::start_network(network_tx.clone(), encoded_rx, vec![peer]);

    // Spawn thread to decode received network packets and send to play_tx for playback
    thread::spawn(move || {
        let mut codec = OpusCodec::new();

        while let Ok(packet) = network_rx.recv() {
            // Decode received Opus packet to audio frame
            let decoded = codec.decode(&packet);
            // Send decoded audio to playback channel
            play_tx.send(decoded).ok();
        }
    });

    // Main thread sleeps forever (keeps the program running)
    loop {
        thread::sleep(std::time::Duration::from_secs(1));
    }
}