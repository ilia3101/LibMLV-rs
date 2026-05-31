
/* Returns duration and result */
fn time<T>(name: Option<&str>, block: impl FnOnce() -> T) -> (f64, T) {
    let start = std::time::Instant::now();
    let result = block();
    let duration_ms = start.elapsed().as_micros() as f64 / 1000.0;
    name.map(|name| println!("\"{}\" took {:.1} ms", name, duration_ms));
    (duration_ms, result)
}

fn main()
{
    /* Measure time */
    let (rl_time,_) = time(Some("Rawloader"), || {
        let raw = rawloader::decode_file("/Users/ilia/Downloads/M01-1850/M01-1850_000000.dng").unwrap();
    });

    let (mlv_time,_) = time(Some("libMLV"), || {
        let mut reader = mlv::MainReader::open_mlv("/Users/ilia/Pictures/America 64GB card Backup/DCIM/100EOS5D/M01-1850.MLV", None).unwrap();
        let mut decoded = vec![0u16; (reader.width().unwrap() * reader.height().unwrap()) as usize];
        reader.decode_frame(0, &mut decoded).unwrap();
    });

    println!("MLV was {:.1} times faster than Rawloader", rl_time / mlv_time);
}
