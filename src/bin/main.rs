use eframe::egui::{self, ScrollArea, Vec2};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

mod json_thread_listner;

use shadowproxy_gui::utils::RequestData;

struct MyApp {
    active_tab: Tab,
    dock_collector: Arc<Mutex<Vec<RequestData>>>,
    request_store: Arc<Mutex<Vec<RequestData>>>,
    selected_for_show: RequestData,

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
        ctx.request_repaint();
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
                        let headers_str = format!("{:?}", entry.headers);
                        ui.label(headers_str);

                        ui.end_row();
                    }
                }
                );
        });

        ui.separator();
        ui.separator();
        //ui.text_edit_multiline(&mut format!("{:?}",self.selected_for_show));  
        let bottom_window_size = [700.0,50.0];
            
        egui::ScrollArea::vertical()
            .id_source("proxy_show_window_scroll")
            .max_height(bottom_window_size[1])
            .show(ui, |ui| { 
                    ui.add_sized(bottom_window_size, egui::TextEdit::multiline(&mut format!("{:?}",self.selected_for_show)));
                }
            );
    }

    fn repeater_tab(&self, ui: &mut eframe::egui::Ui){
    }
}

fn main() {
    let options = eframe::NativeOptions {
        initial_window_size: Some(Vec2::new(500.0, 400.0)),
        ..Default::default()
    };
    let _ = eframe::run_native("Egui Background Task Example", options, Box::new(|_cc| Box::new(MyApp::default())));
}

