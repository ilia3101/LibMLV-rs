// cargo run --release --bin raw2mlv --features="raw2mlv-deps,cineform" --  "/Volumes/EOS_DIGITAL/DCIM/101CANON/1E9A6283.CRM"  --out-path out.mlv
// use argparse::{ArgumentParser, Store, StoreTrue};
use clap::{Parser, ValueEnum};
use mlv::codec;
use rand::Rng;
use std::error::Error;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(ValueEnum, Debug, Copy, Clone, PartialEq, Eq)]
enum Codec {
    #[value(name = "8")]
    Uncompressed8,
    #[value(name = "10")]
    Uncompressed10,
    #[value(name = "12")]
    Uncompressed12,
    #[value(name = "14")]
    Uncompressed14,
    #[value(name = "16")]
    Uncompressed16,
    #[value(name = "lj92")]
    Lj92,
    #[value(name = "cineform")]
    Cineform,
    #[value(name = "jp2k")]
    Jp2k,
}

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[rustfmt::skip]
#[command(version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(short, long)]
    out_path: String,

    /// Output bit depth. If higher than the input range, it will not be increased. 8/10/12/14/16 supported.
    #[arg(long, value_enum, default_value_t = Codec::Uncompressed14)]
    codec: Codec,

    /// Data multiplier. Use this to remap between bitdepths. Not setting this means auto.
    #[arg(long)]
    multiplier: Option<f32>,

    /// Black level to write to output mlv. Not setting it means auto.
    #[arg(long)]
    black_level: Option<u16>,

    /// Black level to write to output mlv. Not setting it means auto.
    #[arg(long)]
    white_level: Option<u16>,

    /// Framerate to write to output MLV and base timestamps on. Example: 24000 1001 for 23.976
    #[arg(long, num_args = 2, value_names = ["NUM", "DENOM"])]
    frame_rate: Vec<u32>,

    /* /// Centre crop rect, specify resolution in the format of 1920 1080 for example, this area will be cropped out from the center of the image.
    #[arg(long, num_args = 2, value_names = ["WIDTH", "HEIGHT"])]
    centre_crop: Option<Vec<i32>>, */

    /// Round resolution to multiple, default 8,2
    #[arg(long, num_args = 2, value_names = ["X", "Y"])]
    round_resolution: Vec<i32>,

    /// Input files. Either multiple raw still images, or one MLV video to re-encode.
    inputs: Vec<String>,
}

// CFHD_EncodingQuality

fn check_cr3(path: &str) -> u32 {
    use std::env;
    use std::fs::File;
    use std::io::prelude::*;
    use std::io::{BufWriter, BufReader};
    use std::path::Path;

    use rawler::{decoders::RawDecodeParams, rawsource::RawSource};
    use rawler::decoders::cr3::Cr3Decoder;
    use rawler::formats::bmff::Bmff;
    use rawler::RawLoader;
    use rawler::decoders::Decoder;

    let path = Path::new(&path);
    let mut rawfile = RawSource::new(path).unwrap();
    let mut rawfile2 = RawSource::new(path).unwrap();

    let rawler = RawLoader::new();

    use std::time::Instant;
    let now = Instant::now();

    let mut num_images = 1;
    if path.extension() == Some(std::ffi::OsStr::new("cr3")) || path.extension() == Some(std::ffi::OsStr::new("CRM")) {
        println!("Detected CR3 file format");
        let cr3decoder = Cr3Decoder::new(
            &mut rawfile,
            Bmff::new(BufReader::new(std::fs::File::open(&path).expect("Coyldnt open file"))).expect("Failed to create bmff thingy"),
            &rawler
        ).expect("Failed to create CR3 decoder");
        num_images = cr3decoder.raw_image_count().expect("Failed to get number of images in CR3 file");
        println!("Number of images in CR3 file: {}", num_images);
    }

    println!("Number of images in CR3 file: {}", num_images);

    return num_images as u32;
}


fn run_through_cr3(path: &str, mut callback: impl FnMut(&[u16]) -> Result<(),Box<dyn Error>>) -> Result<(), Box<dyn Error>> {
    use std::env;
    use std::fs::File;
    use std::io::prelude::*;
    use std::io::{BufWriter, BufReader};
    use std::path::Path;

    use rawler::{decoders::RawDecodeParams, rawsource::RawSource};
    use rawler::decoders::cr3::Cr3Decoder;
    use rawler::formats::bmff::Bmff;
    use rawler::RawLoader;
    use rawler::decoders::Decoder;
    // use rayon::iter::{IntoParallelIterator, ParallelIterator};

    let path = Path::new(&path);
    let mut rawfile = RawSource::new(path).unwrap();
    let mut rawfile2 = RawSource::new(path).unwrap();

    let rawler = RawLoader::new();

    use std::time::Instant;
    let now = Instant::now();

    let mut num_images = 1;
    if path.extension() == Some(std::ffi::OsStr::new("cr3")) || path.extension() == Some(std::ffi::OsStr::new("CRM")) {
        println!("Detected CR3 file format");
        let cr3decoder = Cr3Decoder::new(
            &mut rawfile,
            Bmff::new(BufReader::new(std::fs::File::open(&path).expect("Coyldnt open file"))).expect("Failed to create bmff thingy"),
            &rawler
        ).expect("Failed to create CR3 decoder");
        num_images = cr3decoder.raw_image_count().expect("Failed to get number of images in CR3 file");
        println!("Number of images in CR3 file: {}", num_images);
    }

    println!("Number of images in CR3 file: {}", num_images);

    // doing this shit gives a tiny speedup
    let cr3decoder = Cr3Decoder::new(
        &mut rawfile,
        Bmff::new(BufReader::new(std::fs::File::open(&path).expect("Coyldnt open file"))).expect("Failed to create bmff thingy"),
        &rawler
    ).expect("Failed to create CR3 decoder");

    let now = Instant::now();
    for idx in (0..num_images).step_by(1) {
        println!("Decoding imahge {idx}");
        let rawimage = cr3decoder.raw_image(&mut rawfile2, &RawDecodeParams {image_index: idx}, false).expect("Failed to decode image");
        if let rawler::RawImageData::Integer(data) = rawimage.data {
            callback(&data)?
        } else {
            // eprintln!("Don't know how to process non-integer raw files");
        }
    }

    // (0..num_images).into_par_iter().for_each(|idx| {
    //     println!("Decoding image {}", idx);
    //     let mut rawfile = RawSource::new(&path).unwrap();
    //     let rawimage = cr3decoder.raw_image(&mut rawfile, &RawDecodeParams {image_index: idx}, false).expect("Failed to decode image");
    //     println!("Black lebel = {:?}", rawimage.blacklevel);
    //     println!("White lebel = {:?}", rawimage.whitelevel);
    //     if let rawler::RawImageData::Integer(data) = rawimage.data {
    //         println!("pixel 0 = {}", data[0]);
    //     }
    // });

    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?}", elapsed);

    Ok(())
}

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
pub fn logwithlin_inverse(y: f64, thresh: f64, stops_range: f64) -> f64 {
    // Precompute the threshold value in output space and the linear slope
    let y_thresh = log1(thresh, stops_range);
    let m = log1_gradient(thresh, stops_range);

    // Guard against division by zero (shouldn't occur with valid inputs)
    if m == 0.0 { return thresh; }

    if y >= y_thresh {
        // Invert the log branch
        2.0_f64.powf((y - 1.0) * stops_range + 1.0)
    } else {
        // Invert the linear branch
        thresh + (y - y_thresh) / m
    }
}

fn log_encode(x: f64) -> f64 {
    logwithlin(x, 0.0061, 9.72)
}

fn log_encode_int(x: u16, bl: u16, max: u16) -> u16 {
    let as_float = ((x as f32 - bl as f32) as f32) / ((max - bl) as f32);
    let as_log = log_encode(as_float as f64) * 65535.0;
    return (as_log + 0.5) as u16;
}

fn log_decode_int(x: u16, bl: u16, max: u16) -> u16 {
    let as_float = logwithlin_inverse(x as f64 / 65535.0, 0.0061, 9.72);
    let in_range = as_float * (max-bl) as f64 + bl as f64;
    return (in_range + 0.5) as u16;
}

pub fn encode_log_14_to_12(raw: u16) -> u16 {
    const BLACK: f32 = 2000.0;
    const WHITE: f32 = 16383.0;

    const OUT_BLACK: f32 = 32.0;
    const OUT_WHITE: f32 = 65535.0;

    // Clamp to valid sensor range
    let x = (raw as f32).clamp(BLACK, WHITE);

    // Normalize to 0..1 after black subtraction
    let norm = (x - BLACK) / (WHITE - BLACK);

    // Log curve strength
    // Higher values = more shadow emphasis.
    const LOG_A: f32 = 500.0;

    let log_norm = (1.0 + LOG_A * norm).ln()
        / (1.0 + LOG_A).ln();

    let encoded = OUT_BLACK
        + log_norm * (OUT_WHITE - OUT_BLACK);

    encoded.round() as u16
}

pub fn decode_log_12_to_14(code: u16) -> u16 {
    const BLACK: f32 = 2000.0;
    const WHITE: f32 = 16383.0;

    const OUT_BLACK: f32 = 32.0;
    // const OUT_WHITE: f32 = 4095.0;
    const OUT_WHITE: f32 = 65535.0;

    const LOG_A: f32 = 500.0;

    let y = ((code as f32).clamp(OUT_BLACK, OUT_WHITE) - OUT_BLACK)
        / (OUT_WHITE - OUT_BLACK);

    let norm = ((1.0 + LOG_A).powf(y) - 1.0) / LOG_A;

    let raw = BLACK + norm * (WHITE - BLACK);

    raw.round() as u16
}

fn redecode(data: &[u16], width: u32, height: u32) -> Vec<u16> {
    // let (encode, decode) = (encode_log_14_to_12, decode_log_12_to_14);
    let (encode, decode) = (|x| log_encode_int(x, 1024, 16383), |x| log_decode_int(x, 1024, 16383));
    let data = data.iter().copied().map(encode).collect::<Vec<_>>();
    let encoded = cineform_sys::Encoder::new(width as u32, height as u32, cineform_sys::sys::CFHD_ENCODING_QUALITY_FILMSCAN3).unwrap().encode(&data).unwrap();
    let cratio = (data.len() * 14) as f64 / (encoded.len() * 8) as f64;
    println!("Compression: {:.2?}", cratio);
    return cineform_sys::Decoder::new().unwrap().decode(&encoded, width, height).unwrap().0.iter().copied().map(decode).collect::<Vec<_>>();
    // return data.to_vec()
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = Args::parse();

    /* Set default args */
    if args.frame_rate.len() == 0 {
        args.frame_rate = vec![24000, 1000];
        /* TODO: if input is MLV, fetch framerate from there! */
    }
    if args.round_resolution.len() == 0 {
        args.round_resolution = vec![8, 2];
    }

    println!("Args = {:#?}", args);

    #[cfg(feature = "cineform")]
    let quality = cineform_sys::sys::CFHD_ENCODING_QUALITY_FILMSCAN1;

    let mut camera_model_name = "Canon EOS 5D Mark II".to_string();
    let mut camera_model_id = 123u32;
    let mut colour_matrix = [
        5309, 10000, -229, 10000, -336, 10000, -6241, 10000, 13265, 10000, 3337, 10000, -817, 10000, 1215, 10000, 6664,
        10000,
    ];

    let mut width = 1920;
    let mut height = 1080;
    let mut black_level = 1024;
    let mut white_level = 15000;
    let mut bayer_pattern = 0x02010100;
    let mut bpp = 14i32;

    let mut num_frames_in_file_1 = 1;

    if let Ok(rl) = rawler::decode_file(&args.inputs[0]) {
        width = rl.width as u16;
        height = rl.height as u16;
        black_level = {
            let blr = rl.blacklevel.levels[0];
            (blr.n as f32 / blr.d as f32) //+ 16.0 //TODO: GH1 and GF3 black lebel needs a + 16
        } as i32;
        white_level = rl.whitelevel.0[0] as i32;
        num_frames_in_file_1 = check_cr3(&args.inputs[0]);
        println!("File has multiple images {:?}", num_frames_in_file_1);
        // if let Ok(count) = rl.raw_image_count() && count >= 1 {
        //     println!("File has multiple images {:?}", rl.raw_image_count())
        // }
    } else {
        panic!("Couldn't read first file");
    }

    let out_file_guid = {
        let timestamp_secs =
            SystemTime::now().duration_since(UNIX_EPOCH).expect("SystemTime before UNIX EPOCH!").as_secs() as u32;
        let rand: u32 = rand::thread_rng().r#gen();
        ((timestamp_secs as u64) << 32) | (rand as u64)
    };

    let file = File::create(args.out_path)?;
    let mut writer = BufWriter::new(file);

    writer.write_all("MLVI".as_bytes())?;
    writer.write_all(&u32::to_le_bytes(52))?;
    writer.write_all(&[b'v', b'2', b'.', b'0', 0, 0, 0, 0])?;
    writer.write_all(&u64::to_le_bytes(out_file_guid))?;
    writer.write_all(&u16::to_le_bytes(0))?;
    writer.write_all(&u16::to_le_bytes(1))?;
    writer.write_all(&u32::to_le_bytes(0))?;
    let videoclass = match args.codec {
        Codec::Cineform => 0x11,
        Codec::Jp2k => 0x201,
        Codec::Lj92 => 0x21,
        _ => 0x01,
    };
    println!("videoclass = {videoclass}");
    writer.write_all(&u16::to_le_bytes(videoclass))?; /* 0=none, 1=RAW, 2=YUV, 3=JPEG, 4=H.264 */
    writer.write_all(&u16::to_le_bytes(0))?; // audioclass, set 1 for wav. 0 for none
    writer.write_all(&u32::to_le_bytes(args.inputs.len() as u32))?; // framecount
    writer.write_all(&u32::to_le_bytes(0))?; // audio framecount
    writer.write_all(&u32::to_le_bytes(args.frame_rate[0]))?; // fpsn num
    writer.write_all(&u32::to_le_bytes(args.frame_rate[1]))?; // fps denom

    writer.write_all("RAWI".as_bytes())?;
    writer.write_all(&u32::to_le_bytes(180))?;
    writer.write_all(&u64::to_le_bytes(10))?;
    writer.write_all(&u16::to_le_bytes(width))?; // xRes: u16, /* Configured video resolution, may differ from payload resolution */
    writer.write_all(&u16::to_le_bytes(height))?; // yRes: u16, /* Configured video resolution, may differ from payload resolution */
    /* the raw_info structure delivered by raw.h of ML Core */
    writer.write_all(&u32::to_le_bytes(1))?; // api_version: u32,
    writer.write_all(&u32::to_le_bytes(0))?; // do_not_use_this: u32,
    writer.write_all(&(height as i32).to_le_bytes())?; // height: i32,
    writer.write_all(&(width as i32).to_le_bytes())?; // width: i32,
    writer.write_all(&1i32.to_le_bytes())?; // pitch: i32,
    writer.write_all(&(1 as i32).to_le_bytes())?; // frame_size: i32,
    writer.write_all(&(bpp as i32).to_le_bytes())?; // bits_per_pixel: i32, // 14
    writer.write_all(&(black_level as i32).to_le_bytes())?; // black_level: i32
    writer.write_all(&(white_level as i32).to_le_bytes())?; // white_level: i32,
    // jpeg_x: i32, jpeg_y: i32, // "DNG JPEG info"
    writer.write_all(&(width as i32).to_le_bytes())?;
    writer.write_all(&(height as i32).to_le_bytes())?;
    // jpeg_width: i32, jpeg_height: i32,
    writer.write_all(&(width as i32).to_le_bytes())?;
    writer.write_all(&(height as i32).to_le_bytes())?;
    writer.write_all(&[0; 16])?; // dng_active_area: [i32; 4] // DNG active sensor area (Y1, X1, Y2, X2)
    writer.write_all(&(0i32).to_le_bytes())?; // exposure_bias: [i32; 2], // DNG Exposure Bias (idk what's that)
    writer.write_all(&(0i32).to_le_bytes())?; // cfa_pattern: i32, // stick to 0x02010100 (RGBG) if you can
    writer.write_all(&(0x02010100i32).to_le_bytes())?; //     calibration_illuminant1: i32,
    writer.write_all(&(2i32).to_le_bytes())?;
    for element in colour_matrix {
        //     color_matrix1: [i32; 18],       // DNG Color Matrix
        writer.write_all(&(element as i32).to_le_bytes())?;
    }
    writer.write_all(&(1100i32).to_le_bytes())?; //     dynamic_range: i32              // EV x100, from analyzing black level and noise (very close to DxO)

    if num_frames_in_file_1 != 1 {
        let mut frame_count = 0u32;
        run_through_cr3(&args.inputs[0], |data: &[u16]| {
            let mut histogram = vec![0u32; 65536];
            for &pix in data.iter() {
                histogram[pix as usize] += 1;
            }
            for val in 5000..=5030 {
                println!("hist[{}] = {}", val, histogram[val]);
            }

            let frame_size_bits = (width as u64 * height as u64 * bpp as u64);
            let mut frame_size_bytes = frame_size_bits / 8;
            if (frame_size_bits * 8) < frame_size_bits {
                frame_size_bytes += 1;
            }
            let mut size = 32;
            writer.write_all("VIDF".as_bytes())?;
            let mut buf = Vec::new();

            // CINEFORM TEST
            let data = data.to_vec();

            if args.codec == Codec::Jp2k {
                let mut enc = bayer_compression::jp2kht::BayerEncoder::new();
                let mut jp2k_buf = vec![0u8; data.len() * 2];
                let size = enc.encode_bayer(width as u32, height as u32, 14, &data, &mut jp2k_buf, 0.008);
                jp2k_buf.truncate(size);
                buf = jp2k_buf;
            } else {
                #[cfg(feature = "cineform")]
                let data = redecode(&data, width as u32, height as u32);

                if args.codec != Codec::Cineform {
                    buf = vec![0; frame_size_bytes as usize];
                    mlv::codec::encode_packed14(&data, &mut buf);
                } else {
                    #[cfg(feature = "cineform")]
                    {
                        if let Ok(e) = cineform_sys::Encoder::new(width as u32, height as u32, quality) {
                            if let Ok(encoded) = e.encode(&data) {
                                buf = encoded;
                            }
                        }
                    }
                }
            }

            writer.write_all(&(32u32 + buf.len() as u32).to_le_bytes())?;
            writer.write_all(&10000u64.to_le_bytes())?;
            writer.write_all(&frame_count.to_le_bytes())?;
            writer.write_all(&[0u8; 8])?;
            writer.write_all(&0u32.to_le_bytes())?;
            writer.write_all(&buf)?;

            frame_count += 1;

            Ok(())
        });
    } else {
        let mut frame_count = 0u32;
        for path in args.inputs {
            // println!("")
            if let Ok(rl) = rawler::decode_file(path) {
                if let rawler::RawImageData::Integer(data) = rl.data {
                    let mut histogram = vec![0u32; 65536];
                    for &pix in data.iter() {
                        histogram[pix as usize] += 1;
                    }
                    for val in 5000..=5030 {
                        println!("hist[{}] = {}", val, histogram[val]);
                    }

                    let frame_size_bits = (width as u64 * height as u64 * bpp as u64);
                    let mut frame_size_bytes = frame_size_bits / 8;
                    if (frame_size_bits * 8) < frame_size_bits {
                        frame_size_bytes += 1;
                    }
                    let mut size = 32;
                    writer.write_all("VIDF".as_bytes())?;
                    let mut buf = Vec::new();


                    // CINEFORM TEST
                    let data = data.to_vec();

                    if args.codec == Codec::Jp2k {
                        let mut enc = bayer_compression::jp2kht::BayerEncoder::new();
                        let mut jp2k_buf = vec![0u8; data.len() * 2];
                        let size = enc.encode_bayer(width as u32, height as u32, 14, &data, &mut jp2k_buf, 0.008);
                        jp2k_buf.truncate(size);
                        buf = jp2k_buf;
                    } else {
                        #[cfg(feature = "cineform")]
                        let data = redecode(&data, width as u32, height as u32);

                        if args.codec != Codec::Cineform {
                            buf = vec![0; frame_size_bytes as usize];
                            mlv::codec::encode_packed14(&data, &mut buf);
                        } else {
                            #[cfg(feature = "cineform")]
                            {
                                if let Ok(e) = cineform_sys::Encoder::new(width as u32, height as u32, quality) {
                                    if let Ok(encoded) = e.encode(&data) {
                                        buf = encoded;
                                    }
                                }
                            }
                        }
                    }

                    writer.write_all(&(32u32 + buf.len() as u32).to_le_bytes())?;
                    writer.write_all(&10000u64.to_le_bytes())?;
                    writer.write_all(&frame_count.to_le_bytes())?;
                    writer.write_all(&[0u8; 8])?;
                    writer.write_all(&0u32.to_le_bytes())?;
                    writer.write_all(&buf)?;

                    frame_count += 1;
                }
            }
        }
    }

    Ok(())
}
