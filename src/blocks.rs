#![allow(non_snake_case)]
// #![allow(non_camel_case_types)]

pub trait FromBytes: Sized {
    fn from_bytes(bytes: &[u8]) -> Option<Self>;
}

// impl FromBytes for Re

macro_rules! define_mlv_block {
    ($name: ident { $($field_name:ident : $field_type:ty),+ }) => {
        #[derive(Debug,Clone,Copy)]
        pub struct $name {
            $($field_name: $field_type),+
        }

        // impl FromBytes for $name {
        //     fn from_bytes(bytes: &[u8]) -> Option<Self> {
        //         if bytes.len() != core::mem::size_of::<Self>() {
        //             return None;
        //         }
        //         let mut offset = 0;
        //         $(
        //             let $field_name = <$field_type>::from_bytes(&bytes[offset..])?;
        //             offset += core::mem::size_of::<$field_type>();
        //         )+
        //         Some(Self { $($field_name),+ })
        //     }
        // }
    };
}

define_mlv_block!{
    Poop {
        block_size: u32,
        timestamp: u64
    }
}

use crate::BlockTag;
use crate::endianness::*;

/* TODO: handle endianness properly. Assume MLV is always little endian. */

const MLV_VERSION_STRING: &str = "2.0";

// pub struct BlockTag ([u8; 4]);
// impl BlockTag {
//     /* Block name must be 4 chaarcters long */
//     pub const fn new(block_name: &str) -> BlockTag {
//         let as_bytes = block_name.as_bytes();
//         if block_name.len() != 4 || as_bytes.len() != 4 {
//             panic!("BlockTag must be constructed from 4 bytes (characters)");
//         }
//         Self::from_bytes(as_bytes[0], as_bytes[1], as_bytes[2], as_bytes[3])
//     }
//     #[inline(always)]
//     pub const fn from_bytes(a: u8, b: u8, c: u8, d: u8) -> BlockTag { BlockTag([a, b, c, d]) }
//     #[inline(always)]
//     pub const fn from_chars(a: char, b: char, c: char, d: char) -> BlockTag {
//         BlockTag::from_bytes(a as u8, b as u8, c as u8, d as u8)
//     }
// }

#[repr(C,packed)]
#[derive(Debug,Clone,Copy)]
pub struct BlockInfo {
    block_type: BlockTag,
    block_size: u32le,
    timestamp: u64le,
}

// "MLVI"
#[repr(C,packed)]
#[derive(Debug,Clone,Copy)]
pub struct FileHeader {
    fileMagic: [u8; 4],         /* Magic Lantern Video file header "MLVI" */
    blockSize: u32le,           /* size of the whole header */
    versionString: [u8; 8],     /* null-terminated C-string of the exact revision of this format */
    fileGuid: u64le,            /* UID of the file (group) generated using hw counter, time of day and PRNG */
    fileNum: u16le,             /* the ID within fileCount this file has (0 to fileCount-1) */
    fileCount: u16le,           /* how many files belong to this group (splitting or parallel) */
    fileFlags: u32le,           /* 1=out-of-order data, 2=dropped frames, 4=single image mode, 8=stopped due to error */
    videoClass: u16le,          /* 0=none, 1=RAW, 2=YUV, 3=JPEG, 4=H.264 */
    audioClass: u16le,          /* 0=none, 1=WAV */
    videoFrameCount: u32le,     /* number of video frames in this file. set to 0 on start, updated when finished. */
    audioFrameCount: u32le,     /* number of audio frames in this file. set to 0 on start, updated when finished. */
    sourceFpsNom: u32le,        /* configured fps in 1/s multiplied by sourceFpsDenom */
    sourceFpsDenom: u32le,      /* denominator for fps. usually set to 1000, but may be 1001 for NTSC */
}

#[repr(C,packed)]
#[derive(Debug,Clone,Copy)]
pub struct Vidf {
    frameNumber: u32le,         /* unique video frame number */
    cropPosX: u16le,            /* specifies from which sensor row/col the video frame was copied (8x2 blocks) */
    cropPosY: u16le,            /* (can be used to process dead/hot pixels) */
    panPosX: u16le,             /* specifies the panning offset which is cropPos, but with higher resolution (1x1 blocks) */
    panPosY: u16le,             /* (it's the frame area from sensor the user wants to see) */
    frameSpace: u32le,          /* size of dummy data before frameData starts, necessary for EDMAC alignment */
    /* uint8_t     frameData[variable]; */
}

#[repr(C,packed)]
#[derive(Debug,Clone,Copy)]
pub struct Audf {
    frameNumber: u32le,         /* unique audio frame number */
    frameSpace: u32le           /* size of dummy data before frameData starts, necessary for EDMAC alignment */
    /* uint8_t     frameData[variable]; */
}

#[repr(C,packed)]
#[derive(Debug,Clone,Copy)]
pub struct Rawi {
    xRes: u16le,                /* Configured video resolution, may differ from payload resolution */
    yRes: u16le,                /* Configured video resolution, may differ from payload resolution */
    raw_info: RawInfo,          /* the raw_info structure delivered by raw.h of ML Core */
}

#[repr(C,packed)]
#[derive(Debug,Clone,Copy)]
pub struct RawInfo {
    api_version: u32le,
    do_not_use_this: u32le,

    height: i32le,
    width: i32le,
    pitch: i32le,
    frame_size: i32le,
    bits_per_pixel: i32le,              // 14

    black_level: i32le,                 // autodetected
    white_level: i32le,                 // somewhere around 13000 - 16000, varies with camera, settings etc
                                        // would be best to autodetect it, but we can't do this reliably yet

    // "DNG JPEG info"
    jpeg_x: i32le, jpeg_y: i32le,
    jpeg_width: i32le, jpeg_height: i32le,

    // DNG active sensor area (Y1, X1, Y2, X2)
    dng_active_area: [i32le; 4],

    exposure_bias: [i32le; 2],        // DNG Exposure Bias (idk what's that)
    cfa_pattern: i32le,               // stick to 0x02010100 (RGBG) if you can
    calibration_illuminant1: i32le,
    color_matrix1: [i32le; 18],       // DNG Color Matrix
    dynamic_range: i32le              // EV x100, from analyzing black level and noise (very close to DxO)
}

#[repr(C,packed)]
#[derive(Debug,Clone,Copy)]
pub struct Rawc {
    blockType: [u8; 4],         /* RAWC - raw image capture information */
    blockSize: u32le,           /* sizeof(mlv_rawc_hdr_t) */
    timestamp: u64le,           /* hardware counter timestamp */

    /* see struct raw_capture_info from raw.h */

    /* sensor attributes: resolution, crop factor */
    sensor_res_x: u16le,        /* sensor resolution */
    sensor_res_y: u16le,        /* 2-3 GPixel cameras anytime soon? (to overflow this) */
    sensor_crop: u16le,         /* sensor crop factor x100 */
    reserved: u16le,            /* reserved for future use */

    /* video mode attributes */
    /* (how the sensor is configured for image capture) */
    /* subsampling factor: (binning_x+skipping_x) x (binning_y+skipping_y) */
    binning_x: u8,              /* 3 (1080p and 720p); 1 (crop, zoom) */
    skipping_x: u8,             /* so far, 0 everywhere */
    binning_y: u8,              /* 1 (most cameras in 1080/720p; also all crop modes); 3 (5D3 1080p); 5 (5D3 720p) */
    skipping_y: u8,             /* 2 (most cameras in 1080p); 4 (most cameras in 720p); 0 (5D3) */
    offset_x: i16le,            /* crop offset (top-left active pixel) - optional (SHRT_MIN if unknown) */
    offset_y: i16le,            /* relative to top-left active pixel from a full-res image (FRSP or CR2) */

    /* The captured *active* area (raw_info.active_area) will be mapped
     * on a full-res image (which does not use subsampling) as follows:
     *   active_width  = raw_info.active_area.x2 - raw_info.active_area.x1
     *   active_height = raw_info.active_area.y2 - raw_info.active_area.y1
     *   .x1 (left)  : offset_x + full_res.active_area.x1
     *   .y1 (top)   : offset_y + full_res.active_area.y1
     *   .x2 (right) : offset_x + active_width  * (binning_x+skipping_x) + full_res.active_area.x1
     *   .y2 (bottom): offset_y + active_height * (binning_y+skipping_y) + full_res.active_area.y1
     */
}

/* when audioClass is WAV, this block contains format details  compatible to RIFF */
#[repr(C,packed)]
#[derive(Debug,Clone,Copy)]
pub struct Wavi {
    format: u16le,            /* 1=Integer PCM, 6=alaw, 7=mulaw */
    channels: u16le,          /* audio channel count: 1=mono, 2=stereo */
    samplingRate: u32le,      /* audio sampling rate in 1/s */
    bytesPerSecond: u32le,    /* audio data rate */
    blockAlign: u16le,        /* see RIFF WAV hdr description */
    bitsPerSample: u16le      /* audio ADC resolution */
}

// typedef struct {
//     uint8_t     blockType[4];
//     uint32_t    blockSize;    /* total frame size */
//     uint64_t    timestamp;    /* hardware counter timestamp for this frame (relative to recording start) */
//     uint32_t    isoMode;    /* 0=manual, 1=auto */
//     uint32_t    isoValue;    /* camera delivered ISO value */
//     uint32_t    isoAnalog;    /* ISO obtained by hardware amplification (most full-stop ISOs, except extreme values) */
//     uint32_t    digitalGain;    /* digital ISO gain (1024 = 1 EV) - it's not baked in the raw data, so you may want to scale it or adjust the white level */
//     uint64_t    shutterValue;    /* exposure time in microseconds */
// }  mlv_expo_hdr_t;



// pub trait Block: Sized {
//     fn size() -> u32;
//     fn instance_size() -> u32;
//     fn signature(&self) -> BlockTag;
//     fn timestamp(&self) -> u64;
//     // fn serialise(&self)
//     fn from_bytes(&self, bytes: impl Iterator<Item=u8>) -> Option<Self>;
// }