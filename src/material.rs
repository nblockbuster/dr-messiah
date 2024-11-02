use binrw::binread;
use std::io::SeekFrom;

#[binread]
#[derive(Debug, Clone)]
pub struct MaterialHeader {
    #[br(seek_before=SeekFrom::Current(0x3), temp)]
    _size: u16,
    #[br(map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string(), count = _size)]
    pub id1: String,
    #[br(temp)]
    _size: u16,
    #[br(map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string(), count = _size)]
    pub id2: String,
    pub unk1: u32,
    #[br(temp)]
    _size: u16,
    #[br(map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string(), count = _size)]
    pub data: String,
}
