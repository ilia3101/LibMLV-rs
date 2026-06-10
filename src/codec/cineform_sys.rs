#![allow(unused, non_camel_case_types, non_upper_case_globals)]
use std::ffi::c_void;
use std::os::raw::{c_int, c_ushort};

// ---- Type aliases matching CineForm opaque pointer types ----

/// Opaque handle to an encoder instance.
pub type CFHD_EncoderRef = *mut c_void;
/// Opaque handle to a decoder instance.
pub type CFHD_DecoderRef = *mut c_void;
/// Opaque handle to a metadata interface.
pub type CFHD_MetadataRef = *mut c_void;
/// Opaque handle to an encoder pool.
pub type CFHD_EncoderPoolRef = *mut c_void;
/// Opaque handle to an encoded sample buffer.
pub type CFHD_SampleBufferRef = *mut c_void;
/// Opaque handle to a sample decoder (C++ ISampleDecoder*).
pub type CFHD_SampleDecoderRef = *mut c_void;

// ---- Scalar type aliases ----

pub type CFHD_Error = c_int;
pub type CFHD_EncodingQuality = c_int;
pub type CFHD_EncodedFormat = c_int;
pub type CFHD_EncodingFlags = u32;
pub type CFHD_DecodedResolution = c_int;
pub type CFHD_DecodingFlags = u32;
pub type CFHD_MetadataType = c_int;
pub type CFHD_MetadataTag = u32;
pub type CFHD_MetadataSize = i32;
pub type CFHD_SampleInfoTag = c_int;
pub type CFHD_FieldType = c_int;
pub type CFHD_BayerFormat = c_int;
pub type CFHD_VideoSelect = c_int;
pub type CFHD_Stereo3DType = c_int;
pub type CFHD_StereoFlags = c_int;
pub type CFHD_MetadataTrack = c_int;

// ---- Pixel formats ----

#[allow(non_camel_case_types)]
#[repr(i32)]
pub enum CFHD_PixelFormat {
    CFHD_PIXEL_FORMAT_UNKNOWN = 0,
    // Encoder & Decoder formats
    CFHD_PIXEL_FORMAT_BGRA = 0x42475241, // 'BGRA' RGBA 8-bit 4:4:4:4 inverted
    CFHD_PIXEL_FORMAT_BGRa = 0x42475261, // 'BGRa' RGBA 8-bit 4:4:4:4
    CFHD_PIXEL_FORMAT_RG24 = 0x52473234, // 'RG24' RGB 8-bit 4:4:4 inverted
    CFHD_PIXEL_FORMAT_2VUY = 0x32767579, // '2vuy' Y'CbCr 8-bit 4:2:2
    CFHD_PIXEL_FORMAT_YUY2 = 0x59555932, // 'YUY2' Y'CbCr 8-bit 4:2:2
    CFHD_PIXEL_FORMAT_B64A = 0x62363461, // 'b64a' ARGB 16-bits per component
    CFHD_PIXEL_FORMAT_RG48 = 0x52473438, // 'RG48' 16-bit RGB
    CFHD_PIXEL_FORMAT_YU64 = 0x59553634, // 'YU64'
    CFHD_PIXEL_FORMAT_V210 = 0x76323130, // 'v210'
    CFHD_PIXEL_FORMAT_RG30 = 0x52473330, // 'RG30'
    CFHD_PIXEL_FORMAT_AB10 = 0x41423130, // 'AB10' A2B10G10R10
    CFHD_PIXEL_FORMAT_AR10 = 0x41523130, // 'AR10' A2R10G10B10
    CFHD_PIXEL_FORMAT_R210 = 0x72323130, // 'r210' DPX packed
    CFHD_PIXEL_FORMAT_DPX0 = 0x44505830, // 'DPX0' DPX packed
    CFHD_PIXEL_FORMAT_NV12 = 0x4e563132, // 'NV12' Planar YUV 4:2:0
    CFHD_PIXEL_FORMAT_YV12 = 0x59563132, // 'YV12' Planar YUV 4:2:0
    CFHD_PIXEL_FORMAT_R408 = 0x52343038, // 'R408' Y'CbCrA 8-bit 4:4:4:4
    CFHD_PIXEL_FORMAT_V408 = 0x56343038, // 'V408' Y'CbCrA 8-bit 4:4:4:4
    CFHD_PIXEL_FORMAT_BYR4 = 0x42595234, // 'BYR4' Raw Bayer 16-bit
    // Decoder only
    CFHD_PIXEL_FORMAT_BYR2 = 0x42595232, // 'BYR2' Raw Bayer pixel data
    CFHD_PIXEL_FORMAT_WP13 = 0x57503133, // 'WP13' signed 16-bit RGB
    CFHD_PIXEL_FORMAT_W13A = 0x57313341, // 'W13A' signed 16-bit RGBA
    CFHD_PIXEL_FORMAT_YUYV = 0x79757976, // 'yuyv' YUYV 8-bit 4:2:2
    // Encoder only
    CFHD_PIXEL_FORMAT_BYR5 = 0x42595235, // 'BYR5' Raw Bayer 12-bit packed
    CFHD_PIXEL_FORMAT_B48R = 0x62343872, // 'b48r' RGB 16-bit
    CFHD_PIXEL_FORMAT_RG64 = 0x52473634, // 'RG64' 16-bit RGBA
    // Avid pixel formats
    CFHD_PIXEL_FORMAT_CT_UCHAR = 0x61767538,       // 'avu8' Avid 8-bit CbYCrY
    CFHD_PIXEL_FORMAT_CT_10BIT_2_8 = 0x61763238,   // 'av28'
    CFHD_PIXEL_FORMAT_CT_SHORT_2_14 = 0x61323134,  // 'a214'
    CFHD_PIXEL_FORMAT_CT_USHORT_10_6 = 0x61313036, // 'a106'
    CFHD_PIXEL_FORMAT_CT_SHORT = 0x61763136,       // 'av16'
    CFHD_PIXEL_FORMAT_UNC_ARGB_444 = 0x61723130,   // 'ar10'
}

// ---- Error codes ----

pub const CFHD_ERROR_OKAY: CFHD_Error = 0;
pub const CFHD_ERROR_INVALID_ARGUMENT: CFHD_Error = 1;
pub const CFHD_ERROR_NOT_INITIALIZED: CFHD_Error = 2;
pub const CFHD_ERROR_BADFORMAT: CFHD_Error = 3;
pub const CFHD_ERROR_UNEXPECTED: CFHD_Error = 4;
pub const CFHD_ERROR_LICENSING: CFHD_Error = 5;
pub const CFHD_ERROR_BAD_LICENSE: CFHD_Error = 6;
pub const CFHD_ERROR_OUTOFMEMORY: CFHD_Error = 7;
pub const CFHD_ERROR_NOT_OPEN: CFHD_Error = 8;
pub const CFHD_ERROR_ALREADY_OPEN: CFHD_Error = 9;
pub const CFHD_ERROR_BAD_RESOLUTION: CFHD_Error = 10;
pub const CFHD_ERROR_TIMEOUT: CFHD_Error = 11;
pub const CFHD_ERROR_STILL_ENCODING: CFHD_Error = 12;
pub const CFHD_ERROR_METADATA_TOO_LARGE: CFHD_Error = 13;
pub const CFHD_ERROR_UNEXPECTED_EOF: CFHD_Error = 14;
pub const CFHD_ERROR_END_OF_TAG: CFHD_Error = 15;

// ---- Encoding quality ----

pub const CFHD_ENCODING_QUALITY_FIXED: CFHD_EncodingQuality = 0;
pub const CFHD_ENCODING_QUALITY_LOW: CFHD_EncodingQuality = 1;
pub const CFHD_ENCODING_QUALITY_MEDIUM: CFHD_EncodingQuality = 2;
pub const CFHD_ENCODING_QUALITY_HIGH: CFHD_EncodingQuality = 3;
pub const CFHD_ENCODING_QUALITY_FILMSCAN1: CFHD_EncodingQuality = 4;
pub const CFHD_ENCODING_QUALITY_FILMSCAN2: CFHD_EncodingQuality = 5;
pub const CFHD_ENCODING_QUALITY_FILMSCAN3: CFHD_EncodingQuality = 6;
pub const CFHD_ENCODING_QUALITY_KEYING: CFHD_EncodingQuality = 5 | (0x04000000u32 as c_int);
pub const CFHD_ENCODING_QUALITY_ONE_EIGHTH_UNCOMPRESSED: CFHD_EncodingQuality = 1 << 8;
pub const CFHD_ENCODING_QUALITY_QUARTER_UNCOMPRESSED: CFHD_EncodingQuality = 2 << 8;
pub const CFHD_ENCODING_QUALITY_THREE_EIGHTH_UNCOMPRESSED: CFHD_EncodingQuality = 3 << 8;
pub const CFHD_ENCODING_QUALITY_HALF_UNCOMPRESSED: CFHD_EncodingQuality = 4 << 8;
pub const CFHD_ENCODING_QUALITY_FIVE_EIGHTH_UNCOMPRESSED: CFHD_EncodingQuality = 5 << 8;
pub const CFHD_ENCODING_QUALITY_THREE_QUARTER_UNCOMPRESSED: CFHD_EncodingQuality = 6 << 8;
pub const CFHD_ENCODING_QUALITY_SEVEN_EIGHTH_UNCOMPRESSED: CFHD_EncodingQuality = 7 << 8;
pub const CFHD_ENCODING_QUALITY_UNCOMPRESSED: CFHD_EncodingQuality = 16 << 8;
pub const CFHD_ENCODING_QUALITY_UNC_NO_STORE: CFHD_EncodingQuality = (32 | 16) << 8;
pub const CFHD_ENCODING_QUALITY_DEFAULT: CFHD_EncodingQuality = 4;

// ---- Encoded format ----

pub const CFHD_ENCODED_FORMAT_YUV_422: CFHD_EncodedFormat = 0;
pub const CFHD_ENCODED_FORMAT_RGB_444: CFHD_EncodedFormat = 1;
pub const CFHD_ENCODED_FORMAT_RGBA_4444: CFHD_EncodedFormat = 2;
pub const CFHD_ENCODED_FORMAT_BAYER: CFHD_EncodedFormat = 3;
pub const CFHD_ENCODED_FORMAT_YUVA_4444: CFHD_EncodedFormat = 4;
pub const CFHD_ENCODED_FORMAT_UNKNOWN: CFHD_EncodedFormat = 5;

// ---- Encoding flags ----

pub const CFHD_ENCODING_FLAGS_NONE: CFHD_EncodingFlags = 0;
pub const CFHD_ENCODING_FLAGS_YUV_INTERLACED: CFHD_EncodingFlags = 1 << 0;
pub const CFHD_ENCODING_FLAGS_YUV_2FRAME_GOP: CFHD_EncodingFlags = 1 << 1;
pub const CFHD_ENCODING_FLAGS_YUV_601: CFHD_EncodingFlags = 1 << 2;
pub const CFHD_ENCODING_FLAGS_CURVE_APPLIED: CFHD_EncodingFlags = 1 << 4;
pub const CFHD_ENCODING_FLAGS_CURVE_GAMMA22: CFHD_EncodingFlags = 0;
pub const CFHD_ENCODING_FLAGS_CURVE_LOG90: CFHD_EncodingFlags = 1 << 5;
pub const CFHD_ENCODING_FLAGS_CURVE_LINEAR: CFHD_EncodingFlags = 1 << 6;
pub const CFHD_ENCODING_FLAGS_CURVE_CUSTOM: CFHD_EncodingFlags = 1 << 7;
pub const CFHD_ENCODING_FLAGS_RGB_STUDIO: CFHD_EncodingFlags = 1 << 8;
pub const CFHD_ENCODING_FLAGS_APPEND_THUMBNAIL: CFHD_EncodingFlags = 1 << 9;
pub const CFHD_ENCODING_FLAGS_WATERMARK_THUMBNAIL: CFHD_EncodingFlags = 1 << 10;
pub const CFHD_ENCODING_FLAGS_LARGER_OUTPUT: CFHD_EncodingFlags = 1 << 11;

// ---- Decoded resolution ----

pub const CFHD_DECODED_RESOLUTION_UNKNOWN: CFHD_DecodedResolution = 0;
pub const CFHD_DECODED_RESOLUTION_FULL: CFHD_DecodedResolution = 1;
pub const CFHD_DECODED_RESOLUTION_HALF: CFHD_DecodedResolution = 2;
pub const CFHD_DECODED_RESOLUTION_QUARTER: CFHD_DecodedResolution = 3;
pub const CFHD_DECODED_RESOLUTION_THUMBNAIL: CFHD_DecodedResolution = 4;

// ---- Decoding flags ----

pub const CFHD_DECODING_FLAGS_NONE: CFHD_DecodingFlags = 0;
pub const CFHD_DECODING_FLAGS_IGNORE_OUTPUT: CFHD_DecodingFlags = 1 << 0;
pub const CFHD_DECODING_FLAGS_MUST_SCALE: CFHD_DecodingFlags = 1 << 1;
pub const CFHD_DECODING_FLAGS_USE_RESOLUTION: CFHD_DecodingFlags = 1 << 2;
pub const CFHD_DECODING_FLAGS_INTERNAL_ONLY: CFHD_DecodingFlags = 1 << 3;

// ---- Sample info tags ----

pub const CFHD_SAMPLE_INFO_CHANNELS: CFHD_SampleInfoTag = 0;
pub const CFHD_SAMPLE_DISPLAY_WIDTH: CFHD_SampleInfoTag = 1;
pub const CFHD_SAMPLE_DISPLAY_HEIGHT: CFHD_SampleInfoTag = 2;
pub const CFHD_SAMPLE_KEY_FRAME: CFHD_SampleInfoTag = 3;
pub const CFHD_SAMPLE_PROGRESSIVE: CFHD_SampleInfoTag = 4;
pub const CFHD_SAMPLE_ENCODED_FORMAT: CFHD_SampleInfoTag = 5;
pub const CFHD_SAMPLE_SDK_VERSION: CFHD_SampleInfoTag = 6;
pub const CFHD_SAMPLE_ENCODE_VERSION: CFHD_SampleInfoTag = 7;

// ---- Metadata types ----

pub const METADATATYPE_UNKNOWN: CFHD_MetadataType = 0;
pub const METADATATYPE_STRING: CFHD_MetadataType = 1;
pub const METADATATYPE_UINT32: CFHD_MetadataType = 2;
pub const METADATATYPE_UINT16: CFHD_MetadataType = 3;
pub const METADATATYPE_UINT8: CFHD_MetadataType = 4;
pub const METADATATYPE_FLOAT: CFHD_MetadataType = 5;
pub const METADATATYPE_DOUBLE: CFHD_MetadataType = 6;
pub const METADATATYPE_GUID: CFHD_MetadataType = 7;
pub const METADATATYPE_XML: CFHD_MetadataType = 8;
pub const METADATATYPE_LONG_HEX: CFHD_MetadataType = 9;
pub const METADATATYPE_CINEFORM: CFHD_MetadataType = 10;
pub const METADATATYPE_HIDDEN: CFHD_MetadataType = 11;
pub const METADATATYPE_TAG: CFHD_MetadataType = 12;

// ---- Metadata track ----

pub const METADATATYPE_ORIGINAL: CFHD_MetadataTrack = 0;
pub const METADATATYPE_ORIGINAL_FILTERED: CFHD_MetadataTrack = 1;
pub const METADATATYPE_MODIFIED: CFHD_MetadataTrack = 2;
pub const METADATATYPE_MODIFIED_FILTERED: CFHD_MetadataTrack = 3;
pub const METADATATYPE_MODIFIED_RIGHT: CFHD_MetadataTrack = 6;
pub const METADATATYPE_MODIFIED_RIGHT_FILTERED: CFHD_MetadataTrack = 7;
pub const METADATATYPE_MODIFIED_LEFT: CFHD_MetadataTrack = 10;
pub const METADATATYPE_MODIFIED_LEFT_FILTERED: CFHD_MetadataTrack = 11;

// ---- Field type ----

pub const CFHD_FIELD_TYPE_UNKNOWN: CFHD_FieldType = 0;
pub const CFHD_FIELD_TYPE_PROGRESSIVE: CFHD_FieldType = 1;
pub const CFHD_FIELD_TYPE_UPPER_FIELD_FIRST: CFHD_FieldType = 2;
pub const CFHD_FIELD_TYPE_LOWER_FIELD_FIRST: CFHD_FieldType = 3;

// ---- Bayer format ----

pub const CFHD_BAYER_FORMAT_UNKNOWN: CFHD_BayerFormat = -1;
pub const CFHD_BAYER_FORMAT_RED_GRN: CFHD_BayerFormat = 0;
pub const CFHD_BAYER_FORMAT_GRN_RED: CFHD_BayerFormat = 1;
pub const CFHD_BAYER_FORMAT_GRN_BLU: CFHD_BayerFormat = 2;
pub const CFHD_BAYER_FORMAT_BLU_GRN: CFHD_BayerFormat = 3;

// ---- Demosaic / debayer type ----

pub const DEMOSAIC_USER_DEFAULT: c_int = 0;
pub const DEMOSAIC_BILINEAR: c_int = 1;
pub const DEMOSAIC_MATRIX5x5: c_int = 2;
pub const DEMOSAIC_ADVANCED_SMOOTH: c_int = 3;
pub const DEMOSAIC_ADVANCED_DETAIL1: c_int = 4;
pub const DEMOSAIC_ADVANCED_DETAIL2: c_int = 5;
pub const DEMOSAIC_ADVANCED_DETAIL3: c_int = 6;

// ---- Video select ----

pub const VIDEO_SELECT_DEFAULT: CFHD_VideoSelect = 0;
pub const VIDEO_SELECT_LEFT_EYE: CFHD_VideoSelect = 1;
pub const VIDEO_SELECT_RIGHT_EYE: CFHD_VideoSelect = 2;
pub const VIDEO_SELECT_BOTH_EYES: CFHD_VideoSelect = 3;

// ---- Stereo 3D type ----

pub const STEREO3D_TYPE_DEFAULT: CFHD_Stereo3DType = 0;
pub const STEREO3D_TYPE_STACKED: CFHD_Stereo3DType = 1;
pub const STEREO3D_TYPE_SIDEBYSIDE: CFHD_Stereo3DType = 2;
pub const STEREO3D_TYPE_FIELDS: CFHD_Stereo3DType = 3;
pub const STEREO3D_TYPE_ONION: CFHD_Stereo3DType = 4;
pub const STEREO3D_TYPE_DIFFERENCE: CFHD_Stereo3DType = 5;
pub const STEREO3D_TYPE_FREEVIEW: CFHD_Stereo3DType = 7;
pub const STEREO3D_TYPE_ANAGLYPH_RED_CYAN: CFHD_Stereo3DType = 16;
pub const STEREO3D_TYPE_ANAGLYPH_RED_CYAN_BW: CFHD_Stereo3DType = 17;
pub const STEREO3D_TYPE_ANAGLYPH_BLU_YLLW: CFHD_Stereo3DType = 18;
pub const STEREO3D_TYPE_ANAGLYPH_BLU_YLLW_BW: CFHD_Stereo3DType = 19;
pub const STEREO3D_TYPE_ANAGLYPH_GRN_MGTA: CFHD_Stereo3DType = 20;
pub const STEREO3D_TYPE_ANAGLYPH_GRN_MGTA_BW: CFHD_Stereo3DType = 21;
pub const STEREO3D_TYPE_ANAGLYPH_OPTIMIZED: CFHD_Stereo3DType = 22;

// ---- Stereo flags ----

pub const STEREO_FLAGS_DEFAULT: CFHD_StereoFlags = 0;
pub const STEREO_FLAGS_SWAP_EYES: CFHD_StereoFlags = 1;
pub const STEREO_FLAGS_SPEED_3D: CFHD_StereoFlags = 2;

// ---- Metadata flags ----

pub const METADATAFLAG_FILTERED: c_int = 1;
pub const METADATAFLAG_MODIFIED: c_int = 2;
pub const METADATAFLAG_RIGHT_EYE: c_int = 4;
pub const METADATAFLAG_LEFT_EYE: c_int = 8;

// ---- FFI declarations (core encoder / decoder) ----

unsafe extern "C" {
    // Encoder API
    pub fn CFHD_OpenEncoder(encoder_ref_out: *mut CFHD_EncoderRef, allocator: *mut c_void) -> CFHD_Error;
    pub fn CFHD_PrepareToEncode(
        encoder_ref: CFHD_EncoderRef,
        frame_width: c_int,
        frame_height: c_int,
        pixel_format: CFHD_PixelFormat,
        encoded_format: CFHD_EncodedFormat,
        encoding_flags: CFHD_EncodingFlags,
        encoding_quality: CFHD_EncodingQuality,
    ) -> CFHD_Error;
    pub fn CFHD_EncodeSample(encoder_ref: CFHD_EncoderRef, frame_buffer: *mut c_void, frame_pitch: c_int)
    -> CFHD_Error;
    pub fn CFHD_GetSampleData(
        encoder_ref: CFHD_EncoderRef,
        sample_data_out: *mut *mut c_void,
        sample_size_out: *mut usize,
    ) -> CFHD_Error;
    pub fn CFHD_CloseEncoder(encoder_ref: CFHD_EncoderRef) -> CFHD_Error;

    // Decoder API
    pub fn CFHD_OpenDecoder(decoder_ref_out: *mut CFHD_DecoderRef, allocator: *mut c_void) -> CFHD_Error;
    pub fn CFHD_PrepareToDecode(
        decoder_ref: CFHD_DecoderRef,
        output_width: c_int,
        output_height: c_int,
        output_format: CFHD_PixelFormat,
        decoded_resolution: CFHD_DecodedResolution,
        decoding_flags: CFHD_DecodingFlags,
        sample_ptr: *mut c_void,
        sample_size: usize,
        actual_width_out: *mut c_int,
        actual_height_out: *mut c_int,
        actual_format_out: *mut CFHD_PixelFormat,
    ) -> CFHD_Error;
    pub fn CFHD_DecodeSample(
        decoder_ref: CFHD_DecoderRef,
        sample_ptr: *mut c_void,
        sample_size: usize,
        output_buffer: *mut c_void,
        output_pitch: c_int,
    ) -> CFHD_Error;
    pub fn CFHD_CloseDecoder(decoder_ref: CFHD_DecoderRef) -> CFHD_Error;
}

// ---- FFI declarations (extended API) ----

unsafe extern "C" {
    // ── Encoder: queries, licensing, metadata, watermark ──

    pub fn CFHD_GetInputFormats(
        encoder_ref: CFHD_EncoderRef,
        input_format_array: *mut CFHD_PixelFormat,
        input_format_array_length: c_int,
        actual_input_format_count_out: *mut c_int,
    ) -> CFHD_Error;

    pub fn CFHD_SetEncodeLicense(encoder_ref: CFHD_EncoderRef, license_key: *const u8) -> CFHD_Error;

    pub fn CFHD_SetEncodeLicense2(encoder_ref: CFHD_EncoderRef, license_key: *const u8, level: *mut u32) -> CFHD_Error;

    pub fn CFHD_GetEncodeThumbnail(
        encoder_ref: CFHD_EncoderRef,
        sample_ptr: *mut c_void,
        sample_size: usize,
        output_buffer: *mut c_void,
        output_buffer_size: usize,
        flags: u32,
        ret_width: *mut usize,
        ret_height: *mut usize,
        ret_size: *mut usize,
    ) -> CFHD_Error;

    pub fn CFHD_MetadataOpen(metadata_ref_out: *mut CFHD_MetadataRef) -> CFHD_Error;

    pub fn CFHD_MetadataAdd(
        metadata_ref: CFHD_MetadataRef,
        tag: u32,
        metadata_type: CFHD_MetadataType,
        size: usize,
        data: *mut u32,
        temporary: bool,
    ) -> CFHD_Error;

    pub fn CFHD_MetadataAttach(encoder_ref: CFHD_EncoderRef, metadata_ref: CFHD_MetadataRef) -> CFHD_Error;

    pub fn CFHD_MetadataClose(metadata_ref: CFHD_MetadataRef) -> CFHD_Error;

    pub fn CFHD_ApplyWatermark(
        frame_buffer: *mut c_void,
        frame_width: c_int,
        frame_height: c_int,
        frame_pitch: c_int,
        pixel_format: CFHD_PixelFormat,
    );

    // ── Encoder pool (async encoding) ──

    pub fn CFHD_CreateEncoderPool(
        encoder_pool_ref_out: *mut CFHD_EncoderPoolRef,
        encoder_thread_count: c_int,
        job_queue_length: c_int,
        allocator: *mut c_void,
    ) -> CFHD_Error;

    pub fn CFHD_GetAsyncInputFormats(
        encoder_pool_ref: CFHD_EncoderPoolRef,
        input_format_array: *mut CFHD_PixelFormat,
        input_format_array_length: c_int,
        actual_input_format_count_out: *mut c_int,
    ) -> CFHD_Error;

    pub fn CFHD_PrepareEncoderPool(
        encoder_pool_ref: CFHD_EncoderPoolRef,
        frame_width: c_ushort,
        frame_height: c_ushort,
        pixel_format: CFHD_PixelFormat,
        encoded_format: CFHD_EncodedFormat,
        encoding_flags: CFHD_EncodingFlags,
        encoding_quality: CFHD_EncodingQuality,
    ) -> CFHD_Error;

    pub fn CFHD_SetEncoderPoolLicense(encoder_pool_ref: CFHD_EncoderPoolRef, license_key: *const u8) -> CFHD_Error;

    pub fn CFHD_SetEncoderPoolLicense2(
        encoder_pool_ref: CFHD_EncoderPoolRef,
        license_key: *const u8,
        level: *mut u32,
    ) -> CFHD_Error;

    pub fn CFHD_AttachEncoderPoolMetadata(
        encoder_pool_ref: CFHD_EncoderPoolRef,
        metadata_ref: CFHD_MetadataRef,
    ) -> CFHD_Error;

    pub fn CFHD_StartEncoderPool(encoder_pool_ref: CFHD_EncoderPoolRef) -> CFHD_Error;

    pub fn CFHD_StopEncoderPool(encoder_pool_ref: CFHD_EncoderPoolRef) -> CFHD_Error;

    pub fn CFHD_EncodeAsyncSample(
        encoder_pool_ref: CFHD_EncoderPoolRef,
        frame_number: u32,
        frame_buffer: *mut c_void,
        frame_pitch: isize,
        metadata_ref: CFHD_MetadataRef,
    ) -> CFHD_Error;

    pub fn CFHD_WaitForSample(
        encoder_pool_ref: CFHD_EncoderPoolRef,
        frame_number_out: *mut u32,
        sample_buffer_ref_out: *mut CFHD_SampleBufferRef,
    ) -> CFHD_Error;

    pub fn CFHD_TestForSample(
        encoder_pool_ref: CFHD_EncoderPoolRef,
        frame_number_out: *mut u32,
        sample_buffer_ref_out: *mut CFHD_SampleBufferRef,
    ) -> CFHD_Error;

    pub fn CFHD_GetEncodedSample(
        sample_buffer_ref: CFHD_SampleBufferRef,
        sample_data_out: *mut *mut c_void,
        sample_size_out: *mut usize,
    ) -> CFHD_Error;

    pub fn CFHD_GetSampleThumbnail(
        sample_buffer_ref: CFHD_SampleBufferRef,
        thumbnail_buffer: *mut c_void,
        buffer_size: usize,
        flags: u32,
        actual_width_out: *mut c_ushort,
        actual_height_out: *mut c_ushort,
        pixel_format_out: *mut CFHD_PixelFormat,
        actual_size_out: *mut usize,
    ) -> CFHD_Error;

    pub fn CFHD_ReleaseSampleBuffer(
        encoder_pool_ref: CFHD_EncoderPoolRef,
        sample_buffer_ref: CFHD_SampleBufferRef,
    ) -> CFHD_Error;

    pub fn CFHD_ReleaseEncoderPool(encoder_pool_ref: CFHD_EncoderPoolRef) -> CFHD_Error;

    // ── Decoder: queries, licensing, metadata, utilities ──

    pub fn CFHD_GetOutputFormats(
        decoder_ref: CFHD_DecoderRef,
        sample_ptr: *mut c_void,
        sample_size: usize,
        output_format_array: *mut CFHD_PixelFormat,
        output_format_array_length: c_int,
        actual_output_format_count_out: *mut c_int,
    ) -> CFHD_Error;

    pub fn CFHD_GetSampleInfo(
        decoder_ref: CFHD_DecoderRef,
        sample_ptr: *mut c_void,
        sample_size: usize,
        tag: CFHD_SampleInfoTag,
        value: *mut c_void,
        buffer_size: usize,
    ) -> CFHD_Error;

    pub fn CFHD_GetPixelSize(pixel_format: CFHD_PixelFormat, pixel_size_out: *mut u32) -> CFHD_Error;

    pub fn CFHD_GetImagePitch(
        image_width: u32,
        pixel_format: CFHD_PixelFormat,
        image_pitch_out: *mut i32,
    ) -> CFHD_Error;

    pub fn CFHD_GetImageSize(
        image_width: u32,
        image_height: u32,
        pixel_format: CFHD_PixelFormat,
        videoselect: CFHD_VideoSelect,
        stereotype: CFHD_Stereo3DType,
        image_size_out: *mut u32,
    ) -> CFHD_Error;

    pub fn CFHD_SetLicense(decoder_ref: CFHD_DecoderRef, license_key: *const u8) -> CFHD_Error;

    pub fn CFHD_SetActiveMetadata(
        decoder_ref: CFHD_DecoderRef,
        metadata_ref: CFHD_MetadataRef,
        tag: u32,
        metadata_type: CFHD_MetadataType,
        data: *mut c_void,
        size: u32,
    ) -> CFHD_Error;

    pub fn CFHD_GetThumbnail(
        decoder_ref: CFHD_DecoderRef,
        sample_ptr: *mut c_void,
        sample_size: usize,
        output_buffer: *mut c_void,
        output_buffer_size: usize,
        flags: u32,
        ret_width: *mut usize,
        ret_height: *mut usize,
        ret_size: *mut usize,
    ) -> CFHD_Error;

    pub fn CFHD_ClearActiveMetadata(decoder_ref: CFHD_DecoderRef, metadata_ref: CFHD_MetadataRef) -> CFHD_Error;

    pub fn CFHD_CreateImageDeveloper(
        decoder_ref: CFHD_DecoderRef,
        image_width: u32,
        image_height: u32,
        source_video_channels: u32,
        pixel_format_src: CFHD_PixelFormat,
        pixel_format_dst: CFHD_PixelFormat,
    ) -> CFHD_Error;

    // ── Metadata reading ──

    pub fn CFHD_OpenMetadata(metadata_ref_out: *mut CFHD_MetadataRef) -> CFHD_Error;

    pub fn CFHD_InitSampleMetadata(
        metadata_ref: CFHD_MetadataRef,
        track: CFHD_MetadataTrack,
        sample_data: *mut c_void,
        sample_size: usize,
    ) -> CFHD_Error;

    pub fn CFHD_ReadMetadataFromSample(
        metadata_ref: CFHD_MetadataRef,
        data_out: *mut *mut c_void,
        size_out: *mut usize,
    ) -> CFHD_Error;

    pub fn CFHD_ReadMetadata(
        metadata_ref: CFHD_MetadataRef,
        tag: *mut CFHD_MetadataTag,
        metadata_type: *mut CFHD_MetadataType,
        data: *mut *mut c_void,
        size: *mut CFHD_MetadataSize,
    ) -> CFHD_Error;

    pub fn CFHD_FindMetadata(
        metadata_ref: CFHD_MetadataRef,
        tag: CFHD_MetadataTag,
        metadata_type: *mut CFHD_MetadataType,
        data: *mut *mut c_void,
        size: *mut CFHD_MetadataSize,
    ) -> CFHD_Error;

    pub fn CFHD_CloseMetadata(metadata_ref: CFHD_MetadataRef) -> CFHD_Error;

    // ── Conversion utilities ──

    pub fn ConvertToOutputBuffer(
        input_buffer: *mut c_void,
        input_pitch: c_int,
        input_format: c_int,
        output_buffer: *mut c_void,
        output_pitch: c_int,
        output_format: CFHD_PixelFormat,
        width: c_int,
        height: c_int,
        byte_swap_flag: c_int,
    ) -> CFHD_Error;

    pub fn ScaleToOutputBuffer(
        input_buffer: *mut c_void,
        input_width: c_int,
        input_height: c_int,
        input_pitch: c_int,
        input_format: c_int,
        output_buffer: *mut c_void,
        output_width: c_int,
        output_height: c_int,
        output_pitch: c_int,
        output_format: CFHD_PixelFormat,
        byte_swap_flag: c_int,
    ) -> CFHD_Error;

    // ── Sample decoder factory ──

    pub fn CFHD_CreateSampleDecoder(allocator: *mut c_void, license: *const u8) -> CFHD_SampleDecoderRef;
}
