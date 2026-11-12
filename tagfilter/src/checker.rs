// ---  TODO: make tws2pool save pool data ---

use std::collections::HashSet;
use std::fs;
use std::fs::OpenOptions;
use std::path::{Path, PathBuf};
use e621_api;
use e621_api::posts::Post;

// --- read folder for posts and pools ---
/// Function for pool mode where we need the name of the root folder
/// where the JSON came from (the pool)
fn read_pools(dir: &Path) -> Vec<PathBuf> {
	let mut pools: Vec<PathBuf> = Vec::new();
	if dir.is_dir() {
		for entry in fs::read_dir(dir).unwrap() {
			let entry = entry.unwrap();
			let path = entry.path();
			if path.is_dir() {
				pools.push(path);
			}
		}
	}
	pools
}

pub fn entry(dir: &Path, blacklist: &Path, pool_mode: bool) {
	let bl = read_blacklist(blacklist);
	if pool_mode {
		let pools = read_pools(dir);
		for pool in pools {
			let name = get_pool_name(&pool);
			let mut pool_tags: HashSet<String> = HashSet::new();
			let jsons = get_jsons(&pool);
			for json in jsons {
				let all_tags = read_json_tags(&json);
				for tag in all_tags {
					pool_tags.insert(tag);
				}
			}
			if hash_search(bl.clone(), pool_tags) {
				println!("Pool {} is blacklisted!", name);
			} else {
				// TODO: Something else if pool is fine
			}
		}
	} else {
		let mut post_tags: HashSet<String> = HashSet::new();
		let jsons = get_jsons(&dir);
		for json in jsons {
			let all_tags = read_json_tags(&json);
			for tag in all_tags {
				post_tags.insert(tag);
			}
			let post = parse_json(&json);
			let post_id = post.id;
			if hash_search(bl.clone(), post_tags.clone()) {
				println!("Post {} is blacklisted!", post_id);
			} else {
				// TODO: Something else if post is fine
			}
		}
	}
}

fn get_pool_name(pool: &Path) -> String {
	let path = pool.to_str().unwrap();
	let path_split = path.split('\\');
	let split_collect = path_split.collect::<Vec<&str>>();
	let pool_name = split_collect.last().unwrap();
	pool_name.to_string()
}

// ---  get JSON for posts or pools ---
fn get_jsons(dir: &Path) -> Vec<PathBuf> {
	let mut jsons: Vec<PathBuf> = Vec::new();
	if dir.is_dir() {
		for entry in fs::read_dir(dir).unwrap() {
			let entry = entry.unwrap();
			let path = entry.path();
			if path.is_file() && path.extension().unwrap() == "json" {
				jsons.push(path);
			}
		}
	}
	jsons
}

// ---  read JSON for tags ---
fn read_json_tags(file: &Path) -> Vec<String> {
	let post = parse_json(file);
	let mut tags: Vec<String> = Vec::new();
	for category in post.tags {
		for tag in category {
			tags.push(tag);
		}
	}
	tags
}

fn parse_json(json: &Path) -> Post {
	let file = OpenOptions::new()
		.read(true)
		.write(false)
		.create(false)
		.append(false)
		.open(json)
		.unwrap();
	let post: Post = serde_json::from_reader(file).expect("JSON was not well-formatted");
	post
}

// ---  read blacklist ---
fn read_blacklist(file: &Path) -> Vec<String> {
	let mut blacklist: Vec<String> = Vec::new();
	let content = fs::read_to_string(&file).unwrap();
	for line in content.lines() {
		blacklist.push(line.to_string());
	}
	blacklist
}

// ---  search JSON for blacklist ---
fn hash_search(blacklist: Vec<String>, post_tags: HashSet<String>) -> bool {
	blacklist.iter().all(|tag| post_tags.contains(tag))
}
// ---  return result of search ---


