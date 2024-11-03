use rusb::{Device, Devices, GlobalContext};

const DEV_ID_VENDOR: u16 = 0x1B8E;
const DEV_ID_PRODUCT: u16 = 0xC003;

const NORMAL_ID_VENDOR: u16 = 0x18D1;
const NORMAL_ID_PRODUCT: u16 = 0x4E40;

#[derive(Debug, PartialEq, Eq)]
pub enum Mode {
	Normal,
	Development,
}

pub struct CarThing(pub Device<GlobalContext>);

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

pub struct CarThings<'a>(pub Devices<'a, GlobalContext>);

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
