use std::{io::{Read, Seek, SeekFrom, BufReader}, mem::MaybeUninit};

use crate::{BlockHeader, BlockTag, endianness::*};

fn read_into_struct<T: Copy>(mut file: impl Read) -> Option<T> {
    unsafe {
        println!("size of T is {}", core::mem::size_of::<T>());
        let mut uninit = MaybeUninit::<T>::uninit();
        file.read_exact(core::slice::from_raw_parts_mut(uninit.as_mut_ptr() as *mut u8, core::mem::size_of::<T>())).ok()?;
        Some(uninit.assume_init())
    }
}

/* TODO: better end-of-file, IO error and other error handling */
#[derive(Debug)]
pub struct BlockReader<Reader> {
    pub file: Reader,
    block: Option<BlockHeader>,
    position: u64,
}

/* Read a block at current position in a file */
fn read_block(file: &mut impl Read) -> Option<BlockHeader> {
    let mut header = [0; 16];
    file.read_exact(&mut header).ok()?;

    /* Get block info */
    let block_type = BlockTag([header[0], header[1], header[2], header[3]]);
    let block_size = u32le::new(u32::from_le_bytes([header[4], header[5], header[6], header[7]]));
    let time_stamp = u64le::new(u64::from_le_bytes([
        header[8], header[9], header[10], header[11],
        header[12], header[13], header[14], header[15],
    ]));

    Some(BlockHeader{block_type, block_size, time_stamp})
}

impl<Reader: Read + Seek> BlockReader</* BufReader< */Reader/* > */>
{
    /* TODO: use these arguments: (chunks: impl AsRef<[impl Read]>) */
    pub fn new(mut file: /* BufReader< */Reader/* > */, _size: usize) -> Option<Self> {
        file.seek(SeekFrom::Start(0)).ok()?;
        let mut first_block: BlockHeader = read_into_struct(&mut file)?;
        // let mut first_block = read_block(&mut file)?;
        println!("first_block: {:#?}", first_block);
        /* Check that the block is "MLVI", correct size (52), and that version (timestamp) is "v2.0" */
        if first_block.block_type == "MLVI" &&
           first_block.time_stamp == u64::from_le_bytes(['v' as u8, '2' as u8, '.' as u8, '0' as u8, 0,0,0,0]) &&
           first_block.block_size == 52
        {
            first_block.time_stamp = 0.into();
            Some(Self { file, block: Some(first_block), position: 0 })
        }
        else { None }
    }

    #[inline]
    pub fn next_block(&mut self) {
        if let Some(info) = self.block {
            self.position += info.block_size.get() as u64;
            match self.file.seek(SeekFrom::Start(self.position)) {
            // match self.file.seek_relative((info.block_size - 16) as i64) {
                Ok(_) => self.block = read_block(&mut self.file),
                Err(_) => self.block = None,
            }
        }
    }

    #[inline]
    pub fn get_block_position(&self) -> u64 { self.position }

    #[inline]
    pub fn get_block_info(&self) -> Option<BlockHeader> { self.block }

    #[inline]
    pub fn get_current_block_bytes(&mut self, offset: u32, out: &mut [u8]) -> Option<usize> {
        self.file.seek(SeekFrom::Start(self.position + offset as u64)).ok()?;
        Some(self.file.read(out).ok()?)
    }

    #[inline]
    pub fn get_block_data(&mut self, out: &mut[u8]) -> Result<usize,()> {
        self.file.seek(SeekFrom::Start(self.position + 16)).ok();
        self.file.read(out).map_err(|_| ())
    }
}