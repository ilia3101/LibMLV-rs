use std::io::{BufReader, Seek, SeekFrom};
use std::fs::File;

// fn open_file(path: &str) -> Option<(BufReader<File>, u64)> {
//     let mut file = File::open(path).ok()?;
//     file.seek(SeekFrom::End(0)).ok()?;
//     let size = file.stream_position().ok()?;
//     Some((BufReader::new(file), size))
// }
// let (the_file, file_size) = open_file(file_name).unwrap();
// let mut reader = MLVReader::new(the_file, file_size as usize).unwrap();
// reader.next();

pub fn main()
{
    println!("mlv_file_hdr size: {}", std::mem::size_of::<mlv::blocks::FileHeader>());
    println!("IndexEntry size: {}", std::mem::size_of::<mlv::index::IndexEntry>());
    println!("FileLocation size: {}", std::mem::size_of::<mlv::FileLocation>());
    println!("Option<FileLocation> size: {}", std::mem::size_of::<Option<mlv::FileLocation>>());
    println!("FileLocation alignment: {}", std::mem::align_of::<mlv::FileLocation>());
    // panic!("s");

    let p = mlv::FileLocation::new(12,777777777777);
    if let Some(p) = p {
        println!("{:#?}", p.get_chunk());
        println!("{:#?}", p.get_offset());
    }
    println!("{:#?}", p);

    // /* Reader test */
    let file_name = &std::env::args().collect::<Vec<_>>()[1];
    println!("{file_name}");
    // let file = FileDataSource::open(file_name).unwrap();
    // println!("{file:#?}");

    // let mut reader = mlv::mlv_reader::MLVReader::new(std::io::BufReader::new(std::fs::File::open(file_name).unwrap()), 0);
    let start = std::time::SystemTime::now();
    let mut reader = mlv::MainReader::open_mlv(file_name).unwrap();
    let end = std::time::SystemTime::now();

    println!("reader: {:#?}", reader);
    reader.print_blocks();

    println!("File indexed and loaded in {:.1?} ms", (end.duration_since(start).unwrap().as_micros()) as f64 / 1000.0);
}