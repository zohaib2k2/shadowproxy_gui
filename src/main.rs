/*
 * Copyright (C) [2025] [Zohaib Zafar]
 *
 * 
 * This program is free software:
 * not just free as in free beer but also 'free' as in freedom.
 * you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use eframe::egui::{self, ScrollArea, Vec2};


use reqwest::Client;
use tokio::runtime::Runtime;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use std::collections::HashMap;
use serde_json::{Value, from_str};
use std::sync::{Arc, Mutex};
use std::thread;
use url;
use std::time::Duration;

mod servers;
mod utils;

use crate::utils::structs::RequestData;
use crate::utils::converters::decompress_response;

use log::{info, warn, error};
use log4rs;

/// A basic logger for all the activities.


/// The `MyApp` struct represents the core application state for an `egui`-based proxy tool.
/// It manages tabs, request storage, request selection, response handling, and proxy/recon state.

struct MyApp {
    /// The currently active tab in the UI.
    active_tab: Tab,

    /// A shared, thread-safe collection for storing intercepted HTTP requests.
    /// Used to temporarily collect requests before they are processed.
    dock_collector: Arc<Mutex<Vec<RequestData>>>,

    /// A shared, thread-safe storage for all captured HTTP requests.
    /// This stores the main history of intercepted requests.
    request_store: Arc<Mutex<Vec<RequestData>>>,

    /// The request currently selected for display in the UI.
    selected_for_show: RequestData,

     /// A shared, thread-safe string that holds the HTTP response body.
    /// This allows multiple parts of the UI to access or modify the response.
    response_text: Arc<Mutex<String>>,

    repeater_request: Arc<Mutex<Vec<RequestData>>>,

    selected_repeater_request: RequestData,
    selected_repeater_request_text : String,

    repeater_response: Arc<Mutex<String>>,
     /// A flag to indicate whether the proxy should stop running.
    /// `true` means the proxy should stop; `false` means it's active.
    stop_proxy: bool,

    /// A flag to indicate whether the reconnaissance process should stop.
    /// `true` means recon should stop; `false` means it's active.
    stop_recon: bool,
    
    show_proxy_context_menu: bool,

}

#[derive(PartialEq)]
enum Tab {
    Recon,
    Proxy,
    Repeater,
}

impl Default for MyApp {
    /// Provides a default implementation for `MyApp`.
    /// 
    /// This initializes shared request storage, spawns a background thread
    /// to capture web requests, and sets default values for various UI elements.
    fn default() -> Self {
        
        let request_store = Arc::new(Mutex::new(vec![]));
        let request_store_clone = Arc::clone(&request_store);
        
        let google_dork_collector = Arc::new(Mutex::new(vec![]));
        let google_dork_collector_clone = Arc::clone(&google_dork_collector); 

        let repeater_request = Arc::new(Mutex::new(vec![]));
        let repeater_request_clone = Arc::clone(&repeater_request);

        // Start background thread for capturing web requests
        thread::spawn(
            move || { 
                servers::json_thread_listner::start_warp_server(request_store_clone);
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
            repeater_request: repeater_request,
            selected_repeater_request: RequestData::empty(),
            repeater_response: Arc::new(Mutex::new("".to_string())),
            response_text: Arc::new(Mutex::new("Cliick button".to_string())),
            stop_proxy: false,
            stop_recon: false,
            show_proxy_context_menu: false,
            selected_repeater_request_text : String::from(""),
        }

    }
}

impl eframe::App for MyApp {
    /// This method is called on each frame to update the application's UI.
    /// It handles displaying the tab buttons, updating the active tab,
    /// and rendering the appropriate content based on the selected tab.
    ///
    /// - **Tab Switching**: Buttons ("Recon", "Proxy", "Repeater") allow the user to
    ///   switch between the respective tabs by updating the `active_tab`.
    /// - **UI Layout**: The UI is organized using horizontal layout (`ui.horizontal`) and
    ///   a separator (`ui.separator`).
    /// - **Tab Content Rendering**: The content for the currently active tab is shown by
    ///   matching the `active_tab` with the corresponding method (`capture_google_dork_tab`,
    ///   `proxy_tab`, `repeater_tab`).
    ///
    /// **Repaint Request**: The UI is set to periodically repaint every 600 milliseconds
    /// to ensure the UI stays up to date.
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
    /// Displays the Google Dork capture tab.
    ///
    /// This function is responsible for rendering the UI elements related to
    /// capturing Google Dork search results.
    fn capture_google_dork_tab(&self, ui: &mut egui::Ui){

    }


    /// Displays the Proxy tab, which captures and displays HTTP requests.
    ///
    /// - Provides "Start" and "Stop" buttons to control request capturing.
    /// - Displays captured requests in a scrollable table.
    /// - Allows selecting a request to view details.
    /// - Provides a "Send Request" button to resend a selected request.
    /// - Displays request and response data in two separate scrollable text panels.
    fn proxy_tab(&mut self, ui: &mut egui::Ui) {
        ui.label("Captured Requests:");
        ui.horizontal( |ui| {
            if ui.button("Stop").clicked(){
                self.stop_proxy = true;
            }
            if ui.button("Start").clicked(){
                self.stop_proxy = false;
            }

        }) ;   
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
                        if self.stop_proxy {
                            continue;
                        }
                        if ui.button(format!("{}",index + 1)).clicked(){
                            self.selected_for_show = RequestData{
                                    request_type:entry.request_type.clone(),
                                    http_version:entry.http_version.clone(),
                                    method:entry.method.clone(),
                                    url:entry.url.clone(),
                                    headers:entry.headers.clone(),
                                    body:entry.body.clone()};
                            
                            //println!("{}",self.selected_for_show.headers);
                            let e1 = &self.selected_for_show.headers.replace("\"","\\\"").replace("'","\"");
                            let headers1: HashMap<String,String> = serde_json::from_str(e1).unwrap();
                            for (key, value) in headers1 {
                                println!("{}: {}", key, value);
                            }

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
        ui.horizontal( |ui| {
            if ui.button("Send Request").clicked(){
                let selected_req_for_show = self.selected_for_show.clone();
                self.send_request(ui.ctx().clone(),selected_req_for_show,Tab::Proxy);
            }
            if ui.button("Send to repeater").clicked(){
                self.selected_repeater_request = self.selected_for_show.clone(); 

                self.update_selected_text(); 
            }

        });
        ui.horizontal(|ui| {
            
            //ui.text_edit_multiline(&mut format!("{:?}",self.selected_for_show));  
            let bottom_window_size = [500.0,185.0];
            

            egui::ScrollArea::both()
                .id_source("proxy_show_window_scroll")
                .max_height(bottom_window_size[1])
                .min_scrolled_width(bottom_window_size[0])
                .max_width(bottom_window_size[0])
                .min_scrolled_height(bottom_window_size[1])
                .show(ui, |ui| { 
                       let ui_resp = ui.add_sized(bottom_window_size, egui::TextEdit::multiline(&mut format!("{:?}",self.selected_for_show)));
                       if ui_resp.clicked_by(egui::PointerButton::Secondary) {
                           self.show_proxy_context_menu = true;
                       }

                       if self.show_proxy_context_menu {
                           egui::menu::bar(ui, |ui| {
                                ui.menu_button("Context Menu", |ui|{
                                    if ui.button("Copy").clicked(){
                                        ui.output_mut(|o| o.copied_text = "Hello world!".to_string())
                                    }
                                })
                           }
                           );
                       }
                    }
                );
            
            egui::ScrollArea::both()
                .id_source("response")
                .max_height(bottom_window_size[1])
                .max_width(bottom_window_size[0])
                .show(ui, |ui| { 
                        let ui_resp = ui.add_sized(bottom_window_size, egui::TextEdit::multiline(&mut format!("{}",self.response_text.lock().unwrap())));

                    }
                );
        });

    
    }
     /// Displays the Repeater tab, which allows modifying and resending requests.
    ///
    /// This function is responsible for rendering the UI elements of the
    /// request repeater feature.
    fn repeater_tab(&mut self, ui: &mut eframe::egui::Ui){
        ui.separator();
    
        let available_height = ui.available_height();
        let available_width = ui.available_width();
        ui.horizontal( |ui| {
            if ui.button("Send").clicked(){
                self.send_request(ui.ctx().clone(),self.selected_repeater_request.clone(),Tab::Repeater);
            }
            if ui.button("Print Req").clicked(){
                println!("{}",self.selected_repeater_request_text);
            }
        });

        ui.horizontal( |ui| {
            egui::ScrollArea::both()
                .id_source("request_rep")
                .max_height(available_height * 0.95)
                .max_width(available_width/2.0)
                .min_scrolled_height(available_height)
                .show(ui, |ui| {
                    ui.add_sized([available_width/2.0, available_height*0.95],
                        egui::TextEdit::multiline(&mut self.selected_repeater_request_text)
                            .desired_width(f32::INFINITY)
                        );
                    });
            egui::ScrollArea::both()
                .id_source("respone_rep")
                .max_height(available_height * 0.95)
                .max_width(available_width/2.0)
                .min_scrolled_height(available_height)
                .show(ui, |ui| {
                    // Yar ab ma kud-kooshi kar lon ga muj se 
                    // nai banta gui.
                    ui.add_sized([available_width/2.0, available_height*0.95],

                        egui::TextEdit::multiline(&mut self.repeater_response.lock().unwrap().to_string())
                            .desired_width(f32::INFINITY));
                    
                    });
                

        });
            



    }
    
    fn update_selected_text(&mut self){
        self.selected_repeater_request_text = self.selected_repeater_request.to_string();

    }



    /// Sends the currently selected request asynchronously.
    ///
    /// - Clones the selected request data.
    /// - Uses `reqwest::Client` to make an HTTP request.
    /// - Attaches headers and request body if provided.
    /// - Updates `response_text` with the server's response or an error message.
    /// - Runs the request in a separate thread to avoid blocking the UI.
    fn send_request(&self, ctx: egui::Context, selected_req: RequestData,tab: Tab) {
        let selected_request = selected_req;

        let response_text = match tab {
            Tab::Proxy =>  Arc::clone(&self.response_text),
            Tab::Repeater => Arc::clone(&self.repeater_response),
            _ => {
                info!("Wrong tab!");
                panic!();
            }
        };
            
        

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

                let req = if selected_request.headers.is_empty(){
                    req
                } else {
                    let json_header_trans = crate::utils::converters::json_to_header_map(selected_request.headers.as_str());
                    // okay so this implies that ProperRequestData would have HashMap as its 
                    req.headers(json_header_trans.unwrap())
                };

                

                match req.send().await {
                    Ok(resp) => {
                        
                        // Variable to store the Content-Encoding value
                        let content_encoding: Option<String> = resp 
                            .headers()
                            .get("Content-Encoding")
                            .and_then(|value| value.to_str().ok())
                            .map(|s| s.to_string());
                        
                        /*
                        println!("Headers founds!!");
                        println!("================");
                        println!("Encoding {:?}",resp.headers().get("Content-Encoding"));
                        println!("{}",resp.status());
                        */
                        let str_found :String;
                        if let Ok(text) = resp.bytes().await {

                            match decompress_response(&text,content_encoding.as_deref()) {
                                Ok(decompressed) => {
                                    /*
                                    println!(
                                        "Decompressed response: {}",
                                        String::from_utf8_lossy(&decompressed)
                                    );*/
                                    str_found = String::from_utf8(decompressed).expect("Failed to Convert bytes to String");
                                }
                                Err(e) => {
                                    eprintln!("Error: {}", e);
                                    str_found = String::from("Wrong sequence of UTF-8 bytes.");
                                }
                            }

                             *response_text.lock().unwrap() = str_found;
                                
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
    log4rs::init_file("log4rs.yaml", Default::default()).unwrap();

    info!("Application Starting");
    let options = eframe::NativeOptions {
        initial_window_size: Some(Vec2::new(500.0, 400.0)),
        ..Default::default()
    };
    let _ = eframe::run_native("Egui Background Task Example", options, Box::new(|_cc| Box::new(MyApp::default())));
}

