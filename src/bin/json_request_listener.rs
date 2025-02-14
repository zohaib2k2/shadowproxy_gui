use warp::Filter;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct IncomingData {
    name: String,
    age: u32,
}

#[tokio::main]
async fn main() {
    // Define a warp filter for POST requests on `/data`
    let data_route = warp::path("data")
        .and(warp::post())
        .and(warp::body::json())
        .map(|body: serde_json::Value| {
            println!("Received JSON Data: {:?}",body );
            warp::reply()
        });

    // Start the server on 127.0.0.1:5000
    warp::serve(data_route).run(([0, 0, 0, 0], 5000)).await;
}

