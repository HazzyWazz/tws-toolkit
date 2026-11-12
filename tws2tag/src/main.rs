mod local_worker;
pub mod online_worker;

use std::path::Path;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
	/// Directory to operate on
	#[arg(short, long)]
	directory: String,

	/// Configuration file containing booru data
	#[arg(short, long)]
	config: String,
	
	/// Optional output directory for JSON data
	#[arg(short, long)]
	output: String,
	
	/// Set tool to verification mode
	#[arg(short, long, action)]
	verify: bool,
}


#[tokio::main]
async fn main() {

	let args = Args::parse();
	let directory = Path::new(&args.directory);
	let conf = Path::new(&args.config);
	
	let json_out = Path::new(&args.output);

	let e = json_out.to_str().unwrap().to_owned() + "\\post_archive.txt";
	let archive_path = Path::new(&e);

	let verify_mode = args.verify;


	
	if !verify_mode {
		local_worker::parse_directory(directory, conf, &archive_path, json_out).await;
	} else {
		println!("Launched in verification mode...");
		local_worker::verify(directory, json_out, conf).await;
	}

}
