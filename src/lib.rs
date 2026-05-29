pub mod blocks;
pub mod decode;
pub mod utils;
pub mod lj92;

pub enum MLVError {
    CorruptFile,
}

/************************** Core traits and types ***************************/

pub use utils::{Read, Seek, SeekFrom};

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
#[derive(Default,Debug,Copy,Clone)]
pub struct CoreBlocks {
    pub mlvi: Option<[u8; 52]>,
    pub rawi: Option<[u8; 180]>,
    // pub rawc: Option<blocks::Rawc>,
    pub wavi: Option<[u8; 32]>,
    /* TODO. */
    // pub idnt: Option<(u64, IDNT)>,
    // pub diso: Option<(u64, DISO)>,
    // pub expo: Option<(u64, EXPO)>,
    // pub rtci: Option<(u64, RTCI)>,
    // pub lens: Option<(u64, LENS)>,
    // pub elns: Option<(u64, ELNS)>,
    // pub wbal: Option<(u64, WBAL)>,
    // pub styl: Option<(u64, STYL)>,
}

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


/*************************** Implementations with std ***************************/
pub mod block_reader;
pub use block_reader::BlockReader;


/***************** TOP LEVEL READER IMPLEMENTATION *****************/

use std::{io::BufReader, fs::File, path::Path, fmt::Debug};

use crate::decode::decode_packed12;

#[derive(Clone,Copy,Debug,PartialEq)]
pub struct BlockEntry {
    pub block: BlockHeader,
    pub location: FileLocation,
    // pub data: Option<Vec<u8>>,
}

#[derive(Debug)]
pub struct MainReader<Reader> {
    core_blocks: CoreBlocks,
    chunk_files: Vec<BlockReader<Reader>>, /* TODO: abstract the block source/make BlockReader into a trait (?) */
    all_blocks: Vec<BlockEntry>,
    all_vidf: Vec<FileLocation>,
    all_audf: Vec<FileLocation>,
}


#[cfg(feature = "std")]
impl MainReader<utils::ReadSeekFromStdIo<BufReader<File>>>
{
    pub fn open_mlv<P: AsRef<Path>>(path: P) -> Option<Self>
    {
        /* TODO: search for all chunks (and limit to 101) */
        let mut chunk_files = vec![BlockReader::new(utils::ReadSeekFromStdIo(BufReader::new(File::open(path).ok()?)))?];

        /* Create empty reader/index object */
        let mut reader = Self::empty();

        /* TODO: Use rayon par iter maybe?? */
        for (chunk_index, file) in chunk_files.iter_mut().enumerate() {
            loop {
                if let Some(block_info) = file.block_info() {
                    if block_info.block_type != "NULL" {
                        reader.all_blocks.push(
                            BlockEntry {
                                block: block_info,
                                location: FileLocation::new(chunk_index as u8, file.block_position())?
                            }   
                        );
                        /* TODO: put this block loading at the end */
                        fn try_into<const N: usize>(out: &mut Option<[u8; N]>, data: Option<&[u8]>) {
                            if let Some(data) = data {
                                if out.is_none() && data.len() >= N {
                                    *out = Some(core::array::from_fn(|i| data[i]));
                                }
                            }
                        }
                        if block_info.block_type == "MLVI" {
                            try_into(&mut reader.core_blocks.mlvi, file.block_bytes());
                        } else if block_info.block_type == "RAWI" {
                            try_into(&mut reader.core_blocks.rawi, file.block_bytes());
                        } else if block_info.block_type == "WAVI" {
                            try_into(&mut reader.core_blocks.wavi, file.block_bytes());
                        }
                    }
                }
                else { break; }
                file.next_block();
            }
        }

        reader.chunk_files = chunk_files;

        /* Sort by timestamp */
        reader.all_blocks.sort_unstable_by(|a,b| a.block.time_stamp.cmp(&b.block.time_stamp));


        /***************************************************************************/
        /******************************* Finalisation ******************************/
        /***************************************************************************/

        reader.all_vidf = reader.all_blocks.iter().filter_map(|b| {
            (b.block.block_type == "VIDF").then_some(b.location)
        }).collect();

        reader.all_audf = reader.all_blocks.iter().filter_map(|b| {
            (b.block.block_type == "AUDF").then_some(b.location)
        }).collect();

        /***************************************************************************/

        Some(reader)
    }
}

macro_rules! time {
    ($block:block) => {{
        let start = std::time::Instant::now();
        let result = {$block};
        let duration_ms = start.elapsed().as_micros() as f64 / 1000.0;
        println!("Operation took {:.1} ms", duration_ms);
        result
    }};
}

pub enum FrameCountInfo {
    NotAllParsed{total: u32, dropped: u32},
    AllParsed{total: u32, dropped: u32},
}

impl<Reader> MainReader<Reader>
{
    pub fn width(&self) -> Option<u32> {
        blocks::get_u16(&self.core_blocks.rawi?, blocks::RAWI.field_offset("xRes")?).map(|x| x as u32)
    }
    pub fn height(&self) -> Option<u32> {
        blocks::get_u16(&self.core_blocks.rawi?, blocks::RAWI.field_offset("yRes")?).map(|x| x as u32)
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
    pub fn is_compressed(&self) -> Option<bool> {
        const MLV_VIDEO_CLASS_FLAG_LJ92: u16 = 0x20;
        let class = blocks::get_u16(&self.core_blocks.mlvi?, blocks::MLVI.field_offset("videoClass")?);
        Some(class? & MLV_VIDEO_CLASS_FLAG_LJ92 != 0)
    }

    fn empty() -> Self {
        Self {
            chunk_files: vec![],
            core_blocks: CoreBlocks::default(),
            all_blocks: Vec::new(),
            all_vidf: vec![],
            all_audf: vec![],
        }
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

    pub fn decode_frame<'a>(&mut self, idx: u32, output: &'a mut [u16]) -> Option<&'a [u16]>
    where
        Reader: Read + Seek
    {
        println!("decoding 1...");

        let file_location = self.all_vidf.get(idx as usize)?;
        let file = &mut self.chunk_files[file_location.chunk() as usize].file;
        // println!("decoding 2...");
        file.seek(SeekFrom::Start(file_location.offset() + 4)).ok()?;

        let mut size = [0u8; 4];
        file.read_exact(&mut size).ok()?;
        file.seek(SeekFrom::Current(20)).ok()?; /* Skip Timestamp and other VIDF fields to get to offset */

        let mut data_offset = [0u8; 4];
        file.read_exact(&mut data_offset).ok()?;

        let size = u32::from_le_bytes(size);
        let data_offset = u32::from_le_bytes(data_offset);

        file.seek(SeekFrom::Current(data_offset as i64)).ok()?; /* Skip to frame data */

        let frame_data_size = (size - (data_offset + 32)) as usize;
        let mut data = Vec::with_capacity(frame_data_size);
        unsafe { data.set_len(frame_data_size) }
        file.read_exact(&mut data).ok()?;

        /* Decode the frame */
        time! {{
            // let max_length = (self.width()? * self.height()?) as usize;
            match (self.bitdepth()?, self.is_compressed()?) {
                (14, false) => decode::decode_packed14(&data, output),
                (12, false) => decode::decode_packed12(&data, output),
                (10, false) => decode::decode_packed10(&data, output),
                (_, true) => {decode::decode_lj92(&data, output);},
                _ => {}, /* Unsupported format */
            }
        }}

        return Some(&output[..]);
    }
}
