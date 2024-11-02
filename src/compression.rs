use std::io::Read;

#[derive(Debug, Clone, Copy)]
pub enum CompressionType {
    None,
    LZ4,
    Lzma,
    Zstd,
    Zlib,
    G108Lz4,
    G108Zstd,
    Offset, // actual compression magic is right after b"CCCC"?
}

pub fn get_compression_type(buf: &[u8]) -> Option<CompressionType> {
    match &buf[0..4] {
        b"NNNN" => Some(CompressionType::None),
        &[0xe2, 0x06, ..] => Some(CompressionType::Zlib),
        b"LZMA" => Some(CompressionType::Lzma),
        b"1084" => Some(CompressionType::G108Lz4),
        b"ZZZ4" => Some(CompressionType::LZ4),
        b"108D" => Some(CompressionType::G108Zstd),
        b"ZSTD" => Some(CompressionType::Zstd),
        b"CCCC" => Some(CompressionType::Offset),
        _ => None,
    }
}

pub fn decompress(compression_type: CompressionType, buf: &[u8]) -> Result<Vec<u8>, anyhow::Error> {
    let mut buf = buf.to_vec();
    match compression_type {
        CompressionType::G108Lz4 | CompressionType::G108Zstd => {
            let xor_size = (buf.len() - 8).clamp(0, 256);
            for x in buf[8..8 + xor_size].iter_mut() {
                *x ^= 0x5E;
            }
        }
        _ => {}
    }
    let decsize = u32::from_le_bytes(buf[4..8].try_into().unwrap());
    let mut decompressed = Vec::new();
    match compression_type {
        CompressionType::None => {
            decompressed = buf[0..].to_vec();
        }
        CompressionType::Zlib => {
            let buf = unxor_zlib(&mut buf);
            let mut decoder = flate2::read::ZlibDecoder::new(&buf[0..]);
            decoder.read_to_end(&mut decompressed)?;
        }
        CompressionType::Lzma => {
            let mut reader = std::io::Cursor::new(&buf[8..]);
            let option = lzma_rs::decompress::Options {
                unpacked_size: lzma_rs::decompress::UnpackedSize::UseProvided(Some(decsize.into())),
                memlimit: None,
                allow_incomplete: false,
            };
            lzma_rs::lzma_decompress_with_options(&mut reader, &mut decompressed, &option)?;
        }
        CompressionType::G108Lz4 | CompressionType::LZ4 => {
            decompressed.resize(decsize as usize, 0);
            lz4_flex::decompress_into(&buf[8..], &mut decompressed)?;
        }
        CompressionType::G108Zstd | CompressionType::Zstd => {
            decompressed.resize(decsize as usize, 0);
            decompressed = zstd::decode_all(&buf[8..])?;
        }
        CompressionType::Offset => {
            if let Some(compression_type) = get_compression_type(&buf[0x4..]) {
                decompressed = decompress(compression_type, &buf[0x4..buf.len() - 20])?;
            }
        }
    };
    Ok(decompressed)
}

fn unxor_zlib(buf: &mut [u8]) -> &[u8] {
    let offset = (buf.len() - 8) % 37;
    let end = 128 - offset;
    let end = end.min(buf.len());
    let head = &mut buf[..end];
    for x in head.iter_mut() {
        *x ^= 154;
    }
    let end = if end == buf.len() { end } else { buf.len() - 8 };

    &buf[..end]
}
