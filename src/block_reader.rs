use crate::{Read, Seek, SeekFrom};
use crate::BlockHeader;

/* TODO: completely refactor this whole idea */

/* TODO: better end-of-file, IO error and other error handling */
#[derive(Debug)]
pub struct BlockReader<Reader, const MAX_BYTES: usize = 256> {
    pub file: Reader,
    block: Option<BlockHeader>,
    block_data: [u8; MAX_BYTES],
    position: u64,
}

impl<Reader: Read + Seek, const MAX_BYTES: usize> BlockReader<Reader, MAX_BYTES>
{
    /* TODO: use these arguments: (chunks: impl AsRef<[impl Read]>) */
    pub fn new(file: Reader) -> Option<Self> {
        let mut block_reader = Self { file, block_data: [0;MAX_BYTES], block: None, position: 0 };
        block_reader.file.seek(SeekFrom::Start(0)).ok()?;
        block_reader.next_block()?;
        Some(block_reader)
    }

    #[inline]
    pub fn next_block(&mut self) -> Option<BlockHeader> {
        self.position += self.block.map(|b| b.block_size).unwrap_or(0) as u64;
        let mut bytes = [0; 16];
        self.block = self.file.read_exact(&mut bytes).map(|_| BlockHeader::from_bytes(bytes)).ok();
        if let Some(header) = self.block {
            if header.block_size >= 16 {
                if header.block_size as usize <= MAX_BYTES {
                    header.to_bytes(&mut self.block_data[0..16]);
                    self.file.read_exact(&mut self.block_data[16..(header.block_size as usize)]).ok()?;
                } else {
                    header.to_bytes(&mut self.block_data[0..16]);
                    self.file.read_exact(&mut self.block_data[16..]).ok()?;
                    self.file.seek(SeekFrom::Current((header.block_size - (MAX_BYTES as u32)) as i64)).ok()?;
                }
            } else {
                /* Impossibly small block size. TODO: use an error type here? */
                return None;
            }
        } return self.block;
    }

    /* Returns up to MAX_BYTES bytes of the current block (excluding 16-byte header) */
    #[inline]
    pub fn block_bytes(&mut self) -> Option<&[u8]> {
        Some(&self.block_data[0..(self.block.as_ref()?.block_size as usize).min(MAX_BYTES)])
    }

    #[inline]
    pub fn block_position(&self) -> u64 { self.position }

    #[inline]
    pub fn block_info(&self) -> Option<BlockHeader> { self.block }
}