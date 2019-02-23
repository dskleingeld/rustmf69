
//indicates reading op. after binary OR with reg adress this gives the spi command for reading a reg
const READ_OP: u8 =  0b01111111;
//indicates writing op. after binary OR with reg adress this gives the spi command for writing a reg
const WRITE_OP: u8 = 0b10000000;
//to take advantage of the built in AES/CRC limit the packet size to FIFO size
pub const MAX_PACKET_SIZE: usize = 61; //(66 bytes - 3 bytes overhead - 2 bytes crc)


impl Register {
    pub fn read_address(&self) -> u8 {
        *self as u8 & READ_OP
    }

    pub fn write_address(&self) -> u8 {
        *self as u8 | WRITE_OP
    }
}

//list of good defaults for registers that should not change during operation.
pub const DEFAULT_RADIO_CONFIG: [(Register, u8); 13] =
[
	(Register::Datamodul, Datamodul::Datamode_Packet.bits | Datamodul::Modulationtype_Fsk.bits ),

	(Register::Rxbw, RxBw::Dccfreq_010.bits | RxBw::Mant_16.bits | RxBw::Exp_2.bits ),
  //only enable interrupts on Dio0
  (Register::Diomapping1, DioMapping1::Dio0_01.bits ),
  //disable other output pins to save power
  (Register::Diomapping2, DioMapping2::Clkout_Off.bits ),
  // Writing To This Bit Ensures That The Fifo & Status Flags Are Reset
  (Register::Irqflags2, IrqFlags2::Fifooverrun.bits ),

  (Register::Syncconfig, SyncConfig::On.bits | SyncConfig::Fifofill_Auto.bits |
	                             SyncConfig::Size_2.bits | SyncConfig::Tol_0.bits ),//TODO remove
  //Default Is 2 Bits For The Sync Value, Thus We Need To Set 2 Syncvalues
  (Register::Syncvalue1, 0x2d ), // Attempt To Make This Compatible With Sync1 Byte Of Rfm12b Lib
	//syncvalue2 (used as network id) is set in the filtering setup function

  (Register::Packetconfig1, PacketConfig1::Format_Variable.bits | PacketConfig1::Dcfree_Off.bits |
                                  PacketConfig1::Crc_On.bits | PacketConfig1::Crcautoclear_On.bits |
                                  PacketConfig1::Adrsfiltering_Off.bits ),//TODO remove

  (Register::Fifothresh, FifoThresh::Txstart_Fifonotempty.bits | FifoThresh::Value.bits ), // Tx On Fifo Not Empty

  // run DAGC continuously in RX mode for Fading Margin Improvement, recommended default for AfcLowBetaOn=0
  (Register::Testdagc, TestDagc::Improved_Lowbeta0.bits ),

	//Frequency Deviation setting,
	(Register::Fdevmsb, Fdev::Msb_50000.bits ),
	(Register::Fdevlsb, Fdev::Lsb_50000.bits ),

	(Register::Rssithresh, 220 ), // Must Be Set To Dbm = (-Sensitivity / 2), Default Is 0xe4 = 228 So -114dbm
];

pub const FXOSC: u32 =	32_000_000; //crystal osscilator frequency
pub const RF69_FSTEP: f32 = 61.03515625; // == FXOSC / 2^19 = 32MHz / 2^19 (p13 in datasheet)

//#define register extraction regex: #define (\w*)( *)(\w*)
//and list code: \t$1$2= $3,\n
#[allow(dead_code)]
#[derive(Copy, Clone)]
pub enum Register {
	Fifo          = 0x00,
	Opmode        = 0x01,
	Datamodul     = 0x02,
	Bitratemsb    = 0x03,
	Bitratelsb    = 0x04,
	Fdevmsb       = 0x05,
	Fdevlsb       = 0x06,
	Frfmsb        = 0x07,
	Frfmid        = 0x08,
	Frflsb        = 0x09,
	Osc1          = 0x0a,
	Afcctrl       = 0x0b,
	Lowbat        = 0x0c,
	Listen1       = 0x0d,
	Listen2       = 0x0e,
	Listen3       = 0x0f,
	Version       = 0x10,
	Palevel       = 0x11,
	Paramp        = 0x12,
	Ocp           = 0x13,
	Agcref        = 0x14,
	Agcthresh1    = 0x15,
	Agcthresh2    = 0x16,
	Agcthresh3    = 0x17,
	Lna           = 0x18,
	Rxbw          = 0x19,
	Afcbw         = 0x1a,
	Ookpeak       = 0x1b,
	Ookavg        = 0x1c,
	Ookfix        = 0x1d,
	Afcfei        = 0x1e,
	Afcmsb        = 0x1f,
	Afclsb        = 0x20,
	Feimsb        = 0x21,
	Feilsb        = 0x22,
	Rssiconfig    = 0x23,
	Rssivalue     = 0x24,
	Diomapping1   = 0x25,
	Diomapping2   = 0x26,
	Irqflags1     = 0x27,
	Irqflags2     = 0x28,
	Rssithresh    = 0x29,
	Rxtimeout1    = 0x2a,
	Rxtimeout2    = 0x2b,
	Preamblemsb   = 0x2c,
	Preamblelsb   = 0x2d,
	Syncconfig    = 0x2e,
	Syncvalue1    = 0x2f,
	Syncvalue2    = 0x30,
	Syncvalue3    = 0x31,
	Syncvalue4    = 0x32,
	Syncvalue5    = 0x33,
	Syncvalue6    = 0x34,
	Syncvalue7    = 0x35,
	Syncvalue8    = 0x36,
	Packetconfig1 = 0x37,
	Payloadlength = 0x38,
	Nodeadrs      = 0x39,
	Broadcastadrs = 0x3a,
	Automodes     = 0x3b,
	Fifothresh    = 0x3c,
	Packetconfig2 = 0x3d,
	Aeskey1       = 0x3e,
	Aeskey2       = 0x3f,
	Aeskey3       = 0x40,
	Aeskey4       = 0x41,
	Aeskey5       = 0x42,
	Aeskey6       = 0x43,
	Aeskey7       = 0x44,
	Aeskey8       = 0x45,
	Aeskey9       = 0x46,
	Aeskey10      = 0x47,
	Aeskey11      = 0x48,
	Aeskey12      = 0x49,
	Aeskey13      = 0x4a,
	Aeskey14      = 0x4b,
	Aeskey15      = 0x4c,
	Aeskey16      = 0x4d,
	Temp1         = 0x4e,
	Temp2         = 0x4f,
	Testlna       = 0x58,
	Testpa1       = 0x5a,
	Testpa2       = 0x5c,
	Testdagc      = 0x6f,
}

#[allow(dead_code)]
bitflags! {
pub struct OpMode: u8 {
	const Sequencer_Off = 0b10000000;
	//const Sequencer_On  = 0;

	const Listen_On     = 0b01000000;
	//const Listen_Off    = 0;

	const Listenabort   = 0b00100000;

	const Sleep         = 0b000;
	const Standby       = 0b001;
	const Synthesizer   = 0b010;
	const Transmitter   = 0b011;
	const Receiver      = 0b100;

	const Mode = Self::Sleep.bits | Self::Standby.bits | Self::Synthesizer.bits |
	             Self::Transmitter.bits| Self::Receiver.bits;
}
}

#[allow(dead_code)]
bitflags! {
struct Datamodul: u8 {
	const Datamode_Packet            = 0b00000000;
	const Datamode_Continuous        = 0b01000000;
	const Datamode_Continuousnobsync = 0b01100000;
	const Modulationtype_Fsk         = 0b00000000;
	const Modulationtype_Ook         = 0b00001000;
	const Modulationshaping_00       = 0b00000000;
	const Modulationshaping_01       = 0b10000001;
	const Modulationshaping_10       = 0b10000010;
	const Modulationshaping_11       = 0b10000011;
}
}

#[allow(dead_code)]
bitflags! {
pub struct Bitrate: u8 {
	const Msb_1200= 0x68;
	const Lsb_1200= 0x2b;
	const Msb_2400= 0x34;
	const Lsb_2400= 0x15;
	const Msb_4800= 0x1a;
	const Lsb_4800= 0x0b;
	const Msb_9600= 0x0d;
	const Lsb_9600= 0x05;
	const Msb_19200= 0x06;
	const Lsb_19200= 0x83;
	const Msb_38400= 0x03;
	const Lsb_38400= 0x41;
	const Msb_38323= 0x03;
	const Lsb_38323= 0x43;
	const Msb_34482= 0x03;
	const Lsb_34482= 0xa0;
	const Msb_76800= 0x01;
	const Lsb_76800= 0xa1;
	const Msb_153600= 0x00;
	const Lsb_153600= 0xd0;
	const Msb_57600= 0x02;
	const Lsb_57600= 0x2c;
	const Msb_115200= 0x01;
	const Lsb_115200= 0x16;
	const Msb_12500= 0x0a;
	const Lsb_12500= 0x00;
	const Msb_25000= 0x05;
	const Lsb_25000= 0x00;
	const Msb_50000= 0x02;
	const Lsb_50000= 0x80;
	const Msb_100000= 0x01;
	const Lsb_100000= 0x40;
	const Msb_150000= 0x00;
	const Lsb_150000= 0xd5;
	const Msb_200000= 0x00;
	const Lsb_200000= 0xa0;
	const Msb_250000= 0x00;
	const Lsb_250000= 0x80;
	const Msb_300000= 0x00;
	const Lsb_300000= 0x6b;
	const Msb_32768= 0x03;
	const Lsb_32768= 0xd1;
	// Custom Bit Rates
	const Msb_55555= 0x02;
	const Lsb_55555= 0x40;
	const Msb_200kbps= 0x00;
	const Lsb_200kbps= 0xa0;
}
}

#[allow(dead_code)]
bitflags! {
struct Fdev: u8 {
	const Msb_2000= 0x00;
	const Lsb_2000= 0x21;
	const Msb_5000= 0x00;
	const Lsb_5000= 0x52;
	const Msb_7500= 0x00;
	const Lsb_7500= 0x7b;
	const Msb_10000= 0x00;
	const Lsb_10000= 0xa4;
	const Msb_15000= 0x00;
	const Lsb_15000= 0xf6;
	const Msb_20000= 0x01;
	const Lsb_20000= 0x48;
	const Msb_25000= 0x01;
	const Lsb_25000= 0x9a;
	const Msb_30000= 0x01;
	const Lsb_30000= 0xec;
	const Msb_35000= 0x02;
	const Lsb_35000= 0x3d;
	const Msb_40000= 0x02;
	const Lsb_40000= 0x8f;
	const Msb_45000= 0x02;
	const Lsb_45000= 0xe1;
	const Msb_50000= 0x03;
	const Lsb_50000= 0x33;
	const Msb_55000= 0x03;
	const Lsb_55000= 0x85;
	const Msb_60000= 0x03;
	const Lsb_60000= 0xd7;
	const Msb_65000= 0x04;
	const Lsb_65000= 0x29;
	const Msb_70000= 0x04;
	const Lsb_70000= 0x7b;
	const Msb_75000= 0x04;
	const Lsb_75000= 0xcd;
	const Msb_80000= 0x05;
	const Lsb_80000= 0x1f;
	const Msb_85000= 0x05;
	const Lsb_85000= 0x71;
	const Msb_90000= 0x05;
	const Lsb_90000= 0xc3;
	const Msb_95000= 0x06;
	const Lsb_95000= 0x14;
	const Msb_100000= 0x06;
	const Lsb_100000= 0x66;
	const Msb_110000= 0x07;
	const Lsb_110000= 0x0a;
	const Msb_120000= 0x07;
	const Lsb_120000= 0xae;
	const Msb_130000= 0x08;
	const Lsb_130000= 0x52;
	const Msb_140000= 0x08;
	const Lsb_140000= 0xf6;
	const Msb_150000= 0x09;
	const Lsb_150000= 0x9a;
	const Msb_160000= 0x0a;
	const Lsb_160000= 0x3d;
	const Msb_170000= 0x0a;
	const Lsb_170000= 0xe1;
	const Msb_180000= 0x0b;
	const Lsb_180000= 0x85;
	const Msb_190000= 0x0c;
	const Lsb_190000= 0x29;
	const Msb_200000= 0x0c;
	const Lsb_200000= 0xcd;
	const Msb_210000= 0x0d;
	const Lsb_210000= 0x71;
	const Msb_220000= 0x0e;
	const Lsb_220000= 0x14;
	const Msb_230000= 0x0e;
	const Lsb_230000= 0xb8;
	const Msb_240000= 0x0f;
	const Lsb_240000= 0x5c;
	const Msb_250000= 0x10;
	const Lsb_250000= 0x00;
	const Msb_260000= 0x10;
	const Lsb_260000= 0xa4;
	const Msb_270000= 0x11;
	const Lsb_270000= 0x48;
	const Msb_280000= 0x11;
	const Lsb_280000= 0xec;
	const Msb_290000= 0x12;
	const Lsb_290000= 0x8f;
	const Msb_300000= 0x13;
	const Lsb_300000= 0x33;
}
}

#[allow(dead_code)]
bitflags! {
struct Frf: u8 {
	const Msb_314= 0x4E;
	const Mid_314= 0x80;
	const Lsb_314= 0x00;
	const Msb_315= 0x4E;
	const Mid_315= 0xC0;
	const Lsb_315= 0x00;
	const Msb_316= 0x4F;
	const Mid_316= 0x00;
	const Lsb_316= 0x00;
	const Msb_433= 0x6C;
	const Mid_433= 0x40;
	const Lsb_433= 0x00;
	const Msb_434= 0x6C;
	const Mid_434= 0x80;
	const Lsb_434= 0x00;
	const Msb_435= 0x6C;
	const Mid_435= 0xC0;
	const Lsb_435= 0x00;
	const Msb_863= 0xD7;
	const Mid_863= 0xC0;
	const Lsb_863= 0x00;
	const Msb_864= 0xD8;
	const Mid_864= 0x00;
	const Lsb_864= 0x00;
	const Msb_865= 0xD8;
	const Mid_865= 0x40;
	const Lsb_865= 0x00;
	const Msb_866= 0xD8;
	const Mid_866= 0x80;
	const Lsb_866= 0x00;
	const Msb_867= 0xD8;
	const Mid_867= 0xC0;
	const Lsb_867= 0x00;
	const Msb_868= 0xD9;
	const Mid_868= 0x00;
	const Lsb_868= 0x00;
	const Msb_869= 0xD9;
	const Mid_869= 0x40;
	const Lsb_869= 0x00;
	const Msb_870= 0xD9;
	const Mid_870= 0x80;
	const Lsb_870= 0x00;
	const Msb_902= 0xE1;
	const Mid_902= 0x80;
	const Lsb_902= 0x00;
	const Msb_903= 0xE1;
	const Mid_903= 0xC0;
	const Lsb_903= 0x00;
	const Msb_904= 0xE2;
	const Mid_904= 0x00;
	const Lsb_904= 0x00;
	const Msb_905= 0xE2;
	const Mid_905= 0x40;
	const Lsb_905= 0x00;
	const Msb_906= 0xE2;
	const Mid_906= 0x80;
	const Lsb_906= 0x00;
	const Msb_907= 0xE2;
	const Mid_907= 0xC0;
	const Lsb_907= 0x00;
	const Msb_908= 0xE3;
	const Mid_908= 0x00;
	const Lsb_908= 0x00;
	const Msb_909= 0xE3;
	const Mid_909= 0x40;
	const Lsb_909= 0x00;
	const Msb_910= 0xE3;
	const Mid_910= 0x80;
	const Lsb_910= 0x00;
	const Msb_911= 0xE3;
	const Mid_911= 0xC0;
	const Lsb_911= 0x00;
	const Msb_912= 0xE4;
	const Mid_912= 0x00;
	const Lsb_912= 0x00;
	const Msb_913= 0xE4;
	const Mid_913= 0x40;
	const Lsb_913= 0x00;
	const Msb_914= 0xE4;
	const Mid_914= 0x80;
	const Lsb_914= 0x00;
	const Msb_915= 0xE4;
	const Mid_915= 0xC0;
	const Lsb_915= 0x00;
	const Msb_916= 0xE5;
	const Mid_916= 0x00;
	const Lsb_916= 0x00;
	const Msb_917= 0xE5;
	const Mid_917= 0x40;
	const Lsb_917= 0x00;
	const Msb_918= 0xE5;
	const Mid_918= 0x80;
	const Lsb_918= 0x00;
	const Msb_919= 0xE5;
	const Mid_919= 0xC0;
	const Lsb_919= 0x00;
	const Msb_920= 0xE6;
	const Mid_920= 0x00;
	const Lsb_920= 0x00;
	const Msb_921= 0xE6;
	const Mid_921= 0x40;
	const Lsb_921= 0x00;
	const Msb_922= 0xE6;
	const Mid_922= 0x80;
	const Lsb_922= 0x00;
	const Msb_923= 0xE6;
	const Mid_923= 0xC0;
	const Lsb_923= 0x00;
	const Msb_924= 0xE7;
	const Mid_924= 0x00;
	const Lsb_924= 0x00;
	const Msb_925= 0xE7;
	const Mid_925= 0x40;
	const Lsb_925= 0x00;
	const Msb_926= 0xE7;
	const Mid_926= 0x80;
	const Lsb_926= 0x00;
	const Msb_927= 0xE7;
	const Mid_927= 0xC0;
	const Lsb_927= 0x00;
	const Msb_928= 0xE8;
	const Mid_928= 0x00;
	const Lsb_928= 0x00;
}
}

#[allow(dead_code)]
bitflags! {
struct RxBw: u8 {
	const Dccfreq_000 = 0x00;
	const Dccfreq_001 = 0x20;
	const Dccfreq_010 = 0x40;  // Recommended Default
	const Dccfreq_011 = 0x60;
	const Dccfreq_100 = 0x80;  // Reset Value
	const Dccfreq_101 = 0xa0;
	const Dccfreq_110 = 0xc0;
	const Dccfreq_111 = 0xe0;

	const Mant_16 = 0x00;  // Reset Value
	const Mant_20 = 0x08;
	const Mant_24 = 0x10;  // Recommended Default

	const Exp_0 = 0x00;
	const Exp_1 = 0x01;
	const Exp_2 = 0x02;
	const Exp_3 = 0x03;
	const Exp_4 = 0x04;
	const Exp_5 = 0x05;  // Recommended Default
	const Exp_6 = 0x06;  // Reset Value
	const Exp_7 = 0x07;
}
}

//see table 21 and 22 in datasheet
#[allow(dead_code)]
bitflags! {
pub struct DioMapping1: u8 {
	const Dio0_00 = 0x00;  // Default
	const Dio0_01 = 0x40;
	const Dio0_10 = 0x80;
	const Dio0_11 = 0xc0;

	const Dio1_00 = 0x00;  // Default
	const Dio1_01 = 0x10;
	const Dio1_10 = 0x20;
	const Dio1_11 = 0x30;

	const Dio2_00 = 0x00;  // Default
	const Dio2_01 = 0x04;
	const Dio2_10 = 0x08;
	const Dio2_11 = 0x0c;

	const Dio3_00 = 0x00;  // Default
	const Dio3_01 = 0x01;
	const Dio3_10 = 0x02;
	const Dio3_11 = 0x03;
}
}

//see table 21 and 22 in datasheet
#[allow(dead_code)]
bitflags! {
struct DioMapping2: u8 {
	const Dio4_00 = 0x00;  // Default
	const Dio4_01 = 0x40;
	const Dio4_10 = 0x80;
	const Dio4_11 = 0xc0;

	const Dio5_00 = 0x00;  // Default
	const Dio5_01 = 0x10;
	const Dio5_10 = 0x20;
	const Dio5_11 = 0x30;

	const Clkout_32 = 0x00;
	const Clkout_16 = 0x01;
	const Clkout_8 = 0x02;
	const Clkout_4 = 0x03;
	const Clkout_2 = 0x04;
	const Clkout_1 = 0x05;  // Reset Value
	const Clkout_Rc = 0x06;
	const Clkout_Off = 0x07;  // Recommended default
}
}

#[allow(dead_code)]
bitflags! {
pub struct IrqFlags1: u8 {
	const Modeready            = 0x80;
	const Rxready              = 0x40;
	const Txready              = 0x20;
	const Plllock              = 0x10;
	const Rssi                 = 0x08;
	const Timeout              = 0x04;
	const Automode             = 0x02;
	const SyncaddrESSMATCH     = 0x01;
}
}

#[allow(dead_code)]
bitflags! {
pub struct IrqFlags2: u8 {
	const Fifofull             = 0x80;
	const Fifonotempty         = 0x40;
	const Fifolevel            = 0x20;
	const Fifooverrun          = 0x10;
	const Packetsent           = 0x08;
	const Payloadready         = 0x04;
	const Crcok                = 0x02;
	const Lowbat               = 0x01;
}
}

#[allow(dead_code)]
bitflags! {
pub struct SyncConfig: u8 {
	const On                = 0x80;
	const Off               = 0x00;

	const Fifofill_Auto     = 0x00;
	const Fifofill_Manual   = 0x40;

	const Size_1            = 0x00;
	const Size_2            = 0x08;
	const Size_3            = 0x10;
	const Size_4            = 0x18;
	const Size_5            = 0x20;
	const Size_6            = 0x28;
	const Size_7            = 0x30;
	const Size_8            = 0x38;
	const Size = Self::Size_1.bits | Self::Size_2.bits | Self::Size_3.bits |
	             Self::Size_4.bits | Self::Size_5.bits | Self::Size_6.bits |
	             Self::Size_7.bits | Self::Size_8.bits;

	const Tol_0             = 0x00;
	const Tol_1             = 0x01;
	const Tol_2             = 0x02;
	const Tol_3             = 0x03;
	const Tol_4             = 0x04;
	const Tol_5             = 0x05;
	const Tol_6             = 0x06;
	const TOL_7             = 0x07;
}
}

#[allow(dead_code)]
bitflags! {
pub struct PacketConfig1: u8 {
	//const Format_Fixed       = 0x00; //not format variable
	const Format_Variable    = 0x80;

	const Dcfree_Off         = 0x00;
	const Dcfree_Manchester  = 0x20;
	const Dcfree_Whitening   = 0x40;

	const Crc_On             = 0x10;
	const Crc_Off            = 0x00;

	const Crcautoclear_On    = 0x00;
	const Crcautoclear_Off   = 0x08;

	const Adrsfiltering_Off            = 0x00; //not the others
	const Adrsfiltering_Node           = 0x02;
	const Adrsfiltering_Nodebroadcast  = 0x04;
	const Adrsfiltering = Self::Adrsfiltering_Off.bits | Self::Adrsfiltering_Node.bits |
	                      Self::Adrsfiltering_Nodebroadcast.bits;
}
}

#[allow(dead_code)]
bitflags! {
struct FifoThresh: u8 {
	const Txstart_Fifothresh   = 0x00;
	const Txstart_Fifonotempty = 0x80;
	const Value                = 0x0f;
}
}

#[allow(dead_code)]
bitflags! {
pub struct PacketConfig2: u8 {
	const Rxrestartdelay_1bit        = 0x00;
	const Rxrestartdelay_2bits       = 0x10;
	const Rxrestartdelay_4bits       = 0x20;
	const Rxrestartdelay_8bits       = 0x30;
	const Rxrestartdelay_16bits      = 0x40;
	const Rxrestartdelay_32bits      = 0x50;
	const Rxrestartdelay_64bits      = 0x60;
	const Rxrestartdelay_128bits     = 0x70;
	const Rxrestartdelay_256bits     = 0x80;
	const Rxrestartdelay_512bits     = 0x90;
	const Rxrestartdelay_1024bits    = 0xa0;
	const Rxrestartdelay_2048bits    = 0xb0;
	const Rxrestartdelay_None        = 0xc0;
	const Rxrestart                  = 0x04;

	const Autorxrestart_On           = 0x02;
	const Autorxrestart_Off          = 0x00;

	const Aes_On                     = 0x01;
}
}

#[allow(dead_code)]
bitflags! {
struct TestDagc: u8 {
	const Normal            = 0x00;
	const Improved_Lowbeta1 = 0x20;
	const Improved_Lowbeta0 = 0x30;
}
}

#[allow(dead_code)]
bitflags! {
pub struct PaLevel: u8 {
	const Pa0_On = 0x80;  // Default
	const Pa1_On = 0x40;
	const Pa2_On = 0x20;
	const Power = 0b11111;
}
}
