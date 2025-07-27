/*
 * Copyright (C) [2025] [Zohaib Zafar]
 *
 *
 * This program is free software:
 * not just free as in free ice-cream but also 'free' as in freedom.
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

use genpdf::*;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use reqwest::Client;
use serde_json::{from_str, Value};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{thread, vec};
use tokio::runtime::Runtime;
use url;
use utils::structs::{IntoHeaderMap, IntoMethod, RequestDataProper};
use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};
use html_escape;

mod servers;
mod utils;



use crate::utils::converters::decompress_response;
use crate::utils::structs::RequestData;

use log::{error, info, warn};
use log4rs;

/// A basic logger for all the activities.

/// The `MyApp` struct represents the core application state for an `egui`-based proxy tool.
/// It manages tabs, request storage, request selection, response handling, and proxy/recon state.

struct MyApp {
    /// The currently active tab in the UI.
    active_tab: Tab,

    /// Search url
    search_url: String,

    /// show generate pdf popup
    show_pop_up : bool,

    save_pdf_filename: String,
    /// A shared, thread-safe collection for storing intercepted HTTP requests.
    /// Used to temporarily collect requests before they are processed.
    dock_collector: Arc<Mutex<Vec<String>>>,

    dork_link_searcher : String,

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
    selected_repeater_request_text: String,

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
    Intro,
    Recon,
    Proxy,
    Repeater,
    encoder,
    decoder,
    //bruteforcer
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
        thread::spawn(move || {
            servers::json_thread_listner::start_warp_server(request_store_clone);
        });

        thread::spawn(move || {
            servers::dorklistiner::start_link_server(google_dork_collector_clone);
        });
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
            active_tab: Tab::Intro,
            save_pdf_filename: "output.pdf".to_owned(),
            show_pop_up: false,
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
            selected_repeater_request_text: String::from(""),
            dork_link_searcher: String::from(""),
            search_url: String::from(""),
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
                if ui.button("Intro").clicked() {
                    self.active_tab = Tab::Intro;
                }
                if ui.button("Reconnisense").clicked() {
                    self.active_tab = Tab::Recon;
                }
                if ui.button("Proxy").clicked() {
                    self.active_tab = Tab::Proxy;
                }

                if ui.button("Repeater").clicked() {
                    self.active_tab = Tab::Repeater;
                }

                if ui.button("Encoder").clicked() {
                    self.active_tab = Tab::encoder;
                }
                if ui.button("Decoder").clicked() {
                    self.active_tab = Tab::decoder;
                }
            });

            ui.separator();

            match self.active_tab {
                Tab::Intro => self.intro_tab(ui),
                Tab::Recon => self.capture_google_dork_tab(ui),
                Tab::Proxy => self.proxy_tab(ui),
                Tab::Repeater => self.repeater_tab(ui),
                Tab::encoder => self.encoder_tab(ui),
                Tab::decoder => self.decoder_tab(ui),
            }
        });

        // Request repaint to update the UI periodically
        ctx.request_repaint_after(Duration::from_millis(600));
        if self.show_pop_up {
            egui::Window::new("Save as PDF")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label("Enter file name (e.g., report.pdf):");
                    ui.text_edit_singleline(&mut self.save_pdf_filename);

                    ui.horizontal(|ui| {
                        if ui.button("Save").clicked() {
                            
                            self.genrate_pdf(
                                self.selected_repeater_request_text.clone(),
                                self.repeater_response.lock().unwrap().to_string(),
                                self.save_pdf_filename.clone(),
                            );
                            self.show_pop_up = false;
                        }

                        if ui.button("Cancel").clicked() {
                            self.show_pop_up = false;
                        }
                    });
                });
        }
    }
}

impl MyApp {

    fn intro_tab(&mut self, ui: &mut egui::Ui) {
    // Wrap content in a ScrollArea to make it scrollable
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            // Center the content vertically and horizontally
            ui.vertical_centered(|ui| {
                // Add some spacing at the top
                ui.add_space(20.0);

                // Title with a bold, large font
                ui.heading(
                    egui::RichText::new("Welcome to Shadow Proxy")
                        .strong()
                        .size(28.0)
                        .color(egui::Color32::from_rgb(255, 255, 255)),
                );

                // Subtitle or tagline
                ui.label(
                    egui::RichText::new("A powerful, intuitive tool for network analysis and testing")
                        .italics()
                        .size(16.0)
                        .color(egui::Color32::from_rgb(200, 200, 200)),
                );

                // Add spacing before the feature grid
                ui.add_space(30.0);

                // Use a grid layout to display features with equal column widths
                egui::Grid::new("intro_features_grid")
                    .num_columns(2)
                    .spacing([40.0, 20.0])
                    .min_col_width(ui.available_width() / 2.0 - 20.0) // Ensure equal column widths
                    .show(ui, |ui| {
                        // Feature 1: Recon
                        ui.vertical(|ui| {
                            // Set a maximum width for the vertical component
                            ui.set_max_width(ui.available_width());
                            let recon_heading_resp = ui.button(
                                egui::RichText::new("Recon")
                                    .strong()
                                    .size(18.0)
                                    .color(egui::Color32::from_rgb(100, 200, 255)),
                            );

                            
                            if recon_heading_resp.clicked() {
                                self.active_tab = Tab::Recon;
                            }
                            ui.label(
                                egui::RichText::new(
                                    "Discover and enumerate network targets with advanced reconnaissance tools. Scan for open ports, services, and vulnerabilities effortlessly."
                                )
                                .size(14.0),
                            );
                        });

                        // Feature 2: Proxy
                        ui.vertical(|ui| {
                            // Set a maximum width for the vertical component
                            ui.set_max_width(ui.available_width());
                            if ui.button(
                                egui::RichText::new("Proxy")
                                    .strong()
                                    .size(18.0)
                                    .color(egui::Color32::from_rgb(100, 200, 255)),
                            ).clicked() {
                                self.active_tab = Tab::Proxy;
                            }
                            ui.label(
                                egui::RichText::new(
                                    "Intercept and analyze HTTP/HTTPS traffic in real-time. Modify requests and responses to test application behavior."
                                )
                                .size(14.0),
                            );
                        });
                        ui.end_row();

                        // Feature 3: Repeater
                        ui.vertical(|ui| {
                            // Set a maximum width for the vertical component
                            ui.set_max_width(ui.available_width());
                            if ui.button(
                                egui::RichText::new("Repeater")
                                    .strong()
                                    .size(18.0)
                                    .color(egui::Color32::from_rgb(100, 200, 255)),
                            ).clicked() {
                                self.active_tab = Tab::Repeater;
                            }
                            ui.label(
                                egui::RichText::new(
                                    "Replay and tweak requests to test server responses. Perfect for debugging and exploring edge cases."
                                )
                                .size(14.0),
                            );
                        });

                        // Feature 4: Encoder
                        ui.vertical(|ui| {
                            // Set a maximum width for the vertical component
                            ui.set_max_width(ui.available_width());
                            if ui.button(
                                egui::RichText::new("Encoder")
                                    .strong()
                                    .size(18.0)
                                    .color(egui::Color32::from_rgb(100, 200, 255)),
                            ).clicked() {
                                self.active_tab = Tab::encoder;
                            }
                            ui.label(
                                egui::RichText::new(
                                    "Encode and decode data in various formats (Base64, URL, Hex, etc.) to assist in crafting payloads and analyzing responses."
                                )
                                .size(14.0),
                            );
                        });
                        ui.end_row();
                    });

                // Add spacing before the closing message
                ui.add_space(30.0);

                // Closing message or call to action
                ui.label(
                    egui::RichText::new(
                        "Get started by selecting a tab above to explore Shadow Proxy's powerful features!"
                    )
                    .size(16.0)
                    .color(egui::Color32::from_rgb(180, 180, 180)),
                );

                // Add a separator line
                ui.add_space(10.0);
                ui.separator();

                // Optional: Add a small footer
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("Built with ❤️ by Zohaib")
                            .size(12.0)
                            .color(egui::Color32::from_rgb(150, 150, 150)),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.hyperlink_to(
                            "Documentation",
                            "https://your-documentation-link.com",
                        );
                        ui.label(" | ");
                        ui.hyperlink_to(
                            "GitHub",
                            "https://github.com/zohaib2k2/shadowproxy_gui",
                        );
                    });
                });

                // Add extra space at the bottom to ensure content doesn't feel cramped
                ui.add_space(20.0);
            });
        });
    }
    //TODO: Implement recon like google dorking
    /// Displays the Google Dork capture tab.
    ///
    /// This function is responsible for rendering the UI elements related to
    /// capturing Google Dork search results.
    fn capture_google_dork_tab(&mut self, ui: &mut egui::Ui) {
        //let mut dork_link_searcher = "".to_string();

        ui.label("Google Dorks");
        //let aval_height = ui.available_height();

        ui.add(egui::TextEdit::singleline( &mut self.dork_link_searcher));
        
        let max_height = ui.available_height() * (0.75);
        // Make the table scroll-able both horizontally and vertically, you motherfucker.
        
        ScrollArea::both()
            .id_source("dorkable_table_scroll")
            .max_height(max_height)
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                egui::Grid::new("Dorking table")
                    .striped(true)
                    .min_col_width(20.0)
                    .show(ui, |ui| {
                        // "Its not death, but dying which is terriable"
                        // -- Henry Fielding (1707)

                        // Table headers
                        ui.label("Index");
                        ui.label("URL");
                        ui.end_row();

                        let log = self.dock_collector.lock().unwrap();

                        for (index, entry) in log.iter().enumerate() {
                            if self.stop_proxy {
                                continue;
                            }
                            if entry.clone().contains(&self.dork_link_searcher.to_string()) {
                                if ui.button(format!("{}", index + 1)).clicked() {
                                    ui.output_mut( |o| o.copied_text = entry.to_string());
                                }
                                    
                                    
                                ui.label(entry);
                                
                                ui.end_row();
                            }
                        }
                    });
            });

        ui.separator();
       
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
        ui.horizontal(|ui| {
            if ui.button("Stop").clicked() {
                self.stop_proxy = true;
            }
            if ui.button("Start").clicked() {
                self.stop_proxy = false;
            }
        });
        let max_height = ui.available_height() * (0.75);
        // Make the table scroll-able both horizontally and vertically, you motherfucker.
        ui.add(
            egui::TextEdit::singleline(&mut self.search_url)
                .hint_text("Type to search url...")
                .desired_width(200.0),
        );
        ScrollArea::both()
            .id_source("Proxy_table_scroll")
            .max_height(max_height)
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                egui::Grid::new("Proxy table")
                    .striped(true)
                    .min_col_width(20.0)
                    .show(ui, |ui| {
                        // "Its not death, but dying which is terriable"
                        // -- Henry Fielding (1707)

                        // Table headers
                        ui.label("Index");
                        ui.label("Method");
                        ui.label("URL");
                        ui.label("Headers");
                        ui.end_row();

                        let log = self.request_store.lock().unwrap();

                        for (index, entry) in log.iter().enumerate() {
                            if self.stop_proxy {
                                continue;
                            }
                            if entry.url.clone().contains(&self.search_url) {
                                if ui.button(format!("{}", index + 1)).clicked() {
                                    self.selected_for_show = RequestData {
                                        request_type: entry.request_type.clone(),
                                        http_version: entry.http_version.clone(),
                                        method: entry.method.clone(),
                                        url: entry.url.clone(),
                                        headers: entry.headers.clone(),
                                        body: entry.body.clone(),
                                    };

                                    //println!("{}",self.selected_for_show.headers);
                                    let e1 = &self
                                        .selected_for_show
                                        .headers
                                        .replace("\"", "\\\"")
                                        .replace("'", "\"");
                                    let headers1: HashMap<String, String> =
                                        serde_json::from_str(e1).unwrap();
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
                    });
            });

        ui.separator();
        ui.horizontal(|ui| {
            if ui.button("Send Request").clicked() {
                let selected_req_for_show = self.selected_for_show.clone();

                self.send_request(ui.ctx().clone(), selected_req_for_show, Tab::Proxy);
            }
            if ui.button("Send to repeater").clicked() {
                self.update_selected_text();
                self.selected_repeater_request = self.selected_for_show.clone();
            }
        });
        ui.horizontal(|ui| {
            //ui.text_edit_multiline(&mut format!("{:?}",self.selected_for_show));
            let bottom_window_size = [500.0, 185.0];

            egui::ScrollArea::both()
                .id_source("proxy_show_window_scroll")
                .max_height(bottom_window_size[1])
                .min_scrolled_width(bottom_window_size[0])
                .max_width(bottom_window_size[0])
                .min_scrolled_height(bottom_window_size[1])
                .show(ui, |ui| {
                    let ui_resp = ui.add_sized(
                        bottom_window_size,
                        egui::TextEdit::multiline(&mut format!("{:?}", self.selected_for_show)),
                    );
                    if ui_resp.clicked_by(egui::PointerButton::Secondary) {
                        self.show_proxy_context_menu = true;
                    }

                    if self.show_proxy_context_menu {
                        egui::menu::bar(ui, |ui| {
                            ui.menu_button("Context Menu", |ui| {
                                if ui.button("Copy").clicked() {
                                    ui.output_mut(|o| o.copied_text = "Hello world!".to_string())
                                }
                            })
                        });
                    }
                });

            egui::ScrollArea::both()
                .id_source("response")
                .max_height(bottom_window_size[1])
                .max_width(bottom_window_size[0])
                .show(ui, |ui| {
                    let ui_resp = ui.add_sized(
                        bottom_window_size,
                        egui::TextEdit::multiline(&mut format!(
                            "{}",
                            self.response_text.lock().unwrap()
                        )),
                    );
                });
        });
    }
    /// Displays the Repeater tab, which allows modifying and resending requests.
    ///
    /// This function is responsible for rendering the UI elements of the
    /// request repeater feature.
    fn repeater_tab(&mut self, ui: &mut eframe::egui::Ui) {
        ui.separator();

        let available_height = ui.available_height();
        let available_width = ui.available_width();
        ui.horizontal(|ui| {
            if ui.button("Send").clicked() {
                self.send_request(
                    ui.ctx().clone(),
                    self.selected_repeater_request.clone(),
                    Tab::Repeater,
                );
                let _n = self.selected_repeater_request_text.clone();

                //println!("\n\n\nString: {}",_n);
                let pased_request = self.selected_repeater_request_text.parse::<RequestData>();
                let _v = self.selected_repeater_request.clone();
                //println!("selected for show: _v \n{:?}",_v);
                match pased_request {
                    Ok(parsed) => {
                        self.selected_repeater_request = parsed;
                    }
                    Err(err_str) => {
                        *self.repeater_response.lock().unwrap() = err_str;
                    }
                }
            }
            if ui.button("Print Req").clicked() {
                println!("{}", self.selected_repeater_request_text);
            }
            if ui.button("Print resp").clicked() {
                println!("{}", self.repeater_response.lock().unwrap())
            }
            if ui.button("Genrate PDF").clicked() {
                self.show_pop_up = true;
            }
        });

        ui.horizontal(|ui| {
            egui::ScrollArea::both()
                .id_source("request_rep")
                .max_height(available_height * 0.95)
                .max_width(available_width / 2.0)
                .min_scrolled_height(available_height)
                .show(ui, |ui| {
                    ui.add_sized(
                        [available_width / 2.0, available_height * 0.95],
                        egui::TextEdit::multiline(&mut self.selected_repeater_request_text)
                            .desired_width(f32::INFINITY),
                    );
                });
            egui::ScrollArea::both()
                .id_source("respone_rep")
                .max_height(available_height * 0.95)
                .max_width(available_width / 2.0)
                .min_scrolled_height(available_height)
                .show(ui, |ui| {
                    // Yar ab ma kud-kooshi kar lon ga muj se
                    // nai banta gui.
                    ui.add_sized(
                        [available_width / 2.0, available_height * 0.95],
                        egui::TextEdit::multiline(
                            &mut self.repeater_response.lock().unwrap().to_string(),
                        )
                        .desired_width(f32::INFINITY),
                    );
                });
        });
    }

    fn url_encode(&self, input: &str) -> String {
        input
            .bytes()
            .map(|b| match b {
                // Unreserved characters according to RFC 3986
                b'A'..=b'Z' |
                b'a'..=b'z' |
                b'0'..=b'9' |
                b'-' | b'_' | b'.' | b'~' => (b as char).to_string(),
                _ => format!("%{:02X}", b),
            })
            .collect()
    }


    const CONTROLS: &AsciiSet = &percent_encoding::NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'_')
    .remove(b'.')
    .remove(b'~');

    fn encoder_tab(&mut self, ui: &mut eframe::egui::Ui) {
         ui.separator();

        // Persistent state for the text input and output
        static mut INPUT_TEXT: String = String::new();
        static mut OUTPUT_TEXT: String = String::new();

        let available_height = ui.available_height();
        let available_width = ui.available_width();
       
         // Split the available space into two columns
    egui::SidePanel::right("encoder_controls")
        .resizable(false)
        .default_width(150.0)
        .show_inside(ui, |ui| {
            ui.vertical(|ui| {
                ui.set_height(available_height);
                
                // Encoding buttons
                if ui.button("URL Encode").clicked() {
                    unsafe {
                        //OUTPUT_TEXT = utf8_percent_encode(&INPUT_TEXT, &CONTROLS).to_string();
                        OUTPUT_TEXT = self.url_encode(&INPUT_TEXT);
                    }
                }
                if ui.button("URL Decode").clicked() {
                    unsafe {
                        OUTPUT_TEXT = urlencoding::decode(&INPUT_TEXT)
                            .unwrap()
                            .to_string();
                    }
                }
                if ui.button("HTML Encode").clicked() {
                    unsafe {
                        OUTPUT_TEXT = html_escape::encode_text(&INPUT_TEXT).to_string();
                    }
                }
                if ui.button("HTML Decode").clicked() {
                    unsafe {
                        OUTPUT_TEXT = html_escape::decode_html_entities(&INPUT_TEXT).to_string();
                    }
                }
                if ui.button("Base64 Encode").clicked() {
                    unsafe {
                        OUTPUT_TEXT = base64::encode(&INPUT_TEXT);
                    }
                }
                if ui.button("Base64 Decode").clicked() {
                    unsafe {
                        OUTPUT_TEXT = base64::decode(&INPUT_TEXT)
                            .map(|bytes| String::from_utf8_lossy(&bytes).to_string())
                            .unwrap_or(INPUT_TEXT.clone());
                    }
                }
                if ui.button("Clear").clicked() {
                    unsafe {
                        INPUT_TEXT.clear();
                        OUTPUT_TEXT.clear();
                    }
                }
            });
        });

    // Main content area with input and output text boxes
    egui::CentralPanel::default().show_inside(ui, |ui| {
        ui.horizontal(|ui| {
            ui.set_height(available_height);
            ui.set_width(available_width - 150.0); // Adjust for side panel width

            // Input and output text areas side by side
            ui.vertical(|ui| {
                ui.label("Input:");
                let input_response = ui.add(
                    egui::TextEdit::multiline(unsafe { &mut INPUT_TEXT })
                        .desired_rows(10)
                        .desired_width((available_width - 170.0) / 2.0),
                );
                //println!("{:?}",inp);
                // Update input text when user types
                if input_response.changed() {
                    unsafe {
                        INPUT_TEXT = INPUT_TEXT.clone();
                    }
                }
            });

            ui.add_space(10.0);

            ui.vertical(|ui| {
                ui.label("Output:");
                ui.add(
                    egui::TextEdit::multiline(unsafe { &mut OUTPUT_TEXT })
                        .desired_rows(10)
                        .desired_width((available_width - 170.0) / 2.0)
                        .interactive(false), // Make output read-only
                );
            });
        });
    });
    }

    fn decoder_tab(&mut self, ui: &mut eframe::egui::Ui) {
        ui.separator();
    }

    fn update_selected_text(&mut self) {
        println!("\n\n\nupdate: {:?}", self.selected_repeater_request);
        println!(
            "\n\n\nupdate .to_string: {:?}",
            self.selected_repeater_request.to_string()
        );
        self.selected_repeater_request_text = self.selected_repeater_request.to_string();
    }

    /// Sends the currently selected request asynchronously.
    ///
    /// - Clones the selected request data.
    /// - Uses `reqwest::Client` to make an HTTP request.
    /// - Attaches headers and request body if provided.
    /// - Updates `response_text` with the server's response or an error message.
    /// - Runs the request in a separate thread to avoid blocking the UI.

    fn send_request(&self, ctx: egui::Context, selected_req: RequestData, tab: Tab) {
        let selected_request = selected_req;

        let response_text = match tab {
            Tab::Proxy => Arc::clone(&self.response_text),
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

                        let req = match tab{
                            Tab::Proxy => {
                                let req =client.request(
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
                                req
                            },
                            Tab::Repeater => {
                                let selected_request = selected_request.to_string();
                                let v = selected_request.clone();
                                println!("{:?}",v);
                                let parsed = selected_request.parse::<RequestDataProper>().unwrap();
                                
                                let req = client.request(parsed.method.parse(), parsed.clone().url);
                                println!("DEBUG URL: {}",parsed.clone().url);
                                let req = if parsed.body.is_empty() {
                                    req
                                } else {
                                    req.body(parsed.body)
                                };
                                
                                let req = if parsed.headers.is_empty() {
                                    req
                                } else {
                                    req.headers(parsed.headers.parse())
                                };

                                req

                            },
                            Tab::Recon => {
                                panic!("");
                            },
                            Tab::encoder => {
                                todo!("implement a panic handler.")
                            },
                            Tab::decoder => {
                                todo!("Implement a panic handler.")
                            }
                            Tab::Intro => {
                                todo!("do nothing")
                            }


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
                        *response_text.lock().unwrap() = "Failed to send request, check your internet connection OR network Setting.".to_string();
                    }
                }
            });
        });
    }

    /// generates a pdf that takes the left panel string of repeater
    /// and right panel string of repeater.
    fn genrate_pdf(&self,left_panel_text: String, right_panel_text: String, filename: String) {
        let font_family =
            genpdf::fonts::from_files("/usr/share/fonts/liberation", "LiberationSans", None)
                .expect("Failed to load font family");
        // Create a document and set the default font family
        let mut doc = genpdf::Document::new(font_family);

        // Change the default settings
        doc.set_title("Demo document");
        // Customize the pages
        let mut decorator = genpdf::SimplePageDecorator::new();
        decorator.set_margins(10);
        doc.set_page_decorator(decorator);
        // Add one or more elements
        doc.push(
            genpdf::elements::Paragraph::new("Repeater/Response Document")
                .aligned(genpdf::Alignment::Center)
                .styled(genpdf::style::Style::new().bold().with_font_size(20)),
        );
        doc.push(
            elements::Paragraph::new("Request:")
                .aligned(Alignment::Left)
                .styled(style::Style::new().bold().with_font_size(15)),
        );
        for line in left_panel_text.split("\n") {
            doc.push(genpdf::elements::Paragraph::new(line));
        }

        doc.push(
            elements::Paragraph::new("Response")
                .aligned(Alignment::Left)
                .styled(style::Style::new().bold().with_font_size(15)),
        );

        for line in right_panel_text.split("\n") {
            doc.push(genpdf::elements::Paragraph::new(line));
        }
        // Render the document and write it to a file
        doc.render_to_file(filename)
            .expect("Failed to write PDF file");
}
}


fn main() {
    log4rs::init_file("log4rs.yaml", Default::default()).unwrap();

    info!("Application Starting");
    let options = eframe::NativeOptions {
        initial_window_size: Some(Vec2::new(500.0, 400.0)),
        ..Default::default()
    };
    let _ = eframe::run_native(
        "Egui Background Task Example",
        options,
        Box::new(|_cc| Box::new(MyApp::default())),
    );
}
