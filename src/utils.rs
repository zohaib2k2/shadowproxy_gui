

use reqwest::Method;
use std::collections::HashMap;
use url::Url;


use flate2::read::{GzDecoder, DeflateDecoder};
use brotli::Decompressor;
use std::io::Read;



/// Represents different verisions of HTTP
///
/// This enum defines the supported HTTP versions and an `Unknown` variant
/// for handling unsupported or unrecognized versions.
#[derive(Debug, PartialEq)]
pub enum HttpVersion {
    /// Represents HTTP/0.9, the initial version of HTTP.
    Http0_9,
    /// Represents HTTP/1.1, the most widely used version of HTTP.
    Http1_1,
    /// Represents HTTP/2.0, which introduces multiplexing and binary framing.
    Http2_0,
    /// Represents HTTP/3.0, which uses QUIC for transport instead of TCP.
    Http3_0,
    /// Represents an unknown or unsupported HTTP version.
    Unknown,
}

/// Represents the HTTP request methods defined in the HTTP/1.1 specification.
///
/// This enum defines the standard HTTP methods used to indicate the desired
/// action to be performed on a resource.
#[derive(Debug, PartialEq)]
pub enum HttpMethod {
    /// Requests a representation of the specified resource.
    GET,
    /// Submits an entity to the specified resource, often causing a change in state.
    POST,
    /// Replaces all current representations of the target resource with the request payload.
    PUT,
    /// Deletes the specified resource.
    DELETE,
    /// Describes the communication options for the target resource.
    OPTIONS,
     /// Establishes a tunnel to the server identified by the target resource.
    CONNECT,
    /// Performs a message loop-back test along the path to the target resource.
    TRACE,
}

/// Represents the data of an HTTP request.
///
/// This struct encapsulates the components of an HTTP request, including the
/// request type, HTTP version, method, URL, headers, and body.
#[derive(Debug)]
pub struct RequestData {
    /// The type of the request (e.g., "HTTP Request").
    pub request_type: String,
    /// The HTTP version used in the request (e.g., "HTTP/1.1").
    pub http_version: String,
    /// The HTTP method used in the request (e.g., "GET").
    pub method: String,
    /// The URL of the resource being requested.
    pub url: String,
    /// The headers included in the request, represented as a single string.
    pub headers: String,
     /// The body of the request, if any.
    pub body: String, 
}



pub struct RequestDataProper {
    pub http_version: HttpVersion,
    pub method: HttpMethod,
    pub url: String,
    pub headers: String,
    pub body: String,
}


impl RequestData{
    /// Creates a new `RequestData` instance with the provided fields.
    ///
    /// # Arguments
    /// * `request_type` - The type of the request (e.g., "HTTP Request").
    /// * `http_version` - The HTTP version used in the request (e.g., "HTTP/1.1").
    /// * `method` - The HTTP method used in the request (e.g., "GET").
    /// * `url` - The URL of the resource being requested.
    /// * `headers` - The headers included in the request, represented as a single string.
    /// * `body` - The body of the request, if any.
    ///
    /// # Examples
    /// ```
    /// use your_crate::RequestData;
    ///
    /// let request = RequestData::new(
    ///     String::from("HTTP Request"),
    ///     String::from("HTTP/1.1"),
    ///     String::from("GET"),
    ///     String::from("https://example.com"),
    ///     String::from("Content-Type: application/json"),
    ///     String::from(""),
    /// );
    ///
    /// assert_eq!(request.url, "https://example.com");
    /// ```
    pub fn new(request_type:String, http_version:String, method: String, url:String, headers: String, body:String ) -> RequestData{
        RequestData {
            request_type,
            http_version,
            method,
            url,
            headers,
            body,
        }
    }
    // This is method is very very ugly - too bad.
    // If i become a Good muslim give charity and pray 
    // Allah wil grant me heaven where all code is beautiful and elegant Ameen.
    pub fn empty() -> RequestData{
       RequestData::new("".to_string(),"".to_string() ,"".to_string() ,"".to_string() ,"".to_string() ,"".to_string()) 

    }

    /// Creates a clone of the current `RequestData` instance.
    ///
    /// This method returns a new `RequestData` instance with the same values as the current one.
    ///
    /// # Examples
    /// ```
    /// use shadowproxy_gui::RequestData;
    ///
    /// let request = RequestData::new(
    ///     String::from("HTTP Request"),
    ///     String::from("HTTP/1.1"),
    ///     String::from("GET"),
    ///     String::from("https://example.com"),
    ///     String::from("Content-Type: application/json"),
    ///     String::from(""),
    /// );
    ///
    /// let cloned_request = request.clone();
    /// assert_eq!(request.url, cloned_request.url);
    /// ```
    pub fn clone(&self)->RequestData{
        RequestData::new(self.request_type.clone(),self.http_version.clone(),self.method.clone(),self.url.clone(),self.headers.clone(),self.body.clone())
    }
}

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

/// Translates the intercepted headers in string,
/// into header. `reqwest`
pub fn json_to_header_map(json_headers: &str) -> Result<HeaderMap, Box<dyn std::error::Error>> {
    
    let corrected_string = json_headers.replace("\"","\\\"").replace("'","\"");


    // Parse the JSON string into a serde_json::Value
    let headers_value: Value = from_str(&corrected_string)?;

    // Convert the Value into a HashMap<String, String>
    let headers_map: HashMap<String, String> = headers_value
        .as_object()
        .ok_or("Invalid JSON object")?
        .iter()
        .map(|(k, v)| {
            let value = v.as_str().ok_or("Header value is not a string")?.to_string();
            Ok((k.to_string(), value))
        })
        .collect::<Result<_, Box<dyn std::error::Error>>>()?;

    // Convert the HashMap into a reqwest::header::HeaderMap
    let mut header_map = HeaderMap::new();
    for (key, value) in headers_map {
        if key.trim().to_lowercase() == "if-modified-since" || key.trim().to_lowercase() == "if-none-match"{
            continue;
        }
        let header_name = HeaderName::from_str(&key)?;
        let header_value = HeaderValue::from_str(&value)?;
        header_map.insert(header_name, header_value);
    }

    Ok(header_map)
}
