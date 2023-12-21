use crate::FileLocation;

#[repr(C,packed)]
pub struct IndexEntry {
    /* Basic identifying information */
    block_type: [u8; 4],
    block_size: u32,
    block_timestamp: u64,

    /* Where the block is, file location and block number */
    location: FileLocation,

    /* One block may have a few index entries to store the entire block's
     * data in the index (for blocks below a size threshold) */
    entry_number: u8,

    /* Data (excluding the first 16 bytes, as that's already contained in this struct) */
    data: [u8; 41]
}

#[repr(packed)]
pub struct BlockInfo2 {
    pub timestamp: u64,
    pub location: FileLocation,
    pub entry_number: u8,
    pub data: [u8; 17]
}

#[derive(Clone,Copy,PartialEq,Eq)]
pub enum BlockType2 {
    FILE,
    VIDF,
    AUDF,
    RAWI,
    WAVI,
    EXPO,
    LENS,
    RTCI,
    IDNT,
    INFO,
    DISO,
    NULL,
    ELVL,
    WBAL,
    STYL,
    MARK,
    VERS,
    Other([u8;4])
}

