//use: cargo run --example rpi
extern crate linux_embedded_hal as hal;
extern crate pi_rmf69;

use hal::spidev::SpidevOptions;
use hal::Spidev;
use hal::Delay;

use pi_rmf69::{Radio, FreqencyBand, Bitrate};

fn main() {
    let mut spi = Spidev::open("/dev/spidev0.0").unwrap();
    let options = SpidevOptions::new()
        .max_speed_hz(500_000)
        .mode(hal::spidev::SPI_MODE_0)
        .build();
		spi.configure(&options).unwrap();
		
		let mut radio = Radio::new(spi, Delay).unwrap();
		radio.init(FreqencyBand::ISM433mhz, 0, 0, Bitrate::Standard);
}
