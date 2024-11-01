use binrw::{binread, BinRead};

#[derive(BinRead, Debug, Clone)]
#[br(magic=b".MESSIAH")]
pub struct ModelHeader {
    pub unk8: u32,
    pub unkc: u32,
    pub unk10: u32,
    pub vertex_count: u32,
    pub index_count: u32,
    #[br(count = 4)]
    pub buffer_layouts: Vec<BufferLayout>,
    pub unk_floats: [f32; 10],
    pub unk2: [u32;4],
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
    // TODO: find out how to separate from texcoord (both 'T', maybe check if earlier layout has 'T' as well as position/color/normal)
    TexcoordTangent,
    Tangent,
    Binormal,
    Color,
    Weights,
    Indices,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BufferFormat {
    Float,
    Byte,
    Short,
    Half
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
    pub normal: [f32; 3],
    // multiple texcoord layers
    pub texcoord: Vec<[f32; 2]>,
    pub tangent: [f32; 4],
    pub binormal: [f32; 4],
    pub color: [f32; 4],
    pub weights: [f32; 4],
}


// P3F_N4B_T2F_T2F - Posion 3 Float, Normal 4 Byte, Texcoord 2 Float, Texcoord 2 Float?
// T4H_B4H - Tangent 4 Half, Bitangent 4 Half?

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
            'T' => BufferType::TexcoordTangent,
            'B' => BufferType::Binormal,
            'C' => BufferType::Color,
            'W' => BufferType::Weights,
            'I' => BufferType::Indices,
            _ => panic!("Unknown buffer type: {}", data),
        };

        let size = b.chars().nth(1).unwrap().to_digit(10).unwrap() as u8;

        let buffer_format = match b.chars().nth(2).unwrap() {
            'F' => BufferFormat::Float,
            'B' => BufferFormat::Byte,
            'S' => BufferFormat::Short,
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