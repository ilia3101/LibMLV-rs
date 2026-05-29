use crate::BlockHeader;
use crate::blocks::{block_get_type, block_get_timestamp, block_get_size};

#[derive(Debug,Copy,Clone)]
pub enum ReadBlocksError<ReadErrorType> {
    BlockCutoff,
    TrailingBytes,
    ImpossiblySmallBlockSize,
    FileTooSmall,
    ReadError(ReadErrorType),
}

/* TODO: make this an std-gated feature */
use std::io::{self, Read, Seek, SeekFrom};
pub fn read_wrapper<File>(mut file: File) -> impl FnMut(u64, &mut [u8]) -> io::Result<()>
where
    File: Read + Seek
{
    let mut pos = 0u64;
    file.rewind();

    return move |read_pos: u64, out: &mut [u8]| {
        if read_pos != pos {
            file.seek_relative(read_pos as i64 - pos as i64)?;
        }
        pos = read_pos + out.len() as u64;
        file.read_exact(out).map(|_| ())
    }
}

pub fn read_blocks<const MAX_BLOCK_BYTES: usize, ReadError>(
    file_length: u64,
    /* Pos, out buffer. If this returns error, the function returns an error and exits */
    mut read_exact: impl FnMut(u64, &mut [u8]) -> Result<(), ReadError>,
    /* Return true to continue, false to exit early, args: data, block offset */
    mut block_data_callback: impl FnMut(&[u8], u64) -> bool,
) -> Result<(), ReadBlocksError<ReadError>> {
    let mut buf = [0u8; MAX_BLOCK_BYTES];
    let mut pos = 0u64;

    if file_length < 16 {
        return Err(ReadBlocksError::FileTooSmall)
    }

    loop {
        /* Due to checks, this should always succeed */
        if let Err(e) = read_exact(pos, &mut buf[0..16]) {
            return Err(ReadBlocksError::ReadError(e))
        }
        let block_size = u32::from_le_bytes([buf[4],buf[5],buf[6],buf[7]]);

        if block_size < 16 {
            return Err(ReadBlocksError::ImpossiblySmallBlockSize)
        }

        let next_block_pos = pos + block_size as u64;

        if next_block_pos > file_length {
            println!("File length = {file_length}, nextpos = {next_block_pos}");
            // TODO: Add mechanism for leniency to cut-off blocks? Eg slightly cut off final frame
            return Err(ReadBlocksError::BlockCutoff)
        } else {
            let read_end = (block_size as usize).min(MAX_BLOCK_BYTES);
            if let Err(e) = read_exact(pos+16, &mut buf[16..read_end]) {
                return Err(ReadBlocksError::ReadError(e))
            }
            block_data_callback(&buf[0..read_end], pos);
            if next_block_pos == file_length {
                /* End of file! */
                return Ok(())
            } else if file_length - next_block_pos < 16 {
                /* Data after next block is less than 16 bytes which is the minimum size for a block */
                return Err(ReadBlocksError::TrailingBytes)
            } else { /* Fine */ }

            pos = next_block_pos;
        }
    }
}
