pub mod blocks;
pub mod codec;
pub use util_types::*;
pub mod block_reader;

pub enum MLVError {
    CorruptFile,
}

/************************** Core traits and types ***************************/

#[cfg_attr(rustfmt, rustfmt_skip)]
mod util_types {
    use super::*;

    #[derive(Debug,Clone,Copy,PartialEq,Eq,PartialOrd)]
    pub struct BlockHeader {
        pub block_type: BlockTag,
        pub block_size: u32,
        pub time_stamp: u64,
    }

    impl BlockHeader {
        #[inline]
        pub const fn from_bytes([a,b,c,d,e,f,g,h,i,j,k,l,m,n,o,p]: [u8; 16]) -> Self {
            Self {
                block_type: BlockTag([a,b,c,d]),
                block_size: u32::from_le_bytes([e,f,g,h]),
                time_stamp: u64::from_le_bytes([i,j,k,l,m,n,o,p]),
            }
        }
        #[inline]
        pub fn to_bytes(&self, out: &mut [u8]) {
            out[0..4].copy_from_slice(&self.block_type.0);
            out[4..8].copy_from_slice(&self.block_size.to_le_bytes());
            out[8..16].copy_from_slice(&self.time_stamp.to_le_bytes());
        }
    }

    #[derive(Clone,Copy,PartialEq,Eq,PartialOrd)]
    #[repr(transparent)]
    pub struct BlockTag (pub [u8;4]);

    impl BlockTag {
        #[inline]
        pub const fn new(s: &str) -> Option<Self> {
            let s = s.as_bytes();
            if s.len() == 4 {
                Some(Self([s[0],s[1],s[2],s[3]]))
            } else { return None; }
        }
    }

    impl PartialEq<&str> for BlockTag {
        #[inline]
        fn eq(&self, s: &&str) -> bool {
            s.len() == 4 && s.chars().count() == 4 && self.0.iter().zip(s.chars()).all(|(a,b)| *a == b as u8)
        }
    }

    impl PartialEq<str> for BlockTag {
        #[inline] fn eq(&self, s: &str) -> bool { self == s }
    }

    impl Debug for BlockTag {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            match core::str::from_utf8(&self.0) {
                Ok(s) => write!(f, "{s}"),
                Err(_) => write!(f, "[{:#02x},{:#02x},{:#02x},{:#02x}]", self.0[0],self.0[1],self.0[2],self.0[3]),
            }
        }
    }

    /* Core blocks providing information about an MLV clip */
    #[derive(Debug, Copy, Clone)]
    pub struct CoreBlocks {
        pub mlvi: Option<[u8; 52]>,
        pub rawi: Option<[u8; 180]>,
        pub wavi: Option<[u8; 32]>,
        pub idnt: Option<[u8; 84]>,
        /* CURV log curve lookup table: (block header bytes, LUT entries). */
        pub curv: Option<([u8; 16], [u16; blocks::CURV_MAX_LUT_LEN])>,
        // pub rawc: Option<blocks::Rawc>,
        // pub diso: Option<(u64, DISO)>,
        // pub expo: Option<(u64, EXPO)>,
        // pub rtci: Option<(u64, RTCI)>,
        // pub lens: Option<(u64, LENS)>,
        // pub elns: Option<(u64, ELNS)>,
        // pub wbal: Option<(u64, WBAL)>,
        // pub styl: Option<(u64, STYL)>,
    }

    impl Default for CoreBlocks {
        fn default() -> Self {
            Self {
                mlvi: None,
                rawi: None,
                wavi: None,
                idnt: None,
                curv: None,
            }
        }
    }

    // TODO: simplify this, it was premature optimisation
    #[derive(Clone,Copy,PartialEq,Eq)]
    #[repr(transparent)]
    pub struct FileLocation ([u8;6]);

    impl FileLocation {
        #[inline]
        pub fn new(chunk: u8, offset: u64) -> Option<Self> {
            let [x,y,z,a,b,c,d,e] = offset.to_be_bytes();
            (x == 0 && y == 0 && z == 0).then_some(Self([chunk,a,b,c,d,e]))
        }
        #[inline]
        pub fn offset(self) -> u64 {
            let Self([_,a,b,c,d,e]) = self;
            u64::from_be_bytes([0,0,0,a,b,c,d,e])
        }
        #[inline]
        pub fn chunk(self) -> u8 { self.0[0] }
        #[inline]
        pub fn apply_offset(self, offset: i64) -> Option<Self> {
            Self::new(self.chunk(), u64::try_from((self.offset() as i64).checked_add(offset)?).ok()?)
        }
    }

    impl core::fmt::Debug for FileLocation {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            let (chunk, pos) = (self.chunk(), self.offset());
            write!(f, "FileLocation {{ chunk: {chunk}, pos: {pos} }}")
            // f.debug_struct("FileLocation").field("chunk", &chunk).field("offset", &pos).finish()
        }
    }
}

/***************** TOP LEVEL READER IMPLEMENTATION *****************/

#[cfg(feature = "std")]
use std::{fmt::Debug, fs::File, io::BufReader, path::Path};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BlockEntry {
    pub block: BlockHeader,
    pub location: FileLocation,
}

// An MLV file is presented using one of these
pub trait DataSource {
    type ReadError;
    fn num_files(&self) -> u32;
    fn file_size(&self, file: u32) -> u64;
    fn read_exact(&mut self, file: u32, offset: u64, out: &mut [u8]) -> Result<(), Self::ReadError>;
}

#[cfg(feature = "std")]
#[derive(Debug)]
pub struct FileDataSource {
    file_lengths: Vec<u64>,
    file_positions: Vec<u64>,
    files: Vec<std::io::BufReader<std::fs::File>>,
}

#[cfg(feature = "std")]
impl FileDataSource {
    pub fn new(path: impl AsRef<Path>) -> Result<Self, Box<dyn std::error::Error>> {
        //TODO: find chunk files (.M00 .M01 etc)
        let file = File::open(path)?;
        let filesize = file.metadata()?.len();
        Ok(Self {
            file_lengths: vec![filesize],
            file_positions: vec![0],
            files: vec![BufReader::new(file)],
        })
    }
}

#[cfg(feature = "std")]
impl DataSource for FileDataSource {
    type ReadError = Box<dyn std::error::Error>;
    fn num_files(&self) -> u32 {
        self.files.len() as u32
    }
    fn file_size(&self, file: u32) -> u64 {
        return self.file_lengths[file as usize];
    }
    fn read_exact(&mut self, chunk: u32, read_pos: u64, out: &mut [u8]) -> Result<(), Self::ReadError> {
        use std::io::Read;
        let pos = self.file_positions[chunk as usize];
        if read_pos != pos {
            self.files[chunk as usize].seek_relative(read_pos as i64 - pos as i64)?;
        }
        self.file_positions[chunk as usize] = read_pos + out.len() as u64;
        Ok(self.files[chunk as usize].read_exact(out).map(|_| ())?)
    }
}

/** Main class for reading MLV files */
#[derive(Debug)]
pub struct MainReader<DataSource> {
    pub core_blocks: CoreBlocks,
    pub chunk_files: DataSource, /* TODO: maybe don't keep this inside of this object and have it be external!!! */
    pub all_blocks: Vec<BlockEntry>,
    /** All AUDF blocks (file location of Block, timestamp, data offset, data length) */
    pub all_audf: Vec<(FileLocation, u64, u64, u32)>,
    /** All VIDF blocks (file location of Block, timestamp, data offset, data length) */
    pub all_vidf: Vec<(FileLocation, u64, u64, u32)>,
}

#[cfg(feature = "std")]
impl MainReader<FileDataSource> {
    pub fn open_mlv_from_path<P: AsRef<Path>>(path: P, max_frames: Option<u32>) -> Option<Self> {
        /* TODO: search for all chunks (and limit to 101) */
        let ds = FileDataSource::new(path).ok()?;
        Self::open_mlv(ds, max_frames)
    }
}

impl<DataSrc: DataSource> MainReader<DataSrc> {
    pub fn open_mlv(mut ds: DataSrc, max_frames: Option<u32>) -> Option<Self> {
        /* Create empty reader/index object */
        let mut core_blocks = CoreBlocks::default();
        let mut all_blocks = vec![];
        let mut all_vidf = vec![];
        let mut all_audf = vec![];

        let mut num_vidf = 0u32;

        /* TODO: Use rayon par iter maybe?? */
        for chunk_index in 0..ds.num_files() {
            let file_length = ds.file_size(chunk_index);
            let mut curv_location: Option<FileLocation> = None;
            let _result = block_reader::read_blocks::<200, _>(
                file_length,
                |pos: u64, out: &mut [u8]| ds.read_exact(chunk_index, pos, out),
                |block_bytes: &[u8], block_position: u64| {
                    let block_info = BlockHeader::from_bytes(*block_bytes[0..16].first_chunk().unwrap());
                    if block_info.block_type != "NULL" {
                        /* Skip null blocks */
                        let location = FileLocation::new(chunk_index as u8, block_position).unwrap();
                        all_blocks.push(BlockEntry { block: block_info, location });
                        /* TODO: put this block loading at the end */
                        fn try_into<const N: usize>(out: &mut Option<[u8; N]>, data: Option<&[u8]>) {
                            if let Some(data) = data {
                                if out.is_none() && data.len() >= N {
                                    *out = Some(core::array::from_fn(|i| data[i]));
                                }
                            }
                        }
                        if block_info.block_type == "MLVI" {
                            try_into(&mut core_blocks.mlvi, Some(block_bytes));
                        } else if block_info.block_type == "RAWI" {
                            try_into(&mut core_blocks.rawi, Some(block_bytes));
                        } else if block_info.block_type == "WAVI" {
                            try_into(&mut core_blocks.wavi, Some(block_bytes));
                        } else if block_info.block_type == "IDNT" {
                            try_into(&mut core_blocks.idnt, Some(block_bytes));
                        } else if block_info.block_type == "CURV" {
                            /* Store the header now; the LUT may exceed the block-read window. */
                            let mut header = [0u8; 16];
                            header.copy_from_slice(&block_bytes[0..16]);
                            core_blocks.curv = Some((header, [0u16; blocks::CURV_MAX_LUT_LEN]));
                            curv_location = Some(location);
                        } else if block_info.block_type == "VIDF" {
                            let block_size = block_info.block_size;
                            let frame_data_offset = u32::from_le_bytes(*block_bytes[28..].first_chunk().unwrap());
                            let offset_in_file = block_position + 32 + frame_data_offset as u64;
                            let frame_data_size = (block_size as u32 - (frame_data_offset as u32 + 32)) as u32;
                            all_vidf.push((location, block_info.time_stamp, offset_in_file, frame_data_size)); // TODO: block
                            num_vidf += 1;
                            if let Some(max_frames) = max_frames
                                && max_frames == num_vidf
                            {
                                return true;
                            }
                        } else if block_info.block_type == "AUDF" {
                            let block_size = block_info.block_size;
                            let frame_data_offset = u32::from_le_bytes(*block_bytes[20..].first_chunk().unwrap());
                            let offset_in_file = block_position + 24 + frame_data_offset as u64;
                            let frame_data_size = (block_size as u32 - (frame_data_offset as u32 + 24)) as u32;
                            all_audf.push((location, block_info.time_stamp, offset_in_file, frame_data_size)); // TODO: block
                        }
                    }
                    return true;
                },
            );
            // println!("Result = {:?}", result);

            /* Read the CURV lookup table if a curv block was found in the file. */
            if let (Some((header, lut)), Some(location)) = (&mut core_blocks.curv, curv_location) {
                let block_size = u32::from_le_bytes(*header[4..8].first_chunk().unwrap());
                let lut_len = ((block_size.saturating_sub(16)) / 2).min(blocks::CURV_MAX_LUT_LEN as u32) as usize;
                if lut_len > 0 {
                    let mut lut_bytes = vec![0u8; lut_len * 2];
                    if ds.read_exact(location.chunk() as u32, location.offset() + 16, &mut lut_bytes).is_ok() {
                        for (i, chunk) in lut_bytes.chunks_exact(2).enumerate() {
                            lut[i] = u16::from_le_bytes([chunk[0], chunk[1]]);
                        }
                    }
                }
            }
        }

        /* Sort by timestamp (MLVI always first) */
        all_blocks.sort_unstable_by(|a, b| {
            let ta = if a.block.block_type == "MLVI" { 0 } else { a.block.time_stamp };
            let tb = if b.block.block_type == "MLVI" { 0 } else { b.block.time_stamp };
            ta.cmp(&tb)
        });
        all_vidf.sort_unstable_by(|a, b| a.1.cmp(&b.1));
        all_audf.sort_unstable_by(|a, b| a.1.cmp(&b.1));

        Some(Self { chunk_files: ds, core_blocks, all_blocks, all_vidf, all_audf })
    }
}

impl<DataSrc> MainReader<DataSrc> {
    pub fn width(&self) -> Option<u32> {
        blocks::get_u16(&self.core_blocks.rawi?, blocks::RAWI.field_offset("xRes")?).map(|x| x as u32)
    }

    pub fn height(&self) -> Option<u32> {
        blocks::get_u16(&self.core_blocks.rawi?, blocks::RAWI.field_offset("yRes")?).map(|x| x as u32)
    }

    pub fn fps(&self) -> Option<(u32, u32)> {
        let nom = blocks::get_u32(&self.core_blocks.mlvi?, blocks::MLVI.field_offset("sourceFpsNom")?)?;
        let denom = blocks::get_u32(&self.core_blocks.mlvi?, blocks::MLVI.field_offset("sourceFpsDenom")?)?;
        Some((nom, denom))
    }

    pub fn black_level(&self) -> Option<i32> {
        blocks::get_i32(&self.core_blocks.rawi?, blocks::RAWI.field_offset("black_level")?)
    }

    pub fn white_level(&self) -> Option<i32> {
        blocks::get_i32(&self.core_blocks.rawi?, blocks::RAWI.field_offset("white_level")?)
    }

    pub fn bitdepth(&self) -> Option<i32> {
        blocks::get_i32(&self.core_blocks.rawi?, blocks::RAWI.field_offset("bits_per_pixel")?)
    }

    /** CURV lookup table entries (valid slice length is derived from the block header). */
    pub fn curve_lut(&self) -> Option<&[u16]> {
        let (header, lut) = self.core_blocks.curv.as_ref()?;
        let block_size = u32::from_le_bytes(*header[4..8].first_chunk().unwrap());
        let lut_len = ((block_size.saturating_sub(16)) / 2).min(blocks::CURV_MAX_LUT_LEN as u32) as usize;
        Some(&lut[..lut_len])
    }

    pub fn colour_matrix(&self) -> Option<[[f32; 3]; 3]> {
        let get = |i: u32| {
            Some(blocks::get_i32(&self.core_blocks.rawi?, blocks::RAWI.field_offset("color_matrix1")? + 4 * i)? as f32)
        };
        Some([
            [get(0)? / get(1)?, get(2)? / get(3)?, get(4)? / get(5)?],
            [get(6)? / get(7)?, get(8)? / get(9)?, get(10)? / get(11)?],
            [get(12)? / get(13)?, get(14)? / get(15)?, get(16)? / get(17)?],
        ])
    }

    pub const MLV_VIDEO_CLASS_FLAG_LJ92: u16 = 0x20;
    pub const MLV_VIDEO_CLASS_FLAG_JP2K: u16 = 0x10;

    pub fn videoclass(&self) -> Option<u16> {
        blocks::get_u16(&self.core_blocks.mlvi?, blocks::MLVI.field_offset("videoClass")?)
    }

    pub fn is_lj92(&self) -> Option<bool> {
        Some((self.videoclass()? & Self::MLV_VIDEO_CLASS_FLAG_LJ92) != 0)
    }

    pub fn is_jp2k(&self) -> Option<bool> {
        Some((self.videoclass()? & Self::MLV_VIDEO_CLASS_FLAG_JP2K) != 0)
    }

    pub fn is_compressed(&self) -> Option<bool> {
        Some(self.is_jp2k()? | self.is_lj92()?)
    }

    pub fn audio_sample_rate(&self) -> Option<u32> {
        blocks::get_u32(&self.core_blocks.wavi?, blocks::WAVI.field_offset("samplingRate")?)
    }

    pub fn audio_channels(&self) -> Option<u16> {
        blocks::get_u16(&self.core_blocks.wavi?, blocks::WAVI.field_offset("channels")?)
    }

    pub fn audio_bits_per_sample(&self) -> Option<u16> {
        blocks::get_u16(&self.core_blocks.wavi?, blocks::WAVI.field_offset("bitsPerSample")?)
    }

    pub fn print_blocks(&self) {
        for b in self.all_blocks.iter() {
            if b.block.block_type != "VIDF" && b.block.block_type != "AUDF" {
                println!("{:?} : {} bytes", b.block.block_type, b.block.block_size);
            }
        }
        println!("Total blocks: {}", self.all_blocks.len());
    }

    pub fn num_frames(&self) -> u32 {
        self.all_vidf.len() as u32
    }

    pub fn frame_data_location_and_size(&self, idx: u32) -> Option<(FileLocation, u32)> {
        let (file_location, _timestamp, pos, size) = *self.all_vidf.get(idx as usize)?;
        Some((FileLocation::new(file_location.chunk(), pos)?, size))
    }

    // TODO: better error handling than just returning option
    // returns none if out buffer is not big enough
    pub fn get_frame_payload<'a>(&mut self, idx: u32, mut out: &'a mut [u8]) -> Option<&'a [u8]>
    where
        DataSrc: DataSource,
    {
        let (file_location, frame_data_size) = self.frame_data_location_and_size(idx)?;
        if out.len() < frame_data_size as usize {
            return None; // output buffer too small
        } else {
            out = &mut out[0..frame_data_size as usize];
            let _result =
                self.chunk_files.read_exact(file_location.chunk() as u32, file_location.offset(), &mut out).ok()?;
            return Some(out);
        }
    }

    pub fn decode_frame<'a>(&mut self, idx: u32, output: &'a mut [u16]) -> Option<&'a [u16]>
    where
        DataSrc: DataSource,
    {
        let (_file_location, frame_data_size) = self.frame_data_location_and_size(idx)?;

        // TODO: allow passing temporary buffer for frame decode
        let mut data = Vec::with_capacity(frame_data_size as usize);
        unsafe { data.set_len(frame_data_size as usize) }

        self.get_frame_payload(idx, &mut data);

        /*************************** Decode the frame ***************************/
        match (self.bitdepth()?, self.is_lj92()?, self.is_jp2k()?) {
            (14, false, false) => codec::decode_packed14(&data, output),
            (12, false, false) => codec::decode_packed12(&data, output),
            (10, false, false) => codec::decode_packed10(&data, output),
            (_, true, false) => {
                /* TODO: consider using CURV with lj92 as well?? */
                codec::decode_lj92(&data, output).expect("Lj92 failed");
            }
            (_, false, true) => {
                #[cfg(feature = "jpeg2000")]
                {
                    let decoder = codec::jpeg2000::Decoder::new();
                    let mut decoded_i32 = Vec::with_capacity(output.len());
                    unsafe { decoded_i32.set_len(output.len()) };
                    decoder.decode_into(&data, &mut decoded_i32);

                    /* JP2K frames written by compress_mlv are log-encoded;
                     * these MLV files pretty much MUST have a CURV block. TODO: consider handling of this... */
                    if let Some(lut) = self.curve_lut() {
                        for i in 0..output.len() {
                            output[i] = lut[((decoded_i32[i] & 0xFFFF) as u16) as usize];
                        }
                    } else {
                        for i in 0..output.len() {
                            output[i] = (decoded_i32[i] & 0xFFFF) as u16;
                        }
                    }
                }
            } /* Unsupported format */
            _ => {
                return None;
            }
        }

        return Some(&output[..]);
    }

    /* Intended for 16 bit 44.1khz stereo audio mainly. Returns interleaved stereo I think.
     * TODO: make this a flatmappable iterator */
    pub fn read_audio(&mut self) -> Option<Vec<i16>>
    where
        DataSrc: DataSource,
    {
        let mut audio_buffer = vec![];
        let mut chunk_buffer = vec![];
        for &(location, _timstamp, pos, size) in &self.all_audf {
            chunk_buffer.clear();
            chunk_buffer.reserve(size as usize);
            unsafe {
                chunk_buffer.set_len(size as usize);
            }
            self.chunk_files.read_exact(location.chunk() as u32, pos, &mut chunk_buffer).ok()?;
            for chunk in chunk_buffer.as_chunks().0.iter() {
                audio_buffer.push(i16::from_le_bytes(*chunk))
            }
        }
        Some(audio_buffer)
    }
}
