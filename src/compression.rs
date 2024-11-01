#[derive(Debug, Clone, Copy)]
pub enum CompressionType {
    LZ4,
    Lzma,
    Zstd,
    Zlib,
    G108Lz4,
    G108Zstd,
}

pub fn get_compression_type(buf: &[u8]) -> Option<CompressionType> {
    match &buf[0..4] {
        //&[0xe2, 0x06, ..] => Some(CompressionType::Zlib),
        //b"LZMA" => Some(CompressionType::Lzma),
        b"0184" => Some(CompressionType::G108Lz4),
        b"ZZZ4" => Some(CompressionType::LZ4),
        b"108D" => Some(CompressionType::G108Zstd),
        b"ZSTD" => Some(CompressionType::Zstd),
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
    let mut decompressed = vec![0; decsize as usize];
    match compression_type {
        // CompressionType::Lzma => {
        //     let mut decompressor = liblzma::read::XzDecoder::new(&buf[8..]);
        //     decompressor.read_to_end(&mut decompressed)?;
        // }
        CompressionType::G108Lz4 | CompressionType::LZ4 => {
            lz4_flex::decompress_into(&buf[8..], &mut decompressed)?;
        }
        CompressionType::G108Zstd | CompressionType::Zstd => {
            decompressed = zstd::decode_all(&buf[8..])?;
        }
        // CompressionType::Zlib => {
        //     let mut decoder = flate2::read::ZlibDecoder::new(&buf[8..]);
        //     decoder.read_to_end(&mut decompressed)?;
        // }
        _ => unimplemented!(),
    };
    Ok(decompressed)
}
