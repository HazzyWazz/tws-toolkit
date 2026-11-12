use crate::local_worker::definitions::Config;
use crate::online_worker;
use e621_api;
use reqwest_middleware::ClientWithMiddleware;
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use reqwest::StatusCode;
use sqlite::RowIndex;

mod definitions;
mod database;
pub mod downloader;

static USER_AGENT: &str = "tws2tag v0.2.0 | https://t.me/HazzyXn";

/// Creates a file that will contain all processed post IDs
/// One ID per line
///
/// # Examples
///
/// ```
/// let archive_path = let archive_path = Path::new("./archive.txt");
///
/// let archive_file = create_archive(archive_path);
/// ```
fn create_archive(file: PathBuf) -> File {
	if !file.exists() {
		println!("Creating archive {}", file.display());
		OpenOptions::new()
			.read(true)
			.write(true)
			.append(true)
			.create(true)
			.open(file).unwrap()
	} else {
		println!("Archive exists, opening...");
		OpenOptions::new()
			.read(true)
			.write(true)
			.append(true)
			.create(false)
			.open(file).unwrap()
	}
}

/// Adds an entry to an archive file
///
/// # Examples
///
/// ```
/// let archive_path = let archive_path = Path::new("./archive.txt");
/// let archive_file = create_archive(archive_path);
///
/// let id: usize = 2414962;
/// add_to_archive(&archive_file, id);
/// ```
fn add_to_archive(mut archive: &File, id: usize) {
	// println!("Adding {} to archive", id);
	let archive_line = id.to_string() + "\n";
	archive.write(archive_line.as_bytes()).expect("Unable to write to archive file!");
}

/// Checks whether an ID already exists within an archive file
/// and returns `true` if it does.
///
/// # Examples
///
/// ```
/// let archive_path = Path::new("./archive.txt");
/// let id: usize = 4941136;
///
/// let id_exists = check_archive(archive_path, id);
/// ```
fn check_archive(archive_path: &Path, id: usize) -> bool {
	let mut archive_contains: bool = false;

	let mut archive = File::open(archive_path).expect("Unable to open archive file!");
	let mut archive_buffer: String = String::new();
	archive.read_to_string(&mut archive_buffer).expect("Unable to read archive file!");

	let archive_vec: Vec<&str> = archive_buffer.split('\n').collect();

	if archive_vec.contains(&id.to_string().as_str()) {
		archive_contains = true;
	}

	archive_contains
}


/// Reads the configuration file into a `Config`
///
/// # Examples
///
/// ```
/// let config_path = Path::new("./config.conf");
///
/// let config = read_conf(&config_path);
/// ```
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

/// Walks and parses a directory. This should be a folder containing
/// files downloaded through the TWS app, and using the default download
/// naming format, which is `%artist%-%id%.%ext%`. The output folder
/// is used in a later function and is where the post's JSON data will
/// be stored. Despite being a few kilobytes in size, these files can
/// take up **UP TO 1 GIGABYTE TOTAL** depending on your filesystem's settings.
/// This should only happen if you have thousands of files to process.
///
/// # Examples
///
/// ```
/// let download_directory = Path::new("./Downloads/TWS") ;
/// let config_path = Path::new("./config.conf");
/// let archive_path = Path::new("./archive.txt");
/// let output_path = Path::new("./output");
///
/// parse_directory(download_directory, config_path, archive_path, output);
/// ```
pub async fn parse_directory(dir: &Path, config: &Path, archive_path: &Path, output: &Path) {

	let archive = create_archive(archive_path.to_path_buf());
	let files = get_files_from_directory(&dir);
	let total = files.len();
	let config = read_conf(config);

	let limiter = online_worker::create_global_limiter(config.booru_rate_limit_millis);
	let token = e621_api::auth::create_auth_token(&config.username, &config.api_key);
	let headers = online_worker::create_global_headers(USER_AGENT, Some(&token));
	let global_client = online_worker::create_global_client(headers, limiter);

	println!("Trying to authenticate...");
	let profile = e621_api::auth::get_profile(Option::from(token)).await;
	if profile.name == "Anonymous" {
		println!("Unauthenticated! Your metadata files might miss fields")
	} else {
		println!("Authenticated as: {}", profile.name);
	}

	println!("Found {} files", total);

	let mut counter: usize = 0;

	println!();
	for file in files {
		counter += 1;
		let file_hash = get_file_hash(&file);
		let id = get_id_from_path(&file, output);
		let percentage = (counter as f64 / total as f64) * 100.0;
		if id != 0 {
			print!("\r[{counter}/{total}] ({percentage:.4}%) Processing {id}");
			if !check_archive(&archive_path, id) {
				let metadata = fetch_post_metadata(&config, id, global_client.clone()).await;
				if metadata != "NOT_FOUND".to_string() {
					save_post_metadata(id, metadata, &file_hash, output, &config);
					add_to_archive(&archive, id);
				} else {
					println!("Post {id} not found!")
				}
			} else {
				println!("\nPost {id} is a duplicate! Ignoring...\n");
			}
		} else {
			println!("Post with id 0! This should not happen, check file names and get_id_from_path!");
		}
	}
	println!();
}

/// Walks a directory and returns a vector of each file's path
/// This function will only look at files which are not named "post_archive" and
/// are don't have a .json extension.
///
/// # Examples
///
/// ```
/// let download_directory = Path::new("./Downloads/TWS");
///
/// let files = get_files_from_directory(download_directory);
/// ```
fn get_files_from_directory(dir: &Path) -> Vec<PathBuf> {
	let mut files = Vec::new();

	println!("Reading files, this may take a while...");

	let entries = fs::read_dir(dir).expect("Cannot read directory");

	let mut counter: usize = 0;

	println!();
	for entry in entries {
		counter += 1;
		print!("\r[{}] Processing files...", counter);
		let entry = entry.expect("IO Error").path();
		if entry.is_file()
			&& !entry.file_name().unwrap().to_str().unwrap().contains("post_archive")
			&& !entry.file_name().unwrap().to_str().unwrap().contains("bad_files")
			&& entry.extension().unwrap() != "json"
		{
			files.push(entry);
		}
	}
	println!();
	files
}


/// Extracts the ID from a file's path.
/// This function should handle cases where an artist has dashes
/// or if the folder contains a duplicate either called artist-id-1 (or other number) and
/// artist-id (1) (or other number).
///
/// # Examples
///
/// ```
/// let download_directory = Path::new("./Downloads/TWS");
/// let output_path = Path::new("./output");
/// let files = get_files_from_directory(download_directory);
///
/// let mut ids: Vec<usize> = Vec::new();
///
/// for file in files {
/// 	let id = get_id_from_path(&file, output_path);
/// 	ids.push(id);
/// }
/// ```
fn get_id_from_path(file: &PathBuf, output: &Path) -> usize {
	let mut post_id: usize = 0;
	let file_name = file.file_name()
		.expect("Cannot read file name")
		.to_str()
		.expect("Cannot convert to string");

	if file_name.contains("-") {
		let file_name_split = file_name.split('-')
			.collect::<Vec<&str>>();

		post_id = check_file_name(file_name_split);

	} else {
		println!("\nFile name [{}] didn't match expected format!", file_name);
		// let offender = file_name;
		let list = output.join( Path::new("bad_files.txt"));
		if !list.exists() {
			println!("Creating non-matching file list {}", list.display());
			let mut list_file = OpenOptions::new()
				.read(true)
				.write(true)
				.append(true)
				.create(true)
				.open(list).unwrap();

			list_file.write((file_name.to_owned() + "\n").as_bytes()).expect("Unable to write to list file!");
		} else {
			println!("File lists exists, opening...");
			let mut list_file = OpenOptions::new()
				.read(true)
				.write(true)
				.append(true)
				.create(false)
				.open(list).unwrap();

			list_file.write((file_name.to_owned() + "\n").as_bytes()).expect("Unable to write to list file!");
		}
	}
	post_id
}

/// Checks if the file name is the expected format and extracts the post ID from it.
fn check_file_name(file_name_split: Vec<&str>) -> usize {
	let post_id: usize;
	if file_name_split.last().expect("Could not get last element of file name vector").len() < 6 {
		let file_name_len = file_name_split.len();
		// I am assuming all files are normal and don't use more than 4 characters for extension
		// this counts for webm, jpeg, etc.
		//
		// some-artist-id-1.exte
		// [some, artist, id, 1.exte]
		let last_index = file_name_len - 1; // 1.exte
		let id_index = last_index - 1;	// id
		post_id = file_name_split[id_index].to_string().parse::<usize>().unwrap();
	} else {
		let mut split_split = file_name_split.last()
			.expect("Cannot get file name")
			.split(".")
			.collect::<Vec<&str>>();


		if split_split[0].contains(&" ") {
			// println!("ss[0] contained space [{}]", split_split[0]);
			// this should handle filenames like artist-id (1).ext
			// first being split into id (1).ext and then here cut at " "
			// to leave id
			if split_split[0].contains(&"(") {
				split_split[0] = split_split[0].split(" ").collect::<Vec<&str>>()[0];
			} else {
				// and this if it's artist - id.ext
				split_split[0] = split_split[0].split(" ").collect::<Vec<&str>>()[1];
			}

		}
		post_id = split_split[0].parse::<usize>().expect(&format!("Cannot convert {id} to usize ({fname:?})", id=split_split[0], fname=split_split));
	}
	post_id
}

/// Fetches a post's JSON data from the booru.
/// It returns the JSON response's "post" field, which is then used in a later function
/// to be converted into a `e621_api::post::Post`
///
/// # Examples
///
/// ```
/// let config_path = Path::new("./config.conf");
/// let id: usize = 4984983;
/// let config = read_conf(config_path);
///
/// let metadata = fetch_post_metadata(config, id);
/// ```
async fn fetch_post_metadata(config: &Config, post_id: usize, global_client: ClientWithMiddleware) -> String {
	let post_url = config.booru_url.to_owned() + &post_id.to_string() + ".json?v2=true&mode=extended";
	let mut result: String = String::from("NOT_FOUND");

	let post_request = global_client.get(post_url).build().expect("Failed to build response");
	let post_response = global_client.execute(post_request).await.expect("Failed to get post metadata");

	match post_response.status() {
		StatusCode::OK => {
			let json = post_response.json::<serde_json::Value>()
				//.json::<definitions::Root>()
				.await
				.expect("Cannot convert to json");
			result = json.to_string();
		}
		StatusCode::NOT_MODIFIED => {
			panic!("Post metadata is not modified!");
		}
		_ => {
			println!("GET returned error! {}", post_response.status().as_str());
			println!("{:#?}", post_response.status());
		}
	}
	result
}

/// Saves the JSON metadata of a post. The file's name resembles gallery-dl's
/// in that it's booru_id_md5.json
///
/// # Examples
///
/// ```
/// let id: usize = 3402730;
/// let config_path = Path::new("./config.conf");
/// let config = read_conf(config_path);
/// let metadata = fetch_post_metadata(config, id);
/// let output_path = Path::new("./output");
///
/// // For the MD5 hash, you can either use `localworker::get_file_hash()` or
/// // or fetch it from the metadata. Each has their risk of being wrong
/// // according to the other.
///
/// save_post_metadata(id, metadata, /* md5 */, output, config);
/// ```
fn save_post_metadata(post_id: usize, metadata: String, md5: &String, dir: &Path, config: &Config) {
	let response_file_name = config.booru_name.to_owned() + "_" + &post_id.to_string() + "_" + md5 + ".json";
	let resp_output_path = dir.join(response_file_name);

	fs::write(resp_output_path, metadata).expect("Cannot write to file");
}

/// Gets the input file's MD5 hash and returns it as a String.
///
/// # Examples
///
/// ```
/// let input_file = Path::new("./Download/TWS/tateoftot-5758951.png");
/// let file_hash = get_file_hash(&input_file);
/// ```
fn get_file_hash(file: &PathBuf) -> String {
	let file_data = fs::read(file).expect("Cannot read file");
	let digest = md5::compute(file_data);
	format!("{:x}", digest)
}

/// Verification function used in verification mode. It goes through all files in a directory,
/// gets all JSON files from the output directory (see main()) and:
/// - Checks if the posts have already been checked by fetching it from a local SQLite database
/// - If not, calculates each file's hash and adds its ID, hash and file name to the database
/// - Loads and parses all JSON files into a `Vec<definitions::Root>`
/// - Checks if the file's hash matches what's in its metadata (which is matched by ID)
/// - If not, log it in the database
/// - Redownloads the file from the URL within the JSON file
/// - Verifies the file names of matching IDs with the freshly downloaded file
/// 	(note: this step will report mismatches if the extensions don't match)
///
/// # Examples
///
/// ```
/// let downloads = Path::new("./Download/TWS");
/// let responses = Path::new("./output");
/// let config_file = Path::new("./config.conf");
///
///	verify(downloads, responses, config_file)
/// ```
pub async fn verify(dir: &Path, json: &Path, conf: &Path) {
	let files = get_files_from_directory(&dir);
	let jsons = get_jsons(&json);
	database::create_database();

	// let mut valid_posts: HashMap<usize, String> = HashMap::new();

	let mut ver_counter = 0;

	// get file ids and their hashes
	println!();
	for file in files {
		ver_counter += 1;
		let id = get_id_from_path_verify(&file);
		let file_name = file.file_name().unwrap().to_str().unwrap();
		if database::get_post(id) == "NOT_FOUND" {
			print!("\r[{ver_counter}] Verifying file {id}");
			let file_hash = get_file_hash(&file);
			database::add_file(id, file_hash, String::from(file_name));
		}
	}

	println!();

	let mut parsed_jsons: Vec<e621_api::posts::Post> = Vec::new();

	let mut json_counter = 0;

	// get previously fetched JSON responses
	for response in jsons {
		if response.extension().unwrap() == "json" {
			json_counter += 1;
			print!("\r[{}] Parsing {}", json_counter, response.file_name().unwrap().display());
			let file = OpenOptions::new()
				.read(true)
				.write(false)
				.append(false)
				.create(false)
				.open(&response)
				.expect("Could not open JSON file");

			let parsed: e621_api::posts::Post = serde_json::from_reader(file).expect("Cannot parse JSON file");
			parsed_jsons.push(parsed);
		}
	}
	println!();

	let mut counter = 0;
	let total_jsons = parsed_jsons.len();

	let mut mismatch = 0;

	println!("Found {} JSON files", total_jsons);

	for response in parsed_jsons.clone() {
		counter += 1;
		let current_id = response.id;
		let current_hash = response.files.meta.md5;
		let file_post_hash = database::get_post(current_id as usize);
		print!("\r[{counter}/{total_jsons}]Processing {current_id}");

		if file_post_hash != current_hash {
			mismatch += 1;
			database::add_mismatch(current_id as usize, current_hash, file_post_hash);
		}
	}
	println!();
	println!("Found {} MD5 mismatches", mismatch);

	let mismatched_posts = database::get_mismatched();
	let mut mismatched_jsons: Vec<e621_api::posts::Post> = Vec::new();
	for mp in mismatched_posts.clone() {
		for json in parsed_jsons.clone() {
			if json.id == mp.0 {
				mismatched_jsons.push(json);
			}
		}
	}

	let config = read_conf(&conf);

	let limiter = online_worker::create_global_limiter(config.booru_rate_limit_millis);
	let token = e621_api::auth::create_auth_token(&config.username, &config.api_key);
	let headers = online_worker::create_global_headers(USER_AGENT, Some(&token));
	let global_client = online_worker::create_global_client(headers, limiter);

	downloader::fix_mismatch(mismatched_jsons, conf, json, global_client).await;

	// verify file names
	// let fixed = fs::read_dir("./output").expect("Cannot read output dir");
	let fixed_path = json.join("fixed");
	let fixed = fs::read_dir(fixed_path).expect("Cannot read output dir");
	for entry in fixed {
		let file = entry.expect("Cannot get entry").path();
		if file.is_file()
			&& file.extension().unwrap() != "json"
			&& !file.file_name().unwrap().to_str().unwrap().contains("post_archive")
			&& !file.file_name().unwrap().to_str().unwrap().contains("bad_files")
			&& !file.file_name().unwrap().to_str().unwrap().contains("(")
		{
			let file_name = file.file_name().expect("Couldn't get file name").to_str().unwrap();
			let id = file_name
				.split('-')
				.collect::<Vec<&str>>()
				.last()
				.unwrap()
				.to_owned()
				.split('.')
				.collect::<Vec<&str>>()[0]
				.parse::<i64>()
				.expect(&format!("Cannot parse id of file {} | Split was: {}", file_name, file_name.split('-').collect::<Vec<&str>>().last().unwrap()));

			for post in mismatched_posts.clone() {
				let pid = post.0;
				let pname = post.1;

				if id == pid {
					if file_name != pname {
						println!("Downloaded file {} does not match with post {} (ID: {})", file_name, pname, id);
					}
				}
			}
		}
	}
}

/// Helper function for `verify()` because I was dumb
/// Basically the same as get_id_from_path except it doesn't output the file name to bad_files.txt
fn get_id_from_path_verify(file: &PathBuf) -> usize {
	let post_id: usize;
	let file_name = file.file_name()
		.expect("Cannot read file name")
		.to_str()
		.expect("Cannot convert to string");

	if file_name.contains("-") {
		let file_name_split = file_name.split('-')
			.collect::<Vec<&str>>();
		post_id = check_file_name(file_name_split);
	} else {
		println!("File name [{}] didn't match expected format! Ignoring...", file_name);
		post_id = 0;
	}
	post_id
}

/// Helper function for `verify()` because I was dumb
/// Similar to `get_files_from_directory()`, but it only checks for
/// .json files.
fn get_jsons(dir: &Path) -> Vec<PathBuf> {
	let mut files = Vec::new();

	println!("Reading files, this may take a while...");

	let mut counter = 0;

	let entries = fs::read_dir(dir).expect("Cannot read directory");
	println!();
	for entry in entries {
		counter += 1;
		print!("\r[{}] Processing files...", counter);
		let entry = entry.expect("IO Error").path();
		if entry.is_file() && entry.extension().unwrap() == "json" {
			files.push(entry);
		}
	}
	println!();
	files
}
