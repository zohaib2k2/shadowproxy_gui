use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::{Arc, Mutex};
use tokio::runtime::Runtime;
use warp::Filter;

pub fn start_link_server(dork_store: Arc<Mutex<Vec<String>>>) {
    let rt = Runtime::new().expect("Failed to create runtime");

    rt.block_on(async {
        let dork_store_clone = dork_store.clone();

        let data_route = warp::path("data")
            .and(warp::post())
            .and(warp::body::json())
            .map(move |body: Value| {
                let mut data_store = dork_store_clone.lock().unwrap(); // Lock the mutex

                // Debug the type of JSON received
                if let Some(dork) = body.get("dork").and_then(Value::as_str) {
                    //For debuggin

                    //println!("Request Type: {}", request_type);
                    //println!("Method: {}", method);
                    //println!("URL: {}", url);

                    println!("{}", dork);
                    //data_store.push(dork.to_string());
                    // for debuging
                } else {
                    println!("Missing one or more required fields");
                }

                warp::reply()
            });

        warp::serve(data_route).run(([0, 0, 0, 0], 65535)).await;
    });
}
