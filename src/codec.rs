#![cfg_attr(rustfmt, rustfmt_skip)]
use crate::lj92::{Lj92, Lj92Error};

#[inline]
pub fn decode_lj92(data: &[u8], out: &mut [u16]) -> Result<(), Lj92Error> {
    match Lj92::open(data) {
        Ok(mut lj92) => lj92.decode(out, 0, None),
        Err(err) => Err(err),
    }
}

#[inline]
pub fn decode_packed14(data: &[u8], out: &mut [u16]) {
    for (i, o) in data.as_chunks::<14>().0.iter().zip(out.as_chunks_mut::<8>().0.iter_mut()) {
        *o = decode14(*i);
    }
}

#[inline]
pub fn decode_packed12(data: &[u8], out: &mut [u16]) {
    for (i, o) in data.as_chunks::<6>().0.iter().zip(out.as_chunks_mut::<4>().0.iter_mut()) {
        *o = decode12(*i);
    }
}

#[inline]
pub fn decode_packed10(data: &[u8], out: &mut [u16]) {
    for (i, o) in data.as_chunks::<10>().0.iter().zip(out.as_chunks_mut::<8>().0.iter_mut()) {
        *o = decode10(*i);
    }
}

/* TODO: provide direct bitdepth to bitdepth for re-encoding */

/* Internal implementations */

#[inline(always)]
const fn ones(n: u8) -> u16 {
    !(0xFFFF << n)
}

/* Decodes 8 pixels from 14 bytes of 14-bit packed data */
#[inline(always)]
const fn decode14([a, b, c, d, e, f, g, h, i, j, k, l, m, n]: [u8; 14]) -> [u16; 8] {
    let mask = ones(14);
    let word_a: u16 = u16::from_le_bytes([a, b]);
    let word_b: u16 = u16::from_le_bytes([c, d]);
    let word_c: u16 = u16::from_le_bytes([e, f]);
    let word_d: u16 = u16::from_le_bytes([g, h]);
    let word_e: u16 = u16::from_le_bytes([i, j]);
    let word_f: u16 = u16::from_le_bytes([k, l]);
    let word_g: u16 = u16::from_le_bytes([m, n]);
    return [
        word_a >> 2,
        ((word_a << 12) | (word_b >> 4)) & mask,
        ((word_b << 10) | (word_c >> 6)) & mask,
        ((word_c << 8) | (word_d >> 8)) & mask,
        ((word_d << 6) | (word_e >> 10)) & mask,
        ((word_e << 4) | (word_f >> 12)) & mask,
        ((word_f << 2) | (word_g >> 14)) & mask,
        word_g & mask,
    ];
}

/* Decodes 4 pixels from 6 bytes of 12-bit packed data */
#[inline(always)]
const fn decode12([a, b, c, d, e, f]: [u8; 6]) -> [u16; 4] {
    let mask = ones(12);
    let word_a: u16 = u16::from_le_bytes([a, b]);
    let word_b: u16 = u16::from_le_bytes([c, d]);
    let word_c: u16 = u16::from_le_bytes([e, f]);
    return [
        word_a >> 4,
        ((word_a << 8) | (word_b >> 8)) & mask,
        ((word_b << 4) | (word_c >> 12)) & mask,
        word_c & mask,
    ];
}

/* Decodes 8 pixels from 10 bytes of 10-bit packed data */
#[inline(always)]
const fn decode10([a, b, c, d, e, f, g, h, i, j]: [u8; 10]) -> [u16; 8] {
    let mask = ones(10);
    let word_a: u16 = u16::from_le_bytes([a, b]);
    let word_b: u16 = u16::from_le_bytes([c, d]);
    let word_c: u16 = u16::from_le_bytes([e, f]);
    let word_d: u16 = u16::from_le_bytes([g, h]);
    let word_e: u16 = u16::from_le_bytes([i, j]);
    return [
        word_a >> 6,
        ((word_a << 4) | (word_b >> 12)) & mask,
        (word_b >> 2) & mask,
        ((word_b << 8) | (word_c >> 8)) & mask,
        ((word_c << 2) | (word_d >> 14)) & mask,
        ((word_c << 12) | (word_d >> 4)) & mask,
        ((word_d << 6) | (word_e >> 10)) & mask,
        word_e & mask,
    ];
}

#[inline]
pub fn encode_packed14(data: &[u16], out: &mut [u8]) {
    for (i, o) in data.as_chunks::<8>().0.iter().zip(out.as_chunks_mut::<14>().0.iter_mut()) {
        *o = encode14(*i);
    }
}

#[inline]
pub fn encode_packed12(data: &[u16], out: &mut [u8]) {
    for (i, o) in data.as_chunks::<4>().0.iter().zip(out.as_chunks_mut::<6>().0.iter_mut()) {
        *o = encode12(*i);
    }
}

#[inline]
pub fn encode_packed10(data: &[u16], out: &mut [u8]) {
    for (i, o) in data.as_chunks::<8>().0.iter().zip(out.as_chunks_mut::<10>().0.iter_mut()) {
        *o = encode10(*i);
    }
}

/* Encodes 8 pixels into 14 bytes of 14-bit packed data */
#[inline(always)]
const fn encode14([p0, p1, p2, p3, p4, p5, p6, p7]: [u16; 8]) -> [u8; 14] {
    let mask = ones(14);
    let word_a: u16 = ((p0 & mask) << 2) | ((p1 & mask) >> 12);
    let word_b: u16 = ((p1 & mask) << 4) | ((p2 & mask) >> 10);
    let word_c: u16 = ((p2 & mask) << 6) | ((p3 & mask) >> 8);
    let word_d: u16 = ((p3 & mask) << 8) | ((p4 & mask) >> 6);
    let word_e: u16 = ((p4 & mask) << 10) | ((p5 & mask) >> 4);
    let word_f: u16 = ((p5 & mask) << 12) | ((p6 & mask) >> 2);
    let word_g: u16 = ((p6 & mask) << 14) | (p7 & mask);
    let [a, b] = word_a.to_le_bytes();
    let [c, d] = word_b.to_le_bytes();
    let [e, f] = word_c.to_le_bytes();
    let [g, h] = word_d.to_le_bytes();
    let [i, j] = word_e.to_le_bytes();
    let [k, l] = word_f.to_le_bytes();
    let [m, n] = word_g.to_le_bytes();
    return [a, b, c, d, e, f, g, h, i, j, k, l, m, n];
}

/* Encodes 4 pixels into 6 bytes of 12-bit packed data */
#[inline(always)]
const fn encode12([p0, p1, p2, p3]: [u16; 4]) -> [u8; 6] {
    let mask = ones(12);
    let word_a: u16 = ((p0 & mask) << 4) | ((p1 & mask) >> 8);
    let word_b: u16 = ((p1 & mask) << 8) | ((p2 & mask) >> 4);
    let word_c: u16 = ((p2 & mask) << 12) | (p3 & mask);
    let [a, b] = word_a.to_le_bytes();
    let [c, d] = word_b.to_le_bytes();
    let [e, f] = word_c.to_le_bytes();
    return [a, b, c, d, e, f];
}

/* Encodes 8 pixels into 10 bytes of 10-bit packed data */
#[inline(always)]
const fn encode10([p0, p1, p2, p3, p4, p5, p6, p7]: [u16; 8]) -> [u8; 10] {
    let mask = ones(10);
    let word_a: u16 = ((p0 & mask) << 6) | ((p1 & mask) >> 4);
    let word_b: u16 = ((p1 & mask) << 12) | ((p2 & mask) << 2) | ((p3 & mask) >> 8);
    let word_c: u16 = ((p3 & mask) << 8) | ((p4 & mask) >> 2);
    let word_d: u16 = ((p4 & mask) << 14) | ((p5 & mask) << 4) | ((p6 & mask) >> 6);
    let word_e: u16 = ((p6 & mask) << 10) | (p7 & mask);
    let [a, b] = word_a.to_le_bytes();
    let [c, d] = word_b.to_le_bytes();
    let [e, f] = word_c.to_le_bytes();
    let [g, h] = word_d.to_le_bytes();
    let [i, j] = word_e.to_le_bytes();
    return [a, b, c, d, e, f, g, h, i, j];
}
