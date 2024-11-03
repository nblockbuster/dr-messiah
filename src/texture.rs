use std::{io::Read, path::Path};

use binrw::{BinRead, BinReaderExt};

#[derive(BinRead, Debug, Clone)]
#[br(repr = u8)]
enum SamplerFilter {
    FNone = 0,
    Point = 1,
    Linear = 2,
    Anisotropic = 3,
}

#[derive(BinRead, Debug, Clone)]
#[br(repr = u8)]
enum SampleAddress {
    ANone = 0,
    Wrap = 1,
    Mirror = 2,
    Clamp = 3,
    FromTexture = 4,
}

#[derive(BinRead, Debug, Clone)]
#[br(repr = u8)]
enum PixelFormat {
    R32G32B32A32 = 3,
    A16B16G16R16 = 4,
    R8G8B8A8 = 5,
    B5G6R5 = 6,
    A8L8 = 7,
    G16R16 = 8,
    G16R16F = 9,
    G32R32F = 10,
    R16F = 11,
    L8 = 12,
    L16 = 13,
    A8 = 14,
    FloatRGB = 15,
    FloatRGBA = 255, // ??
    D24 = 16,
    D32 = 17,
    BC1 = 18,
    BC2 = 19,
    BC3 = 20,
    BC4 = 21,
    BC5 = 22,
    BC6S = 23,
    BC6U = 24,
    BC7 = 25,
    PVRTC2_RGB = 27,
    PVRTC2_RGBA = 28,
    PVRTC4_RGB = 29,
    PVRTC4_RGBA = 30,
    ETC1 = 31,
    ETC2RGB = 32,
    ETC2RGBA = 33,
    ATC_RGBA_E = 34,
    ATC_RGBA_I = 35,
    ASTC_4x4_LDR = 36,
    ASTC_5x4_LDR = 37,
    ASTC_5x5_LDR = 38,
    ASTC_6x5_LDR = 39,
    ASTC_6x6_LDR = 40,
    ASTC_8x5_LDR = 41,
    ASTC_8x6_LDR = 42,
    ASTC_8x8_LDR = 43,
    ASTC_10x5_LDR = 44,
    ASTC_10x6_LDR = 45,
    ASTC_10x8_LDR = 46,
    ASTC_10x10_LDR = 47,
    ASTC_12x10_LDR = 48,
    ASTC_12x12_LDR = 49,
    ShadowDepth = 50,
    ShadowDepth32 = 51,
    R10G10B10A2 = 52,
    R32U = 53,
    R11G11B10 = 54,
    ASTC_4x4_HDR = 55,
    ASTC_5x4_HDR = 56,
    ASTC_5x5_HDR = 57,
    ASTC_6x5_HDR = 58,
    ASTC_6x6_HDR = 59,
    ASTC_8x5_HDR = 60,
    ASTC_8x6_HDR = 61,
    ASTC_8x8_HDR = 62,
    ASTC_10x5_HDR = 63,
    ASTC_10x6_HDR = 64,
    ASTC_10x8_HDR = 65,
    ASTC_10x10_HDR = 66,
    ASTC_12x10_HDR = 67,
    ASTC_12x12_HDR = 68,
    R32G32B32A32UI = 69,
}

#[derive(BinRead, Debug, Clone)]
#[br(repr = u8)]
enum TextureType {
    Texture1D = 0,
    Texture2D = 1,
    Texture3D = 2,
    Cube = 3,
    Texture2DArray = 4,
    CubeArray = 5,
    Array = 6,
}

#[derive(BinRead, Debug, Clone)]
#[br(repr = u8)]
enum TextureCompressionPresets {
    Default = 0,
    NormalMap = 1,
    DisplacementMap = 2,
    Grayscale = 3,
    HDR = 4,
    NormalMapUncompress = 5,
    NormalMapBC5 = 6,
    VectorMap = 7,
    Uncompressed = 8,
    LightMap = 9,
    EnvMap = 10,
    MixMap = 11,
    UI = 12,
    TerrainBlock = 13,
    TerrainIndex = 14,
    NormalMapCompact = 15,
    cBC6H = 16,
    cBC7 = 17,
    LightProfile = 18,
    LUTHDR = 19,
    LUTLOG = 20,
    TerrainNormalMap = 21,
}

#[derive(BinRead, Debug, Clone)]
#[br(repr = u8)]
enum ETextureLODGroup {
    World = 0,
    WorldNormalMap = 1,
    WorldSpecular = 2,
    Character = 3,
    CharacterNormalMap = 4,
    CharacterSpecular = 5,
    Weapon = 6,
    WeaponNormalMap = 7,
    WeaponSpecular = 8,
    Cinematic = 9,
    Effect = 10,
    EffectUnfiltered = 11,
    Sky = 12,
    Gui = 13,
    RenderTarget = 14,
    ShadowMap = 15,
    LUT = 16,
    TerrainBlockMap = 17,
    TerrainIndexMap = 18,
    TerrainLightMap = 19,
    ImageBaseReflection = 20,
}

#[derive(BinRead, Debug, Clone)]
#[br(repr = u8)]
enum ETextureMipGen {
    FromTextureGroup = 0,
    Simple = 1,
    Sharpen = 2,
    NoMip = 3,
    Blur = 4,
    AlphaDistribution = 5,
}

// #[bitfield]
// #[derive(BinRead, Debug, Clone)]
// #[br(map = Self::from_bytes)]
// pub struct Flags {
//     is_srgb: bool,
//     is_dynamic_range: bool,
//     none_compression: bool,
//     compression_no_alpha: bool,
//     none_mip_downgrading: bool,
//     _reserved: B3,
// }

#[derive(BinRead, Debug, Clone)]
pub struct TexHeader {
    mag_filter: SamplerFilter,
    min_filter: SamplerFilter,
    mip_filter: SamplerFilter,
    address_u: SampleAddress,
    address_v: SampleAddress,
    fmt: PixelFormat,
    miplevel: u8,
    flags: u8,
    compression_preset: TextureCompressionPresets,
    lod_group: ETextureLODGroup,
    mip_gen_preset: ETextureMipGen,
    texture_type: TextureType,
    width: u16,
    height: u16,
    default_color: [f32; 4],
    size: u32,
    unk: u16,
    slice_count: u16,
}

#[derive(BinRead, Debug, Clone)]
pub struct TextureSliceInfo {
    size: u32,
    width: u16,
    height: u16,
    depth: u16,
    pitch_in_byte: u16,
    slice_in_byte: u32,
}

pub fn export_texture(texture_path: &str) -> anyhow::Result<(), anyhow::Error> {
    let mut file = std::fs::File::open(texture_path)?;
    let header: TexHeader = file.read_le()?;
    println!("{:#?}", header);

    for i in 0..header.slice_count {
        let slice_info: TextureSliceInfo = file.read_le()?;
        println!("{:#?}", slice_info);

        if slice_info.slice_in_byte == 0 {
            println!("Empty slice");
            continue;
        }

        let mut data = vec![0; slice_info.slice_in_byte as usize];
        file.read_exact(&mut data)?;
    
        
        let mut image: Vec<u32> = vec![0; slice_info.width as usize * slice_info.height as usize];
        match header.fmt {
            PixelFormat::ASTC_10x10_LDR
            | PixelFormat::ASTC_10x10_HDR => {
                texture2ddecoder::decode_astc_10_10(&data, slice_info.width as usize, slice_info.height as usize, &mut image).unwrap();
            }
            PixelFormat::ASTC_10x5_LDR
            | PixelFormat::ASTC_10x5_HDR => {
                texture2ddecoder::decode_astc_10_5(&data, slice_info.width as usize, slice_info.height as usize, &mut image).unwrap();
            }
            PixelFormat::ASTC_10x6_LDR
            | PixelFormat::ASTC_10x6_HDR => {
                texture2ddecoder::decode_astc_10_6(&data, slice_info.width as usize, slice_info.height as usize, &mut image).unwrap();
            }
            PixelFormat::ASTC_10x8_LDR
            | PixelFormat::ASTC_10x8_HDR => {
                texture2ddecoder::decode_astc_10_8(&data, slice_info.width as usize, slice_info.height as usize, &mut image).unwrap();
            }
            PixelFormat::ASTC_12x10_LDR
            | PixelFormat::ASTC_12x10_HDR => {
                texture2ddecoder::decode_astc_12_10(&data, slice_info.width as usize, slice_info.height as usize, &mut image).unwrap();
            }
            PixelFormat::ASTC_12x12_LDR
            | PixelFormat::ASTC_12x12_HDR => {
                texture2ddecoder::decode_astc_12_12(&data, slice_info.width as usize, slice_info.height as usize, &mut image).unwrap();
            }
            PixelFormat::ASTC_4x4_LDR
            | PixelFormat::ASTC_4x4_HDR => {
                texture2ddecoder::decode_astc_4_4(&data, slice_info.width as usize, slice_info.height as usize, &mut image).unwrap();
            }
            PixelFormat::ASTC_5x4_LDR
            | PixelFormat::ASTC_5x4_HDR => {
                texture2ddecoder::decode_astc_5_4(&data, slice_info.width as usize, slice_info.height as usize, &mut image).unwrap();
            }
            PixelFormat::ASTC_5x5_LDR
            | PixelFormat::ASTC_5x5_HDR => {
                texture2ddecoder::decode_astc_5_5(&data, slice_info.width as usize, slice_info.height as usize, &mut image).unwrap();
            }
            PixelFormat::ASTC_6x5_LDR
            | PixelFormat::ASTC_6x5_HDR => {
                texture2ddecoder::decode_astc_6_5(&data, slice_info.width as usize, slice_info.height as usize, &mut image).unwrap();
            }
            PixelFormat::ASTC_6x6_LDR
            | PixelFormat::ASTC_6x6_HDR => {
                texture2ddecoder::decode_astc_6_6(&data, slice_info.width as usize, slice_info.height as usize, &mut image).unwrap();
            }
            PixelFormat::ASTC_8x5_LDR
            | PixelFormat::ASTC_8x5_HDR => {
                texture2ddecoder::decode_astc_8_5(&data, slice_info.width as usize, slice_info.height as usize, &mut image).unwrap();
            }
            PixelFormat::ASTC_8x6_LDR
            | PixelFormat::ASTC_8x6_HDR => {
                texture2ddecoder::decode_astc_8_6(&data, slice_info.width as usize, slice_info.height as usize, &mut image).unwrap();
            }
            PixelFormat::ASTC_8x8_LDR
            | PixelFormat::ASTC_8x8_HDR => {
                texture2ddecoder::decode_astc_8_8(&data, slice_info.width as usize, slice_info.height as usize, &mut image).unwrap();
            }
            _ => {
                println!("Unsupported format {:?}", header.fmt);
            }
        }

        let mut img = image::ImageBuffer::new(slice_info.width as u32, slice_info.height as u32);
        for (x, y, pixel) in img.enumerate_pixels_mut() {
            let color = image[(y * slice_info.width as u32 + x) as usize];
            *pixel = image::Rgba([
                ((color >> 16) & 0xFF) as u8,
                ((color >> 8) & 0xFF) as u8,
                (color & 0xFF) as u8,
                ((color >> 24) & 0xFF) as u8,
            ]);
        }
        
        let output_path = Path::new(texture_path).with_extension("png");
        img.save(output_path)?;
    }

    Ok(())
}