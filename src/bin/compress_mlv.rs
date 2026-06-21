// example command: cargo run --release --bin compress_mlv --features="compress-mlv" -- --codec jp2k-high --output output.MLV --input /path/to/input.MLV
use clap::{Parser, ValueEnum};
use mlv::{BlockHeader, FileDataSource, block_reader};
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};

#[derive(Parser, Debug)]
#[command(about, long_about = None)]
struct Args {
    /** Input: Either a single MLV file (chunks will be found automatically, or can be specified), a canon CRM file, or multiple still images. */
    inputs: Vec<String>,

    /** Output MLV file */
    #[arg(long, short)]
    output: String,

    /** Compression codec (default: jp2k-high) - I recommend jp2k-high for 3K+ video and jp2k-very-high for 1080p. If you have 5k+ and don't intend to crop, jp2k-medium might be good enough. */
    #[arg(long, value_enum, default_value_t = CompressionCodec::Jp2kHigh)]
    codec: CompressionCodec,

    /** Enabled by default, as vertical stripes can interfere with compression in an ugly way and become uncorrectable. */
    #[arg(long, default_value_t = true)]
    fix_vertical_stripes: bool,
}

#[derive(ValueEnum, Clone, Copy, PartialEq, Eq, Debug)]
enum CompressionCodec {
    #[value(name = "jp2k-low")]
    Jp2kLow,
    #[value(name = "jp2k-medium")]
    Jp2kMedium,
    #[value(name = "jp2k-high")]
    Jp2kHigh,
    #[value(name = "jp2k-very-high")]
    Jp2kVeryHigh,
    #[value(name = "jp2k-visually-lossless")]
    Jp2kVisuallyLossless,
}

impl CompressionCodec {
    fn quant(&self) -> f64 {
        match self {
            CompressionCodec::Jp2kLow => 0.01,                // Visible artifacts
            CompressionCodec::Jp2kMedium => 0.0065, // Good for 5K+ resolution if you have no intention of cropping
            CompressionCodec::Jp2kHigh => 0.0045,   // good for 3k+ only
            CompressionCodec::Jp2kVeryHigh => 0.0032, // good for 1080
            CompressionCodec::Jp2kVisuallyLossless => 0.0015, // (go up one quality level if you're a perfectionist. TODO: decide on rules)
        }
    }
}

// Some crap I came up with using desmos - this needs more consideration to improve compression and stop wasting so many code values on dark/noisy images as it currently is
mod log {
    fn log1(x: f64, stops_range: f64) -> f64 {
        (x.log2() - 1.0) / (1.0f64.log2() - (2.0f64.powf(-stops_range).log2())) + 1.0
    }

    fn log1_gradient(x: f64, stops_range: f64) -> f64 {
        1.0 / (x * stops_range * core::f64::consts::LN_2)
    }

    fn logwithlin(x: f64, thresh: f64, stops_range: f64) -> f64 {
        if x < thresh {
            log1_gradient(thresh, stops_range) * (x - thresh) + log1(thresh, stops_range)
        } else {
            log1(x, stops_range)
        }
    }

    /// Inverse of `withlin`. Given y = withlin(x, thresh, stops_range), returns x.
    fn logwithlin_inverse(y: f64, thresh: f64, stops_range: f64) -> f64 {
        // Precompute the threshold value in output space and the linear slope
        let y_thresh = log1(thresh, stops_range);
        let m = log1_gradient(thresh, stops_range);

        // Guard against division by zero (shouldn't occur with valid inputs)
        if m == 0.0 {
            return thresh;
        }

        if y >= y_thresh {
            // Invert the log branch
            2.0_f64.powf((y - 1.0) * stops_range + 1.0)
        } else {
            // Invert the linear branch
            thresh + (y - y_thresh) / m
        }
    }

    const LOG_THRESH: f64 = 0.0061;
    const LOG_STOPS: f64 = 9.72;

    fn log_encode(x: f64) -> f64 {
        logwithlin(x, LOG_THRESH, LOG_STOPS)
    }

    #[inline]
    pub fn log_encode_int(x: u16, bl: u16, max: u16, max_value: u16) -> u16 {
        let as_float = ((x.saturating_sub(bl) as f32) / ((max - bl) as f32)).min(1.0);
        let as_log = log_encode(as_float as f64) * max_value as f64;
        return ((as_log + 0.5) as u16).min(max_value);
    }

    #[inline]
    pub fn log_decode_int(x: u16, bl: u16, max: u16, max_value: u16) -> u16 {
        let x = x.min(max_value);
        let as_float = logwithlin_inverse(x as f64 / max_value as f64, LOG_THRESH, LOG_STOPS);
        let in_range = as_float * (max - bl) as f64 + bl as f64;
        return (in_range + 0.5) as u16;
    }
}

use log::*;

// TODO: adapt to bayer pattern.
fn get_vertical_stripes_coeffs(data: &[u16], width: usize, bl: u16) -> [f32; 8] {
    let bayer = 0; // Set this to 0 or 1 to determine which pixels are bayer green

    // get green only pixels
    let mut green_only = Vec::<u16>::with_capacity(data.len() / 2 - width);
    unsafe { green_only.set_len(data.len() / 2 - width) };
    for (in_rows, out_row) in data.chunks(width * 2).zip(green_only.chunks_mut(width)) {
        for i in 0..width {
            let in_idx = i + if i % 2 == bayer { width } else { 0 };
            out_row[i] = in_rows[in_idx];
        }
    }
    let data = &green_only;

    fn blur8(inrow: &[u16], width: i32, out: &mut [u16]) {
        let get = |x: i32| inrow[x.clamp(0, width - 1) as usize] as i32;
        let mut blur_value_a: i32 = (1..=4).map(get).fold(get(0) * 4, |acc, x| acc + x);
        let mut blur_value_b: i32 = (1..=3).map(get).fold(get(0) * 5, |acc, x| acc + x);
        let (mut leftptr_a, mut rightptr_a) = (-3, 5);
        let (mut leftptr_b, mut rightptr_b) = (-4, 4);
        for i in 0..out.len() {
            out[i] = ((blur_value_a + blur_value_b) / 16) as u16;
            blur_value_a = blur_value_a + (get(rightptr_a) - get(leftptr_a));
            blur_value_b = blur_value_b + (get(rightptr_b) - get(leftptr_b));
            leftptr_a += 1;
            rightptr_a += 1;
            leftptr_b += 1;
            rightptr_b += 1;
        }
    }
    let mut blurred = Vec::<u16>::with_capacity(data.len());
    unsafe { blurred.set_len(data.len()) };
    // box blur size 8 each row to ignore gradients
    for (row, out) in data.chunks(width).zip(blurred.chunks_mut(width)) {
        blur8(row, width as i32, out)
    }

    // estimate white level. TODO: do histogram perhaps to ignore any crazy outliers
    let mut max_value = data.iter().copied().max().unwrap_or(65535);

    const NUM_BINS: usize = 4096;
    const EV_RANGE: f32 = 1.5;
    let mut diff_hists = [[0u32; NUM_BINS]; 8];

    // Padding. TODO: does it need it?
    let (col_start, col_end) = (0, width);

    for (y, (row, blurred)) in data.chunks(width).zip(blurred.chunks(width)).enumerate() {
        for (x, (pix, blurred)) in row[col_start..col_end].iter().zip(blurred[col_start..col_end].iter()).enumerate() {
            let col_idx = x % 8;
            let (pix, blurred) = (*pix as i32 - bl as i32, *blurred as i32 - bl as i32);
            let diff = (blurred as f32 / pix as f32).log2();
            let bin = (((diff + EV_RANGE) / (EV_RANGE * 2.0)) * NUM_BINS as f32).round() as i32;
            let bin_idx = bin.min((NUM_BINS - 1) as i32).max(0) as usize;
            // TODO: be conditional about the pixel value, exclude overexposed or too dark??!
            diff_hists[col_idx][bin_idx] += 1;
        }
    }

    // find median of each columns histogram bin position
    let sums = diff_hists.map(|hist| hist.iter().sum::<u32>());
    // println!("diff_hists: {:?}", diff_hists);
    // println!("sums: {:?}", sums);
    let median_bin_positions = diff_hists.iter().zip(sums.iter()).map(|(hist, hist_sum)| {
        let (mut bin, mut sum) = (0, 0);
        for (bin, value) in hist.iter().enumerate() {
            sum += value;
            if sum > (*hist_sum) / 2 {
                return bin;
            }
        }
        return NUM_BINS - 1;
    });

    // convert bin position to multiplier
    let mut multipliers =
        median_bin_positions.map(|x| (((x as f32) / NUM_BINS as f32) * (EV_RANGE * 2.0) - EV_RANGE).exp2());

    core::array::from_fn(|_| multipliers.next().unwrap_or(1.0))
}

fn apply_vertical_stripe_coeffs(data: &mut [u16], width: usize, bl: u16, coeffs: [f32; 8]) {
    for row in data.chunks_mut(width) {
        for chunk in row.as_chunks_mut::<8>().0.iter_mut() {
            for i in 0..8 {
                chunk[i] = ((chunk[i] as f32 - bl as f32) as f32 * coeffs[i] + bl as f32).round() as u16;
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let input_path = &args.inputs[0];
    let output_path = &args.output;

    // let input_paths = args.inputs.map(|p| PathBuf::from());

    let codec = args.codec;
    println!("Codec: {:?} (quant: {})", codec, codec.quant());

    println!("{:?}", args.inputs);

    let mut reader = mlv::MainReader::open_mlv_from_path(input_path, None).expect("Couldn't open MLV");
    let bl = reader.black_level().unwrap();
    let wl = reader.white_level().unwrap();
    let mut input_file = BufReader::new(File::open(input_path).expect("Failed to open input file"));

    let mut data_source = FileDataSource::new(&input_path)?;

    // Curve LUT
    const LOG_MAX_VALUE: u16 = 4095;
    let lut_encode: Vec<u16> =
        (0u32..65536).map(|i| log_encode_int(i as u16, bl as u16, 16383, LOG_MAX_VALUE)).collect();
    let lut_decode: Vec<u16> =
        (0..=LOG_MAX_VALUE).map(|i| log_decode_int(i, bl as u16, 16383, LOG_MAX_VALUE)).collect();
    let curv_block = "CURV"
        .as_bytes()
        .iter()
        .copied()
        .chain((16u32 + lut_decode.len() as u32 * 2).to_le_bytes().into_iter())
        .chain((1u64).to_le_bytes().into_iter())
        .chain(lut_decode.into_iter().flat_map(|x| x.to_le_bytes().into_iter()));

    // Write output
    let mut output_file = BufWriter::new(File::create(output_path).expect("Failed to create output file"));

    // build a list of blocks to write (in the case of MLV this is simple, pretty much just copy old ones)
    let mut blocks_to_write: Vec<Vec<u8>> = vec![];

    let is_mlv = true;
    if is_mlv {
        // Find first MLVI. TODO: set file count to 1 if input is chunked
        for block in &reader.all_blocks2 {
            if block.is_type("MLVI") {
                input_file.seek(SeekFrom::Start(block.loc.offset())).unwrap();
                let mut block_buf = vec![0; block.size() as usize];
                input_file.read_exact(&mut block_buf).unwrap();
                let video_class: u16 = 0x11; // just raw is 0x01, 0x11 is jp2k
                block_buf[32..34].copy_from_slice(&u16::to_le_bytes(video_class));
                blocks_to_write.push(block_buf);
                break;
            }
        }

        // Write all non MLVI blocks. TODO: modify reader to store them in memory instead of having to re-read all of them?
        for block in &reader.all_blocks2 {
            if !block.is_type("MLVI") && !block.is_type("VIDF") && !block.is_type("AUDF") {
                input_file.seek(SeekFrom::Start(block.loc.offset())).unwrap();
                let mut block_buf = vec![0; block.size() as usize];
                input_file.read_exact(&mut block_buf).unwrap();
                blocks_to_write.push(block_buf);
            }
        }
    }

    blocks_to_write.push(curv_block.collect());

    if is_mlv {
        // write all audio AUDF
        for block in &reader.all_blocks2 {
            if block.is_type("AUDF") {
                input_file.seek(SeekFrom::Start(block.loc.offset())).unwrap();
                let mut block_buf = vec![0; block.size() as usize];
                input_file.read_exact(&mut block_buf).unwrap();
                blocks_to_write.push(block_buf);
            }
        }
    }

    // write all blocks in blocks_to_write
    for block in blocks_to_write {
        output_file.write_all(&block)?;
    }

    // get vertical stripe coeffs from first frame
    let mut frame_buf = vec![0; (reader.width().unwrap() * reader.height().unwrap()) as usize];

    let mut vscoeffs = [1.0; 8];
    if args.fix_vertical_stripes {
        reader.decode_frame(0, &mut frame_buf);
        let vscoeffs = get_vertical_stripes_coeffs(&frame_buf, reader.width().unwrap() as usize, bl as u16);
        println!("Vertical stripe multipliers = {:.4?}", vscoeffs);
    }

    println!("bl = {}", bl);

    let mut jp2k_encoder = if codec.is_jp2k() { Some(mlv::codec::jpeg2000::BayerEncoder::new()) } else { None };
    let mut encoded_sizes_sum = 0;

    for i in tqdm::tqdm(0..reader.num_frames()) {
        reader.decode_frame(i, &mut frame_buf);

        if args.fix_vertical_stripes {
            apply_vertical_stripe_coeffs(&mut frame_buf, reader.width().unwrap() as usize, bl as u16, vscoeffs);
        }

        let mut buf = vec![];
        let mut logged = frame_buf.iter().copied().map(|x| lut_encode[x as usize]).collect::<Vec<_>>();
        if codec.is_jp2k() {
            let mut jp2k_buf = vec![0u8; (reader.width().unwrap() as usize * reader.height().unwrap() as usize * 4)];
            let size = jp2k_encoder.as_mut().unwrap().encode_bayer(
                reader.width().unwrap() as u32,
                reader.height().unwrap() as u32,
                12,
                &logged,
                &mut jp2k_buf,
                codec.quant() as f32,
            );
            jp2k_buf.truncate(size);
            buf = jp2k_buf;
        }

        encoded_sizes_sum += buf.len();

        // write it now...
        output_file.write_all("VIDF".as_bytes())?;
        output_file.write_all(&(32u32 + buf.len() as u32).to_le_bytes())?;
        output_file.write_all(&10000u64.to_le_bytes())?;
        output_file.write_all(&(i as u32).to_le_bytes())?;
        output_file.write_all(&[0u8; 8])?;
        output_file.write_all(&0u32.to_le_bytes())?;
        output_file.write_all(&buf)?;
    }

    output_file.flush().expect("Failed to flush output");
    let input_size = std::fs::metadata(input_path)?.len() as f64;
    let output_size = std::fs::metadata(output_path)?.len() as f64;
    println!("Done! Wrote sorted MLV to {output_path}");
    println!(
        "File size reduction: {:.1}:1 ({:.1} MB -> {:.1} MB)",
        input_size / output_size,
        input_size / 1_000_000.0,
        output_size / 1_000_000.0
    );
    // include average block size
    let bidepth = 14;
    let uncomp_size = reader.all_blocks2.len() as u64 * 32
        + (reader.width().unwrap() as u64 * reader.height().unwrap() as u64 * bidepth * reader.num_frames() as u64) / 8;
    println!(
        "Compression ratio compared to uncompressed: {:.1}x ({:.1} MB -> {:.1} MB)",
        uncomp_size as f64 / encoded_sizes_sum as f64,
        uncomp_size as f64 / 1_000_000.0,
        output_size / 1_000_000.0
    );

    Ok(())
}
