//use: cargo run --example rpi
extern crate linux_embedded_hal as hal;
extern crate pi_rmf69;

use hal::spidev::SpidevOptions;
use hal::Spidev;
use hal::Pin;
use hal::sysfs_gpio::Direction;
use hal::Delay;

use std::{thread, time::Duration};

use pi_rmf69::{radio, FreqencyBand, Bitrate};

fn main() {
    let cs = Pin::new(8);
    cs.export().unwrap();
    cs.set_direction(Direction::Out).unwrap();

    let mut spi = Spidev::open("/dev/spidev0.1").unwrap();
    let options = SpidevOptions::new()
        .max_speed_hz(pi_rmf69::SPI_SPEED)
        .mode(pi_rmf69::SPI_MODE)
        .build();
		spi.configure(&options).unwrap();
		
		let mut radio = radio(spi, cs, Delay)
			.adress(0)
			.broadcast(2)
			.freqency_band(FreqencyBand::ISM433mhz)
			.fixed_package_length(16)
			.network_id(core::num::NonZeroU8::new(1).unwrap())
			.bitrate(Bitrate::Lowest)
			.power_level(31)
		.build();

		radio.init().unwrap();

		println!("radio init without problems!");

		loop {
			radio.send_blocking(5, &[1,2,3]).unwrap();
			thread::sleep(Duration::from_millis(10));
		}
}
