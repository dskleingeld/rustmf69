#[macro_use]
extern crate bitflags;

//working off example (spi): https://github.com/japaric/mfrc522/blob/master/src/lib.rs
//another example: https://github.com/JohnDoneth/hd44780-driver/blob/master/examples/raspberrypi/src/main.rs
//#![no_std] //FIXME TODO remove all std lib dependencies
extern crate embedded_hal as hal;
use hal::blocking::spi;
use hal::digital::OutputPin;
use hal::spi::{Mode, Phase, Polarity};
use hal::blocking::delay::{DelayMs, DelayUs};

mod registers;
use registers::Register;

pub struct Radio<SPI, CS, D> {
	spi: SPI,
	cs: CS,
	delay: D,
	mode: RadioMode,
	power_level: u8, //TODO restrict max-min
}

enum RadioMode {
	Sleep = 0, // Xtal Off
	Standby = 1, // Xtal On
	Synth = 2, // Pll On
	Rx = 3, // Rx Mode
	Tx = 4, // Tx Mode
}

#[allow(dead_code)]
pub enum Bitrate { 
	Low, 
	High,
	Standard, 
}

#[allow(dead_code)]
pub enum FreqencyBand {
	ISM315mhz = 31, // Non Trivial Values To Avoid Misconfiguration
	ISM433mhz = 43,
	ISM868mhz = 86,
	ISM915mhz = 91,
}

impl<SPI,CS, D, E> Radio<SPI, CS, D>
where SPI: spi::Transfer<u8, Error = E> + spi::Write<u8, Error = E>,
      D: DelayMs<u16>+DelayUs<u16>,
      CS: OutputPin,
      E : std::fmt::Debug {

  pub fn new(spi: SPI, cs: CS, delay: D) ->	Result<Self, ()> {
    Ok(Radio { spi, cs, delay, mode: RadioMode::Rx, power_level: 31 })
  }

	pub fn init(&mut self, freq_band: FreqencyBand, node_id: u8, network_id: u8, speed: Bitrate) -> Result<(),()> {

		//self.cs.set_high();

		//check if the radio responds by seeing if we can change a register
		let mut synced = false;
		for _attempt in 0..100 {
			self.write_reg(Register::Syncvalue1, 0xAA); //170
			self.delay.delay_ms(1);
			if self.read_reg(Register::Syncvalue1) == 0xAA {
				synced = true;
				break;
			}
		}
		if !synced {return Err(())}

		synced =	false;
		for _attempt in 0..100 {
			self.write_reg(Register::Syncvalue1, 0x55); //85
			self.delay.delay_ms(1);
			if self.read_reg(Register::Syncvalue1) == 0x55 {
				synced = true;
				break;
			}
		}
		if !synced {return Err(())}
		
		//configure the radio chips for normal use
		self.configure_radio(freq_band, speed);

		Ok(())
	}

	pub fn configure_radio(&mut self, freq_band: FreqencyBand, speed: Bitrate){


	}


	// pub fn send(uint8_t toAddress, const void* buffer, uint8_t bufferSize, bool requestACK=false) {
	
	
	// }

	fn write_reg(&mut self, addr: Register, value: u8) {
		let to_write: [u8; 2] = [addr.write_address(), value];

		//self.cs.set_low();
		self.spi.write(&to_write).unwrap();
		//self.cs.set_high();
		self.delay.delay_us(15u16);
	}

	fn read_reg(&mut self, addr: Register) -> u8{
		let mut to_transfer: [u8; 2] = [addr.read_address(), 0];

		//self.cs.set_low();
		let to_transfer = self.spi.transfer(&mut to_transfer).unwrap();
		//self.cs.set_high();
		self.delay.delay_us(15u16);

		let awnser = to_transfer[1];
		awnser
	}

}
