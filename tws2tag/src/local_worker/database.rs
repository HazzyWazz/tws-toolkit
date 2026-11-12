use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use sqlite;
use sqlite::State;

/// Creates a local database and returns its File.
/// 
/// The database will contain the tables "files", "mismatched", "fixed" and "deleted"
pub fn create_database() -> File {
	let database_file = OpenOptions::new()
		.read(true)
		.write(true)
		.create(true)
		.open("./database.sqlite")
		.unwrap();

	let conn = sqlite::open("./database.sqlite").expect("Could not open database file");

	conn.execute("CREATE TABLE IF NOT EXISTS files (id INTEGER UNIQUE NOT NULL PRIMARY KEY, hash VARCHAR(64) NOT NULL, file_name VARCHAR(64) NOT NULL)")
		.expect("Could not create table 'files'");
	conn.execute("CREATE TABLE IF NOT EXISTS mismatched (id INTEGER UNIQUE NOT NULL PRIMARY KEY, hash_from_json VARCHAR(64) NOT NULL, hash_from_file VARCHAR(64) NOT NULL)")
		.expect("Could not create table 'mismatched'");
	conn.execute("CREATE TABLE IF NOT EXISTS fixed (id INTEGER UNIQUE NOT NULL PRIMARY KEY)")
		.expect("Could not create table 'fixed'");
	conn.execute("CREATE TABLE IF NOT EXISTS deleted (id INTEGER UNIQUE NOT NULL PRIMARY KEY)")
		.expect("Could not create table 'deleted'");

	println!("Created database file");
	database_file
}

/// Adds an ID to the "fixed" table
pub fn add_download(id: usize) {
	let conn = sqlite::open("./database.sqlite").expect("Could not open database file");
	let query = format!("INSERT INTO fixed (id) VALUES ({})", id);
	conn.execute(&query).expect("Could not insert into fixed");
}

/// Adds an ID to the "deleted" table
pub fn add_deleted(id: usize) {
	let conn = sqlite::open("./database.sqlite").expect("Could not open database file");
	let query = format!("INSERT INTO deleted (id) VALUES ({})", id);
	conn.execute(&query).expect("Could not insert into deleted");
}

/// Fetches and returns the IDs in the "fixed" table
pub fn fetch_downloads() -> Vec<i64> {
	let mut downloads: Vec<i64> = Vec::new();
	let conn = sqlite::open("./database.sqlite").expect("Could not open database file");
	let query = "SELECT id FROM fixed".to_string();

	let mut statement = conn.prepare(query).expect("Could not get statement");

	while let Ok(State::Row) = statement.next() {
		let id = statement.read::<i64,_ >("id").unwrap();
		downloads.push(id);
	}
	
	downloads
}

/// Adds a file, its hash and name to the "files" table
/// This table is used along with mismatch to check file integrity
pub fn add_file(id: usize, hash: String, file_name: String) {
	let conn = sqlite::open("./database.sqlite").expect("Could not open database file");
	let query = format!("INSERT INTO files (id, hash, file_name) VALUES ({id}, \"{hash}\", \"{name}\") ON CONFLICT(id) DO UPDATE SET file_name=\"{name}\"", id=id, hash=hash, name=file_name);
	// println!("INSERT QUERY: {:?}", query);
	conn.execute(query).expect("Could not insert file");
	// println!("Inserted file {}", id);
}

/// Adds an ID and the differing hashes to the "mismatched" table
pub fn add_mismatch(id: usize, json_hash: String, file_hash: String) {
	let conn = sqlite::open("./database.sqlite").expect("Could not open database file");
	let query = format!("INSERT OR IGNORE INTO mismatched (id, hash_from_json, hash_from_file) VALUES ({}, \"{}\", \"{}\")", id, json_hash, file_hash);
	conn.execute(query).expect("Could not insert mismatch");
}

/// Returns the hash of a provided ID from the "files" table if it exists, otherwise returns "NOT_FOUND"
pub fn get_post(id: usize) -> String {
	let conn = sqlite::open("./database.sqlite").expect("Could not open database file");

	let query = format!("SELECT hash FROM files WHERE id = {}", id);
	let check_exists = format!("SELECT exists({}) AS row_exists", &query);

	let mut checker = conn.prepare(check_exists).expect("Could not check exists");
	let e: i64 = checker
		.iter()
		.last()
		.unwrap()
		.unwrap()
		.read(0);

	if e > 0 {
		let mut statement = conn.prepare(query).expect("Could not get post");
		let result = statement
			.iter()
			.last()
			.unwrap();
		let row = result.unwrap();
		let hash = String::from(row.read::<&str, usize>(0));
		if hash.len() > 0 {
			return hash;
		}
	}
	"NOT_FOUND".to_string()
}

/// Returns a hashmap of the IDs and their file names from the tables "files" and "mismatched"
pub fn get_mismatched() -> HashMap<i64, String> {
	let conn = sqlite::open("./database.sqlite").expect("Could not open database file");

	let mut mismatched: HashMap<i64, String> = HashMap::new();

	let query = "SELECT mismatched.id, files.file_name FROM files INNER JOIN mismatched ON files.id = mismatched.id";

	let mut statement = conn.prepare(query).expect("Could not get statement");

	while let Ok(State::Row) = statement.next() {
		let id = statement.read::<i64,_ >("id").unwrap();
		let file_name = statement.read::<String,_ >("file_name").unwrap();
		mismatched.insert(id, file_name);
	}

	println!("Fetched {} mismatched posts", mismatched.len());
	mismatched
}
