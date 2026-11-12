pub mod checker;

use std::path::Path;
use clap::Parser;

/// A tool to find posts and pools without specific tags extracted by tws2pool
/// or found by tws2tag
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct CmdArgs {
	/// Path to input folder
	#[arg(short, long)]
	input: String,

	/// Path to tag blacklist
	#[arg(short, long)]
	blacklist: String,

	/// Output file
	#[arg(short, long)]
	output: String,

	/// Pool mode
	#[arg(long, action=clap::ArgAction::SetFalse)]
	pool: Option<bool>,
}

fn main() {
	let args = CmdArgs::parse();
	let input = Path::new(&args.input);
	let blacklist = Path::new(&args.blacklist);
	let output = Path::new(&args.output);
	let pool_mode = args.pool.unwrap();

	checker::entry(input, blacklist, pool_mode)
}