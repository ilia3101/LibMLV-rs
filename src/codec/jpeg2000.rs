//TODO: make this less sloppy
#![allow(unused)]
use std::ffi::c_void;
use std::os::raw::c_int;

#[allow(non_camel_case_types)]
type c_uint = u32;

unsafe extern "C" {
    fn ojph_encoder_new() -> *mut c_void;
    fn ojph_encoder_set_image(
        e: *mut c_void,
        w: c_uint,
        h: c_uint,
        num_comps: c_uint,
        bit_depth: c_uint,
        is_signed: c_int,
    );
    fn ojph_encoder_set_lossless(e: *mut c_void, lossless: c_int);
    fn ojph_encoder_set_decompositions(e: *mut c_void, n: c_uint);
    fn ojph_encoder_set_quantization(e: *mut c_void, q: f32);
    fn ojph_encoder_encode_into(e: *mut c_void, pixels: *const i32, out_buf: *mut u8, out_buf_size: usize) -> usize;
    fn ojph_encoder_free(e: *mut c_void);

    fn ojph_decoder_new() -> *mut c_void;
    fn ojph_decoder_probe(
        d: *mut c_void,
        data: *const u8,
        size: usize,
        w: *mut c_uint,
        h: *mut c_uint,
        num_comps: *mut c_uint,
        bit_depth: *mut c_uint,
        is_signed: *mut c_int,
    ) -> c_int;
    fn ojph_decoder_decode_into(
        d: *mut c_void,
        data: *const u8,
        size: usize,
        out_pixels: *mut i32,
        out_pixels_cap: usize,
        out_w: *mut c_uint,
        out_h: *mut c_uint,
        out_num_comps: *mut c_uint,
    ) -> usize;
    fn ojph_decoder_free(d: *mut c_void);
}

pub struct Encoder {
    inner: *mut c_void,
}

impl Encoder {
    pub fn new() -> Self {
        let inner = unsafe { ojph_encoder_new() };
        assert!(!inner.is_null(), "failed to create encoder");
        Self { inner }
    }

    pub fn set_image(&self, w: u32, h: u32, num_comps: u32, bit_depth: u32, is_signed: bool) {
        unsafe {
            ojph_encoder_set_image(self.inner, w, h, num_comps, bit_depth, is_signed as c_int);
        }
    }

    pub fn set_lossless(&self, lossless: bool) {
        unsafe { ojph_encoder_set_lossless(self.inner, lossless as c_int) };
    }

    pub fn set_decompositions(&self, n: u32) {
        unsafe { ojph_encoder_set_decompositions(self.inner, n) };
    }

    pub fn set_quantization(&self, q: f32) {
        unsafe { ojph_encoder_set_quantization(self.inner, q) };
    }

    /// Encode into a caller-provided buffer. Returns bytes written, or 0 if too small.
    /// Safe upper bound for buffer size: `w * h * nc * 2`.
    pub fn encode_into(&self, pixels: &[i32], out: &mut [u8]) -> usize {
        unsafe { ojph_encoder_encode_into(self.inner, pixels.as_ptr(), out.as_mut_ptr(), out.len()) }
    }
}

impl Drop for Encoder {
    fn drop(&mut self) {
        unsafe { ojph_encoder_free(self.inner) };
    }
}

pub struct Decoder {
    inner: *mut c_void,
}

impl Decoder {
    pub fn new() -> Self {
        let inner = unsafe { ojph_decoder_new() };
        assert!(!inner.is_null(), "failed to create decoder");
        Self { inner }
    }

    pub fn probe(&self, data: &[u8]) -> Result<(u32, u32, u32, u32, bool), i32> {
        unsafe {
            let mut w: c_uint = 0;
            let mut h: c_uint = 0;
            let mut nc: c_uint = 0;
            let mut bd: c_uint = 0;
            let mut signed: c_int = 0;
            let r = ojph_decoder_probe(
                self.inner,
                data.as_ptr(),
                data.len(),
                &mut w,
                &mut h,
                &mut nc,
                &mut bd,
                &mut signed,
            );
            if r != 0 {
                return Err(r);
            }
            Ok((w, h, nc, bd, signed != 0))
        }
    }

    /// Decode into a caller-provided pixel buffer. Returns (width, height, num_components)
    /// if successful, or None if the buffer was too small.
    /// Pre-allocate with `probe()`: `out.len() >= w * h * nc`.
    pub fn decode_into(&self, data: &[u8], out: &mut [i32]) -> Option<(u32, u32, u32)> {
        unsafe {
            let mut w: c_uint = 0;
            let mut h: c_uint = 0;
            let mut nc: c_uint = 0;
            let n = ojph_decoder_decode_into(
                self.inner,
                data.as_ptr(),
                data.len(),
                out.as_mut_ptr(),
                out.len(),
                &mut w,
                &mut h,
                &mut nc,
            );
            if n == 0 {
                return None;
            }
            Some((w, h, nc))
        }
    }
}

impl Drop for Decoder {
    fn drop(&mut self) {
        unsafe { ojph_decoder_free(self.inner) };
    }
}

unsafe impl Send for Encoder {}
unsafe impl Send for Decoder {}

/*
 * Bayer format:
 * version: u32 // 1
 * offset_a: u32
 * size_a: u32
 * offset_b: u32,
 * size_b: u32,
 * offset_c: u32,
 * size_c: u32,
 * offset_d: u32
 * size_d: u32
 */

pub struct BayerEncoder {
    encoders: Vec<Encoder>,
    buffers: Vec<Vec<i32>>,
    encoded_buffers: Vec<Vec<u8>>,
}

impl BayerEncoder {
    pub fn new() -> Self {
        Self {
            encoders: vec![Encoder::new(), Encoder::new(), Encoder::new(), Encoder::new()],
            buffers: vec![vec![], vec![], vec![], vec![]],
            encoded_buffers: vec![vec![], vec![], vec![], vec![]],
        }
    }

    pub fn encode_bayer(
        &mut self,
        width: u32,
        height: u32,
        bits: u32,
        data: &[u16],
        out: &mut [u8],
        quant: f32, // recomeneded 0.008 for medium, 0.0015 for visually lossless
    ) -> usize {
        for i in 0..4 {
            self.buffers[i].clear();
            self.buffers[i].reserve_exact((width * height) as usize / 4);
            unsafe { self.buffers[i].set_len((width * height) as usize / 4) };
        }
        for i in 0..4 {
            self.encoded_buffers[i].clear();
            self.encoded_buffers[i].reserve_exact((width * height) as usize * 2);
            unsafe { self.encoded_buffers[i].set_len((width * height) as usize * 2) };
        }
        for i in 0..4 {
            self.encoders[i].set_quantization(quant);
            self.encoders[i].set_image(width / 2, height / 2, 1, bits, false);
        }
        fn encode_quarter(
            encoder: &mut Encoder,
            buf: &[u16],
            buf32: &mut [i32],
            width: u32,
            height: u32,
            out: &mut [u8],
            off_x: u32,
            off_y: u32,
        ) -> usize {
            for y in 0..(height / 2) {
                let in_row_off = ((y * 2 + off_y) * width);
                let in_row = &buf[in_row_off as usize..(in_row_off + width) as usize];
                let out_row = &mut buf32[(y * width / 2) as usize..((y + 1) * width / 2) as usize];
                for x in 0..(width / 2) {
                    out_row[x as usize] = in_row[(x * 2 + off_x) as usize] as i32;
                }
            }
            return encoder.encode_into(buf32, out);
        }
        let mut enc_sizes = [0, 0, 0, 0];

        // this loop can be done in parallel
        // for i in 0..4 {
        //     let x_off = [0, 1, 0, 1][i];
        //     let y_off = [0, 0, 1, 1][i];
        //     enc_sizes[i] = encode_quarter(
        //         &mut self.encoders[i],
        //         data,
        //         &mut self.buffers[i],
        //         width,
        //         height,
        //         &mut self.encoded_buffers[i],
        //         x_off,
        //         y_off,
        //     );
        // }

        std::thread::scope(|s| {
            let mut handles = [None, None, None, None];
            for i in 0..4 {
                let x_off = [0u32, 1, 0, 1][i];
                let y_off = [0u32, 0, 1, 1][i];
                let enc = self.encoders.as_ptr().wrapping_add(i) as usize;
                let buf_in = data.as_ptr() as usize;
                let buf_in_len = data.len();
                let buf32 = self.buffers[i].as_mut_ptr() as usize;
                let buf32_len = self.buffers[i].len();
                let out_buf = self.encoded_buffers[i].as_mut_ptr() as usize;
                let out_buf_len = self.encoded_buffers[i].len();
                handles[i] = Some(s.spawn(move || {
                    let enc = unsafe { &mut *(enc as *mut Encoder) };
                    let buf_in = unsafe { std::slice::from_raw_parts(buf_in as *const u16, buf_in_len) };
                    let buf32 = unsafe { std::slice::from_raw_parts_mut(buf32 as *mut i32, buf32_len) };
                    let out_buf = unsafe { std::slice::from_raw_parts_mut(out_buf as *mut u8, out_buf_len) };
                    encode_quarter(enc, buf_in, buf32, width, height, out_buf, x_off, y_off)
                }));
            }
            for (i, h) in handles.into_iter().enumerate() {
                enc_sizes[i] = h.unwrap().join().unwrap();
            }
        });

        let mut out_ptr = out;

        fn add_bytes<'a>(out_ptr: &'a mut [u8], bytes: &[u8]) -> &'a mut [u8] {
            out_ptr[0..bytes.len()].copy_from_slice(bytes);
            &mut out_ptr[bytes.len()..]
        }

        let off_a = 36usize;
        let off_b = off_a + enc_sizes[0];
        let off_c = off_b + enc_sizes[1];
        let off_d = off_c + enc_sizes[2];

        out_ptr = add_bytes(out_ptr, &u32::to_le_bytes(1));
        out_ptr = add_bytes(out_ptr, &u32::to_le_bytes(off_a as u32));
        out_ptr = add_bytes(out_ptr, &u32::to_le_bytes(enc_sizes[0] as u32));
        out_ptr = add_bytes(out_ptr, &u32::to_le_bytes(off_b as u32));
        out_ptr = add_bytes(out_ptr, &u32::to_le_bytes(enc_sizes[1] as u32));
        out_ptr = add_bytes(out_ptr, &u32::to_le_bytes(off_c as u32));
        out_ptr = add_bytes(out_ptr, &u32::to_le_bytes(enc_sizes[2] as u32));
        out_ptr = add_bytes(out_ptr, &u32::to_le_bytes(off_d as u32));
        out_ptr = add_bytes(out_ptr, &u32::to_le_bytes(enc_sizes[3] as u32));

        out_ptr = add_bytes(out_ptr, &self.encoded_buffers[0][0..enc_sizes[0]]);
        out_ptr = add_bytes(out_ptr, &self.encoded_buffers[1][0..enc_sizes[1]]);
        out_ptr = add_bytes(out_ptr, &self.encoded_buffers[2][0..enc_sizes[2]]);
        out_ptr = add_bytes(out_ptr, &self.encoded_buffers[3][0..enc_sizes[3]]);

        return (9 * 4) + enc_sizes[0] + enc_sizes[1] + enc_sizes[2] + enc_sizes[3];
    }
}

pub struct BayerDecoder {
    decoders: Vec<Decoder>,
    buffers: Vec<Vec<i32>>,
}

impl BayerDecoder {
    pub fn new() -> Self {
        Self {
            decoders: vec![Decoder::new(), Decoder::new(), Decoder::new(), Decoder::new()],
            buffers: vec![vec![], vec![], vec![], vec![]],
        }
    }

    pub fn decode_bayer(&mut self, data: &[u8], width: u32, height: u32, out: &mut [u16]) {
        // Parse header: version + 8 u32s (off, size)*4
        let h = |i: usize| u32::from_le_bytes(data[i * 4..(i + 1) * 4].try_into().unwrap());
        assert!(h(0) == 1, "unknown bayer format version");
        let sizes = [h(2) as usize, h(4) as usize, h(6) as usize, h(8) as usize];
        let offsets = [h(1) as usize, h(3) as usize, h(5) as usize, h(7) as usize];

        // for c in 0..4 {
        //     let encoded = &data[offsets[c]..offsets[c] + sizes[c]];
        //     let (dw, dh, _nc, _bd, _signed) =
        //         self.decoders[c].probe(encoded).expect("probe failed");
        //     let pix_count = (dw * dh) as usize;
        //     self.buffers[c].clear();
        //     self.buffers[c].reserve_exact(pix_count);
        //     unsafe { self.buffers[c].set_len(pix_count) };
        //     self.decoders[c]
        //         .decode_into(encoded, &mut self.buffers[c])
        //         .expect("bayer decode failed");
        // }
        let mut pix_counts = [0usize; 4];

        // Pre-allocate buffers so raw pointers have valid capacity
        let est = ((width / 2) * (height / 2)) as usize;
        for c in 0..4 {
            self.buffers[c].clear();
            if self.buffers[c].capacity() < est {
                self.buffers[c].reserve_exact(est);
            }
        }

        std::thread::scope(|s| {
            let mut handles = [None, None, None, None];
            for c in 0..4 {
                let encoded = &data[offsets[c]..offsets[c] + sizes[c]];
                let dec = self.decoders.as_ptr().wrapping_add(c) as usize;
                let buf = self.buffers[c].as_mut_ptr() as usize;
                let buf_cap = self.buffers[c].capacity();
                handles[c] = Some(s.spawn(move || {
                    let dec = unsafe { &mut *(dec as *mut Decoder) };
                    let (dw, dh, _nc, _bd, _signed) = dec.probe(encoded).expect("probe failed");
                    let n = (dw * dh) as usize;
                    assert!(n <= buf_cap, "bayer decode buffer too small");
                    let buf = unsafe { std::slice::from_raw_parts_mut(buf as *mut i32, n) };
                    dec.decode_into(encoded, buf).expect("bayer decode failed");
                    n
                }));
            }
            for (c, h) in handles.into_iter().enumerate() {
                pix_counts[c] = h.unwrap().join().unwrap();
            }
        });
        for c in 0..4 {
            unsafe { self.buffers[c].set_len(pix_counts[c]) };
        }

        // Reconstruct full frame from 4 bayer channels
        // Channel order: [0]=x0y0, [1]=x1y0, [2]=x0y1, [3]=x1y1
        let hw = width / 2;
        let hh = height / 2;
        for y in 0..hh {
            for x in 0..hw {
                let idx = (y * hw + x) as usize;
                out[(y * 2 * width + x * 2) as usize] = self.buffers[0][idx].clamp(0, u16::MAX as i32) as u16;
                out[(y * 2 * width + x * 2 + 1) as usize] = self.buffers[1][idx].clamp(0, u16::MAX as i32) as u16;
                out[((y * 2 + 1) * width + x * 2) as usize] = self.buffers[2][idx].clamp(0, u16::MAX as i32) as u16;
                out[((y * 2 + 1) * width + x * 2 + 1) as usize] = self.buffers[3][idx].clamp(0, u16::MAX as i32) as u16;
            }
        }
    }
}
