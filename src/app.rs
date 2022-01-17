use std::{fs, thread};
use std::collections::hash_map::Entry;
use std::sync::{Arc, mpsc, RwLock};
use std::collections::HashMap;
use eframe::{egui, epi};

use crate::data;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct AppOrganizerApp {
    // Example stuff:
    app_list: Vec<data::AndroidApp>,
    selected: Option<usize>,

    // this how you opt-out of serialization of a member
    #[serde(skip)]
    rt_thread: Option<(thread::JoinHandle<()>, mpsc::Sender<Option<Vec<String>>>)>,
    #[serde(skip)]
    images: Arc<RwLock<HashMap<String, ImageOrTexture>>>,
}

impl Default for AppOrganizerApp {
    fn default() -> Self {
        Self {
            app_list: Vec::new(),
            selected: None,
            rt_thread: None,
            images: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl epi::App for AppOrganizerApp {
    fn name(&self) -> &str {
        "Android App Organizer"
    }

    /// Called once before the first frame.
    fn setup(
        &mut self,
        _ctx: &egui::CtxRef,
        _frame: &epi::Frame,
        _storage: Option<&dyn epi::Storage>,
    ) {
        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        #[cfg(feature = "persistence")]
        if let Some(storage) = _storage {
            *self = epi::get_value(storage, epi::APP_KEY).unwrap_or_default()
        }
        get_images(&self.app_list, &mut self.rt_thread, &self.images);
    }

    /// Called by the frame work to save state before shutdown.
    /// Note that you must enable the `persistence` feature for this to work.
    #[cfg(feature = "persistence")]
    fn save(&mut self, storage: &mut dyn epi::Storage) {
        epi::set_value(storage, epi::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::CtxRef, frame: &epi::Frame) {
        let Self { app_list, selected, rt_thread, images} = self;

        // Examples of how to create different panels and windows.
        // Pick whichever suits you.
        // Tip: a good default choice is to just keep the `CentralPanel`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Import from LMA text").clicked() {
                        match native_dialog::FileDialog::new().show_open_single_file() {
                            Ok(Some(p)) => {
                                match fs::read_to_string(p) {
                                    Ok(t) => {
                                        *app_list = data::parse_lma_text(t);
                                        *selected = None;
                                        get_images(app_list, rt_thread, images);
                                    },
                                    Err(e) => {
                                        native_dialog::MessageDialog::new()
                                            .set_type(native_dialog::MessageType::Error)
                                            .set_text(&*e.to_string())
                                            .show_alert()
                                            .expect("There was an error showing the open failure error alert.");
                                    },
                                }
                            },
                            Ok(None) | Err(_) => {},
                        }
                    }
                    if ui.button("Import from JSON").clicked() {
                        match native_dialog::FileDialog::new()
                            .add_filter("JSON Text", &["json"])
                            .show_open_single_file() {
                            Ok(Some(p)) => {
                                match fs::read_to_string(p) {
                                    Ok(t) => {
                                        match serde_json::from_str(&*t) {
                                            Ok(v) => {
                                                *app_list = v;
                                                *selected = None;
                                                get_images(app_list, rt_thread, images);
                                            },
                                            Err(e) => {
                                                native_dialog::MessageDialog::new()
                                                    .set_type(native_dialog::MessageType::Error)
                                                    .set_text(&*e.to_string())
                                                    .show_alert()
                                                    .expect("There was an error showing the open failure error alert.");
                                            }
                                        }
                                        *selected = None;
                                    },
                                    Err(e) => {
                                        native_dialog::MessageDialog::new()
                                            .set_type(native_dialog::MessageType::Error)
                                            .set_text(&*e.to_string())
                                            .show_alert()
                                            .expect("There was an error showing the open failure error alert.");
                                    },
                                }
                            },
                            Ok(None) | Err(_) => {},
                        }
                    }
                    if ui.button("Save to JSON").clicked() {
                        save_to_json(app_list, false)
                    }
                    if ui.button("Save to prettified JSON").clicked() {
                        save_to_json(app_list, true)
                    }
                    if ui.button("Clear").clicked() {
                        *app_list = Vec::new();
                        *selected = None;
                    }
                    if ui.button("Quit").clicked() {
                        frame.quit();
                    }
                });
            });
        });

        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.heading("Imported apps");
            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.with_layout(egui::Layout::top_down_justified(egui::Align::Min), |ui| {
                    // ui.spacing_mut().item_spacing.y = 0.0;

                    for (i, app) in app_list.iter().enumerate() {
                        if ui.selectable_label(selected.as_ref()
                                                   .map(|s| app_list[*s].name == app.name)
                                                   .unwrap_or(false), &app.name)
                            .clicked() {
                            *selected = Some(i);
                        }
                    }
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::warn_if_debug_build(ui);
            match selected {
                Some(i) => {
                    ui.heading(&app_list[*i].name);
                    let img = {
                        let mut imgs = images.write().unwrap();
                        let entry = imgs.entry(app_list[*i].id.clone());
                        match entry {
                            Entry::Occupied(mut o) => {
                                match o.get() {
                                    ImageOrTexture::Image(im) => {
                                        let t = frame.alloc_texture(im.clone());
                                        o.insert(ImageOrTexture::Texture(t));
                                        Some(t)
                                    }
                                    ImageOrTexture::Texture(t) => Some(t.clone())
                                }
                            }
                            Entry::Vacant(_) => None
                        }
                    };
                    if let Some(t) = img {
                        ui.image(t, egui::Vec2::splat(100.0));
                    }
                    ui.label(format!("id: {}", app_list[*i].id));
                    ui.checkbox(&mut app_list[*i].delete, "To delete");
                    ui.add_sized(ui.available_size(), egui::TextEdit::multiline(&mut app_list[*i].notes));
                },
                None => {
                    ui.heading("Nothing selected");
                },
            }
        });
    }

    fn on_exit(&mut self) {
        if let Some((t, s)) = self.rt_thread.take() {
            s.send(None).unwrap();
            t.join().unwrap();
        }
    }
}

fn save_to_json(app_list: &Vec<data::AndroidApp>, pretty: bool) {
    match native_dialog::FileDialog::new()
        .add_filter("JSON Text", &["json"])
        .set_filename("apps.json")
        .show_save_single_file() {
        Ok(Some(p)) => {
            match if pretty { serde_json::to_string_pretty(app_list) } else { serde_json::to_string(app_list) }
                .map(|s| fs::write(p, s)) {
                Ok(Ok(_)) => {},
                Ok(Err(e)) => {
                    native_dialog::MessageDialog::new()
                        .set_type(native_dialog::MessageType::Error)
                        .set_text(&*e.to_string())
                        .show_alert()
                        .expect("There was an error showing the save failure error alert.");
                }
                Err(e) => {
                    native_dialog::MessageDialog::new()
                        .set_type(native_dialog::MessageType::Error)
                        .set_text(&*e.to_string())
                        .show_alert()
                        .expect("There was an error showing the save failure error alert.");
                }
            }
        },
        Ok(None) => {},
        Err(e) => {
            native_dialog::MessageDialog::new()
                .set_type(native_dialog::MessageType::Error)
                .set_text(&*e.to_string())
                .show_alert()
                .expect("There was an error showing the save failure error alert.");
        }
    }
}

type AsyncResult<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[tokio::main]
async fn async_main(rx: mpsc::Receiver<Option<Vec<String>>>, images: Arc<RwLock<HashMap<String, ImageOrTexture>>>) {
    loop {
        match rx.recv().unwrap() {
            Some(ids_list) => {
                let mut tasks = Vec::new();
                for id in ids_list.iter() {
                    tasks.push(tokio::task::spawn(dispatch_get_icon(id.clone(), images.clone())));
                }
            },
            None => break,
        }
    }
}

async fn dispatch_get_icon(id: String, images: Arc<RwLock<HashMap<String, ImageOrTexture>>>) -> AsyncResult<()> {
    {
        let imgs = images.read().unwrap();
        if imgs.contains_key(&*id.clone()) {
            return Ok(());
        }
    }
    let img = tokio::task::spawn(data::get_icon(id.clone())).await?.ok_or("Error fetching icon.")?;
    let size = [img.width() as usize, img.height() as usize];
    let pixels = img.into_vec();
    let final_img = epi::Image::from_rgba_unmultiplied(size, &pixels);
    {
        let mut imgs = images.write().unwrap();
        (*imgs).insert(id.clone(), ImageOrTexture::Image(final_img.clone()));
    }
    Ok(())
}

enum ImageOrTexture {
    Image(epi::Image),
    Texture(egui::TextureId),
}

fn get_images(app_list: &Vec<data::AndroidApp>,
              rt_thread: &mut Option<(thread::JoinHandle<()>, mpsc::Sender<Option<Vec<String>>>)>,
              images: &Arc<RwLock<HashMap<String, ImageOrTexture>>>) {
    match rt_thread {
        Some((_t, s)) => {
            s.send(Some(app_list.iter().map(|a| a.id.clone()).collect())).unwrap();
        },
        None => {
            let (tx, rx) = mpsc::channel();
            let imgs = images.clone();
            *rt_thread = Some({
                let t = thread::Builder::new()
                    .name("image-fetcher".to_string())
                    .spawn(move || async_main(rx, imgs))
                    .unwrap();
                tx.send(Some(app_list.iter().map(|a| a.id.clone()).collect())).unwrap();
                (t, tx)
            });
        },
    }
}
