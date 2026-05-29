#![allow(non_snake_case)]
// #![allow(non_camel_case_types)]


/********************** Type defiintions and mactos **********************/

/* Why did i invent my own reflection */

const SIGNED_BIT: u8 = 0x80;

#[derive(Debug,Clone,Copy)]
#[repr(u8)]
pub enum PrimitiveType {
    U8 = 1, U16 = 2, U32 = 4, U64 = 8,
    I8 = 1 | SIGNED_BIT, I16 = 2 | SIGNED_BIT,
    I32 = 4 | SIGNED_BIT, I64 = 8 | SIGNED_BIT,
}

impl PrimitiveType {
    #[inline] pub const fn size(self) -> u32 { self as u32 & !SIGNED_BIT as u32 }
}

#[derive(Debug,Clone,Copy)]
pub enum FieldType {
    Single(PrimitiveType),
    Array(PrimitiveType, u32),
}

impl FieldType {
    #[inline]
    pub const fn size(self) -> u32 {
        match self {
            FieldType::Single(t) => t.size(),
            FieldType::Array(t, len) => t.size() * len,
        }
    }
}

#[derive(Debug,Copy,Clone)]
pub struct FieldDefinition {
    pub name: &'static str,
    pub data_type: FieldType,
}

#[derive(Debug,Copy,Clone)]
pub struct BlockDefinition {
    pub start_offset: u32,
    pub fields: &'static[FieldDefinition],
}

impl BlockDefinition {
    #[inline]
    pub const fn field_offset(&self, field_name: &'static str) -> Option<u32> {
        let mut off = self.start_offset;
        let mut f = 0;
        while f < self.fields.len() {
            let field = &self.fields[f];
            const fn is_same(a: &[u8], b: &[u8]) -> bool {
                let mut i = 0;
                while i < a.len() { if a[i] != b[i] { return false; } i += 1; }
                a.len() == b.len()
            }
            if is_same(field.name.as_bytes(), field_name.as_bytes()) { return Some(off); }
            off += field.data_type.size() as u32;
            f += 1;
        }
        return None;
    }

    #[inline]
    pub const fn size(&self) -> u32 {
        let mut size = self.start_offset;
        let mut f = 0;
        while f < self.fields.len() {
            size += self.fields[f].data_type.size();
            f += 1;
        }
        size
    }
}

macro_rules! mlv_all_block_def {
    ($($block_name: ident {
        $($name:ident : $ty:tt),* $(,)?
    })*) => {
        $(pub const $block_name: BlockDefinition = mlv_block_def! {
            $($name : $ty),*
        };)*

        pub const MLV_BLOCKS: &[(&str, &BlockDefinition)] = &[
            $((stringify!($block_name), &$block_name),)*
        ];
    };
}

macro_rules! mlv_block_def {
    ($($name:ident : $ty:tt),* $(,)?) => {
        BlockDefinition {
            start_offset: 16,
            fields: &[
                $(field!(stringify!($name), $ty),)*
            ]
        }
    };
}

macro_rules! field {
    ($name:expr, [$elem_ty:tt; $len:expr]) => {
        FieldDefinition {
            name: $name,
            data_type: FieldType::Array(type_to_fdt!($elem_ty), $len),
        }
    };
    ($name:expr, $ty:tt) => {
        FieldDefinition {
            name: $name,
            data_type: FieldType::Single(type_to_fdt!($ty)),
        }
    };
}

macro_rules! type_to_fdt {
    (u8) => { PrimitiveType::U8 };
    (u16) => { PrimitiveType::U16 };
    (u32) => { PrimitiveType::U32 };
    (u64) => { PrimitiveType::U64 };
    (i8) => { PrimitiveType::I8 };
    (i16) => { PrimitiveType::I16 };
    (i32) => { PrimitiveType::I32 };
    (i64) => { PrimitiveType::I64 };
    (_) => { compile_error!("Unknown type") };
}

macro_rules! impl_get_field {
    ($fun_name:ident, $fun_ty:tt, $ty_size:expr) => {
        pub fn $fun_name(data: &[u8], offset: u32) -> Option<$fun_ty> {
            let mut buf = [0u8; $ty_size];
            buf.copy_from_slice(data.get(offset as usize..(offset + $ty_size) as usize)?);
            Some($fun_ty::from_le_bytes(buf))
        }
    };
}

impl_get_field!(get_u8, u8, 1);
impl_get_field!(get_u16, u16, 2);
impl_get_field!(get_u32, u32, 4);
impl_get_field!(get_u64, u64, 8);

impl_get_field!(get_i8, i8, 1);
impl_get_field!(get_i16, i16, 2);
impl_get_field!(get_i32, i32, 4);
impl_get_field!(get_i64, i64, 8);

/* Block functions for working directly with blocks as bytes */
pub fn block_get_type<'a>(data: &'a [u8]) -> Option<&'a str> {
    str::from_utf8(&data[0..4]).ok()
}

pub fn block_get_timestamp(data: &[u8]) -> Option<u64> {
    if block_get_type(data) == Some("MLVI") {
        Some(0)
    } else {
        Some(u64::from_le_bytes(*data[8..16].first_chunk()?))
    }
}

pub fn block_get_size(data: &[u8]) -> Option<u32> {
    Some(u32::from_le_bytes(*data[4..8].first_chunk()?))
}

/*********************************************************************************
                                 BLOCK TYPE DEFINITIONS
*********************************************************************************/

mlv_all_block_def! {
    MLVI {
        // fileMagic: [u8; 4],         /* Magic Lantern Video file header "MLVI" */
        // blockSize: u32,           /* size of the whole header */
        // versionString: [u8; 8],     /* null-terminated C-string of the exact revision of this format */
        fileGuid: u64,            /* UID of the file (group) generated using hw counter, time of day and PRNG */
        fileNum: u16,             /* the ID within fileCount this file has (0 to fileCount-1) */
        fileCount: u16,           /* how many files belong to this group (splitting or parallel) */
        fileFlags: u32,           /* 1=out-of-order data, 2=dropped frames, 4=single image mode, 8=stopped due to error */
        videoClass: u16,          /* 0=none, 1=RAW, 2=YUV, 3=JPEG, 4=H.264 */
        audioClass: u16,          /* 0=none, 1=WAV */
        videoFrameCount: u32,     /* number of video frames in this file. set to 0 on start, updated when finished. */
        audioFrameCount: u32,     /* number of audio frames in this file. set to 0 on start, updated when finished. */
        sourceFpsNom: u32,        /* configured fps in 1/s multiplied by sourceFpsDenom */
        sourceFpsDenom: u32,      /* denominator for fps. usually set to 1000, but may be 1001 for NTSC */
    }

    VIDF {
        frameNumber: u32,         /* unique video frame number */
        cropPosX: u16,            /* specifies from which sensor row/col the video frame was copied (8x2 blocks) */
        cropPosY: u16,            /* (can be used to process dead/hot pixels) */
        panPosX: u16,             /* specifies the panning offset which is cropPos, but with higher resolution (1x1 blocks) */
        panPosY: u16,             /* (it's the frame area from sensor the user wants to see) */
        frameSpace: u32,          /* size of dummy data before frameData starts, necessary for EDMAC alignment */
    }

    AUDF {
        frameNumber: u32,         /* unique audio frame number */
        frameSpace: u32           /* size of dummy data before frameData starts, necessary for EDMAC alignment */
        /* uint8_t     frameData[variable]; */
    }

    RAWI {
        xRes: u16,                /* Configured video resolution, may differ from payload resolution */
        yRes: u16,                /* Configured video resolution, may differ from payload resolution */
        // raw_info: RawInfo,          /* the raw_info structure delivered by raw.h of ML Core */

        api_version: u32,
        do_not_use_this: u32,

        height: i32,
        width: i32,
        pitch: i32,
        frame_size: i32,
        bits_per_pixel: i32,              // 14

        black_level: i32,                 // autodetected
        white_level: i32,                 // somewhere around 13000 - 16000, varies with camera, settings etc
                                            // would be best to autodetect it, but we can't do this reliably yet

        // "DNG JPEG info"
        jpeg_x: i32, jpeg_y: i32,
        jpeg_width: i32, jpeg_height: i32,

        // DNG active sensor area (Y1, X1, Y2, X2)
        dng_active_area: [i32; 4],

        exposure_bias: [i32; 2],        // DNG Exposure Bias (idk what's that)
        cfa_pattern: i32,               // stick to 0x02010100 (RGBG) if you can
        calibration_illuminant1: i32,
        color_matrix1: [i32; 18],       // DNG Color Matrix
        dynamic_range: i32              // EV x100, from analyzing black level and noise (very close to DxO)
    }

    WAVI {
        format: u16,            /* 1=Integer PCM, 6=alaw, 7=mulaw */
        channels: u16,          /* audio channel count: 1=mono, 2=stereo */
        samplingRate: u32,      /* audio sampling rate in 1/s */
        bytesPerSecond: u32,    /* audio data rate */
        blockAlign: u16,        /* see RIFF WAV hdr description */
        bitsPerSample: u16      /* audio ADC resolution */
    }

    EXPO {
        isoMode: u32,            /* 0=manual, 1=auto */
        isoValue: u32,           /* camera delivered ISO value */
        isoAnalog: u32,          /* ISO obtained by hardware amplification (most full-stop ISOs, except extreme values) */
        digitalGain: u32,        /* digital ISO gain (1024 = 1 EV) - it's not baked in the raw data, so you may want to scale it or adjust the white level */
        shutterValue: u64,       /* exposure time in microseconds */
    }

    RAWC {
        blockType: [u8; 4],         /* RAWC - raw image capture information */
        blockSize: u32,           /* sizeof(mlv_rawc_hdr_t) */
        timestamp: u64,           /* hardware counter timestamp */

        /* see struct raw_capture_info from raw.h */

        /* sensor attributes: resolution, crop factor */
        sensor_res_x: u16,        /* sensor resolution */
        sensor_res_y: u16,        /* 2-3 GPixel cameras anytime soon? (to overflow this) */
        sensor_crop: u16,         /* sensor crop factor x100 */
        reserved: u16,            /* reserved for future use */

        /* video mode attributes */
        /* (how the sensor is configured for image capture) */
        /* subsampling factor: (binning_x+skipping_x) x (binning_y+skipping_y) */
        binning_x: u8,              /* 3 (1080p and 720p); 1 (crop, zoom) */
        skipping_x: u8,             /* so far, 0 everywhere */
        binning_y: u8,              /* 1 (most cameras in 1080/720p; also all crop modes); 3 (5D3 1080p); 5 (5D3 720p) */
        skipping_y: u8,             /* 2 (most cameras in 1080p); 4 (most cameras in 720p); 0 (5D3) */
        offset_x: i16,            /* crop offset (top-left active pixel) - optional (SHRT_MIN if unknown) */
        offset_y: i16,            /* relative to top-left active pixel from a full-res image (FRSP or CR2) */

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
}

pub const MLV_VERSION_STRING: &str = "2.0";
