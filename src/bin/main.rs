use eframe::egui::{self, ScrollArea, Vec2};

use reqwest::Client;
use reqwest::Method;
use reqwest::Version;
use tokio::runtime::Runtime;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use std::collections::HashMap;
use std::str::FromStr;
use serde_json::{Value, from_str};
use std::sync::{Arc, Mutex};
use std::thread;
use url;
use std::time::Duration;

mod json_thread_listner;

use shadowproxy_gui::utils::RequestData;

fn json_to_header_map(json_headers: &str) -> Result<HeaderMap, Box<dyn std::error::Error>> {
    // Parse the JSON string into a serde_json::Value
    let headers_value: Value = from_str(json_headers)?;

    // Convert the Value into a HashMap<String, String>
    let headers_map: HashMap<String, String> = headers_value
        .as_object()
        .ok_or("Invalid JSON object")?
        .iter()
        .map(|(k, v)| {
            let value = v.as_str().ok_or("Header value is not a string")?.to_string();
            println!("{:?} and {:?}",k,v);
            Ok((k.to_string(), value))
        })
        .collect::<Result<_, Box<dyn std::error::Error>>>()?;

    // Convert the HashMap into a reqwest::header::HeaderMap
    let mut header_map = HeaderMap::new();
    for (key, value) in headers_map {
        let header_name = HeaderName::from_str(&key)?;
        let header_value = HeaderValue::from_str(&value)?;
        header_map.insert(header_name, header_value);
    }

    Ok(header_map)
}

struct MyApp {
    active_tab: Tab,
    dock_collector: Arc<Mutex<Vec<RequestData>>>,
    request_store: Arc<Mutex<Vec<RequestData>>>,
    selected_for_show: RequestData,
    response_text: Arc<Mutex<String>>,
}

#[derive(PartialEq)]
enum Tab {
    Recon,
    Proxy,
    Repeater,
}

impl Default for MyApp {
    fn default() -> Self {
        
        let request_store = Arc::new(Mutex::new(vec![]));
        let request_store_clone = Arc::clone(&request_store);
        
        let google_dork_collector = Arc::new(Mutex::new(vec![]));
        let google_dork_collector_clone = Arc::clone(&google_dork_collector); 


        // Start background thread for capturing web requests
        thread::spawn(
            move || {

                json_thread_listner::start_warp_server(request_store_clone);
            }
        );
        /*
        thread::spawn(move || {
            let mut count = 0;
            loop {
                thread::sleep(Duration::from_secs(1)); // Simulate capturing a web request
                let mut log = requests_clone.lock().unwrap();
                log.push(format!("Captured request #{}", count));
                count += 1;
            }
        });
        */

        Self {
            
            active_tab: Tab::Recon,
            dock_collector: google_dork_collector,
            request_store: request_store,
            selected_for_show: RequestData::empty(),
            response_text: Arc::new(Mutex::new("Cliick button".to_string())),
        }

    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Reconnisense").clicked() {
                    self.active_tab = Tab::Recon;
                }
                if ui.button("Proxy").clicked() {
                    self.active_tab = Tab::Proxy;
                }

                if ui.button("Repeater").clicked(){
                    self.active_tab = Tab::Repeater;
                }
            });

            ui.separator();

            match self.active_tab {
                Tab::Recon => self.capture_google_dork_tab(ui),
                Tab::Proxy => self.proxy_tab(ui),
                Tab::Repeater => self.repeater_tab(ui),
            }
        });

        // Request repaint to update the UI periodically
        ctx.request_repaint_after(Duration::from_millis(600));
    }
}

impl MyApp {
    
    //TODO: Implement recon like google dorking
    fn capture_google_dork_tab(&self, ui: &mut egui::Ui){

    }



    fn proxy_tab(&mut self, ui: &mut egui::Ui) {
        ui.label("Captured Requests:");
        
        let max_height = ui.available_height() * (0.75);
        // Make the table scroll-able both horizontally and vertically, you motherfucker.
        ScrollArea::both()
            .id_source("Proxy_table_scroll")
            .max_height(max_height)
            .auto_shrink([false; 2])
            .show(ui, |ui| {
            egui::Grid::new("Proxy table")
                .striped(true)
                .min_col_width(20.0)
                .show ( ui, |ui|{
                    // "Its not death, but dying which is terriable"
                    // -- Henry Fielding (1707)
                    
                    // Table headers
                    ui.label("Index");
                    ui.label("Method");
                    ui.label("URL");
                    ui.label("Headers");
                    ui.end_row();
                

                    let log = self.request_store.lock().unwrap();

                    for (index,entry) in log.iter().enumerate() {
                        if ui.button(format!("{}",index + 1)).clicked(){
                            self.selected_for_show = RequestData{
                                    request_type:entry.request_type.clone(),
                                    http_version:entry.http_version.clone(),
                                    method:entry.method.clone(),
                                    url:entry.url.clone(),
                                    headers:entry.headers.clone(),
                                    body:entry.body.clone()};
                            
                            //println!("{:?}",self.selected_for_show);
                        }
                        ui.label(&entry.method);
                        ui.label(&entry.url);
                        // Display headers as a JSON string
                        // let headers_str = format!("{:?}", entry.headers);

                        ui.end_row();
                    }
                }
                );
        });

        ui.separator();
        ui.separator();
        if ui.button("Send Request").clicked(){
            self.send_request(ui.ctx().clone());
        }
        ui.horizontal(|ui| {
            
            //ui.text_edit_multiline(&mut format!("{:?}",self.selected_for_show));  
            let bottom_window_size = [400.0,200.0];
            

            egui::ScrollArea::both()
                .id_source("proxy_show_window_scroll")
                .max_height(bottom_window_size[1])
                .max_width(bottom_window_size[0])
                .auto_shrink([false; 2])
                .show(ui, |ui| { 
                        ui.add_sized(bottom_window_size, egui::TextEdit::multiline(&mut format!("{:?}",self.selected_for_show)));
                    }
                );
            
            egui::ScrollArea::both()
                .id_source("response")
                .max_height(bottom_window_size[1])
                .max_width(bottom_window_size[0])
                .auto_shrink([false;2])
                .show(ui, |ui| { 
                        ui.add_sized(bottom_window_size, egui::TextEdit::multiline(&mut format!("{}",self.response_text.lock().unwrap())));
                    }
                );
        });

    
    }

    fn repeater_tab(&self, ui: &mut eframe::egui::Ui){
    }

    fn send_request(&self, ctx: egui::Context) {
        let selected_request = self.selected_for_show.clone();
        let response_text = Arc::clone(&self.response_text);

        std::thread::spawn(move || {
            let runtime = Runtime::new().unwrap();
            runtime.block_on(async {
                let client = Client::new();
                let req = client.request(
                    selected_request.method.parse().unwrap(), 
                    &selected_request.url
                );

                let req = if selected_request.body.is_empty() {
                    req
                } else {
                    req.body(selected_request.body.clone())
                };

                let json_header_trans = json_to_header_map(selected_request.headers.as_str());
                //println!("Something :{:?}",json_header_trans.unwrap());
                //let req = req.headers(json_header_trans.unwrap());

                match req.send().await {
                    Ok(resp) => {
                        if let Ok(text) = resp.text().await {
                            *response_text.lock().unwrap() = text;
                        }
                    }
                    Err(_) => {
                        *response_text.lock().unwrap() = "Failed to send request".to_string();
                    }
                }
            });
        });
    }
}

fn main() {
    let options = eframe::NativeOptions {
        initial_window_size: Some(Vec2::new(500.0, 400.0)),
        ..Default::default()
    };
    let _ = eframe::run_native("Egui Background Task Example", options, Box::new(|_cc| Box::new(MyApp::default())));
}

