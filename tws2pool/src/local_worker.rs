use std::collections::HashSet;
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;
use reqwest_middleware::ClientWithMiddleware;
use crate::parser::TwsPoolEntry;
use crate::parser::get_pools;
use crate::{gallery, online_worker, parser};
use e621_api;
use e621_api::pools;


pub async fn entry(tws_file: &Path, output_path: &Path, cookie_file: &Path, username: &str, api_key: &str) {

	let limiter = online_worker::create_global_limiter(500);
	let headers = online_worker::create_global_headers(e621_api::auth::USER_AGENT,
								Option::from(e621_api::auth::create_auth_token(username, api_key).as_str()));
	let client = online_worker::create_global_client(headers, limiter);


	let gf_path = output_path.join("gallery_file.txt");
	let mut gallery_file = OpenOptions::new()
		.read(true)
		.write(true)
		.append(true)
		.create(true)
		.open(gf_path.clone()).unwrap();

	let tws_pools = read_file(tws_file);
	let pools = get_pools(tws_pools);

	for pool in pools.clone() {
		parser::save_pool_id(gallery_file.try_clone().unwrap(), pool);
	}
	
	let gallery_output = gallery::download(&*gf_path.clone(), output_path, cookie_file).expect("Error while running gallery-dl");
	match gallery_output.status.code().unwrap() {
		0 => {
			println!("{}", String::from_utf8_lossy(&gallery_output.stdout));
		}
		_ => {
			println!("{}", String::from_utf8_lossy(&gallery_output.stderr));
			println!("{}", String::from_utf8_lossy(&gallery_output.stdout));
		}
	}

	verify_pools(output_path.to_str().unwrap(), pools, client).await;
}

// --- Read TWS export ---
fn read_file(tws_file: &Path) -> Vec<TwsPoolEntry> {
	let file = OpenOptions::new()
		.read(true)
		.write(false)
		.create(false)
		.append(false)
		.open(tws_file).unwrap();
	let entries: Vec<TwsPoolEntry> = serde_json::from_reader(file).unwrap();
	entries
}

// --- Verify pool integrity ---
async fn verify_pools(gdl_root: &str, all_pools: HashSet<i64>, client: ClientWithMiddleware) {
	let mut found_pools: HashSet<i64> = HashSet::new();
	let mut failed_pools = OpenOptions::new()
		.read(true)
		.write(true)
		.create(true)
		.append(true)
		.open(gdl_root.to_owned() + "failed_pools.txt")
		.unwrap();


	let gdl_path = gdl_root.to_owned() + "/e621/pool";
	read_pools(gdl_path.clone(), found_pools.clone());

	for pool in fs::read_dir(gdl_path.clone()).expect("Failed to read pools directory") {
		let entry = pool.unwrap();
		let path = entry.path();
		let pool_id = get_pool_id(&path);
		if path.is_dir() {
			parser::save_pool_metadata(pool_id, &path, client.clone()).await;
		}
	}

	check_missing_pools(all_pools, found_pools.clone(), failed_pools);

	for pool in fs::read_dir(gdl_path).expect("Failed to read pools directory") {
		let entry = pool.unwrap();
		let path = entry.path();
		let mut parsed_pool = e621_api::pools::Pool::new();
		if path.is_dir() {
			for files in fs::read_dir(path.clone()).expect("Failed to read pool files directory") {
				let file_entry = files.unwrap();
				let path = file_entry.path();
				if path.is_file() && path.file_name().unwrap().to_str().unwrap().contains("e621_pool_") {
					parsed_pool = serde_json::from_reader(File::open(path).unwrap()).unwrap();
				}
			}
		}
		match parsed_pool.category {
			pools::PoolCategory::collection | pools::PoolCategory::series => {
				let pool_files = parsed_pool.post_ids.len();
				check_for_missing_posts(&path, pool_files as i64);
			}
			_ => {
				panic!("Uninitialized pool!");
			}
		}
	}

}
// --- vpi2: read pool directory ---
fn read_pools(pools_path: String, mut found_pools: HashSet<i64>) {
	let pools = fs::read_dir(pools_path).expect("Failed to read pools directory");
	for entry in pools {
		let entry = entry.expect("Failed to read pool entry");
		let path = entry.path();
		let pool_id = get_pool_id(&path);
		found_pools.insert(pool_id);
	}
}

// --- vpi3: parse pool id from name ---
fn get_pool_id(pool_path: &Path) -> i64 {
	let path_vec: Vec<&str> = pool_path.to_str().unwrap().split('\\').collect();
	let pool_name = path_vec.last().unwrap().to_string();
	let pool_id: i64 = pool_name.split(' ')
		.collect::<Vec<&str>>()[0]
		.parse()
		.expect("Failed to parse pool name");
	pool_id
}

// --- vpi4: check if all pools has found pools ---
fn check_missing_pools(all_pools: HashSet<i64>, found_pools: HashSet<i64>, mut failed_pools: File) {
	for pool in all_pools {
		if !found_pools.contains(&pool) {
			let pool_url = format!("https://e621.net/pools/{pool}\n");
			failed_pools.write(pool.to_string().as_bytes()).expect("Error while writing to failed pool file.");
		}
	}
}

// --- vpi5: check if found pools are complete ---
fn check_for_missing_posts(pool_path: &Path, expected: i64) {
	let mut count = fs::read_dir(pool_path).expect("Failed to read pools directory").count();
	count = (count-1)/3;
	let id = get_pool_id(pool_path);
	match count {
		count if count < expected as usize => {
			println!("Pool {id} is missing posts!");
		},
		count if count == expected as usize => {},
		count if count > expected as usize => {
			println!("Pool {id} is has more posts than expected!");
		},
		_ => panic!("Invalid file structure!")
	}
}
