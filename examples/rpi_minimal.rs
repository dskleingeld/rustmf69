//use: cargo run --example rpi
extern crate linux_embedded_hal as hal;
extern crate pi_rmf69;

use hal::spidev::SpidevOptions;
use hal::Spidev;
use hal::Pin;
use hal::sysfs_gpio::Direction;
use hal::Delay;

use pi_rmf69::{radio, FreqencyBand};

fn main() {
    let cs = Pin::new(8);
    cs.export().unwrap();
    cs.set_direction(Direction::Out).unwrap();

    let mut spi = Spidev::open("/dev/spidev0.1").unwrap();
    let options = SpidevOptions::new()
        .max_speed_hz(pi_rmf69::SPI_SPEED)
        .mode(hal::spidev::SPI_MODE_0)
        .build();
		spi.configure(&options).unwrap();

		let mut radio = radio(spi, cs, Delay)
			.adress(0)
			.freqency_band(FreqencyBand::ISM433mhz)
			.build();
		radio.init().unwrap();

		println!("radio init without problems!");
		radio.send_blocking(5, &[1,2,3]).unwrap();
		println!("radio send done!");
}
