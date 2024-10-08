use clap::{Parser, Subcommand};

const DEV_ID_VENDOR: u16 = 0x1B8E;
const DEV_ID_PRODUCT: u16 = 0xC003;

const NORMAL_ID_VENDOR: u16 = 0x18d1;
const NORMAL_ID_PRODUCT: u16 = 0x4e40;

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
		SubCommand::FindDevice => find_device(),
	}
}

fn find_device() -> Result<(), Box<dyn core::error::Error>> {
	for device in rusb::devices()?.iter() {
		let device_descriptor = device.device_descriptor()?;

		if device_descriptor.vendor_id() == DEV_ID_VENDOR
			&& device_descriptor.product_id() == DEV_ID_PRODUCT
		{
			println!("Found device booted in USB burn mode");
		} else if device_descriptor.vendor_id() == NORMAL_ID_VENDOR
			&& device_descriptor.product_id() == NORMAL_ID_PRODUCT
		{
			println!("Found device booted in normal mode");
		}
	}

	Ok(())
}
