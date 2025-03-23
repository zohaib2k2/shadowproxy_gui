

use warp::Filter;
use serde::{Deserialize, Serialize};
use tokio::runtime::Runtime;
use serde_json::Value;
use std::sync::{Arc,Mutex};

#[derive(Debug, Deserialize, Serialize)]
struct IncomingData {
    name: String,
    age: u32,
}

use crate::utils::structs::RequestData;
// Function to start the Warp server
// later i may have to pass egui object Ui to make these request visable. 
pub fn start_warp_server(data_store: Arc<Mutex<Vec<RequestData>>>) {
    let rt = Runtime::new().expect("Failed to create runtime");

    rt.block_on(async {
        let data_store_clone = data_store.clone();

        let data_route = warp::path("data")
            .and(warp::post())
            .and(warp::body::json())
            .map( move |body: Value| {
            
            let mut data_store = data_store_clone.lock().unwrap(); // Lock the mutex

                  // Debug the type of JSON received
            if let (Some(request_type),Some(http_version) ,Some(method), Some(url),Some(headers),Some(body)) = (
                body.get("type").and_then(Value::as_str),
                body.get("http_version").and_then(Value::as_str),
                body.get("method").and_then(Value::as_str),
                body.get("url").and_then(Value::as_str),

                body.get("headers").and_then(Value::as_str),
                body.get("body").and_then(Value::as_str),
            ) {
    
                //For debuggin
                
                //println!("Request Type: {}", request_type);
                //println!("Method: {}", method);
                //println!("URL: {}", url);
                
                
                let req_data = RequestData{
                    request_type : request_type.to_string(),
                    http_version: http_version.to_string(),
                    method: method.to_string(),
                    url: url.to_string(),
                    headers: headers.to_string(),
                    body: body.to_string(),
                };
                
                data_store.push(req_data);
                // for debuging

            } else {
                println!("Missing one or more required fields: type, method, url");
            }

            // Extract headers if they exist
            /*
            if let Some(headers) = body.get("headers") {
                println!("Headers: {:?}", headers);
            } else {
                println!("Headers field is missing");
            } 
            */

            warp::reply()
            });


        warp::serve(data_route).run(([0, 0, 0, 0], 5000)).await;
    });
}

