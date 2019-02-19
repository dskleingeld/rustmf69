//working off example (spi): https://github.com/japaric/mfrc522/blob/master/src/lib.rs
//another example: https://github.com/JohnDoneth/hd44780-driver/blob/master/examples/raspberrypi/src/main.rs
//#![no_std] //FIXME TODO remove all std lib dependencies
extern crate embedded_hal as hal;

#[macro_use]
extern crate bitflags;

use core::num::NonZeroU8;
use hal::blocking::spi;
use hal::digital::OutputPin;
use hal::spi::{Mode, Phase, Polarity};
use hal::blocking::delay::{DelayMs, DelayUs};

mod registers;
use registers::{Register, RF69_FSTEP, FXOSC};
mod builder;
pub use builder::{RadioBuilder,radio};

pub struct Radio<SPI, CS, DELAY> {
	spi: SPI,
	cs: CS,
	delay: DELAY,

	freq_band: FreqencyBand, //non optional
	freq: u32,
	bitrate: Bitrate,   //optional (default = smthg)
	power_level: u8, //optional (default, max)

	network_filtering: Option<NonZeroU8>,
	adress_filtering: AddressFiltering,

	mode: RadioMode,
	package_len: PackageLength,

	register_flags: RegisterFlags
}

//local copy of register flags to save register read operations
struct RegisterFlags {
	mode: registers::OpMode,
	sync: registers::SyncConfig,
	config1: registers::PacketConfig1,
}

impl Default for RegisterFlags {
	fn default() -> Self {
		Self {
			mode: registers::OpMode::Standby
		          & !registers::OpMode::Sequencer_Off
		          & !registers::OpMode::Listen_On,
			sync: registers::SyncConfig::On
			      | registers::SyncConfig::Fifofill_Auto
			      | registers::SyncConfig::Size_2
			      | registers::SyncConfig::Tol_0,
			config1: registers::PacketConfig1::Format_Variable
			      | registers::PacketConfig1::Dcfree_Off
			      | registers::PacketConfig1::Crc_On
			      | registers::PacketConfig1::Crcautoclear_On
			      | registers::PacketConfig1::Adrsfiltering_Off,
		}
	}
}

enum AddressFiltering {
	None,
	AddressOnly(u8),
	AddressOrBroadcast((u8,u8)), //(addr, broadcast_addr)
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
	Lowest,
	Low, 
	Standard,
	High,
	Custom(u32),
}

impl Default for Bitrate {
	fn default() -> Self {
		Bitrate::Standard
	}
}

#[allow(dead_code)]
#[derive(Debug,Clone)]
pub enum PackageLength {
	Fixed(u8), //in bytes
	Max(u8),
}

impl Default for PackageLength {
	fn default() -> Self {
		PackageLength::Fixed(16)
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

		self.set_package_filtering();
		self.set_bitrate();
		self.set_frequency();
		self.set_payload_length();
	}

	fn set_payload_length(&mut self){
		match self.package_len {
			PackageLength::Fixed(len) => {
				self.register_flags.config1 -= registers::PacketConfig1::Format_Variable;
				self.write_reg(Register::Payloadlength, len);
			},
			PackageLength::Max(len) => {
				self.register_flags.config1 |= registers::PacketConfig1::Format_Variable;
				self.write_reg(Register::Payloadlength, len);
			},
		}
		self.write_reg(Register::Packetconfig1, self.register_flags.config1.bits());
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
	fn set_package_filtering(&mut self) {
		use registers::SyncConfig;
		use registers::PacketConfig1;

		match self.network_filtering {
			None =>	{//switch to one sync word (second one is used as network id)
				self.register_flags.sync = (self.register_flags.sync - SyncConfig::Size) | SyncConfig::Size_1;
				self.write_reg(Register::Syncconfig, self.register_flags.sync.bits());
			},
			Some(network_id) => {
				self.register_flags.sync = (self.register_flags.sync - SyncConfig::Size) | SyncConfig::Size_2;
				self.write_reg(Register::Syncconfig, self.register_flags.sync.bits());
				self.write_reg(Register::Syncvalue2, network_id.get());
			},
		}

		self.register_flags.config1 -= PacketConfig1::Adrsfiltering;
		match self.adress_filtering {
			AddressFiltering::None => {
				self.register_flags.config1 |= PacketConfig1::Adrsfiltering_Off;
				self.write_reg(Register::Packetconfig1, self.register_flags.config1.bits());
			},
			AddressFiltering::AddressOnly(node_addr) => {
				self.register_flags.config1 |= PacketConfig1::Adrsfiltering_Node;
				self.write_reg(Register::Packetconfig1, self.register_flags.config1.bits());
				self.write_reg(Register::Nodeadrs, node_addr);
			},
			AddressFiltering::AddressOrBroadcast((node_addr,broadcast_addr)) => {
				self.register_flags.config1 |= PacketConfig1::Adrsfiltering_Nodebroadcast;
				self.write_reg(Register::Packetconfig1, self.register_flags.config1.bits());
				self.write_reg(Register::Nodeadrs, node_addr);
				self.write_reg(Register::Broadcastadrs, broadcast_addr);
			},
		}
	}



	fn set_bitrate(&mut self) {
		//bitrate reg value: F_xosc / bitrate (b/s)
		match self.bitrate {
			Bitrate::Lowest => {
				self.write_reg(Register::Bitratemsb, registers::Bitrate::Msb_1200.bits());
				self.write_reg(Register::Bitratelsb, registers::Bitrate::Lsb_1200.bits());
			},
			Bitrate::Low => {
				self.write_reg(Register::Bitratemsb, registers::Bitrate::Msb_55555.bits());
				self.write_reg(Register::Bitratelsb, registers::Bitrate::Lsb_55555.bits());
			},
			Bitrate::High => {
				self.write_reg(Register::Bitratemsb, registers::Bitrate::Msb_200kbps.bits());
				self.write_reg(Register::Bitratelsb, registers::Bitrate::Lsb_200kbps.bits());
			},
			Bitrate::Standard => {
				self.write_reg(Register::Bitratemsb, registers::Bitrate::Msb_100000.bits());
				self.write_reg(Register::Bitratelsb, registers::Bitrate::Lsb_100000.bits());
			},
			Bitrate::Custom(bitrate) => {
				let msb = (FXOSC/bitrate >> 8) as u8;
				let lsb = (FXOSC/bitrate) as u8;
				self.write_reg(Register::Bitratemsb, msb);
				self.write_reg(Register::Bitratelsb, lsb);
			},
		}
	}


	fn switch_freq(&mut self) -> Result<(),()> {
		let frf = (self.freq as f32 / RF69_FSTEP) as u32; // divide down by FSTEP to get FRF
		if self.mode == RadioMode::Tx {
			self.switch_transeiver_mode_blocking(RadioMode::Rx)?;
			self.write_reg(Register::Frfmsb, (frf >> 16) as u8);
			self.write_reg(Register::Frfmid, (frf >> 8) as u8);
			self.write_reg(Register::Frflsb, frf as u8);
			self.switch_transeiver_mode_blocking(RadioMode::Tx)?;
		} else {
			let old_mode = self.mode;
			self.write_reg(Register::Frfmsb, (frf >> 16) as u8);
			self.write_reg(Register::Frfmid, (frf >> 8) as u8);
			self.write_reg(Register::Frflsb, frf as u8);
			self.switch_transeiver_mode_blocking(RadioMode::FreqSynth)?;
			self.switch_transeiver_mode_blocking(old_mode)?;
		}
		Ok(())
	}

	//see page 38 in the datasheet,
	//TODO research Fdev and do that too
	fn set_frequency(&mut self) -> Result<(),()> {
	  if !self.register_flags.mode.contains(registers::OpMode::Sequencer_Off) {
	  	self.register_flags.mode |= registers::OpMode::Sequencer_Off;
	  	self.write_reg(Register::Opmode, self.register_flags.mode.bits());

	  	self.switch_freq()?;

	  	self.register_flags.mode -= registers::OpMode::Sequencer_Off;
	  	self.write_reg(Register::Opmode, self.register_flags.mode.bits());
	  } else {
	  	self.switch_freq()?;
	  }
		Ok(())
	}

	fn switch_transceiver_mode(&mut self, new_mode: RadioMode) {
		use registers::OpMode;

		let old_flag = self.register_flags.mode - OpMode::Mode;
		self.register_flags.mode = match new_mode {
			RadioMode::Sleep => old_flag | OpMode::Sleep, // Xtal Off
			RadioMode::Standby => old_flag | OpMode::Standby, // Xtal On
			RadioMode::FreqSynth => old_flag | OpMode::Synthesizer, // Pll On
			RadioMode::Rx => old_flag | OpMode::Receiver, // Rx Mode
			RadioMode::Tx => old_flag | OpMode::Transmitter, // Tx Mode
		};
		self.write_reg(Register::Opmode, self.register_flags.mode.bits());
		self.mode = new_mode;
	}

	fn switch_transeiver_mode_blocking(&mut self, new_mode: RadioMode) -> Result<(),()>{
		use registers::IrqFlags1;

		self.switch_transceiver_mode(new_mode);
		for _attempt in 0..10 {//try for one millisecond
			let interrupt_flag = IrqFlags1::from_bits(self.read_reg(Register::Irqflags1)).unwrap();
			if interrupt_flag.contains(IrqFlags1::Modeready){
				return Ok(())
			}
			self.delay.delay_us(100u16);
		}
		Err(())
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

