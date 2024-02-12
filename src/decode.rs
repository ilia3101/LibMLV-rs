#[inline]
pub fn decode_packed14(data: &[u8]) -> impl Iterator<Item = u16> + '_ {
    data.chunks_exact(14)
        .flat_map(|bytes| {
            let word_a: u16 = u16::from_le_bytes([bytes[0], bytes[1]]);
            let word_b: u16 = u16::from_le_bytes([bytes[2], bytes[3]]);
            let word_c: u16 = u16::from_le_bytes([bytes[4], bytes[5]]);
            let word_d: u16 = u16::from_le_bytes([bytes[6], bytes[7]]);
            let word_e: u16 = u16::from_le_bytes([bytes[8], bytes[9]]);
            let word_f: u16 = u16::from_le_bytes([bytes[10], bytes[11]]);
            let word_g: u16 = u16::from_le_bytes([bytes[12], bytes[13]]);
            return [
                word_a >> 2,
                ((word_a << 12) | (word_b >>  4)) & 0x3FFF,
                ((word_b << 10) | (word_c >>  6)) & 0x3FFF,
                ((word_c <<  8) | (word_d >>  8)) & 0x3FFF,
                ((word_d <<  6) | (word_e >> 10)) & 0x3FFF,
                ((word_e <<  4) | (word_f >> 12)) & 0x3FFF,
                ((word_f <<  2) | (word_g >> 14)) & 0x3FFF,
                word_g & 0x3FFF,
            ].into_iter()
        })
}

#[inline]
pub fn decode_packed12(data: &[u8]) -> impl Iterator<Item=u16> + '_ {
    data.chunks_exact(6).flat_map(|bytes| {
        let word_a: u16 = u16::from_le_bytes([bytes[0], bytes[1]]);
        let word_b: u16 = u16::from_le_bytes([bytes[2], bytes[3]]);
        let word_c: u16 = u16::from_le_bytes([bytes[4], bytes[5]]);
        return [
            word_a >> 4,
            ((word_a << 8) | (word_b >> 8)) & 0x0FFF,
            ((word_b << 4) | (word_c >> 12)) & 0x0FFF,
            word_c & 0x0FFF,
        ].into_iter()
    })
}

#[inline]
pub fn decode_packed10(data: &[u8]) -> impl Iterator<Item=u16> + '_ {
    data.chunks_exact(10).flat_map(|bytes| {
        let word_a: u16 = u16::from_le_bytes([bytes[0], bytes[1]]);
        let word_b: u16 = u16::from_le_bytes([bytes[2], bytes[3]]);
        let word_c: u16 = u16::from_le_bytes([bytes[4], bytes[5]]);
        let word_d: u16 = u16::from_le_bytes([bytes[6], bytes[7]]);
        let word_e: u16 = u16::from_le_bytes([bytes[8], bytes[9]]);
        return [
            word_a >> 6,
            ((word_a << 4) | (word_b >> 12)) & 0x03FF,
            (word_b >> 2) & 0x03FF,
            ((word_b << 8) | (word_c >> 8)) & 0x03FF,
            (word_c << 2) & 0x03FF,
            ((word_c << 12) | (word_d >> 4)) & 0x03FF,
            ((word_d << 6) | (word_e >> 10)) & 0x03FF,
            word_e & 0x03FF,
        ].into_iter()
    })
}

#[inline]
pub fn decode_lj92(data: &[u8]) -> impl Iterator<Item=u16> + '_ {
    todo!("lj92 decoding not implemented yet");
    return data.iter().map(|a| *a as u16);
}