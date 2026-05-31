use crate::lj92::{Lj92, Lj92Error};
use core::array::from_fn as arr;

#[inline]
pub fn decode_lj92(data: &[u8], out: &mut [u16]) -> Result<(),Lj92Error> {
    match Lj92::open(data) {
        Ok(mut lj92) => lj92.decode(out, 0, None),
        Err(err) => Err(err)
    }
}

#[inline]
pub fn decode_packed14(data: &[u8], out: &mut [u16]) {
    let (in_chunk, out_chunk) = (14, 8);
    for (i,o) in data.chunks_exact(in_chunk).zip(out.chunks_exact_mut(out_chunk)) {
        [o[0],o[1],o[2],o[3],o[4],o[5],o[6],o[7]] = decode14(arr(|n| i[n]));
    }
}

#[inline]
pub fn decode_packed12(data: &[u8], out: &mut [u16]) {
    let (in_chunk, out_chunk) = (6, 4);
    for (i,o) in data.chunks_exact(in_chunk).zip(out.chunks_exact_mut(out_chunk)) {
        [o[0],o[1],o[2],o[3]] = decode12(arr(|n| i[n]))
    }
}

#[inline]
pub fn decode_packed10(data: &[u8], out: &mut [u16]) {
    let (in_chunk, out_chunk) = (10, 8);
    for (i,o) in data.chunks_exact(in_chunk).zip(out.chunks_exact_mut(out_chunk)) {
        [o[0],o[1],o[2],o[3],o[4],o[5],o[6],o[7]] = decode10(arr(|n| i[n]));
    }
}

/* TODO: encoding functions and provide direct bitdepth to bitdepth for re-encoding */

/* Internal implementations */

#[inline]
const fn ones(n: u8) -> u16 { !(0xFFFF << n) }

/* Decodes 8 pixels from 14 bytes of 14-bit packed data */
#[inline(always)]
const fn decode14([a,b,c,d,e,f,g,h,i,j,k,l,m,n]: [u8; 14]) -> [u16; 8] {
    let mask = ones(14);
    let word_a: u16 = u16::from_le_bytes([a,b]);
    let word_b: u16 = u16::from_le_bytes([c,d]);
    let word_c: u16 = u16::from_le_bytes([e,f]);
    let word_d: u16 = u16::from_le_bytes([g,h]);
    let word_e: u16 = u16::from_le_bytes([i,j]);
    let word_f: u16 = u16::from_le_bytes([k,l]);
    let word_g: u16 = u16::from_le_bytes([m,n]);
    return [
        word_a >> 2,
        ((word_a << 12) | (word_b >>  4)) & mask,
        ((word_b << 10) | (word_c >>  6)) & mask,
        ((word_c <<  8) | (word_d >>  8)) & mask,
        ((word_d <<  6) | (word_e >> 10)) & mask,
        ((word_e <<  4) | (word_f >> 12)) & mask,
        ((word_f <<  2) | (word_g >> 14)) & mask,
        word_g & mask,
    ]
}

/* Decodes 4 pixels from 6 bytes of 12-bit packed data */
#[inline(always)]
const fn decode12([a,b,c,d,e,f]: [u8; 6]) -> [u16; 4] {
    let mask = ones(12);
    let word_a: u16 = u16::from_le_bytes([a,b]);
    let word_b: u16 = u16::from_le_bytes([c,d]);
    let word_c: u16 = u16::from_le_bytes([e,f]);
    return [
        word_a >> 4,
        ((word_a << 8) | (word_b >> 8)) & mask,
        ((word_b << 4) | (word_c >> 12)) & mask,
        word_c & mask,
    ]
}

/* Decodes 8 pixels from 10 bytes of 10-bit packed data */
#[inline(always)]
const fn decode10([a,b,c,d,e,f,g,h,i,j]: [u8; 10]) -> [u16; 8] {
    let mask = ones(10);
    let word_a: u16 = u16::from_le_bytes([a,b]);
    let word_b: u16 = u16::from_le_bytes([c,d]);
    let word_c: u16 = u16::from_le_bytes([e,f]);
    let word_d: u16 = u16::from_le_bytes([g,h]);
    let word_e: u16 = u16::from_le_bytes([i,j]);
    return [
        word_a >> 6,
        ((word_a << 4) | (word_b >> 12)) & mask,
        (word_b >> 2) & mask,
        ((word_b << 8) | (word_c >> 8)) & mask,
        (word_c << 2) & mask,
        ((word_c << 12) | (word_d >> 4)) & mask,
        ((word_d << 6) | (word_e >> 10)) & mask,
        word_e & mask,
    ]
}
