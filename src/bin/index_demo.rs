macro_rules! time {
    ($name:expr, $block:block) => {{
        let start = std::time::Instant::now();
        let result = {$block};
        let duration_ms = start.elapsed().as_micros() as f64 / 1000.0;
        println!("Operation \"{}\" took {:.1} ms", $name, duration_ms);
        result
    }};
}

pub fn main()
{
    /* Reader test */
    let file_name = &std::env::args().collect::<Vec<_>>()[1];
    println!("{file_name}");

    let start = std::time::SystemTime::now();
    let mut reader = mlv::MainReader::open_mlv(file_name, None).unwrap();
    let end = std::time::SystemTime::now();

    println!("reader: {:#?}", reader);
    reader.print_blocks();

    println!("File indexed and loaded in {:.1?} ms", (end.duration_since(start).unwrap().as_micros()) as f64 / 1000.0);


    for (name, block) in mlv::blocks::MLV_BLOCKS.iter() {
        println!("Block {}: {:#?} bytes", name, block.size());
    }

    let wl = reader.white_level().unwrap() as f32;
    let bl = reader.black_level().unwrap() as f32;
    let width = reader.width().unwrap() as u32;
    let height = reader.height().unwrap() as u32;
    let exposure = 5.0;

    let mut decoded_buf = vec![0u16; (width * height) as usize];

    let num_decodes = 5;

    let start = std::time::Instant::now();
    let mut decoded = reader.decode_frame(0, &mut decoded_buf).unwrap();
    for i in 1..num_decodes {
        decoded = reader.decode_frame(i % reader.num_frames(), &mut decoded_buf).unwrap();
    }
    let duration_ms = start.elapsed().as_micros() as f64 / 1000.0;

    println!("Frame decoding took {:.1} ms", duration_ms / num_decodes as f64);
    let megapixels_per_second: f64 = (width as f64 / 1000. * height as f64 / 1000.) * (num_decodes as f64) * (1000. / duration_ms);
    println!("Decoded {:.1} MPixels in {:.1} ms ({:.1} MPixels/s)", (width as f64 * height as f64 * num_decodes as f64) / 1_000_000., duration_ms, megapixels_per_second);
    // let decoded = reader.decode_frame(0, &mut decoded_buf).unwrap();

    let data = time!("Processing",{ decoded.iter().copied().flat_map(|a| {
        std::iter::repeat((((a as f32 - bl) / (wl-bl) * exposure).sqrt() * 255.0) as u8).take(3)
    }).collect::<Vec<_>>() });

    println!("Width: {}, Height: {}", width, height);

    /* time! {{  */save_bmp(width, height, &data, &mut std::fs::File::create("test.bmp").unwrap()) /* }}; */
}

fn save_bmp(width: u32, height: u32, data: &[u8], file: &mut impl std::io::Write) {
    let header: [u8; 26] = [0x42,0x4D,0,0,0,0,0,0,0,0,26,0,0,0,12,0,0,0,width.to_le_bytes()[0],
        width.to_le_bytes()[1],height.to_le_bytes()[0],height.to_le_bytes()[1],1,0,24,0];
    let dat: Vec<_> = data.chunks_exact(3*width as usize).rev()
        .flat_map(|a| a.chunks_exact(3).flat_map(|a: &[u8]| [a[2],a[1],a[0]].into_iter()))
        .collect();
    file.write_all(&header).unwrap();
    file.write_all(&dat).unwrap();
}
