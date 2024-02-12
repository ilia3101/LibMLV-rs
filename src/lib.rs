pub mod blocks;
pub mod index;
pub mod endianness;
pub mod decode;

/************************** Core traits and types ***************************/

// pub trait Index {
//     fn get_num_blocks_of_type(&self, block_type: BlockTag) -> usize;
//     fn get_pos_of_block(&self, block_type: BlockTag, block_number: usize) -> Option<FileLocation>;
//     // fn iter_blocks_of_type(&self, block_type: BlockTag) -> impl Iterator<Item=(BlockHeader,FileLocation)>;
// }

type TimeStamp = endianness::u64le;

#[derive(Clone,Copy,PartialEq,Eq,Hash)]
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
    #[inline(always)]
    fn eq(&self, s: &&str) -> bool {
        s.len() == 4 && s.chars().count() == 4 && self.0.iter().zip(s.chars()).all(|(a,b)| *a == b as u8)
    }
}

impl PartialEq<str> for BlockTag {
    #[inline(always)] fn eq(&self, s: &str) -> bool { self == s }
}

impl Debug for BlockTag {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match core::str::from_utf8(&self.0) {
            Ok(s) => write!(f, "{s}"),
            Err(_) => write!(f, "[{:#02x},{:#02x},{:#02x},{:#02x}]", self.0[0],self.0[1],self.0[2],self.0[3]),
        }
    }
}

/* A data source trait to represent a set of files, custom implementations possible
 * for no_std uses, but the default implementation uses std */
pub trait DataSource {
    fn get_num_chunks(&self) -> u8;
    fn get_chunk_size(&self, chunk: u8) -> Option<u64>;
    fn read(&mut self, chunk: u8, position: u64, output: &mut [u8]) -> u64;
}

/* Core blocks providing information about an MLV clip */
#[derive(Default,Debug,Copy,Clone)]
pub struct CoreBlocks {
    pub file: Option<blocks::FileHeader>,
    pub rawi: Option<blocks::Rawi>,
    pub rawc: Option<blocks::Rawc>,
    pub wavi: Option<blocks::Wavi>,
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

#[derive(Clone,Copy,PartialEq,Debug)]
#[repr(C)]
pub struct BlockHeader {
    pub block_type: BlockTag,
    pub block_size: endianness::u32le,
    pub time_stamp: TimeStamp,
}

#[derive(Clone,Copy)]
#[repr(transparent)]
pub struct FileLocation ([u8;6]);

impl FileLocation {
    #[inline]
    pub fn new(chunk: u8, offset: u64) -> Option<Self> {
        let [x,y,z,a,b,c,d,e] = offset.to_be_bytes();
        if x != 0 || y != 0 || z != 0 {
            return None; /* Offset is too large */
        } else {
            Some(Self([chunk,a,b,c,d,e]))
        }
    }
    #[inline]
    pub fn get_offset(self) -> u64 {
        let Self([_,a,b,c,d,e]) = self;
        u64::from_be_bytes([0,0,0,a,b,c,d,e])
    }
    #[inline]
    pub fn get_chunk(self) -> u8 { self.0[0] }
    #[inline]
    pub fn apply_offset(self, offset: i64) -> Option<Self> {
        Self::new(self.get_chunk(), u64::try_from((self.get_offset() as i64).checked_add(offset)?).ok()?)
    }
}

// use core::num::NonZeroU64;

// #[derive(Clone,Copy)]
// #[repr(transparent)]
// pub struct FileLocation (NonZeroU64);

// impl FileLocation {
//     #[inline(always)]
//     pub fn new(chunk: u8, offset: u64) -> Option<Self> {
//         Some(Self(NonZeroU64::new(chunk.checked_add(1)? as u64 | offset.checked_shl(8)?)?))
//     }
//     #[inline(always)]
//     pub fn get_chunk(&self) -> u8 { (self.0.get() & 255) as u8 - 1 }
//     #[inline(always)]
//     pub fn get_offset(&self) -> u64 { self.0.get() >> 8 }
// }

impl core::fmt::Debug for FileLocation {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let (chunk, pos) = (self.get_chunk(), self.get_offset());
        write!(f, "FileLocation {{ chunk: {chunk}, pos: {pos} }}")
        // f.debug_struct("FileLocation").field("chunk", &chunk).field("offset", &pos).finish()
    }
}


/*************************** Implementations with std ***************************/
// TODO: add std feature gate to this whole section
pub mod file_data_source;

pub use file_data_source::FileDataSource;

pub mod mlv_reader;
pub use mlv_reader::BlockReader;




/***************** TOP LEVEL READER IMPLEMENTATION *****************/

use std::{io::BufReader, fs::File, path::Path, fmt::Debug, collections::HashMap};

const MAX_BLOCK_STORED_SIZE: usize = 512;

#[derive(Clone,Copy,Debug)]
pub struct VidfFrameInfo {
    pub block_location: FileLocation,
    pub data_offset: u32,
    pub data_bytes: u32, /* Useful when compressed */
}

// #[derive(Debug)]
// pub struct AudfFrameInfo {
//     pub block_location: FileLocation,
//     pub data_offset: u32,
//     pub data_bytes: u32, /* Useful when compressed */
// }

#[derive(Debug)]
pub struct MainReader<Reader> {
    pub core_blocks: CoreBlocks,
    chunk_files: Vec<BlockReader<Reader>>, /* TODO: abstract the block source/make BlockReader into a trait (?) */
    all_blocks: Vec<(BlockHeader,FileLocation,Option<Vec<u8>>)>,
    all_vidf: Vec<FileLocation>,
    all_audf: Vec<FileLocation>,
    // /* Information about blocks based on type: (Location, TimeStamp, Optional data after header) */
    // by_type: HashMap<BlockTag, Vec<(FileLocation,TimeStamp,Option<Vec<u8>>)>>,
}

impl MainReader<BufReader<File>>
{
    #[inline]
    pub fn open_mlv<P: AsRef<Path>>(path: P) -> Option<Self>
    {
        /* TODO: search for chunks (and limit to 101) */
        let mut chunk_files = vec![BlockReader::new(BufReader::new(File::open(path).ok()?), 0)?];

        /* Create empty reader/index object */
        let mut reader = Self::empty();

        /* TODO: Use rayon par iter maybe?? */
        for (chunk_index, file) in chunk_files.iter_mut().enumerate() {
            loop {
                if let Some(block_info) = file.get_block_info() {
                    if block_info.block_type != "NULL" {
                        reader.add_block(
                            block_info,
                            FileLocation::new(chunk_index as u8, file.get_block_position())?,
                            None /* TODO: read block into u8 vec and pass it here.
                            (And then find a better way of doing it with less allocations) */
                        );
                        // if block_info.block_type == 
                    }
                }
                else { break; }
                file.next_block();
            }
        }

        reader.chunk_files = chunk_files;

        /* Set first block's timestamp to 0, as it was filled with version string before */
        // all_blocks[0].0.time_stamp = 0;

        /* Sort by timestamp */
        // all_blocks.sort_unstable_by(|a,b| a.0.time_stamp.cmp(&b.0.time_stamp));

        /* Finalise index (generic) */

        reader.finalise();

        Some(reader)
    }
}

impl<Reader> MainReader<Reader> {
    pub fn get_num_frames(&self) -> u32 {
        self.all_vidf.len() as u32
    }

    pub fn decode_frame(&mut self, idx: u32) -> Option<Vec<u16>>
    where
        Reader: std::io::Read + std::io::Seek
    {
        let file_location = self.all_vidf.get(idx as usize)?;
        let file = &mut self.chunk_files[file_location.get_chunk() as usize].file;
        file.seek(std::io::SeekFrom::Start(file_location.get_offset() + 4)).ok()?;

        let mut size = [0u8; 4];
        file.read_exact(&mut size).ok()?;
        file.seek(std::io::SeekFrom::Current(8)).ok()?; /* Skip Timestamp */
        file.seek(std::io::SeekFrom::Current(12)).ok()?; /* Skip the unnecessary fields in VIDF, to get to offset */

        let mut data_offset = [0u8; 4];
        file.read_exact(&mut data_offset).ok()?;

        let size = u32::from_le_bytes(size);
        let data_offset = u32::from_le_bytes(data_offset);

        file.seek(std::io::SeekFrom::Current(data_offset as i64)).ok()?; /* Skip to frame data */

        let frame_data_size = (size - (data_offset + 32)) as usize;
        let mut data = Vec::with_capacity(frame_data_size);
        unsafe { data.set_len(frame_data_size) }

        file.read_exact(&mut data).ok()?;

        /* TODO: decode 14-bit and compression here */
        let frame_max_length = 1000000000000000; /* TODO: multiply width*height here */
        Some(decode::decode_packed12(&data).take(frame_max_length).collect())
    }
}

impl<T> MainReader<T> {
    fn empty() -> Self {
        Self {
            chunk_files: vec![],
            core_blocks: CoreBlocks::default(),
            all_blocks: Vec::new(),
            all_vidf: vec![],
            all_audf: vec![],
            // by_type: HashMap::new(),
        }
    }

    fn add_block(&mut self, info: BlockHeader, location: FileLocation, data: Option<Vec<u8>>) {
        self.all_blocks.push((info, location, data));
    }

    fn finalise(&mut self) {
        /* Sort blocks */
        self.all_blocks.sort_unstable_by(|a,b| a.2.cmp(&b.2));

        for (block_info, location, data) in self.all_blocks.iter() {
            if block_info.block_type == "VIDF" {
                self.all_vidf.push(*location);
            } else if block_info.block_type == "AUDF" {
                self.all_audf.push(*location);
            } else {
                /* Hashmap by block type? */
                // self.by_type.entry(block_info.block_type).or_insert_with(|| vec![]).push((*location,block_info.time_stamp,data.clone()));
            }
        }
    }
    
    pub fn print_blocks(&self) {
        for (block_info, _location, _data) in self.all_blocks.iter() {
            if block_info.block_type != "VIDF" && block_info.block_type != "AUDF" {
                println!("{:?} : {} bytes", block_info.block_type, block_info.block_size);
            }
        }
        println!("Total blocks: {}", self.all_blocks.len());
    }

    /* Returns latest block of type at timestamp */
    pub fn get_block(&self, block_type: BlockTag, timestamp: TimeStamp) -> Option<&(FileLocation,TimeStamp,Option<Vec<u8>>)> {
        // self.by_type.get(&block_type)?.get(block_number)
        todo!()
    }
}

// #[derive(Debug)]
// pub struct FullIndex {
//     indexed_blocks: Vec<(BlockHeader,FileLocation)>,
//     unindexed_blocks: Vec<(BlockHeader,FileLocation)>,
//     // all_vidf: Vec<Option<(FileLocation, u32)>>,
//     // all_audf: Vec<Option<(FileLocation, u32)>>,
// }






// /***************** READER IMPLEMENTATION *****************/

// // pub struct MLVReader {
// //     file: Reader,
// //     index: Vec<crate::index::IndexEntry>
// // }




// // #[derive(Debug)]
// // pub struct File<T> {files: Vec<T>}
// // impl<T> DataSource for File<T>
// // where
// //     T: Read

// use std::io::Read;

// /* A bufreader is recommended  */
// pub struct BlockReader<T> {
//     file: T,
//     current_block: [u8; 4],
//     current_block_size: u32,
//     current_block_timestamp: u64,
//     current_block_position: u64,
// }

// impl<T> BlockReader<T>
// where
//     T: Read
// {
//     #[inline]
//     pub fn create(file: T) -> Option<Self> {
//         Some(Self { file, current_block: [0; 4], current_block_size: 0, current_block_timestamp: 0, current_block_position: 0 })
//     }

//     #[inline]
//     pub fn next(&mut self) -> Option<[u8; 4]> {
//         todo!()
//     }

//     #[inline]
//     pub fn read_current_block_data(
//         &mut self,
//         offset_from_start_of_block: u32,
//         out: &mut [u8]
//     ) -> Option<u64> {
//         if offset_from_start_of_block < 16 {
//             let header_bytes = self.current_block.iter()
//                 .chain(self.current_block_size.to_le_bytes().iter())
//                 .chain(self.current_block_size.to_le_bytes().iter())
//                 .copied()
//                 .skip(offset_from_start_of_block as usize);
//             let (head,bytes) = out.split_at_mut((offset_from_start_of_block - 16) as usize);
//         }
//         todo!()
//     }
// }



