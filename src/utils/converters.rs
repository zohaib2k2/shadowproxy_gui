use std::collections::HashMap;

use brotli::Decompressor;
use flate2::read::{DeflateDecoder, GzDecoder};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde_json;
use serde_json::Value;
use std::io::Read;

/// Decompress a response body based on the `Content-Encoding` header.
pub fn decompress_response(body: &[u8], encoding: Option<&str>) -> Result<Vec<u8>, &'static str> {
    match encoding {
        Some("gzip") => {
            let mut decoder = GzDecoder::new(body);
            let mut decompressed = Vec::new();
            decoder
                .read_to_end(&mut decompressed)
                .map_err(|_| "Failed to decompress gzip data")?;
            Ok(decompressed)
        }
        Some("deflate") => {
            let mut decoder = DeflateDecoder::new(body);
            let mut decompressed = Vec::new();
            decoder
                .read_to_end(&mut decompressed)
                .map_err(|_| "Failed to decompress deflate data")?;
            Ok(decompressed)
        }
        Some("br") => {
            let mut decompressed = Vec::new();
            Decompressor::new(body, body.len())
                .read_to_end(&mut decompressed)
                .map_err(|_| "Failed to decompress brotli data")?;
            Ok(decompressed)
        }
        _ => Ok(body.to_vec()), // No compression
    }
}

/// json_headers:str to HashMap
/// convert json_headers in &str form into Hashmap.
pub fn json_str_to_hashmap(
    json_headers: &str,
) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
    let corrected_string = json_headers.replace("\"", "\\\"").replace("'", "\"");

    let header_value: Value = serde_json::from_str(&corrected_string)?;

    let header_map: HashMap<String, String> = header_value
        .as_object()
        .ok_or("Invalid JSON object")?
        .iter()
        .map(|(k, v)| {
            let value = v
                .as_str()
                .ok_or("Header value is not a string")?
                .to_string();
            Ok((k.to_string(), value))
        })
        .collect::<Result<_, Box<dyn std::error::Error>>>()?;

    Ok(header_map)
}

/// Translates the intercepted headers in string,
/// into header. `reqwest`
pub fn json_to_header_map(json_headers: &str) -> Result<HeaderMap, Box<dyn std::error::Error>> {
    let corrected_string = json_headers.replace("\"", "\\\"").replace("'", "\"");

    // Parse the JSON string into a serde_json::Value
    let headers_value: Value = serde_json::from_str(&corrected_string)?;

    // Convert the Value into a HashMap<String, String>
    let headers_map: HashMap<String, String> = headers_value
        .as_object()
        .ok_or("Invalid JSON object")?
        .iter()
        .map(|(k, v)| {
            let value = v
                .as_str()
                .ok_or("Header value is not a string")?
                .to_string();
            Ok((k.to_string(), value))
        })
        .collect::<Result<_, Box<dyn std::error::Error>>>()?;

    // Convert the HashMap into a reqwest::header::HeaderMap
    let mut header_map = HeaderMap::new();
    for (key, value) in headers_map {
        if key.trim().to_lowercase() == "if-modified-since"
            || key.trim().to_lowercase() == "if-none-match"
        {
            continue;
        }
        let header_name = HeaderName::from_bytes(&key.as_bytes())?;
        let header_value = HeaderValue::from_str(&value)?;
        header_map.insert(header_name, header_value);
    }

    Ok(header_map)
}
