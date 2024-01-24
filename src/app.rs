use egui::ScrollArea;
use egui_extras::{Column, Size, StripBuilder, TableBuilder};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

use crate::{fetch_collection, BoardGame};

#[derive(Clone)]
pub struct AsyncState {
	games: Vec<BoardGame>,
	is_loading: bool,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct MyApp {
	username: String,

	#[serde(skip)]
	async_state: Arc<Mutex<AsyncState>>,
}

impl Default for MyApp {
	fn default() -> Self {
		Self {
			username: "".to_owned(),
			async_state: Arc::new(Mutex::new(AsyncState {
				games: Vec::new(),
				is_loading: false,
			})),
		}
	}
}

impl MyApp {
	pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
		if let Some(storage) = cc.storage {
			return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
		}

		Default::default()
	}
}

impl eframe::App for MyApp {
	/// Called by the frame work to save state before shutdown.
	fn save(&mut self, storage: &mut dyn eframe::Storage) {
		eframe::set_value(storage, eframe::APP_KEY, self);
	}

	fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
		egui::CentralPanel::default().show(ctx, |ui| {
			ui.horizontal(|ui| {
				ui.label("Username: ");
				ui.text_edit_singleline(&mut self.username);

				if ui.button("Load").clicked() {
					let clone = self.clone();
					let async_state_mutex = Arc::clone(&self.async_state);

					let mut async_state = async_state_mutex.lock().unwrap();
					async_state.is_loading = true;

					// Unlock Mutex guard, before usage again in thread.
					// Mutex::unlock() is unstable.
					drop(async_state);

					#[cfg(not(target_arch = "wasm32"))]
					tokio::spawn(async move {
						let games = fetch_collection(&clone.username).await;
						let mut async_state = async_state_mutex.lock().unwrap();

						if let Ok(games) = games {
							async_state.games = games;
						} else {
							// println!("Fetching the games in BGG failed: {}", games.err().unwrap());
						}

						async_state.is_loading = false;

						// Unlock Mutex guard, before usage again in set_checked.
						// Mutex::unlock() is unstable.
						// drop(boardgames);
					});

					#[cfg(target_arch = "wasm32")]
					wasm_bindgen_futures::spawn_local(async move {
						let games = fetch_collection(&clone.username).await;
						let mut async_state = async_state_mutex.lock().unwrap();

						if let Ok(games) = games {
							async_state.games = games;
						} else {
							// println!("Fetching the games in BGG failed: {}", games.err().unwrap());
						}

						async_state.is_loading = false;

						// Unlock Mutex guard, before usage again in set_checked.
						// Mutex::unlock() is unstable.
						// drop(boardgames);
					});
				}

				let async_state_mutex = Arc::clone(&self.async_state);
				let async_state = async_state_mutex.lock().unwrap();

				if async_state.is_loading {
					ui.spinner();
				}
			});

			let async_state_mutex = Arc::clone(&self.async_state);
			let async_state = async_state_mutex.lock().unwrap();

			StripBuilder::new(ui)
				.size(Size::remainder().at_least(100.0))
				.vertical(|mut strip| {
					strip.cell(|ui| {
						ScrollArea::horizontal().show(ui, |ui| {
							let table = TableBuilder::new(ui)
								.cell_layout(egui::Layout::left_to_right(egui::Align::Center))
								.column(Column::auto())
								.column(Column::auto())
								.column(Column::auto())
								.column(Column::remainder());

							table
								.header(20.0, |mut header| {
									header.col(|ui| {
										ui.strong("Players");
									});
									header.col(|ui| {
										ui.strong("Playtime");
									});
									header.col(|ui| {
										ui.strong("Release");
									});
									header.col(|ui| {
										ui.strong("Title");
									});
								})
								.body(move |mut body| {
									for game in &async_state.games {
										body.row(18.0, |mut row| {
											row.col(|ui| {
												ui.label(format!(
													"{}-{:2}",
													&game.min_players.unwrap_or_default().min(99),
													&game.max_players.unwrap_or_default().min(99)
												));
											});
											row.col(|ui| {
												ui.label(format!(
													"{:3}m",
													&game.playtime.unwrap_or_default().min(999)
												));
											});
											row.col(|ui| {
												ui.label(format!(
													"{:4}",
													&game.year.unwrap_or_default()
												));
											});
											row.col(|ui| {
												ui.label(&game.name);
											});
										})
									}
								});
						});
					});
				});
		});
	}
}
/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
	// Example stuff:
	label: String,

	#[serde(skip)] // This how you opt-out of serialization of a field
	value: f32,
}

impl Default for TemplateApp {
	fn default() -> Self {
		Self {
			// Example stuff:
			label: "Hello World!".to_owned(),
			value: 2.7,
		}
	}
}

impl TemplateApp {
	/// Called once before the first frame.
	pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
		// This is also where you can customize the look and feel of egui using
		// `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

		// Load previous app state (if any).
		// Note that you must enable the `persistence` feature for this to work.
		if let Some(storage) = cc.storage {
			return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
		}

		Default::default()
	}
}

impl eframe::App for TemplateApp {
	/// Called by the frame work to save state before shutdown.
	fn save(&mut self, storage: &mut dyn eframe::Storage) {
		eframe::set_value(storage, eframe::APP_KEY, self);
	}

	/// Called each time the UI needs repainting, which may be many times per second.
	fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
		// Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
		// For inspiration and more examples, go to https://emilk.github.io/egui

		egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
			// The top panel is often a good place for a menu bar:

			egui::menu::bar(ui, |ui| {
				// NOTE: no File->Quit on web pages!
				let is_web = cfg!(target_arch = "wasm32");
				if !is_web {
					ui.menu_button("File", |ui| {
						if ui.button("Quit").clicked() {
							ctx.send_viewport_cmd(egui::ViewportCommand::Close);
						}
					});
					ui.add_space(16.0);
				}

				egui::widgets::global_dark_light_mode_buttons(ui);
			});
		});

		egui::CentralPanel::default().show(ctx, |ui| {
			// The central panel the region left after adding TopPanel's and SidePanel's
			ui.heading("eframe template");

			ui.horizontal(|ui| {
				ui.label("Write something: ");
				ui.text_edit_singleline(&mut self.label);
			});

			ui.add(egui::Slider::new(&mut self.value, 0.0..=10.0).text("value"));
			if ui.button("Increment").clicked() {
				self.value += 1.0;
			}

			ui.separator();

			ui.add(egui::github_link_file!(
				"https://github.com/emilk/eframe_template/blob/master/",
				"Source code."
			));

			ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
				powered_by_egui_and_eframe(ui);
				egui::warn_if_debug_build(ui);
			});
		});
	}
}

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
	ui.horizontal(|ui| {
		ui.spacing_mut().item_spacing.x = 0.0;
		ui.label("Powered by ");
		ui.hyperlink_to("egui", "https://github.com/emilk/egui");
		ui.label(" and ");
		ui.hyperlink_to(
			"eframe",
			"https://github.com/emilk/egui/tree/master/crates/eframe",
		);
		ui.label(".");
	});
}
