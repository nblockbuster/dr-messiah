#![feature(let_chains)]
mod compression;
mod file;
mod material;
mod model;
mod mpk;
mod texture;
mod version;

use binrw::BinReaderExt;
use clap::Parser;
use mpk::{MpkInfo, ResourceList, ResourcesMpkInfo};
use rayon::prelude::*;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::sync::Mutex;
use version::Version;

#[derive(clap::Parser, Debug, Clone)]
#[command(author, version, about, long_about = None, disable_version_flag(true))]
struct Args {
    /// Path to packages directory
    mpkinfo_path: Option<String>,

    /// Try to find names from "patchlist_android64_low.json" by matching md5 hash in resources
    #[arg(short)]
    patchlist: bool,

    /// Game version for the specified packages directory
    #[arg(short)]
    output_path: Option<String>,

    /// Convert all etsb in path to json for readability
    #[arg(short)]
    etsb_path: Option<String>,

    /// Convert model file to cast file
    #[arg(short)]
    model_path: Option<String>,

    /// Manually decompress a file
    #[arg(short)]
    decompress_path: Option<String>,

    /// Manually convert a texture to dds
    #[arg(short)]
    texture_path: Option<String>,

    #[arg(short)]
    version: Option<Version>,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let mpkinfo_path: PathBuf = if let Some(ref mpkinfo_path) = args.mpkinfo_path {
        mpkinfo_path.into()
    } else {
        PathBuf::new()
    };
    let output_path: PathBuf = if let Some(ref output_path) = args.output_path {
        output_path.into()
    } else {
        let path: PathBuf = mpkinfo_path.clone();
        path.with_extension("")
    };
    let version = if let Some(ref version) = args.version {
        version
    } else {
        &Version::ClosedBeta
    };

    println!("mpkinfo_path: {:#?}", mpkinfo_path);
    println!("output_path: {:#?}", output_path);
    println!("version: {:#?}", version);

    if let Some(texture_path) = args.texture_path {
        texture::export_texture(version, &texture_path)?;
        return Ok(());
    }

    if let Some(decompress_path) = args.decompress_path {
        let decompress_path = PathBuf::from(decompress_path);
        let mut file = File::open(&decompress_path)?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;

        if let Some(compression_type) = compression::get_compression_type(&data) {
            println!("{:?}", compression_type);
            data = compression::decompress(version, compression_type, &data)?;
        } else {
            println!(
                "No compression found with bytes {:X?}/{:?}",
                &data[0x0..0x4],
                std::str::from_utf8(&data[0x0..0x4])?
            );
        }

        let mut output_file = File::create(decompress_path.with_extension("decomp"))?;
        output_file.write_all(&data)?;
        return Ok(());
    }

    if let Some(model_path) = args.model_path {
        model::export_model(&model_path)?;
        return Ok(());
    }

    if args.etsb_path.is_some() {
        for file in std::fs::read_dir(args.etsb_path.unwrap())? {
            let file = file?;
            let file_path = file.path();
            if file_path.is_dir() {
                continue;
            }
            println!("{:?}", file_path);
            let mut file = File::open(file_path.clone())?;
            if file.metadata()?.len() < 4 {
                continue;
            }
            if matches!(
                file_path.extension().unwrap().to_str().unwrap(),
                "etsb" | "monb"
            ) || file.read_le::<u16>()? == 0x537C
            {
                let mut data = Vec::new();
                file.seek(SeekFrom::Start(0))?;
                file.read_to_end(&mut data)?;
                if data[0x0..0x4] == [0x7c, 0x53, 0xb6, 0xc8] {
                    data = data[0x8..].to_vec();
                }
                let etsb: serde_json::Value = rmp_serde::from_slice(&data)?;
                let json = serde_json::to_string_pretty(&etsb)?;
                let mut output_file = File::create(file_path.with_extension("ejson"))?;
                output_file.write_all(json.as_bytes())?;
            }
        }
        return Ok(());
    }

    // TODO: add android_high, android_emulator
    let res_list: ResourceList = if args.patchlist {
        serde_json::from_str(&std::fs::read_to_string("patchlist_android64_low.json")?)?
    } else {
        ResourceList {
            android64_common: HashMap::new(),
            android_low: HashMap::new(),
            common: HashMap::new(),
        }
    };

    let mut res_map: HashMap<String, String> = HashMap::new();
    for (path, hash_size) in res_list.android64_common {
        res_map.insert(
            hash_size[0]
                .clone()
                .as_str()
                .unwrap()
                .trim_matches('"')
                .to_string(),
            path,
        );
    }
    for (path, hash_size) in res_list.android_low {
        res_map.insert(
            hash_size[0]
                .clone()
                .as_str()
                .unwrap()
                .trim_matches('"')
                .to_string(),
            path,
        );
    }
    for (path, hash_size) in res_list.common {
        res_map.insert(
            hash_size[0]
                .clone()
                .as_str()
                .unwrap()
                .trim_matches('"')
                .to_string(),
            path,
        );
    }

    if matches!(
        mpkinfo_path.file_name().unwrap().to_str().unwrap(),
        "Resources.mpkinfo" | "Engine.mpkinfo"
    ) {
        let mut mpkinfo_file = File::open(mpkinfo_path.clone())?;
        let resources: ResourcesMpkInfo = mpkinfo_file.read_le()?;
        println!("{:?}", resources.records.len());

        let mut mpk_path = mpkinfo_path.clone();
        let mut mpk_files = Vec::new();

        mpk_path.set_file_name(mpkinfo_path.with_extension("mpk"));
        if mpk_path.exists() {
            println!("{:?}", mpk_path);
            mpk_files.push(Mutex::new(File::open(mpk_path.clone())?));
        }
        for i in 0..=6 {
            mpk_path.set_file_name(format!("Resources{}.mpk", i));
            if mpk_path.exists() {
                println!("{:?}", mpk_path);
                mpk_files.push(Mutex::new(File::open(mpk_path.clone())?));
            }
        }

        let start = std::time::Instant::now();

        resources
            .records
            .par_iter()
            .try_for_each(|record| -> anyhow::Result<()> {
                let file_index = record.flags >> 1;
                let path = format!("{:08x}.{}", record.unk_hash, record.ext)
                    .to_string()
                    .replace("/", "_");
                let (data, file_path) = mpk::extract_file(
                    &mpk_files[file_index as usize],
                    output_path.clone(),
                    record.asset_size as usize,
                    record.mpk_offset as usize,
                    &path,
                    &res_map,
                    &args,
                    version,
                )?;
                std::fs::create_dir_all(file_path.parent().unwrap())?;
                let mut output_file = File::create(file_path)?;
                output_file.write_all(&data)?;
                Ok(())
            })?;

        println!("Elapsed: {:?}", start.elapsed());

        return Ok(());
    }

    let mut mpkinfo_file = File::open(mpkinfo_path.clone())?;
    let mut mpkinfo_vec = Vec::new();
    while let Ok(info) = mpkinfo_file.read_le::<MpkInfo>() {
        // Files with no name or seemingly data
        if info.md5 == "00000000000000000000000000000000" {
            continue;
        }
        mpkinfo_vec.push(info);
    }
    println!("{:?}", mpkinfo_vec.len());

    let mpk_file = File::open(mpkinfo_path.clone().with_extension("mpk"))?;
    let mpk_file = Mutex::new(mpk_file);

    let start = std::time::Instant::now();

    mpkinfo_vec
        .par_iter()
        .try_for_each(|info| -> anyhow::Result<()> {
            let (data, file_path) = mpk::extract_file(
                &mpk_file,
                output_path.clone(),
                info.data_size as usize,
                info.data_start as usize,
                &info.path,
                &res_map,
                &args,
                version,
            )?;
            std::fs::create_dir_all(file_path.parent().unwrap())?;
            let mut output_file = File::create(file_path).unwrap_or_else(|_| {
                panic!("unable to create file {} | md5: {}", info.path, info.md5)
            });
            output_file.write_all(&data)?;
            Ok(())
        })?;

    println!("Elapsed: {:?}", start.elapsed());
    Ok(())
}
