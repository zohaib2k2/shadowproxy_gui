

use reqwest::Method;
use std::collections::HashMap;
use url::Url;


#[derive(Debug, PartialEq)]
pub enum HttpVersion {
    Http0_9,
    Http1_1,
    Http2_0,
    Http3_0,
    Unknown,
}


#[derive(Debug, PartialEq)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
    OPTIONS,
    CONNECT,
    TRACE,
}


#[derive(Debug)]
pub struct RequestData {
    pub request_type: String,
    pub http_version: String, 
    pub method: String,
    pub url: String,
    pub headers: String,
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

    pub fn clone(&self)->RequestData{
        RequestData::new(self.request_type.clone(),self.http_version.clone(),self.method.clone(),self.url.clone(),self.headers.clone(),self.body.clone())
    }
}

