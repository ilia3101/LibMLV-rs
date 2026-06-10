//TODO: make this less sloppy
use super::cineform_sys as sys;

use std::ffi::c_void;
use std::ptr;

pub type Error = sys::CFHD_Error;

fn check(err: sys::CFHD_Error) -> Result<(), Error> {
    if err == sys::CFHD_ERROR_OKAY { Ok(()) } else { Err(err) }
}

// ── Encoder ──

pub struct Encoder {
    inner: sys::CFHD_EncoderRef,
    w: u32,
    h: u32,
    pitch: i32,
}

impl Encoder {
    pub fn new(w: u32, h: u32, quality: sys::CFHD_EncodingQuality) -> Result<Self, Error> {
        unsafe {
            let mut inner: sys::CFHD_EncoderRef = ptr::null_mut();
            check(sys::CFHD_OpenEncoder(&mut inner, ptr::null_mut()))?;
            let res = sys::CFHD_PrepareToEncode(
                inner,
                w as _,
                h as _,
                sys::CFHD_PixelFormat::CFHD_PIXEL_FORMAT_BYR4,
                sys::CFHD_ENCODED_FORMAT_BAYER,
                sys::CFHD_ENCODING_FLAGS_NONE,
                quality,
            );
            if res != sys::CFHD_ERROR_OKAY {
                sys::CFHD_CloseEncoder(inner);
                return Err(res);
            }
            Ok(Self { inner, w, h, pitch: (w * 2) as i32 })
        }
    }

    pub fn encode(&self, data: &[u16]) -> Result<Vec<u8>, Error> {
        assert_eq!(data.len(), (self.w * self.h) as usize);
        unsafe {
            check(sys::CFHD_EncodeSample(self.inner, data.as_ptr() as *mut c_void, self.pitch))?;
            let mut cdata: *mut c_void = ptr::null_mut();
            let mut csize: usize = 0;
            check(sys::CFHD_GetSampleData(self.inner, &mut cdata, &mut csize))?;
            let out = std::slice::from_raw_parts(cdata as *const u8, csize).to_vec();
            Ok(out)
        }
    }
}

impl Drop for Encoder {
    fn drop(&mut self) {
        unsafe {
            sys::CFHD_CloseEncoder(self.inner);
        }
    }
}

// ── Decoder ──

pub struct Decoder {
    inner: sys::CFHD_DecoderRef,
}

impl Decoder {
    pub fn new() -> Result<Self, Error> {
        unsafe {
            let mut inner: sys::CFHD_DecoderRef = ptr::null_mut();
            check(sys::CFHD_OpenDecoder(&mut inner, ptr::null_mut()))?;
            Ok(Self { inner })
        }
    }

    /// Decode a compressed sample. Returns (pixels, width, height).
    pub fn decode(&self, sample: &[u8], w: u32, h: u32) -> Result<(Vec<u16>, u32, u32), Error> {
        unsafe {
            let mut out_w: i32 = 0;
            let mut out_h: i32 = 0;
            let mut out_fmt: sys::CFHD_PixelFormat = sys::CFHD_PixelFormat::CFHD_PIXEL_FORMAT_UNKNOWN;
            check(sys::CFHD_PrepareToDecode(
                self.inner,
                w as _,
                h as _,
                sys::CFHD_PixelFormat::CFHD_PIXEL_FORMAT_BYR4,
                sys::CFHD_DECODED_RESOLUTION_FULL,
                sys::CFHD_DECODING_FLAGS_NONE,
                sample.as_ptr() as *mut c_void,
                sample.len(),
                &mut out_w,
                &mut out_h,
                &mut out_fmt,
            ))?;
            let dst_bytes = (out_w as usize) * (out_h as usize) * 2;
            let mut dst: Vec<u16> = vec![0u16; dst_bytes / 2];
            check(sys::CFHD_DecodeSample(
                self.inner,
                sample.as_ptr() as *mut c_void,
                sample.len(),
                dst.as_mut_ptr() as *mut c_void,
                out_w * 2,
            ))?;
            Ok((dst, out_w as u32, out_h as u32))
        }
    }
}

impl Drop for Decoder {
    fn drop(&mut self) {
        unsafe {
            sys::CFHD_CloseDecoder(self.inner);
        }
    }
}
