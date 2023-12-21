use crate::index::IndexEntry;
use crate::DataSource;

use std::io::{Read, Seek, SeekFrom, BufReader};
use std::fs::File;

#[derive(Debug)]
pub struct FileDataSource<FILE = BufReader<File>> {
    /* Chunk files and their sizes */
    chunks: Vec<(FILE, u64)>,
    /* Chunk's file paths */
    paths: Vec<String>,
}

impl<FILE> DataSource for FileDataSource<FILE>
where
    FILE: Read + Seek
{
    #[inline]
    fn get_num_chunks(&self) -> u8 {
        let num_chunks = self.chunks.len();
        assert!(num_chunks <= 101);
        num_chunks as u8
    }

    #[inline]
    fn get_chunk_size(&self, chunk: u8) -> Option<u64> {
        Some(self.chunks.get(chunk as usize)?.1)
    }

    #[inline]
    fn read(&mut self, chunk: u8, position: u64, output: &mut [u8]) -> u64 {
        if let Some((ref mut file, size)) = self.chunks.get_mut(chunk as usize) {
            if file.seek(SeekFrom::Start(position)).is_ok() {
                file.read(output).unwrap_or(0) as u64
            } else {0}
        } else {0}
    }
}

impl FileDataSource<BufReader<File>> {
    pub fn open(path: &str) -> Option<Self> {
        fn open_file(path: &str) -> Option<(BufReader<File>, u64)> {
            let mut file = File::open(path).ok()?;
            file.seek(SeekFrom::End(0)).ok()?;
            let size = file.stream_position().ok()?;
            Some((BufReader::new(file), size))
        }
        let mut chunks = vec![];
        let mut paths = vec![];
        chunks.push(open_file(path)?);
        paths.push(path.to_string());
        Some(Self{chunks, paths})
    }
    // /* Check multiple cards to support card spanning */
    // pub fn smart_open(path: &str) -> Self {
    //     /* Search for all chunks in directory and on other cards/drives */
    //     /* TODO: use operating system APIs here maybe */
    //     todo!()
    // }
}

impl Clone for FileDataSource<BufReader<File>> {
    fn clone(&self) -> Self {
        todo!()
    }
}
