#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use bgg_egui::MyApp;

// @see https://boardgamegeek.com/wiki/page/BGG_XML_API
// @see https://boardgamegeek.com/xmlapi/collection/cedeber

#[cfg(not(target_arch = "wasm32"))]
mod desktop {
	use clap::Parser;

	/// Simple program to list all board games from a BoardGameGeek user.
	#[derive(Parser, Debug)]
	#[clap(author, version, about, long_about = None)]
	pub struct Args {
		/// BoardGameGeek Username
		#[arg()]
		pub username: Option<String>,

		/// Filter by title with a RegExp
		#[arg(short, long, requires = "username")]
		pub filter: Option<String>,

		/// How long you want to play, in minutes. (+/- 10 minutes)
		#[arg(short, long, requires = "username")]
		pub time: Option<i64>,

		/// How many players
		#[arg(short, long, requires = "username")]
		pub players: Option<i64>,

		/// Export to a TOML file
		#[arg(short, long, requires = "username")]
		pub export: bool,
	}
}

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() {
	use bgg_egui::{export, fetch_collection, filter, output};
	use clap::Parser;

	env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

	// parse the CLI arguments
	let args = desktop::Args::parse();

	if let Some(username) = &args.username {
		// Fetch all games from BGG
		let games = fetch_collection(username).await;

		if games.is_err() {
			println!("Fetching the games in BGG failed: {}", games.err().unwrap());
			return;
		}

		let mut games = games.unwrap();

		// Apply the regex filter if any
		games = match &args.filter {
			Some(regex) => filter(&games, regex),
			None => games,
		};

		// Filter the games by number of players
		if let Some(players) = args.players {
			games.retain(|game| {
				game.min_players.unwrap_or_default() <= players
					&& game.max_players.unwrap_or_default() >= players
			})
		}

		// Filter the games by time (+/- 10 minutes)
		if let Some(time) = args.time {
			games.retain(|game| {
				let playtime = game.playtime.unwrap_or_default();
				playtime <= time + 10 && playtime >= time - 10
			})
		}

		if args.export {
			// Export to TOML
			export(&games);
		} else {
			// Output the list of filtered games in the console.
			output(&games);
		}
	} else {
		let options = eframe::NativeOptions {
			viewport: egui::ViewportBuilder::default()
				.with_inner_size([640.0, 480.0])
				.with_min_inner_size([400.0, 300.0]),
			// renderer: Renderer::Wgpu,
			..Default::default()
		};

		eframe::run_native(
			"Board Game Geek",
			options,
			Box::new(|cc| Box::new(MyApp::new(cc))),
		)
		.expect("failed to start eframe");
	}
}

// When compiling to web using trunk:
#[cfg(target_arch = "wasm32")]
fn main() {
	// Redirect `log` message to `console.log` and friends:
	eframe::WebLogger::init(log::LevelFilter::Debug).ok();

	let web_options = eframe::WebOptions::default();

	wasm_bindgen_futures::spawn_local(async {
		eframe::WebRunner::new()
			.start(
				"the_canvas_id", // hardcode it
				web_options,
				Box::new(|cc| Box::new(MyApp::new(cc))),
			)
			.await
			.expect("failed to start eframe");
	});
}
