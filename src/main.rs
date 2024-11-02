#![deny(clippy::correctness, clippy::suspicious, clippy::complexity)]
#![feature(let_chains)]
mod compression;
mod file;
mod model;
mod mpk;
mod material;

use binrw::BinReaderExt;
use clap::Parser;
use mpk::{MpkInfo, ResourcesMpkInfo};
use std::fs::File;
use std::io::{Read, Seek, Write, SeekFrom};
use std::path::PathBuf;
use std::sync::Mutex;
use rayon::prelude::*;

#[derive(clap::Parser, Debug)]
#[command(author, version, about, long_about = None, disable_version_flag(true))]
struct Args {
    /// Path to packages directory
    mpkinfo_path: Option<String>,

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
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let mpkinfo_path: PathBuf = if let Some(mpkinfo_path) = args.mpkinfo_path {
        mpkinfo_path.into()
    } else {
        PathBuf::new()
    };
    let output_path: PathBuf = if let Some(output_path) = args.output_path {
        output_path.into()
    } else {
        let path: PathBuf = mpkinfo_path.clone();
        path.with_extension("")
    };

    println!("mpkinfo_path: {:#?}", mpkinfo_path);
    println!("output_path: {:#?}", output_path);

    if let Some(decompress_path) = args.decompress_path {
        let decompress_path = PathBuf::from(decompress_path);
        let mut file = File::open(&decompress_path)?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;

        if let Some(compression_type) = compression::get_compression_type(&data) {
            println!("{:?}", compression_type);
            data = compression::decompress(compression_type, &data)?;
        } else {
            println!("No compression found with bytes {:X?}/{:?}", &data[0x0..0x4], std::str::from_utf8(&data[0x0..0x4])?);
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
            if file_path.extension().unwrap() == "etsb" || file.read_le::<u16>()? == 0x537C {
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

    if mpkinfo_path.file_name().unwrap() == "Resources.mpkinfo" {
        let mut mpkinfo_file = File::open(mpkinfo_path.clone())?;
        let resources: ResourcesMpkInfo = mpkinfo_file.read_le()?;
        println!("{:?}", resources.records.len());

        let mut mpk_path = mpkinfo_path.clone();
        let mut mpk_files = Vec::new();

        mpk_path.set_file_name("Resources.mpk");
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
                let mut file_path = output_path
                    .clone()
                    .join(format!("{:08x}.{}", record.unk_hash, record.ext));
                let mut data = vec![0; record.asset_size as usize];
                {
                    let file_index = record.flags >> 1;

                    let mut mpk_file = mpk_files[file_index as usize].lock().unwrap();

                    mpk_file.seek(SeekFrom::Start(record.mpk_offset as u64))?;
                    mpk_file.read_exact(&mut data)?;
                }
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
                    if let Ok(extra_decomp_data) =
                        compression::decompress(compression_type, &data[0x38..])
                    {
                        data.truncate(0x38);
                        data.extend_from_slice(&extra_decomp_data);
                    }
                }
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
        mpkinfo_vec.push(info);
    }
    println!("{:?}", mpkinfo_vec.len());

    let mpk_file = File::open(mpkinfo_path.clone().with_extension("mpk"))?;
    let mpk_file = Mutex::new(mpk_file);

    let start = std::time::Instant::now();

    mpkinfo_vec
        .par_iter()
        .try_for_each(|info| -> anyhow::Result<()> {
            let mut file_path = output_path.clone().join(info.path.clone());

            let mut data = vec![0; info.data_size as usize];
            {
                let mut mpk_file = mpk_file.lock().unwrap();
                mpk_file.seek(SeekFrom::Start(info.data_start as u64))?;
                mpk_file.read_exact(&mut data)?;
            }
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
                if let Ok(extra_decomp_data) =
                    compression::decompress(compression_type, &data[0x38..])
                {
                    data.truncate(0x38);
                    data.extend_from_slice(&extra_decomp_data);
                }
            }
            std::fs::create_dir_all(file_path.parent().unwrap())?;
            let mut output_file = File::create(file_path)?;
            output_file.write_all(&data)?;
            Ok(())
        })?;

    println!("Elapsed: {:?}", start.elapsed());
    Ok(())
}
