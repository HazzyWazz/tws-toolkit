use std::collections::HashSet;
use std::fmt::format;
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use e621_api;
use reqwest_middleware::ClientWithMiddleware;

pub struct Config {
	pub username: String,
	pub api_key: String,
	pub booru_name: String,
	pub booru_url: String,
	pub booru_rate_limit_millis: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TwsPoolEntry {
	#[serde(alias = "a")]
	title: String,
	#[serde(alias = "b")]
	page: u64,
	#[serde(alias = "c")]
	tags: Vec<String>,
	#[serde(alias = "d")]
	date_created: u64,
	#[serde(alias = "e")]
	date_used: u64,
	#[serde(alias = "f")]
	new_posts: u64
}

pub fn read_conf(conf: &Path) -> Config {
	let contents = fs::read_to_string(conf).expect("Cannot read config file");

	let mut username: String = "".to_string();
	let mut api_key: String = "".to_string();
	let mut booru_name: String = "".to_string();
	let mut booru_url: String = "".to_string();
	let mut booru_rate_limit_millis: u64 = 0;

	for line in contents.lines() {
		let line_split = line.split('=').collect::<Vec<&str>>();
		match line_split[0] {
			"username" => { username = line_split[1].to_string() },
			"api_key" => { api_key = line_split[1].to_string(); },
			"booru_name" => { booru_name = line_split[1].to_string(); },
			"booru_url" => { booru_url = line_split[1].to_string(); },
			"booru_rate_limit_millis" => { booru_rate_limit_millis = line_split[1].parse::<u64>().unwrap() },
			_ => continue,
		}
	}

	let conf = Config {
		username,
		api_key,
		booru_name,
		booru_url,
		booru_rate_limit_millis,
	};

	conf
}


// --- Read TWS export ---
// local_worker task


// --- Extract saved pools into TwsPoolEntry ---
pub fn get_pools(entries: Vec<TwsPoolEntry>) -> HashSet<i64> {
	let mut pools: HashSet<i64> = HashSet::new();

	for entry in entries {
		if entry.tags[0].contains(&"pool:".to_string()) && !entry.tags[0].contains(&"inpool:".to_string()) {
			let pool_id: i64 = entry.tags[0].split(':')
				.collect::<Vec<&str>>()
				.last().unwrap()
				.parse().unwrap();
			pools.insert(pool_id);
		}
	}
	pools
}

// --- convert timestamps 'd','e'---
fn timestamp_converter(timestamp: i64) -> DateTime<Utc> {
	DateTime::from_timestamp_millis(timestamp).expect("Invalid timestamp")
}

// --- Save pool data ---
pub(crate) async fn save_pool_metadata(id: i64, output_path: &Path, client: ClientWithMiddleware) {
	let file_name = output_path.join(format!("e621_pool_{id}.json"));
	let output_file = OpenOptions::new()
		.read(true)
		.write(true)
		.create(true)
		.append(false)
		.open(file_name).unwrap();

	let pool_metadata = e621_api::pools::get_pool(id, client).await;
	serde_json::to_writer(output_file, &pool_metadata).unwrap();
}
// --- Save pool to txt to be used with gallery-dl ---
pub(crate) fn save_pool_id(mut gallery_file: File, id: i64) {
	let pool_url = format!("https://e621.net/pools/{id}\n");
	gallery_file.write_all(pool_url.as_bytes()).unwrap();
}
