use binrw::{binread, BinRead};
use serde_json::Value;
use std::{
    collections::HashMap,
    fs::File,
    io::{Read, Seek, SeekFrom},
    path::PathBuf,
    sync::Mutex,
};

use crate::{compression, version::Version, Args};

#[binread]
#[derive(Debug, Clone)]
pub struct MpkInfo {
    #[br(temp)]
    pub path_size: u32,
    #[br(map = |s: Vec<u8>| decode_file_path(&s), count = path_size)]
    pub path: String,
    #[br(seek_before = std::io::SeekFrom::Current(0x8))]
    pub data_size: u32,
    #[br(map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string(), count = 0x20, seek_before = std::io::SeekFrom::Current(0x6))]
    pub md5: String,
    #[br(seek_before = std::io::SeekFrom::Current(0x2), pad_after = 0x4)]
    pub data_start: u32,
}

// https://github.com/cohaereo/gwynn/blob/0c159d1ac12427916074cc3358b2fd2ab66ab56e/crates/gwynn-mpk/src/lib.rs#L28
fn decode_file_path(bytes: &[u8]) -> String {
    // println!("path {:?}", bytes);
    if bytes.len() <= 2
        || ((bytes[0] as char).is_alphanumeric()
            && (bytes[1] as char).is_alphanumeric()
            && bytes[2] == b'/')
    {
        // If the first three bytes are alphanumeric followed by a '/', it's a nameless path and we dont need to decrypt it
        // println!("alphanumeric {:?}", String::from_utf8_lossy(bytes));
        String::from_utf8_lossy(bytes).to_string()
    } else {
        let part_size = bytes.len() % 7;
        let mut decoded = String::new();
        for byte in &bytes[0..part_size] {
            let decoded_byte = (byte) ^ 0x2B;
            decoded.push(decoded_byte as char);
        }

        for byte in &bytes[part_size..] {
            let decoded_byte = (byte) ^ 0x35;
            decoded.push(decoded_byte as char);
        }

        // println!("decoded {:?}", decoded);

        decoded
    }
}

#[binread]
#[derive(Debug, Clone)]
#[br(magic = 0x2_u32)] // Version
pub struct ResourcesMpkInfo {
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

// format is { "name": ["hash", size] }

#[derive(serde::Deserialize, Clone, Debug)]
pub struct ResourceList {
    pub android64_common: HashMap<String, Vec<Value>>,
    pub android_low: HashMap<String, Vec<Value>>,
    pub common: HashMap<String, Vec<Value>>,
}

pub fn extract_file(
    mpk_file: &Mutex<File>,
    output_path: PathBuf,
    data_size: usize,
    data_start: usize,
    info_path: &String,
    res_map: &HashMap<String, String>,
    info: &Args,
    version: &Version,
) -> anyhow::Result<(Vec<u8>, PathBuf), anyhow::Error> {
    let mut data = vec![0; data_size];
    {
        let mut mpk_file = mpk_file.lock().unwrap();
        mpk_file.seek(SeekFrom::Start(data_start as u64))?;
        mpk_file.read_exact(&mut data)?;
    }

    let mut file_path = if info.patchlist {
        // needs to be pre-decompression
        let md5_hash = md5::compute(&data);
        let md5_hash = format!("{:x}", md5_hash);

        let file_path = PathBuf::from(res_map.get(&md5_hash).unwrap_or(info_path));
        output_path.join(file_path)
    } else {
        let file_path = PathBuf::from(info_path);
        output_path.join(file_path)
    };

    if file_path.extension().is_none() {
        file_path.set_extension("bin");
    }

    if data.len() > 0x4
        && let Some(compression_type) = compression::get_compression_type(&data[0x0..])
    {
        data = compression::decompress(version, compression_type, &data[0x0..])?;
    }
    // if data.len() > 0x38
    //     && let Some(compression_type) = compression::get_compression_type(&data[0x38..])
    // {
    //     let mut end = data.len();

    //     if file_path.extension().unwrap().to_str().unwrap() == "4" {
    //         end -= 24;
    //     }

    //     if let Ok(extra_decomp_data) = compression::decompress(compression_type, &data[0x38..end]) {
    //         // TODO: is this bad?
    //         data.truncate(0x38);
    //         data.extend_from_slice(&extra_decomp_data);
    //     }
    // }
    Ok((data, file_path))
}
