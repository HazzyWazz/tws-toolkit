use std::path::Path;
use std::process::{Command, Output};
use e621_api;
fn update() -> Result<Output, std::io::Error> {
	let update_cmd = "gallery-dl -U";

	let output: Result<Output, std::io::Error> = if cfg!(target_os = "windows") {
		Command::new("powershell")
			.args(["/C", update_cmd])
			.output()
	} else {
		Command::new("sh")
			.arg("-c")
			.arg(update_cmd)
			.output()
	};
	output
}

pub fn download(input_urls: &Path, output_path: &Path, cookies: &Path) -> Result<Output, std::io::Error> {
	update().expect("Failed to update. Is gallery-dl in your PATH?");

	let user_agent = e621_api::auth::USER_AGENT.to_string();
	let input_file = input_urls.to_str().unwrap();
	let ep_join = output_path.join("errored.txt");
	let errored_path = Path::new(&ep_join);
	let errored = errored_path.to_str().unwrap();
	let ap_join = output_path.join("archive");
	let archive_path = Path::new(&ap_join);
	let archive = archive_path.to_str().unwrap();
	let destination = output_path.to_str().unwrap();
	let cookie_file = cookies.to_str().unwrap();

	let download_command = format!("gallery-dl \
	--cookies \"{cookie_file}\" \
	--user-agent \"{user_agent}\" \
	-I \"{input_file}\" \
	--destination \"{destination}\" \
	-e \"{errored}\" \
	--sleep-request 1.0 \
	--sleep-429 10 \
	--write-metadata \
	--write-tags \
	--download-archive \"{archive}\"");

	let output: Result<Output, std::io::Error> = if cfg!(target_os = "windows") {
		Command::new("powershell")
			.args(["/C", &download_command])
			.output()
	} else {
		Command::new("sh")
			.arg("-c")
			.arg(download_command)
			.output()
	};
	output
}