#![warn(clippy::all, rust_2018_idioms)]

mod app;
pub use app::MyApp;
pub use app::TemplateApp;
use regex::Regex;
use serde::{Deserialize, Serialize};

// Used for the TOML export. It will export by something like { games: [..] }
#[derive(Debug, Clone, Serialize, Deserialize)]
struct GameExport {
	games: Vec<BoardGame>,
}

// Each board game struct
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoardGame {
	pub id: i64,
	pub name: String,
	pub year: Option<i64>,
	pub min_players: Option<i64>,
	pub max_players: Option<i64>,
	pub playtime: Option<i64>, // minutes
}

#[derive(Debug, Clone, Serialize)]
pub struct User {
	pub id: Option<i64>,
	pub username: String,
}

// Fetch the list of games for a specific user from BGG.
// Also parse the XML and return a list of BoardGames.
pub async fn fetch_collection(username: &str) -> Result<Vec<BoardGame>, reqwest::Error> {
	let mut games: Vec<BoardGame> = Vec::new();

	// Fetch the BGG API ans save the XML as text.
	let resp = reqwest::get(format!(
		"https://boardgamegeek.com/xmlapi/collection/{}",
		username
	))
	.await?
	.text()
	.await
	.unwrap(); // parsing to text should be fine.

	// Parse the XML.
	let doc = match roxmltree::Document::parse(&resp) {
		Ok(doc) => doc,
		Err(_) => {
			// TODO Export parsing error
			return Ok(games);
		}
	};

	// Check if the returned XML does not contain <message>
	if doc.root_element().has_tag_name("message") {
		let message = doc.root_element().text().unwrap();
		println!("{}", message.trim());
		return Ok(vec![]);
	}

	// Check if the returned XML does not contain <errors><error><message>
	if doc.root_element().has_tag_name("errors") {
		let message = doc
			.root_element() // <errors>
			.first_element_child() // <error>
			.unwrap()
			.first_element_child() // <message>
			.unwrap()
			.text()
			.unwrap();
		println!("Error: {}", message.trim());
		return Ok(vec![]);
	}

	// Go trough the XML Nodes and extract useful information.
	// We use unwrap because we believe the XML API is stable and trustable.
	// Crashing the application here means that the API changed anyway.
	for node in
		// If I own the game
		doc
			.descendants()
			.filter(|n| n.has_tag_name("item"))
			.filter(|n| n.attribute("subtype") == Some("boardgame"))
			.filter(|n| {
				n.descendants()
					.find(|n| n.has_tag_name("status"))
					.unwrap()
					.attribute("own") == Some("1")
			}) {
		let id = node.attribute("objectid").unwrap();
		let mut children = node.children();
		let name = children.find(|n| n.has_tag_name("name"));
		let year = children.find(|n| n.has_tag_name("yearpublished"));
		let stats = children.find(|n| n.has_tag_name("stats"));

		// Default values
		let mut playtime = Some(0);
		let mut min_players = Some(0);
		let mut max_players = Some(0);

		if let Some(stats) = stats {
			playtime = stats
				.attribute("playingtime")
				.map(|time| time.parse::<i64>().unwrap_or_default());
			min_players = stats
				.attribute("minplayers")
				.map(|min| min.parse::<i64>().unwrap_or_default());
			max_players = stats
				.attribute("maxplayers")
				.map(|max| max.parse::<i64>().unwrap_or_default());
		}

		// If the game has a name, extract data and add it to the games list.
		if let Some(name) = name {
			let name = name.text().unwrap();
			let year = year.map(|year| year.text().unwrap().parse::<i64>().unwrap_or_default());

			games.push(BoardGame {
				id: id.parse::<i64>().unwrap_or_default(),
				name: String::from(name),
				year,
				playtime,
				min_players,
				max_players,
			});
		}
	}

	// return the games list
	Ok(games)
}

// TODO Fuzzy search
// Filter the games by name
pub fn filter(games: &[BoardGame], regex: &str) -> Vec<BoardGame> {
	let re = Regex::new(regex).unwrap();

	games
		.iter()
		.filter(|game| re.find(&game.name).is_some())
		.cloned()
		.collect()
}

// Output the games list to the console
#[cfg(not(target_arch = "wasm32"))]
pub fn output(games: &[BoardGame]) {
	use std::io::Write;

	use console::{style, Term};
	use unicode_segmentation::UnicodeSegmentation;

	if games.is_empty() {
		println!("No games found.");
		return;
	}

	let mut term = Term::stdout();
	let title_max_length: usize = games
		.iter()
		.map(|g| g.name.graphemes(true).count())
		.max_by(|x, y| x.cmp(y))
		.unwrap();

	for game in games {
		let text = format!(
			"⎮ {} ⎮ {} ⎮ {} ⎮ {}",
			style(format!(
				"{:1} ⇢{:2}p",
				&game.min_players.unwrap_or_default().min(99),
				&game.max_players.unwrap_or_default().min(99)
			))
			.magenta(),
			style(format!(
				"{:3}m",
				&game.playtime.unwrap_or_default().min(999)
			))
			.green(),
			style(format!("{:4}", &game.year.unwrap_or_default())).cyan(),
			&game.name
		);

		// term.move_cursor_right(title_max_length + 1).unwrap();
		// term.write_all("⎮".as_bytes()).unwrap();
		term.write_all(text.as_bytes()).unwrap();
		term.move_cursor_right(title_max_length - game.name.graphemes(true).count() + 1)
			.unwrap();
		term.write_line("⎮").unwrap();
	}
}

// Export to TOML
#[cfg(not(target_arch = "wasm32"))]
pub fn export(games: &[BoardGame]) {
	use std::{fs::File, io::Write};

	let export = GameExport {
		games: games.to_owned(),
	};

	let toml = toml::to_string(&export).unwrap();
	let path = "export.toml";

	let mut output = File::create(path).unwrap();
	output.write_all(toml.as_ref()).unwrap();
}
