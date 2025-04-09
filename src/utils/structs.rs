

use crate::utils::converters::json_str_to_hashmap;
use egui::TextBuffer;
use serde_json;
use std::fmt;
use std::collections::HashMap;
use std::str::FromStr;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};

/// Represents different verisions of HTTP
///
/// This enum defines the supported HTTP versions and an `Unknown` variant
/// for handling unsupported or unrecognized versions.
#[derive(Debug, PartialEq,Clone)]
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
#[derive(Debug, PartialEq,Clone,Copy)]
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
#[derive(Debug,Clone)]
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


#[derive(Debug,Clone)]
pub struct RequestDataProper {
    pub http_version: HttpVersion,
    pub method: HttpMethod,
    pub url: String,
    pub headers: HashMap<String,String>,
    pub body: String,
}



impl FromStr for RequestData {
    type Err = String;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = raw.trim().split("\n\n").collect();
       

        let headers_and_start = parts[0];
        let body = parts[1..].join("\n\n");

        let mut lines = headers_and_start.lines();
        let request_line = lines.next().ok_or("Missing request line")?;
        let host_line =lines.next().ok_or("No host line")?;
        let split_hostline:Vec<String> = host_line.split(":").map(|l|l.to_string()).collect();
        let host = split_hostline.get(1).unwrap();
        let mut parts = request_line.split_whitespace();

        let method = parts.next().ok_or("missing method")?.to_string();
        let url = parts.next().ok_or("Missing url")?.to_string();
        let http_version = parts.next().ok_or("missing http version")?.to_string();

        let headers: Vec<String> = lines.map(|line| line.trim().to_string()).collect();
        let headers_string = headers.join("\n");
        let full_url = format!("https://{}{}",host.trim(),url.trim());

        Ok(RequestData {
            request_type: "HTTP Request".to_string(),
            http_version,
            method,
            url: full_url,
            headers: headers_string,
            body: body.trim().to_string(),
        })
    }
}


impl RequestData{
    /// Creates a new `RequestData` instance with the provided fields.
    ///
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
    /* 
    pub fn to_RequestData(request_str : String) -> RequestData{
        

    }*/
}


impl fmt::Display for RequestData{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut out_put = String::new();

        let parsed_url = match url::Url::parse(&self.url){
            Ok(url) => url,
            Err(_) => {
                out_put.push_str(&format!("Invalid URL: {}\n",self.url));
                return writeln!(f, "Invalid URL eat shit: {}", self.url)
            },
        };

        match parsed_url.fragment() {
            Some(frag) => {
                out_put.push_str(&format!("{} {}{}#{} {}\n",self.method,parsed_url.path(),parsed_url.query().unwrap_or(""),frag,self.http_version));

            }
            None => {

                if parsed_url.query().is_none() {
                    out_put.push_str(&format!("{} {} {}\n",self.method,parsed_url.path(),self.http_version));
                } else {
                    
                    out_put.push_str(&format!("{} {}?{} {}\n",self.method,parsed_url.path(),parsed_url.query().unwrap_or(""),self.http_version));
                }
            }
        }


        if let Some(host) = parsed_url.host() {
            out_put.push_str(&format!("Host: {}\n",host));
        } else {
            out_put.push_str("Host: (none)\n");
        }

        let hashmap = match json_str_to_hashmap(self.headers.as_str()){
            Ok(map) => map,
            Err(e) => {
                println!("Errored Header:[{}]",self.headers.as_str());
                out_put.push_str(&format!("Failed to parse headers another L for me: {}\n",e));
                //return write!(f,"{}",out_put);
                let lines = self.headers.lines();
                let mut hm:HashMap<String, String> = HashMap::new();

                for line in lines {
                    let kv = line.split_once(":").unwrap();

                    hm.insert(kv.0.to_string(), kv.1.to_string());
                    
                }

                let hmc = hm.clone();

                for (k,v) in hmc {
                    println!("DEBUG:");
                    println!("{} {}",k,v);
                } 

                hm
            }
        };
        
        for (key, value) in hashmap.iter(){
            out_put.push_str(&format!("{}: {}\n",key,value));
        }

        out_put.push_str("\n");
        
        if !self.body.is_empty(){
            out_put.push_str(&format!("{}\n",self.body));
        }

        write!(f, "{}",out_put)?;
        Ok(()) 
    }
}



impl FromStr for HttpVersion {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "HTTP/0.9" => Ok(HttpVersion::Http0_9),
            "HTTP/1.1" => Ok(HttpVersion::Http1_1),
            "HTTP/2.0" => Ok(HttpVersion::Http2_0),
            "HTTP/3.0" => Ok(HttpVersion::Http3_0),
            _ => Ok(HttpVersion::Unknown),
        }
    }
}

impl FromStr for HttpMethod {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "GET" => Ok(HttpMethod::GET),
            "POST" => Ok(HttpMethod::POST),
            "PUT" => Ok(HttpMethod::PUT),
            "DELETE" => Ok(HttpMethod::DELETE),
            "OPTIONS" => Ok(HttpMethod::OPTIONS),
            "CONNECT" => Ok(HttpMethod::CONNECT),
            "TRACE" => Ok(HttpMethod::TRACE),
            "Invalid" => {
                println!("Invalid shit, get your shit together!!");
                panic!();
            }
            _ => Err(format!("Unsupported HTTP method zzz worng shit: {}", s.to_string())),
        }
    }
}

impl FromStr for RequestDataProper {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut lines = s.lines().filter(|line| !line.trim().is_empty());
        
        // Parse request line
        let request_line = lines.next().ok_or("Missing request line")?;
        let mut parts = request_line.split_whitespace();
        let method = parts.next().ok_or("Missing HTTP method")?.trim().parse()?;
        let url = parts.next().ok_or("Missing URL")?.to_string();
        let http_version = parts.next().ok_or("Missing HTTP version")?.parse()?;

        // Parse headers
        let mut headers = HashMap::new();
        let mut body_lines = Vec::new();
        let mut is_body = false;
        let mut host = "";

        for line in lines {
            if line.is_empty() {
                is_body = true;
                continue;
            }
            
            if is_body {
                body_lines.push(line);
            } else if let Some((key, value)) = line.split_once(": ") {
                if key.eq_ignore_ascii_case("Host") {
                    host = value;
                } else {
                    headers.insert(key.to_string(), value.to_string());
                }
            }
        }
        
        let body = s.to_string().split("\n\n").nth(1).map(str::trim).filter(|&x| !x.is_empty()).unwrap_or("").to_string();
        let scheme = match http_version {
            HttpVersion::Http0_9 | HttpVersion::Http1_1 => "http",
            HttpVersion::Http2_0 | HttpVersion::Http3_0 => "https",
            _ => "http", // Default to HTTP if unknown
        };
        let full_url = format!("{}://{}/{}", scheme, host.trim().trim_end_matches("/").trim_start_matches("/"), url.trim().trim_end_matches("/").trim_start_matches("/"));    
    

        Ok(RequestDataProper {
            http_version,
            method,
            url: full_url,
            headers,
            body,
        })
    }
}


pub trait IntoHeaderMap {
    fn parse(self) -> HeaderMap;
}

pub trait IntoMethod {
    fn parse(self) -> reqwest::Method;
}

impl IntoMethod for HttpMethod {
    fn parse(self) -> reqwest::Method {
        match self {
            HttpMethod::GET => reqwest::Method::GET,
            HttpMethod::POST => reqwest::Method::POST,
            HttpMethod::CONNECT => reqwest::Method::CONNECT,
            HttpMethod::DELETE => reqwest::Method::DELETE,
            HttpMethod::PUT => reqwest::Method::PUT,
            HttpMethod::OPTIONS => reqwest::Method::OPTIONS,
            HttpMethod::TRACE => reqwest::Method::TRACE,
        }
    }
}

impl IntoHeaderMap for HashMap<String, String> {
    fn parse(self) -> HeaderMap {
        let mut header_map = HeaderMap::new();

        for (key, value) in self {
            if let (Ok(name), Ok(val)) = (
                HeaderName::from_bytes(key.as_bytes()),
                HeaderValue::from_str(&value),
            ) {
                header_map.insert(name, val);
            } else {
                eprintln!("Invalid header: {}: {}", key, value);
            }
        }

        header_map
    }
}
