use std::{
    collections::HashMap,
    fs::File,
    io::{Read, Seek, SeekFrom},
    path::PathBuf,
    sync::Mutex,
};
use sha1::{Sha1, Digest};
use binrw::{binread, BinRead};
use serde_json::Value;

use crate::compression;

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
    pub md5: String,
    #[br(seek_before = std::io::SeekFrom::Current(0x2), pad_after = 0x4)]
    pub data_start: u32,
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
    use_patchlist: bool,
    res_map: &HashMap<String, String>,
) -> anyhow::Result<(Vec<u8>, PathBuf), anyhow::Error> {
    let mut data = vec![0; data_size];
    {
        let mut mpk_file = mpk_file.lock().unwrap();
        mpk_file.seek(SeekFrom::Start(data_start as u64))?;
        mpk_file.read_exact(&mut data)?;
    }

    let mut file_path = if use_patchlist {
        // needs to be pre-decompression
        let md5_hash = md5::compute(&data);
        let md5_hash = format!("{:x}", md5_hash);

        let mut file_path = PathBuf::from(res_map.get(&md5_hash).unwrap_or(info_path));
        if file_path == PathBuf::from(info_path) {
            let uuid_name = PathBuf::from(info_path)
                .file_stem()
                .unwrap()
                .to_string_lossy()
                .to_string();
            let uuid = uuid::Uuid::parse_str(&uuid_name);
            if let Ok(uuid) = uuid {
                if uuid.get_version() == Some(uuid::Version::Sha1) {
                    let mut found = false;
                    for (name, _) in res_map.iter() {
                        let mut hasher = Sha1::new();
                        hasher.update(name.as_bytes());
                        let hash = hasher.finalize();
                        let mut sha1_trunc_data: [u8; 16] = Default::default();
                        sha1_trunc_data.copy_from_slice(&hash[0..16]);
                        let name_uuid = uuid::Builder::from_sha1_bytes(sha1_trunc_data).into_uuid();
                        if name_uuid == uuid {
                            println!("Found UUID match for {:?} in resource list", info_path);
                            file_path = PathBuf::from(name);
                            found = true;
                            break;
                        }
                    }
                    if !found {
                        file_path = PathBuf::from(info_path);
                    }
                }
            };
        }
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
        data = compression::decompress(compression_type, &data[0x0..])?;
    }
    if data.len() > 0x38
        && let Some(compression_type) = compression::get_compression_type(&data[0x38..])
    {
        if let Ok(extra_decomp_data) = compression::decompress(compression_type, &data[0x38..]) {
            // TODO: is this bad?
            data.truncate(0x38);
            data.extend_from_slice(&extra_decomp_data);
        }
    }
    Ok((data, file_path))
}
