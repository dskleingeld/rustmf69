//working off example (spi): https://github.com/japaric/mfrc522/blob/master/src/lib.rs
//another example: https://github.com/JohnDoneth/hd44780-driver/blob/master/examples/raspberrypi/src/main.rs
//#![no_std] //FIXME TODO remove all std lib dependencies
extern crate embedded_hal as hal;

#[macro_use]
extern crate bitflags;

use hal::blocking::spi;
use hal::digital::OutputPin;
use hal::spi::{Mode, Phase, Polarity};
use hal::blocking::delay::{DelayMs, DelayUs};

mod registers;
use registers::{Register, RF69_FSTEP};
mod builder;
pub use builder::{RadioBuilder,radio};

pub struct Radio<SPI, CS, DELAY> {
	spi: SPI,
	cs: CS,
	delay: DELAY,

	freq_band: FreqencyBand, //non optional
	node_id: u8,   //non optional
	network_id: u8,//optional (default = 0)
	bitrate: Bitrate,   //optional (default = smthg)
	power_level: u8, //optional (default, max)

	mode: RadioMode,
}


#[allow(dead_code)]
#[derive(Debug, PartialEq, Clone, Copy)]
enum RadioMode { //rename transeiver?
	Sleep = 0, // Xtal Off
	Standby = 1, // Xtal On
	FreqSynth = 2, // Pll On
	Rx = 3, // Rx Mode
	Tx = 4, // Tx Mode
}

impl Default for RadioMode {
	fn default() -> Self {
		RadioMode::Standby
	}
}

#[allow(dead_code)]
#[derive(Debug,Clone)]
pub enum Bitrate { 
	Low, 
	High,
	Standard, 
}

impl Default for Bitrate {
	fn default() -> Self {
		Bitrate::Standard
	}
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum FreqencyBand {
	ISM315mhz,
	ISM433mhz,
	ISM868mhz,
	ISM915mhz,
}


impl<SPI,CS, D, E> Radio<SPI, CS, D>
where SPI: spi::Transfer<u8, Error = E> + spi::Write<u8, Error = E>,
      D: DelayMs<u16>+DelayUs<u16>,
      CS: OutputPin,
      E : std::fmt::Debug {

	pub fn init(&mut self) -> Result<(),()> {
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
		self.configure_radio();

		Ok(())
	}

	pub fn configure_radio(&mut self){

		self.set_node_id();
		self.set_network_id();
		self.set_rssi_threashold();
		self.set_bitrate();
		self.set_frequency(40);
	}



	// pub fn send(uint8_t toAddress, const void* buffer, uint8_t bufferSize, bool requestACK=false) {
	
	
	// }
/*
		[Register::Syncvalue2 as u8, 1 ],    // Will Be Replaced With Network Id

		//Frequency Deviation setting,
		[Register::Fdevmsb as u8, Fdev::Msb_50000.bits ],
		[Register::Fdevlsb as u8, Fdev::Lsb_50000.bits ],

		[Register::Rssithresh as u8, 220 ], // Must Be Set To Dbm = (-Sensitivity / 2), Default Is 0xe4 = 228 So -114dbm
		[Register::Payloadlength as u8, 66 ], // In Variable Length Mode: The Max Frame Size, Not Used In Tx
		[Register::Nodeadrs as u8, 0 ], //  Address Filtering

		//Bit Rate setting
		[Register::Bitratemsb as u8, Bitrate::Msb_55555.bits ],
		[Register::Bitratelsb as u8, Bitrate::Lsb_55555.bits ],
*/
	fn set_node_id(&self) {
		//TODO
	}

	fn set_rssi_threashold(&self) {
		//TODO

	}

	fn set_network_id(&self) {
		//TODO

	}

	fn set_bitrate(&self) {
		//TODO

	}

	//see page 38 in the datasheet,
	//TODO research Fdev and do that too
	fn set_frequency(&mut self, target_freqency: u32){
		//TODO get freq from band or manually set freq (add funct to builder for this)
		let freqHz = (target_freqency as f32 / RF69_FSTEP) as u32; // divide down by FSTEP to get FRF
	  //TODO disable automatic seqencer if enabled
	  if self.mode == RadioMode::Tx {
	  	//TODO switch to Rx mode
			self.write_reg(Register::Frfmsb, (freqHz >> 16) as u8);
			self.write_reg(Register::Frfmid, (freqHz >> 8) as u8);
			self.write_reg(Register::Frflsb, freqHz as u8);
			//TODO switch back to Tx mode
		} else {
			let old_mode = self.mode;
			self.write_reg(Register::Frfmsb, (freqHz >> 16) as u8);
			self.write_reg(Register::Frfmid, (freqHz >> 8) as u8);
			self.write_reg(Register::Frflsb, freqHz as u8);
			//TODO switch to FreqSynth mode
			//TODO switch back to old mode
		}
	}

	fn switch_transceiver_mode(&mut self, new_mode: RadioMode) {
		use registers::OpMode;

		let old_bitflag = OpMode::from_bits(self.read_reg(Register::Opmode)).unwrap() - OpMode::Mode;
		let new_bitflag = match (new_mode) {
			RadioMode::Sleep => old_bitflag | OpMode::Sleep, // Xtal Off
			RadioMode::Standby => old_bitflag | OpMode::Sleep, // Xtal On
			RadioMode::FreqSynth => old_bitflag | OpMode::Sleep, // Pll On
			RadioMode::Rx => old_bitflag | OpMode::Sleep, // Rx Mode
			RadioMode::Tx => old_bitflag | OpMode::Sleep, // Tx Mode
		};
		self.write_reg(Register::Opmode, new_bitflag.bits());
		self.mode = new_mode;
	}

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

