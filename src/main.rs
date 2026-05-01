// Helper struct to manage alternate screen buffer
use std::io::Write;
struct AltScreen;
impl AltScreen {
    fn enter() {
        // Switch to alternate screen buffer
        print!("\x1B[?1049h");
        std::io::stdout().flush().ok();
    }
}
impl Drop for AltScreen {
    fn drop(&mut self) {
        // Switch back to normal screen buffer
        print!("\x1B[?1049l");
        std::io::stdout().flush().ok();
    }
}

// Import project modules
mod audio;
mod codec;
mod network;

// Import required crates and types
use codec::OpusCodec;
use crossbeam::channel;
use std::net::SocketAddr;
use std::thread;

/// Entry point for the Polycom audio chat application
fn main() {
    // Enter alternate screen buffer for fullscreen UI
    let _altscreen = AltScreen;
    AltScreen::enter();
    // Prompt user for a nickname (not used further in this code, but could be used for identification)
    println!("Enter your nickname:");
    let mut nickname = String::new();
    std::io::stdin().read_line(&mut nickname).unwrap();

    // No need to prompt for peer IP; we will broadcast to everyone on the same port

    // Create channels for communication between threads:
    // mic_tx/mic_rx: raw audio frames from microphone
    // encoded_tx/encoded_rx: Opus-encoded audio packets
    // network_tx/network_rx: received network packets
    // play_tx/play_rx: decoded audio frames for playback
    let (mic_tx, mic_rx) = channel::unbounded::<Vec<f32>>();
    let (encoded_tx, encoded_rx) = channel::unbounded::<Vec<u8>>();
    let (network_tx, network_rx) = channel::unbounded::<(Vec<u8>, SocketAddr)>();
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
    let nickname = nickname.trim().to_string();
    thread::spawn(move || {
        let mut codec = OpusCodec::new();

        while let Ok(frame) = mic_rx.recv() {
            buffer.extend(frame);

            while buffer.len() >= 960 {
                let chunk: Vec<f32> = buffer.drain(..960).collect();
                let encoded = codec.encode(&chunk);
                // Prepend nickname to packet: [len][nickname][opus]
                let nick_bytes = nickname.as_bytes();
                let mut packet = Vec::with_capacity(1 + nick_bytes.len() + encoded.len());
                packet.push(nick_bytes.len() as u8);
                packet.extend_from_slice(nick_bytes);
                packet.extend_from_slice(&encoded);
                encoded_tx_clone.send(packet).ok();
            }
        }
    });

    // Start network thread: broadcast encoded audio and receive packets from anyone
    network::start_network(network_tx.clone(), encoded_rx);

    // Spawn thread to decode received network packets and send to play_tx for playback
    thread::spawn(move || {
        let mut codec = OpusCodec::new();
        use std::collections::HashMap;
        use std::io::{self, Write};
        use std::time::{Duration, Instant};
        let mut known_nicks: HashMap<String, (SocketAddr, Instant)> = HashMap::new();
        let mut my_addr = None;

        // Helper to redraw the "GUI" of current nicknames
        fn redraw_gui(nicks: &HashMap<String, (SocketAddr, Instant)>) {
            // Clear screen and move cursor to top-left (ANSI escape codes)
            print!("\x1B[2J\x1B[H");
            // Print header and nicknames
            println!("=== Polycom Voice Chat ===");
            println!("Currently online:");
            for (nick, (addr, _)) in nicks.iter() {
                // Red color: \x1B[31m ... \x1B[0m
                println!("> \x1B[31m{}\x1B[0m ({})", nick, addr);
            }
            print!("\n(Press Ctrl+C to quit)");
            io::stdout().flush().ok();
        }

        // Redraw every time a new nick is seen or a timeout occurs
        let mut last_gui_update = Instant::now();

        loop {
            // Remove nicks not seen in the last 5 seconds
            let now = Instant::now();
            known_nicks.retain(|_, &mut (_, last_seen)| {
                now.duration_since(last_seen) < Duration::from_secs(5)
            });

            // Non-blocking receive with short timeout to allow periodic GUI refresh
            let packet_opt = match network_rx.recv_timeout(Duration::from_millis(200)) {
                Ok((packet, src_addr)) => Some((packet, src_addr)),
                Err(crossbeam::channel::RecvTimeoutError::Timeout) => None,
                Err(_) => break,
            };

            if let Some((packet, src_addr)) = packet_opt {
                // On first packet, determine our own address (to avoid echo)
                if my_addr.is_none() {
                    if let Ok(sock) = std::net::UdpSocket::bind("0.0.0.0:0") {
                        if let Ok(local_addr) = sock.local_addr() {
                            my_addr = Some(local_addr);
                        }
                    }
                }
                // Ignore our own packets (optional, may not always work with NAT)
                if let Some(my_addr) = my_addr {
                    if src_addr == my_addr {
                        continue;
                    }
                }
                // Extract nickname from packet
                if packet.is_empty() {
                    continue;
                }
                let nick_len = packet[0] as usize;
                if packet.len() < 1 + nick_len {
                    continue;
                }
                let nick = match std::str::from_utf8(&packet[1..1 + nick_len]) {
                    Ok(n) => n,
                    Err(_) => continue,
                };
                // Update last seen for this nick
                let is_new = !known_nicks.contains_key(nick);
                known_nicks.insert(nick.to_string(), (src_addr, now));
                // Redraw GUI if new nick
                if is_new {
                    redraw_gui(&known_nicks);
                }
                // Decode and play audio
                let opus_data = &packet[1 + nick_len..];
                let decoded = codec.decode(opus_data);
                play_tx.send(decoded).ok();
            }

            // Periodically redraw GUI to remove timed-out nicks
            if last_gui_update.elapsed() > Duration::from_secs(1) {
                redraw_gui(&known_nicks);
                last_gui_update = Instant::now();
            }
        }
    });

    // Main thread sleeps forever (keeps the program running)
    loop {
        thread::sleep(std::time::Duration::from_secs(1));
    }
}
