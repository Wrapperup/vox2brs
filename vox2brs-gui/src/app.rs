use std::fs::File;
use std::ops::RangeInclusive;
use std::path::Path;
use std::time::Instant;
use brickadia::save::{BrickOwner, SaveData, User};
use brickadia::write::SaveWriter;
use create_vox::VoxFile;
use eframe::{egui, epi};
use eframe::egui::{Checkbox, Color32, Hyperlink, Label, Layout, RichText, TextEdit, TextStyle, TopBottomPanel};
use eframe::egui::special_emojis::GITHUB;
use rampifier::{Rampifier, RampifierConfig};
use vox2brs::{BrickOutputMode, vox2brs};
use vox2brs::BrickOutputMode::Brick;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))] // if we add new fields, give them default values when deserializing old state
pub struct Vox2BrsApp {
    pub input_file_path: String,
    pub output_directory: String,
    pub save_name: String,
    pub mode: BrickOutputMode,
    pub width: f32,
    pub height: f32,
    pub simplify: bool,
    pub rampify: bool,
}

impl Default for Vox2BrsApp {
    fn default() -> Self {
        Self {
            input_file_path: "input.vox".into(),
            output_directory: "builds".into(),
            save_name: "output".into(),
            mode: BrickOutputMode::Brick,
            width: 1.0,
            height: 1.0,
            simplify: false,
            rampify: false,
        }
    }
}

impl epi::App for Vox2BrsApp {
    fn name(&self) -> &str {
        "vox2brs"
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::CtxRef, frame: &epi::Frame) {
        let input_file_valid = Path::new(&self.input_file_path).exists();
        let output_dir_valid = Path::new(&self.output_directory).is_dir();

        if self.mode == BrickOutputMode::MicroBrick && self.rampify {
            self.mode = BrickOutputMode::Brick;
        }

        self.simplify = self.simplify || self.rampify;

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading(RichText::new("vox2brs").monospace());
                ui.label("v0.1.0");
            });
            ui.label("Convert .vox files into .brs files!");

            ui.separator();

            ui.label(RichText::new("Configuration").heading());
            ui.label("Change how vox2brs converts your voxels into bricks.");
            ui.add_space(10.0);
            egui::Grid::new("paths")
                .num_columns(2)
                .spacing([40.0, 4.0])
                .striped(true)
                .show(ui, |ui| {
                    ui.label("VOX File").on_hover_text("MagicaVoxel Model to convert");
                    ui.horizontal(|ui| {
                        ui.add(TextEdit::singleline(&mut self.input_file_path).desired_width(400.0).text_color(bool_color(input_file_valid)));
                        if ui.button(RichText::new("🗁").color(Color32::from_rgb(255, 206, 70))).clicked() {
                            match nfd2::open_file_dialog(Some("vox"), None).unwrap() {
                                nfd2::Response::Okay(file_path) => {
                                    self.input_file_path = file_path.to_string_lossy().into_owned();
                                    self.save_name = match file_path.file_stem() {
                                        Some(s) => s.to_string_lossy().into_owned(),
                                        None => self.save_name.clone()
                                    };
                                },
                                _ => ()
                            }
                        }
                    });
                    ui.end_row();

                    ui.label("Output Directory").on_hover_text("Where generated save will be written to");
                    ui.horizontal(|ui| {
                        ui.add(TextEdit::singleline(&mut self.output_directory).desired_width(400.0).text_color(bool_color(output_dir_valid)));
                        if ui.button(RichText::new("🗁").color(Color32::from_rgb(255, 206, 70))).clicked() {
                            let default_dir = if output_dir_valid {
                                Some(Path::new(self.output_directory.as_str()))
                            } else {
                                None
                            };

                            match nfd2::open_pick_folder(default_dir).unwrap() {
                                nfd2::Response::Okay(file_path) => {
                                    self.output_directory = file_path.to_string_lossy().into_owned();
                                },
                                _ => ()
                            }
                        }
                    });
                    ui.end_row();

                    ui.label("Save Name");
                    ui.text_edit_singleline(&mut self.save_name);
                    ui.end_row();
                });

            ui.separator();

            egui::Grid::new("options")
                .num_columns(2)
                .spacing([40.0, 4.0])
                .striped(true)
                .show(ui, |ui| {
                    ui.label("Rampify");
                    ui.checkbox(&mut self.rampify, "Rampify the result. NOTE: Disables Microbricks as an option.");
                    ui.end_row();

                    ui.label("Simplify");
                    ui.add_enabled(!self.rampify, Checkbox::new(&mut self.simplify, "Optimizes bricks of the same color conservatively."));
                    ui.end_row();

                    ui.label("Brick Type");
                    egui::ComboBox::from_label("What kind of brick should be output?")
                        .selected_text(format!("{:?}", &mut self.mode))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.mode, BrickOutputMode::Brick, "Brick");
                            ui.selectable_value(&mut self.mode, BrickOutputMode::Plate, "Plate");
                            if !self.rampify {
                                ui.selectable_value(&mut self.mode, BrickOutputMode::MicroBrick, "MicroBrick");
                            }
                            else {
                                ui.horizontal(|ui| {
                                    ui.add_space(ui.spacing().button_padding.x);
                                    ui.colored_label(bool_color(false), "MicroBrick");
                                });
                            }
                        });
                    ui.end_row();

                    ui.label("Brick Size");
                    ui.horizontal(|ui| {
                        let range = RangeInclusive::new(1.0, f32::MAX);

                        ui.label("Width");
                        ui.add(egui::DragValue::new(&mut self.width).clamp_range(range.clone()).speed(1.0));
                        ui.label("Height");
                        ui.add(egui::DragValue::new(&mut self.height).clamp_range(range).speed(1.0));
                    });
                    ui.end_row();
                });

            ui.separator();

            ui.vertical_centered(|ui| {
                if ui.button("Convert VOX to BRS").clicked() {
                    let public = User {
                        name: "vox2brs".into(),
                        id: "a8033bee-6c37-4118-b4a6-cecc1d966133".parse().unwrap(),
                    };

                    let mut save = SaveData::default();

                    // set the first header
                    save.header1.author = public.clone();
                    save.header1.host = Some(public.clone());
                    save.header1.description = "Converted .vox file.".into();

                    // set the second header
                    save.header2
                        .brick_owners
                        .push(BrickOwner::from_user_bricks(public.clone(), 100));

                    save.header2.brick_assets =
                        vec![
                            "PB_DefaultBrick".into(),
                            "PB_DefaultMicroBrick".into(),
                            "PB_DefaultRamp".into(),
                            "PB_DefaultWedge".into(),
                        ];

                    if !Path::new(&self.input_file_path).exists() {
                        println!("Voxel file not found.");
                        return;
                    }

                    let vox_data = VoxFile::load(&self.input_file_path);

                    let mut result = vox2brs(
                        vox_data,
                        save,
                        self.mode,
                        Some(self.width as u32),
                        Some(self.height as u32),
                        self.simplify,
                        self.rampify,
                        0,
                        1,
                        2,
                        3,
                    );

                    let output_file_path = format!("{}\\{}.brs", self.output_directory, self.save_name);

                    match result {
                        Ok(out_save) => {
                            println!("\nWriting save file...");
                            let file = File::create(&output_file_path);

                            match file {
                                Ok(file) => {
                                    SaveWriter::new(file, out_save)
                                        .write()
                                        .unwrap();

                                    println!("Save written to {}", &output_file_path);
                                },
                                Err(error) => {
                                    println!("Could not write to {}", &output_file_path);
                                }
                            }
                        },
                        Err(error) => {
                            println!("Could not convert VOX file.");
                        }
                    }
                }
            });

            TopBottomPanel::bottom("bottom").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("vox2brs");
                    ui.add_space(10.0);
                    let text = format!("{} {}", GITHUB, "Source Code");
                    ui.add(Hyperlink::from_label_and_url(text, "https://github.com/Wrapperup/vox2brs"));
                    ui.add_space(10.0);
                    ui.label("Based on obj2brs & heightmap2brs");
                });
            });
        });
    }
}

pub fn bool_color(b: bool) -> Color32 {
    if b {
        Color32::WHITE
    } else {
        Color32::from_rgb(255, 50, 50)
    }
}
