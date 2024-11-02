use binrw::{binread, BinRead};

#[binread]
#[derive(Debug, Clone)]
pub struct MpkInfo {
    #[br(temp)]
    pub path_size: u32,
    #[br(map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string(), count = path_size)]
    pub path: String,
    #[br(seek_before = std::io::SeekFrom::Current(0x8))]
    pub data_size: u32,
    #[br(map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string(), count = 0x20, seek_before = std::io::SeekFrom::Current(0x6))]
    pub unk16: String,
    #[br(seek_before = std::io::SeekFrom::Current(0x2), pad_after = 0x4)]
    pub data_start: u32,
}

#[binread]
#[derive(Debug, Clone)]
pub struct ResourcesMpkInfo {
    pub version: u32,
    #[br(temp)]
    pub record_num: u32,
    #[br(count = record_num)]
    pub records: Vec<ResourcesMpkRecord>,
}

#[derive(BinRead, Debug, Clone)]
pub struct ResourcesMpkRecord {
    pub asset_size: u32,
    pub flags: u32,
    pub unk: u8,
    #[br(map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string(), count = 0x3)]
    pub ext: String,
    pub unk_hash: u32,
    pub mpk_offset: u32,
}
