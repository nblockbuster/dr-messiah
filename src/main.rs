#![feature(let_chains)]
mod compression;
mod model;
mod mpk;

use binrw::BinReaderExt;
use clap::Parser;
use model::Vertex;
use mpk::{MpkInfo, ResourcesMpkInfo};
use porter_cast::{CastFile, CastId, CastNode, CastPropertyId};
use porter_math::{Vector2, Vector3};
use std::fs::File;
use std::io::{Read, Seek, Write};
use std::path::PathBuf;
use std::sync::Mutex;

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

    /// Convert model file to <collada?>
    #[arg(short)]
    model_path: Option<String>,
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

    if let Some(model_path) = args.model_path {
        let mut mfile = File::open(model_path.clone())?;
        let model: model::ModelHeader = mfile.read_le()?;
        println!("{:?}", model);

        // check all buffer layouts for indices buffer
        let mut indices_buffer = None;
        for layout in &model.buffer_layouts {
            let buffer = model::get_buffers_from_layout(&layout.data);
            for b in buffer.clone() {
                if b.buffer_type == model::BufferType::Indices {
                    indices_buffer = Some(b);
                    break;
                }
            }
            if indices_buffer.is_some() {
                break;
            }
        }

        // default u16, if buffer exists then use buffer size
        let mut indices: Vec<Vec<u32>> = Vec::new();
        for _ in 0..model.index_count / 3 {
            let mut index = Vec::new();
            for _ in 0..3 {
                if let Some(indices_buffer) = indices_buffer {
                    match indices_buffer.size {
                        0x4 => {
                            index.push(mfile.read_le::<u32>()?);
                        }
                        _ => {
                            unimplemented!();
                        }
                    }
                } else {
                    index.push(mfile.read_le::<u16>()? as u32);
                }
            }

            indices.push(index);
        }
        println!("{:?}", indices.len());
        println!("{:?}", indices.last());

        let mut vertices: Vec<Vertex> = vec![Vertex::default(); model.vertex_count as usize];
        for layout in &model.buffer_layouts {
            let buffer = model::get_buffers_from_layout(&layout.data);
            if buffer.is_empty() {
                continue;
            }
            if buffer
                .iter()
                .any(|b| b.buffer_type == model::BufferType::Indices)
            {
                continue; // covered earlier
            }
            if buffer
                .iter()
                .any(|b| b.buffer_type == model::BufferType::Binormal)
            {
                continue; // todo: implement binormals/tangent
            }
            println!("{:?}", buffer);
            for vid in 0..model.vertex_count {
                let mut vertex = Vertex::default();
                for b in &buffer {
                    let mut data = Vec::new();
                    for _ in 0..b.size {
                        match b.buffer_format {
                            model::BufferFormat::Float => {
                                data.push(mfile.read_le::<f32>()?);
                            }
                            model::BufferFormat::Byte => {
                                data.push(mfile.read_le::<u8>()? as f32 / 255.0);
                            }
                            model::BufferFormat::Half => {
                                data.push(mfile.read_le::<u16>()? as f32 / 65535.0);
                            }
                            // model::BufferFormat::Half => {
                            //     data.push(half::f16::from_bits(mfile.read_le::<u16>()?).to_f32());
                            // }
                            _ => {
                                unimplemented!();
                            }
                        }
                        // println!("{:?}", data);
                    }
                    match b.buffer_type {
                        model::BufferType::Position => {
                            vertex.position = [data[0], data[1], data[2]];
                        }
                        model::BufferType::Normal => {
                            vertex.normal = [data[0], data[1], data[2]];
                        }
                        model::BufferType::TexcoordTangent => {
                            vertex.texcoord.push([data[0], data[1]]);
                        }
                        model::BufferType::Tangent => {
                            vertex.tangent = [data[0], data[1], data[2], data[3]];
                        }
                        model::BufferType::Binormal => {
                            vertex.binormal = [data[0], data[1], data[2], data[3]];
                        }
                        model::BufferType::Color => {
                            vertex.color = [data[0], data[1], data[2], data[3]];
                        }
                        model::BufferType::Weights => {
                            vertex.weights = [data[0], data[1], data[2], data[3]];
                        }
                        _ => {}
                    }
                }
                vertices[vid as usize] = vertex;
            }
        }
        println!("{:?}", vertices.len());
        println!("{:?}", vertices.first());
        println!("{:?}", vertices.last());

        let uv_layer_count: u32 =
            vertices.iter().map(|v| v.texcoord.len()).max().unwrap_or(0) as u32;

        let mut castfile = CastFile::new();
        let mut root = CastNode::root();
        let mesh = root.create(CastId::Model).create(CastId::Mesh);

        let uvlayers = mesh.create_property(CastPropertyId::Integer32, "ul");
        uvlayers.push(uv_layer_count);

        let colorlayers = mesh.create_property(CastPropertyId::Integer32, "cl");
        colorlayers.push(1_u32);

        let pos = mesh.create_property(CastPropertyId::Vector3, "vp");
        for vertex in &vertices {
            pos.push(Vector3::new(
                vertex.position[0],
                vertex.position[1],
                vertex.position[2],
            ));
        }

        let norm = mesh.create_property(CastPropertyId::Vector3, "vn");
        for vertex in &vertices {
            norm.push(Vector3::new(
                vertex.normal[0],
                vertex.normal[1],
                vertex.normal[2],
            ));
        }

        let color = mesh.create_property(CastPropertyId::Integer32, "c0");
        for vertex in &vertices {
            let col: u32 = ((vertex.color[0] * 255.0) as u32) << 24
                | ((vertex.color[1] * 255.0) as u32) << 16
                | ((vertex.color[2] * 255.0) as u32) << 8
                | ((vertex.color[3] * 255.0) as u32);
            color.push(col);
        }

        for i in 0..uv_layer_count {
            let uv = mesh.create_property(CastPropertyId::Vector2, format!("u{}", i));
            for vertex in &vertices {
                if i < vertex.texcoord.len() as u32 {
                    uv.push(Vector2::new(
                        vertex.texcoord[i as usize][0],
                        vertex.texcoord[i as usize][1],
                    ));
                } else {
                    uv.push(Vector2::new(0.0, 0.0));
                }
            }
        }

        // let binormal = mesh.create_property(CastPropertyId::Vector3, "vt");
        // for vertex in &vertices {
        //     binormal.push(Vector3::new(
        //         vertex.tangent[0],
        //         vertex.tangent[1],
        //         vertex.tangent[2],
        //     ));
        // }

        // let uv = mesh.create_property(CastPropertyId::Vector2, "u0");
        // for vertex in &vertices {
        //     uv.push(Vector2::new(vertex.texcoord[0], vertex.texcoord[1]));
        // }

        let mut idx = mesh.create_property(CastPropertyId::Integer32, "f");
        for index in indices.iter() {
            idx = idx.push(index[0]);
            idx = idx.push(index[1]);
            idx = idx.push(index[2]);
        }

        castfile.push(root);

        let mut cast_out = File::create(PathBuf::from(model_path).with_extension("cast"))?;
        castfile.write(&mut cast_out)?;

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
                file.seek(std::io::SeekFrom::Start(0))?;
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
            mpk_files.push(File::open(mpk_path.clone())?);
        }
        for i in 0..=6 {
            mpk_path.set_file_name(format!("Resources{}.mpk", i));
            if mpk_path.exists() {
                println!("{:?}", mpk_path);
                mpk_files.push(File::open(mpk_path.clone())?);
            }
        }

        resources
            .records
            .iter()
            .try_for_each(|record| -> anyhow::Result<()> {
                let mut file_path = output_path
                    .clone()
                    .join(format!("{:08x}.{}", record.unk_hash, record.ext));
                let mut data = vec![0; record.asset_size as usize];
                {
                    let file_index = record.flags >> 1;

                    let mut mpk_file = mpk_files[file_index as usize].try_clone()?;

                    mpk_file.seek(std::io::SeekFrom::Start(record.mpk_offset as u64))?;
                    mpk_file.read_exact(&mut data)?;
                    // if data.len() > 0x4 {
                    //     println!("{:X?} ({})", &data[0x0..0x4], String::from_utf8_lossy(&data[0x0..0x4]));
                    // }
                }
                if file_path.extension().is_none() {
                    file_path.set_extension("bin");
                }
                // if file_path.extension().unwrap() == "4" {
                //     file_path.set_extension("tex");
                // }
                if data.len() > 0x4 {
                    if let Some(compression_type) = compression::get_compression_type(&data[0x0..])
                    {
                        data = compression::decompress(compression_type, &data[0x0..])?;
                    }
                }
                if data.len() > 0x38 {
                    if let Some(compression_type) = compression::get_compression_type(&data[0x38..])
                    {
                        if let Ok(extra_decomp_data) =
                            compression::decompress(compression_type, &data[0x38..])
                        {
                            data.truncate(0x38);
                            data.extend_from_slice(&extra_decomp_data);
                        }
                    }
                }
                std::fs::create_dir_all(file_path.parent().unwrap())?;
                let mut output_file = File::create(file_path)?;
                output_file.write_all(&data)?;
                Ok(())
            })?;

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
        .iter_mut()
        .try_for_each(|info| -> anyhow::Result<()> {
            let mut file_path = output_path.clone().join(info.path.clone());

            let mut data = vec![0; info.data_size as usize];
            {
                let mut mpk_file = mpk_file.lock().unwrap();
                mpk_file.seek(std::io::SeekFrom::Start(info.data_start as u64))?;
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
            if data.len() > 0x38 {
                if let Some(compression_type) = compression::get_compression_type(&data[0x38..]) {
                    if let Ok(extra_decomp_data) =
                        compression::decompress(compression_type, &data[0x38..])
                    {
                        data.truncate(0x38);
                        data.extend_from_slice(&extra_decomp_data);
                    }
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
