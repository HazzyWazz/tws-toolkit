mod parser;
mod gallery;
mod local_worker;
mod online_worker;

use std::path::Path;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
	/// Path to TWS2 file
	#[arg(short, long)]
	input: String,

	/// Output folder
	#[arg(short, long)]
	output: String,

	/// Cookie file
	#[arg(long)]
	cookie: String,

	/// tws2tag Config File
	#[arg(long)]
	config: String,
}

#[tokio::main]
async fn main() {
	let args = Args::parse();
	let tws2_file = Path::new(&args.input);
	let output_path = Path::new(&args.output);
	let cookie_file = Path::new(&args.cookie);
	let config_file = Path::new(&args.config);

	let config = parser::read_conf(config_file);

	local_worker::entry(tws2_file, output_path, cookie_file, &config.username, &config.api_key).await;

}
