use std::{fs::File, path::PathBuf};

use anyhow::{Error, Result};
use binrw::{binread, BinRead, BinReaderExt};
use porter_cast::{CastFile, CastId, CastNode, CastPropertyId};
use porter_math::{Vector2, Vector3};

use crate::file::{MessiahHeader, MessiahTypes};

#[derive(BinRead, Debug, Clone)]
pub struct ModelHeader {
    pub _unk0: u32,
    pub _unk4: u32,
    pub vertex_count: u32,
    pub index_count: u32,
    #[br(count = 4)]
    pub buffer_layouts: Vec<BufferLayout>,
    pub _unk_floats: [f32; 10],
    pub _unk2: [u32; 4],
}

#[binread]
#[derive(Debug, Clone)]
pub struct BufferLayout {
    #[br(temp)]
    len: u16,

    #[br(map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string(), count = len)]
    pub data: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BufferType {
    Position,
    Normal,
    Color,
    Texcoord,
    BlendWeight,
    BlendIndices,
    Tangent,
    Binormal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BufferFormat {
    Float,
    Byte,
    Half,
}

#[derive(Debug, Clone, Copy)]
pub struct Buffer {
    pub buffer_type: BufferType,
    pub size: u8,
    pub buffer_format: BufferFormat,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 4],
    pub color: [f32; 4],
    pub texcoords: Vec<[f32; 2]>,
    pub blend_weight: [f32; 4],
    pub blend_indices: [u32; 4],
    pub tangent: [f32; 4],
    pub binormal: [f32; 4],
}

impl Vertex {
    pub fn combine(&self, other: &Vertex) -> Vertex {
        let mut new = Vertex::default();
        if self.position != [0.0; 3] {
            new.position = self.position;
        } else {
            new.position = other.position;
        }
        if self.normal != [0.0; 4] {
            new.normal = self.normal;
        } else {
            new.normal = other.normal;
        }
        if self.color != [0.0; 4] {
            new.color = self.color;
        } else {
            new.color = other.color;
        }
        if !self.texcoords.is_empty() {
            new.texcoords = self.texcoords.clone();
        } else {
            new.texcoords = other.texcoords.clone();
        }
        if self.blend_weight != [0.0; 4] {
            new.blend_weight = self.blend_weight;
        } else {
            new.blend_weight = other.blend_weight;
        }
        if self.blend_indices != [0; 4] {
            new.blend_indices = self.blend_indices;
        } else {
            new.blend_indices = other.blend_indices;
        }
        if self.tangent != [0.0; 4] {
            new.tangent = self.tangent;
        } else {
            new.tangent = other.tangent;
        }
        if self.binormal != [0.0; 4] {
            new.binormal = self.binormal;
        } else {
            new.binormal = other.binormal;
        }
        new
    }
}

// P3F_N4B_T2F_T2F - Posion 3 Float, Normal 4 Byte, Texcoord 2 Float, Texcoord 2 Float
// T4H_B4H - Tangent 4 Half, Binormal 4 Half

pub fn get_buffers_from_layout(data: &str) -> Vec<Buffer> {
    if data == "None" {
        return Vec::new();
    }
    let mut buffers = Vec::new();
    let a = data.split("_").collect::<Vec<&str>>();

    for b in a {
        let buffer_type = match b.chars().next().unwrap() {
            'P' => BufferType::Position,
            'N' => BufferType::Normal,
            'C' => BufferType::Color,
            'T' => BufferType::Texcoord, // and Tangent
            'W' => BufferType::BlendWeight,
            'I' => BufferType::BlendIndices,
            'B' => BufferType::Binormal,
            _ => panic!("Unknown buffer type: {}", data),
        };

        let size = b.chars().nth(1).unwrap().to_digit(10).unwrap() as u8;

        let buffer_format = match b.chars().nth(2).unwrap() {
            'F' => BufferFormat::Float,
            'B' => BufferFormat::Byte,
            'H' => BufferFormat::Half,
            _ => panic!("Unknown buffer format: {}", data),
        };

        buffers.push(Buffer {
            buffer_type,
            size,
            buffer_format,
        });
    }

    buffers
}

pub fn export_model(model_path: &str) -> Result<(), Error> {
    let mut mfile = File::open(model_path)?;
    let fileheader: MessiahHeader = mfile.read_le()?;
    println!("{:?}", fileheader);
    let model = match fileheader.data {
        MessiahTypes::Model(model) => model,
        _ => panic!("Not a model"),
    };
    // println!("{:?}", model);

    let mut indices: Vec<Vec<u32>> = Vec::new();
    let index_stride = if model.vertex_count > 0xFFFF { 4 } else { 2 };
    for _ in 0..model.index_count / 3 {
        let mut index = Vec::new();
        for _ in 0..3 {
            match index_stride {
                2 => {
                    index.push(mfile.read_le::<u16>()? as u32);
                }
                4 => {
                    index.push(mfile.read_le::<u32>()?);
                }
                _ => {
                    unimplemented!();
                }
            }
        }
        indices.push(index);
    }

    let mut vertices: Vec<Vertex> = vec![Vertex::default(); model.vertex_count as usize];
    for layout in &model.buffer_layouts {
        let buffer = get_buffers_from_layout(&layout.data);
        if buffer.is_empty() {
            continue;
        }

        let t_is_tangent = !(buffer.iter().any(|b| b.buffer_type == BufferType::Texcoord)
            && buffer.iter().any(|b| b.buffer_type == BufferType::Position));

        println!("{:?}", buffer);
        for vid in 0..model.vertex_count {
            let mut vertex = Vertex::default();
            for b in &buffer {
                let mut data = Vec::new();
                //println!("{:?}", b.buffer_type);
                for _ in 0..b.size {
                    match b.buffer_format {
                        BufferFormat::Float => {
                            data.push(mfile.read_le::<f32>()?);
                        }
                        BufferFormat::Byte => {
                            data.push(mfile.read_le::<u8>()? as f32);
                        }
                        BufferFormat::Half => {
                            data.push(mfile.read_le::<u16>()? as f32 / 65535.0);
                        }
                    }
                    //println!("{:?}", data);
                }
                match b.buffer_type {
                    BufferType::Position => {
                        vertex.position = [data[0], data[1], data[2]];
                    }
                    BufferType::Normal => {
                        vertex.normal = [data[0], data[1], data[2], data[3]];
                    }
                    BufferType::Color => {
                        vertex.color = [data[0], data[1], data[2], data[3]];
                    }
                    BufferType::Texcoord => {
                        if t_is_tangent {
                            vertex.tangent = [data[0], data[1], data[2], data[3]];
                        } else {
                            vertex.texcoords.push([data[0], data[1]]);
                        }
                    }
                    BufferType::BlendWeight => {
                        vertex.blend_weight = [data[0], data[1], data[2], data[3]];
                    }
                    BufferType::BlendIndices => {
                        vertex.blend_indices = [
                            data[0] as u32,
                            data[1] as u32,
                            data[2] as u32,
                            data[3] as u32,
                        ];
                    }
                    BufferType::Tangent => {
                        vertex.tangent = [data[0], data[1], data[2], data[3]];
                    }
                    BufferType::Binormal => {
                        vertex.binormal = [data[0], data[1], data[2], data[3]];
                    }
                }
            }
            vertices[vid as usize] = vertices[vid as usize].combine(&vertex);
        }
    }
    println!("{:?}", vertices.len());
    println!("{:?}", vertices.first());
    println!("{:?}", vertices.last());

    let uv_layer_count: u32 = vertices
        .iter()
        .map(|v| v.texcoords.len())
        .max()
        .unwrap_or(0) as u32;

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

    // let norm = mesh.create_property(CastPropertyId::Vector3, "vn");
    // for vertex in &vertices {
    //     norm.push(Vector3::new(
    //         vertex.normal[0],
    //         vertex.normal[1],
    //         vertex.normal[2],
    //     ));
    // }

    let color = mesh.create_property(CastPropertyId::Integer32, "c0");
    for vertex in &vertices {
        let col: u32 = (((vertex.color[0] * 255.0) as u32) << 24)
            | (((vertex.color[1] * 255.0) as u32) << 16)
            | (((vertex.color[2] * 255.0) as u32) << 8)
            | ((vertex.color[3] * 255.0) as u32);
        color.push(col);
    }

    for i in 0..uv_layer_count {
        let uv = mesh.create_property(CastPropertyId::Vector2, format!("u{}", i));
        for vertex in &vertices {
            if i < vertex.texcoords.len() as u32 {
                uv.push(Vector2::new(
                    vertex.texcoords[i as usize][0],
                    vertex.texcoords[i as usize][1],
                ));
            } else {
                uv.push(Vector2::new(0.0, 0.0));
            }
        }
    }

    let weights = mesh.create_property(CastPropertyId::Float, "wv");
    for vertex in &vertices {
        weights.push(vertex.blend_weight[0]);
        weights.push(vertex.blend_weight[1]);
        weights.push(vertex.blend_weight[2]);
        weights.push(vertex.blend_weight[3]);
    }

    let bindices = mesh.create_property(CastPropertyId::Integer32, "wb");
    for vertex in &vertices {
        bindices.push(vertex.blend_indices[0]);
        bindices.push(vertex.blend_indices[1]);
        bindices.push(vertex.blend_indices[2]);
        bindices.push(vertex.blend_indices[3]);
    }

    // let tangent = mesh.create_property(CastPropertyId::Vector3, "vt");
    // for vertex in &vertices {
    //     tangent.push(Vector3::new(
    //         vertex.tangent[0],
    //         vertex.tangent[1],
    //         vertex.tangent[2],
    //     ));
    // }

    let idx = mesh.create_property(CastPropertyId::Integer32, "f");
    indices.iter().flatten().for_each(|i| {
        idx.push(*i);
    });

    castfile.push(root);

    let mut cast_out = File::create(PathBuf::from(model_path).with_extension("cast"))?;
    castfile.write(&mut cast_out)?;
    Ok(())
}
