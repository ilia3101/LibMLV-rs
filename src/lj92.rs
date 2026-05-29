/*
lj92.rs
(c) Andrew Baldwin 2014
(c) Ilia Sibiryakov 2024 (translated to Rust)

Permission is hereby granted, free of charge, to any person obtaining a copy of
this software and associated documentation files (the "Software"), to deal in
the Software without restriction, including without limitation the rights to
use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies
of the Software, and to permit persons to whom the Software is furnished to do
so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
*/

#[derive(Debug,Copy,Clone,PartialEq,Eq)]
pub enum Lj92Error {
    Corrupt = -1,
    NoMemory = -2,
    BadHandle = -3,
    TooWide = -4,
    Encoder = -5,
    EndOfImage = -6, /* ilia3101 */
}

#[derive(Debug,Default)]
pub struct Lj92<'a> {
    data: &'a [u8],

    scanstart: usize,
    ix: usize, /* Position in data */
    x: u32, // Width
    y: u32, // Height
    bitdepth: u8, // Bit depth
    components: u8,  // Components(Nf)
    // sssshist: [u32; 16],

    // Huffman table - only one supported, and probably needed
    hufflut: Vec<u16>,
    huffbits: u32,

    // Parse state
    cnt: usize,
    bits: i32, /* Current bits */
}

impl<'a> Lj92<'a> {
    /* Getters */
    pub fn width(&self) -> u32 { self.x }
    pub fn height(&self) -> u32 { self.y }
    pub fn bitdepth(&self) -> u8 { self.bitdepth }
    pub fn components(&self) -> u8 { self.components }

    /* This does what the 'BEH' macro did in the original -
     * getting a Big Endian Half from the input data, with given offset */
    #[inline(always)]
    fn get_be_u16(&self, off: usize) -> u16 {
        u16::from_be_bytes([self.data[self.ix+off], self.data[self.ix+off+1]])
    }

    /* Returns (Self, width, height, bitdepth, components) */
    pub fn open(data: &'a [u8]) -> Result<Lj92, Lj92Error> {
        let mut lj = Self::default();
        lj.data = data;
        let ret = lj.find_soi();
        ret.map(|_| lj)
    }

    /* I have merged lj92_decode and parse_scan into one function,
     * skipping the saving of parameters into the struct */
    pub fn decode(
        &mut self,
        out: &'a mut [u16],
        skip_length: usize,
        linearize: Option<&[u16]>
    ) -> Result<(),Lj92Error> {
        // self.sssshist = [0; 16];
        self.ix = self.scanstart;
        let compcount = self.data[self.ix+2];
        let pred = self.data[self.ix+3+2*compcount as usize];
        if pred > 7 { return Err(Lj92Error::Corrupt); }
        // if (pred==6) { return parsePred6(self); } /* Fast path, TODO: translate it to rust as well? */
        self.ix += self.get_be_u16(0) as usize;
        self.cnt = 0;
        self.bits = 0;

        /* To convert to u16 while overflowing */
        let to_u16 = |x: i32| (x & 0xffff) as u16;

        // First pixel predicted from base value
        let mut diff;
        let mut px;
        let mut left = 0i32;
        let row_out_len = (self.x * self.components as u32) as usize + skip_length;

        for row in 0..(self.y as usize) {
            let row_start = row * row_out_len;
            let prev_row_start = row.saturating_sub(1) * row_out_len;
            let lastrow = |data: &mut [u16], i: usize| -> u16 { data[prev_row_start + i] };
            let thisrow = |data: &mut [u16], i: usize| -> u16 { data[row_start + i] };
            for col in 0..(self.x as usize) {
                let colx = col * self.components as usize;
                for c in 0..(self.components as usize) {
                    px = match (row, col) {
                        (0, 0) => 1 << (self.bitdepth - 1),
                        (0, _) => thisrow(out, colx - self.components as usize + c),
                        (_, 0) => lastrow(out, c),
                        (_, _) => {
                            let prev_colx = colx - self.components as usize;
                            match pred {
                                0 => 0,
                                1 => thisrow(out, prev_colx + c),
                                2 => lastrow(out, colx + c),
                                3 => lastrow(out, prev_colx + c),
                                4 => to_u16(left + lastrow(out, colx + c) as i32 - lastrow(out, prev_colx + c) as i32),
                                5 => to_u16(left + ((lastrow(out, colx + c) as i32 - lastrow(out, prev_colx + c) as i32) >> 1)),
                                6 => to_u16(lastrow(out, colx + c) as i32 + ((left - lastrow(out, prev_colx + c) as i32) >> 1)),
                                7 => to_u16((left + lastrow(out, colx + c) as i32) >> 1),
                                _ => unreachable!("Invalid prediction mode")
                            }
                        }
                    };
                    
                    diff = self.next_diff();
                    left = to_u16((px as i32) + diff) as i32;

                    let linear = if let Some(linearize) = linearize {
                            /* ilia3101: Is this bounds checking really necessary? */
                            if left > linearize.len() as i32 { return Err(Lj92Error::Corrupt); }
                            linearize[left as usize]
                        } else { left as u16 };

                    /* Weird- adding this checked version made it a tiny bit faster than normal indexing??? */
                    if let Some(out) = out.get_mut(row_start + colx + c) {
                        *out = linear;
                    } else { return Ok(()); } /* Todo: return ok... or error? */
                } // c
            } // col
        } // row

        Ok(())
    }

    fn find_soi(&mut self) -> Result<(),Lj92Error> {
        if self.find() == Ok(0xd8) {
            self.parse_image()
        } else { Err(Lj92Error::Corrupt) }
    }

    fn find(&mut self) -> Result<u8,Lj92Error> {
        while self.data[self.ix] != 0xFF && self.ix < (self.data.len()-1) { self.ix += 1; }
        self.ix += 2;
        if self.ix >= self.data.len() { return Err(Lj92Error::EndOfImage); }
        Ok(self.data[self.ix-1])
    }

    fn parse_image(&mut self) -> Result<(),Lj92Error> {
        let mut ret = Ok(());
        while let Ok(next_marker) = self.find() {
            match next_marker {
                0xC4 => ret = self.parse_huff(),
                0xC3 => ret = self.parse_sof3(),
                0xFE => ret = self.parse_block(),
                0xD9 => {break},
                0xDA => {
                    self.scanstart = self.ix;
                    ret = Ok(());
                    break;
                },
                _ => ret = self.parse_block(),
            }

            if ret != Ok(()) {break;}
        }
        return ret;
    }

    fn parse_block(&mut self) -> Result<(),Lj92Error> {
        self.ix += self.get_be_u16(0) as usize;
        if self.ix >= self.data.len() { return Err(Lj92Error::Corrupt); }
        return Ok(());
    }

    fn parse_sof3(&mut self) -> Result<(),Lj92Error> {
        if (self.ix + 6) >= self.data.len() { return Err(Lj92Error::Corrupt); }
        self.y = self.get_be_u16(3) as u32;
        self.x = self.get_be_u16(5) as u32;
        self.bitdepth = self.data[self.ix+2];
        self.components = self.data[self.ix+7];
        self.ix += self.get_be_u16(0) as usize;
        Ok(())
    }

    fn parse_huff(&mut self) -> Result<(),Lj92Error> {
        let mut ret = Err(Lj92Error::Corrupt);
        let huffhead = &self.data[self.ix..]; // xstruct.unpack('>HB16B',self.data[self.ix:self.ix+19])
        let bits = &huffhead[2..];
        /* TODO: why is this weird mutation of the input data here (commenting it out didnt' break anything) */
        // bits[0] = 0; // Because table starts from 1
        let hufflen = u16::from_be_bytes([huffhead[0], huffhead[1]]);
        if (self.ix + hufflen as usize) >= self.data.len() { return ret; }

        /* Calculate huffman direct lut */
        // How many bits in the table - find highest entry
        let huffvals = &self.data[(self.ix+19)..];
        let mut maxbits = 16;
        while maxbits > 0 {
            if bits[maxbits] != 0 { break; }
            maxbits -= 1;
        }
        self.huffbits = maxbits as u32;
        /* Now fill the lut */
        self.hufflut = vec![0u16; 1 << maxbits];
        let mut i = 0;
        let mut hv = 0;
        let mut rv = 0;
        let mut vl = 0; // i
        let mut hcode;
        let mut bitsused = 1;

        while i < (1 << maxbits) {
            if bitsused > maxbits {
                break; // Done. Should never get here!
            }
            if vl >= bits[bitsused] {
                bitsused += 1;
                vl = 0;
                continue;
            }
            if rv == 1 << (maxbits-bitsused) {
                rv = 0;
                vl += 1;
                hv += 1;
                continue;
            }
            hcode = huffvals[hv];
            self.hufflut[i] = ((hcode as u16) << 8) | bitsused as u16;
            i += 1;
            rv += 1;
        }
        ret = Ok(());
        return ret;
    }

    fn next_diff(&mut self) -> i32 {
        let mut bits = self.bits;
        let mut cnt = self.cnt;
        let huffbits = self.huffbits;
        let mut ix = self.ix;
        while cnt < huffbits as usize {
            /* ilia3101: I have modified this line to be more endianness independent */
            let one = self.data[ix] as i32;
            let two = self.data[ix+1] as i32;
            bits = (bits << 16) | (one << 8) | two;
            cnt += 16;
            ix += 2;
            if one == 0xFF {
                bits >>= 8;
                cnt -= 8;
            } else if two == 0xFF { ix += 1; };
        }
        let index = bits >> (cnt - huffbits as usize);
        let ssssused: u16 = self.hufflut[index as usize];
        let usedbits = ssssused & 0xFF;
        let t = ssssused >> 8;
        // self.sssshist[t as usize] += 1;
        cnt -= usedbits as usize;
        let mut keepbitsmask = (1 << cnt ) - 1;
        bits &= keepbitsmask;
        let mut diff;
        if t == 16 {
            diff = 1 << 15;
        } else {
            while cnt < t as usize {
                /* ilia3101: I have modified this line to be more endianness independent */
                let one = self.data[ix] as i32;
                let two = self.data[ix+1] as i32;
                bits = (bits << 16) | (one << 8) | two;
                cnt += 16;
                ix += 2;
                /* Skip the 0 byte after each FF byte */
                if one == 0xFF {
                    bits >>= 8;
                    cnt -= 8;
                } else if two == 0xFF { ix += 1; }
            }
            cnt -= t as usize;
            diff = bits >> cnt;
            let mut vt = 1 << (t - 1);
            if diff < vt {
                vt = (-1 << t) + 1;
                diff += vt;
            }
        }
        keepbitsmask = (1 << cnt)-1;
        self.bits = bits & keepbitsmask;
        self.cnt = cnt;
        self.ix = ix;

        return diff;
    }
}