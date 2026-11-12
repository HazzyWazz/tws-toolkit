use reqwest_middleware::{ClientWithMiddleware};
use std::fs::{OpenOptions};
use std::io::Write;
use std::path::Path;
use crate::local_worker::database::{add_deleted, add_download, fetch_downloads};
use crate::local_worker::{read_conf};
use e621_api::posts::Post;

// This module is specific to TWS's default save format
// that being artist-id.ext, making this customizable
// is in The Plans For The Future:tm: (likely never, sorry)

/// Tries to fix the files with mismatched hashes by redownloading them. 
pub async fn fix_mismatch(jsons: Vec<Post>, conf: &Path, output: &Path, client: ClientWithMiddleware) {
	let downloaded = fetch_downloads();
	let total = jsons.clone().len();
	let mut download_counter = 1;
	for json in jsons.clone() {
		let tags = json.tags;
		let id = json.id;
		let extension = json.files.meta.ext;
		let url = json.files.original.url;
		let artists = tags.artist;
		
		if url.is_some() && !downloaded.contains(&id) {
			let artists_string = artists.join("_");
			let output_file_path = output.join(format!("fixed/{artists_string}-{id}.{extension}"));
			//println!("{}", output_file_path.display());


			download(url.unwrap(), &output_file_path, client.clone()).await;
			add_download(id as usize);
			println!("[{download_counter}/{total}] Successfully downloaded file");

		} else {
			if downloaded.contains(&id) {
				println!("Post found in database");
			} else {
				if json.flags.deleted {
					println!("Post {} was deleted!", id);
					add_deleted(id as usize);
				} else {
					println!("Post {} has no URL but was not deleted! Failed login on fetch? Retrying...", id);
	
					let config = read_conf(conf);

					// let auth_key = e621_api::auth::create_auth_token(&config.username.clone(), &config.api_key.clone());

					let post = e621_api::posts::get_post(id, client.clone()).await;
					let json_url = post.files.original.url.clone();
	
					if json_url.is_some() {
						let artists_string = artists.join("_");
						// let output_file_name = format!("./output/{artists_string}-{id}.{extension}");
						let output_file_path = output.join(format!("fixed/{artists_string}-{id}.{extension}"));
						// let fixed_file = OpenOptions::new()
						// 	.read(true)
						// 	.write(true)
						// 	.create(true)
						// 	.open(output_file_name)
						// 	.expect("Could not open file");
	
						download(json_url.unwrap(), &output_file_path, client.clone()).await;
	
						println!("Successfully downloaded file on retry");
					} else {
						println!("URL Missing after retry. Is the API correct?");
					}
					
				}
			}
		}
		download_counter += 1;
	}
}

/// Downloads a file from the input URL to the output File
async fn download(url: String, output: &Path, client: ClientWithMiddleware) {
	let file_request = client.get(url).build().expect("Failed to build response");
	let file_response = client.execute(file_request).await.expect("Failed to get file");
	let content = file_response.bytes().await.expect("Failed to get bytes");

	println!("dl | got path: {:?}", output);
	
	let mut fixed_file = OpenOptions::new()
		.read(true)
		.write(true)
		.create_new(true)
		.open(output)
		.expect(&format!("Could not open file {}", output.to_str().unwrap()));

	fixed_file.write_all(&*content).expect("Error while trying to write to file");
}
