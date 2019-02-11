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


//indicates reading op. after binary OR with reg adress this gives the spi command for reading a reg
const READ_OP: u8 = 0b1111111;
//indicates writing op. after binary OR with reg adress this gives the spi command for writing a reg
const WRITE_OP: u8 = 0b10000000;


impl Register {
    pub fn read_address(&self) -> u8 {
        *self as u8 & 0x7F
    }

    pub fn write_address(&self) -> u8 {
        *self as u8 | 0x80
    }
}
