// Run with: cargo run --features audio-export --bin extract_audio -- <input.mlv> [output.wav]

pub fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: extract_audio <input.mlv> [output.wav]");
        std::process::exit(1);
    }
    let input = &args[1];
    let output = if args.len() > 2 { args[2].as_str() } else { "output.wav" };

    let mut reader = mlv::MainReader::open_mlv(input).expect("Failed to open MLV file");

    if let Some(sample_rate) = reader.audio_sample_rate()
        && let Some(channels) = reader.audio_channels()
        && let Some(bits_per_sample) = reader.audio_bits_per_sample()
    {
        println!("Sample rate: {sample_rate}\nChannels: {channels}\nBits per sample: {bits_per_sample}");

        let audio_data = reader.read_audio().expect("Failed to read audio data");

        let spec = hound::WavSpec {
            channels,
            sample_rate,
            bits_per_sample,
            sample_format: hound::SampleFormat::Int,
        };
        let mut writer = hound::WavWriter::create(output, spec).expect("Failed to create WAV file");

        for sample in &audio_data {
            writer.write_sample(*sample).expect("Failed to write sample");
        }
        writer.finalize().expect("Failed to finalize WAV file");

        println!("Wrote {} samples to {}", audio_data.len(), output);
    } else {
        println!("couldn't read audio metadata");
    }
}
