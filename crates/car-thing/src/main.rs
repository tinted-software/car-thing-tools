use clap::{Parser, Subcommand};
use rusb::{Device, Devices, GlobalContext};

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

const DEV_ID_VENDOR: u16 = 0x1B8E;
const DEV_ID_PRODUCT: u16 = 0xC003;

const NORMAL_ID_VENDOR: u16 = 0x18D1;
const NORMAL_ID_PRODUCT: u16 = 0x4E40;

#[derive(Debug, PartialEq, Eq)]
pub enum Mode {
	Normal,
	Development,
}

pub struct CarThing(Device<GlobalContext>);

impl CarThing {
	pub fn mode(&self) -> Result<Mode, rusb::Error> {
		let descriptor = self.0.device_descriptor()?;

		if descriptor.vendor_id() == DEV_ID_VENDOR
			&& descriptor.product_id() == DEV_ID_PRODUCT
		{
			Ok(Mode::Development)
		} else if descriptor.vendor_id() == NORMAL_ID_VENDOR
			&& descriptor.product_id() == NORMAL_ID_PRODUCT
		{
			Ok(Mode::Normal)
		} else {
			Err(rusb::Error::NotSupported)
		}
	}
}

pub struct CarThings<'a>(Devices<'a, GlobalContext>);

impl<'a> Iterator for CarThings<'a> {
	type Item = CarThing;

	fn next(&mut self) -> Option<Self::Item> {
		loop {
			match self.0.next() {
				Some(device) => match device.device_descriptor() {
					Ok(descriptor) => {
						if (descriptor.vendor_id() == DEV_ID_VENDOR
							&& descriptor.product_id() == DEV_ID_PRODUCT)
							|| (descriptor.vendor_id() == NORMAL_ID_VENDOR
								&& descriptor.product_id() == NORMAL_ID_PRODUCT)
						{
							return Some(CarThing(device));
						} else {
							continue;
						}
					}
					Err(_error) => {
						return None;
					}
				},
				None => return None,
			}
		}
	}
}
