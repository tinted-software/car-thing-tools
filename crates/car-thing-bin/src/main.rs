use car_thing_lib::CarThings;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
	#[clap(subcommand)]
	command: SubCommand,
}

#[derive(Subcommand)]
enum SubCommand {
	/// Find car thing devices and print the device mode
	FindDevice,
}

fn main() -> Result<(), Box<dyn core::error::Error>> {
	let args = Cli::parse();

	match args.command {
		SubCommand::FindDevice => {
			let devices = rusb::devices()?;

			for car_thing in CarThings(devices.iter()) {
				println!(
					"Found car thing booted in {:?} mode",
					car_thing.mode()?
				);
			}
		}
	}

	Ok(())
}
