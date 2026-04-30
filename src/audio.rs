
// Import necessary traits from the cpal crate for audio device and stream handling
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
// Import Sender type from crossbeam for thread-safe channel communication
use crossbeam::channel::Sender;

/// Starts capturing audio from the default input device and sends audio data through the provided channel.
///
/// # Arguments
/// * `tx` - A Sender channel to send audio samples (as Vec<f32>) to another thread.
pub fn start_input(tx: Sender<Vec<f32>>) {
    // Get the default audio host (platform-specific backend)
    let host = cpal::default_host();
    // Get the default input device (e.g., microphone)
    let device = host.default_input_device().expect("No input device");
    // Get the default input configuration (sample rate, channels, etc.)
    let config = device.default_input_config().unwrap();

    // Build the input audio stream
    let stream = device.build_input_stream(
        &config.into(), // Convert config to the required format
        move |data: &[f32], _| {
            // This closure is called whenever new audio input data is available
            // Convert the input slice to a Vec and send it through the channel
            tx.send(data.to_vec()).ok();
        },
        move |err: cpal::StreamError| {
            // This closure is called if an error occurs in the input stream
            eprintln!("input error: {:?}", err);
        },
        None, // No additional stream configuration
    ).unwrap();

    // Start the input stream
    stream.play().unwrap();
    // Park the thread to keep it alive while the stream runs
    std::thread::park(); // keep thread alive
}

/// Starts playing audio to the default output device using data received from the provided channel.
///
/// # Arguments
/// * `rx` - A Receiver channel to receive audio samples (as Vec<f32>) from another thread.
pub fn start_output(rx: crossbeam::channel::Receiver<Vec<f32>>) {
    let host = cpal::default_host();
    let device = host.default_output_device().expect("No output device");
    let config = device.default_output_config().unwrap();

    // Build the output audio stream
    let stream = device.build_output_stream(
        &config.into(), // Convert config to the required format
        move |output: &mut [f32], _| {
            // This closure is called whenever the output device needs more audio data
            // Try to receive audio data from the channel (non-blocking)
            if let Ok(data) = rx.try_recv() {
                // Copy received audio data into the output buffer
                for (o, i) in output.iter_mut().zip(data.iter()) {
                    *o = *i;
                }
            }
            // If no data is available, output buffer remains unchanged (may produce silence or repeat last buffer)
        },
        move |err: cpal::StreamError| {
            // This closure is called if an error occurs in the output stream
            eprintln!("output error: {:?}", err);
        },
        None, // No additional stream configuration
    ).unwrap();

    // Start the output stream
    stream.play().unwrap();
    // Park the thread to keep it alive while the stream runs
    std::thread::park();
}