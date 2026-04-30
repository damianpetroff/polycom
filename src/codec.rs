
// Import Opus encoder/decoder types and configuration enums
use opus::{Encoder, Decoder, Application, Channels};

/// Struct to encapsulate Opus audio codec functionality (encoding and decoding)
pub struct OpusCodec {
    encoder: Encoder, // Opus encoder for compressing audio
    decoder: Decoder, // Opus decoder for decompressing audio
}

impl OpusCodec {
    /// Create a new OpusCodec with default settings (48kHz, mono, VoIP application)
    pub fn new() -> Self {
        // Create an Opus encoder for 48kHz mono audio, optimized for VoIP
        let encoder = Encoder::new(48000, Channels::Mono, Application::Voip).unwrap();
        // Create an Opus decoder for 48kHz mono audio
        let decoder = Decoder::new(48000, Channels::Mono).unwrap();

        Self { encoder, decoder }
    }

    /// Encode a slice of f32 audio samples into Opus-compressed bytes
    ///
    /// # Arguments
    /// * `input` - Slice of audio samples (f32) to encode
    ///
    /// # Returns
    /// * Vec<u8> - Encoded Opus packet
    pub fn encode(&mut self, input: &[f32]) -> Vec<u8> {
        // Allocate buffer for encoded data (size is more than enough for typical Opus frames)
        let mut buf = vec![0u8; 4000];
        // Encode the input samples into Opus format
        let len = self.encoder.encode_float(input, &mut buf).unwrap();
        // Return only the used portion of the buffer
        buf[..len].to_vec()
    }

    /// Decode an Opus packet into a vector of f32 audio samples
    ///
    /// # Arguments
    /// * `input` - Slice of Opus-encoded bytes to decode
    ///
    /// # Returns
    /// * Vec<f32> - Decoded audio samples
    pub fn decode(&mut self, input: &[u8]) -> Vec<f32> {
        // Allocate buffer for decoded samples (960 samples = 20ms at 48kHz mono)
        let mut buf = vec![0f32; 960];
        // Decode the Opus packet into audio samples
        let len = self.decoder.decode_float(input, &mut buf, false).unwrap();
        // Return only the used portion of the buffer
        buf[..len].to_vec()
    }
}