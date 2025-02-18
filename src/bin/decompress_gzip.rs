use flate2::read::GzDecoder;
use std::io::Read;

/// Decompress gzip-encoded data
fn decompress_gzip(data: &[u8]) -> Result<Vec<u8>, &'static str> {
    let mut decoder = GzDecoder::new(data);
    let mut decompressed = Vec::new();
    decoder
        .read_to_end(&mut decompressed)
        .map_err(|_| "Failed to decompress gzip data")?;
    Ok(decompressed)
}

fn main() {
    // Example: Simulated gzip-compressed HTTP response body
    let gzip_data: &[u8] = &[
        0x1f, 0x8b, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0xcb, 0x48, 0xcd, 0xc9, 0xc9,
        0x57, 0x28, 0xcf, 0x2f, 0xca, 0x49, 0x01, 0x00, 0x85, 0x11, 0x4a, 0x0d, 0x0b, 0x00, 0x00,
        0x00,
    ];

    // Decompress the data
    match decompress_gzip(gzip_data) {
        Ok(decompressed) => {
            println!("Decompressed data: {:?}", String::from_utf8_lossy(&decompressed));
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }
}
