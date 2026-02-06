//! Interfaces to the serial ports exposed in x86 I/O port space.

use mrld::x86::io::*;
use spin;

/// Representing a serial port in the x86 I/O port address space. 
pub struct SerialPort<const PORT: u16> { 
    /// Has this port been initialized by the kernel? 
    initialized: bool,
}

// It seems like a serial port is implemented as a set of eight I/O ports. 
// "Luckily" for us, there is no substantial public AMD documentation on the 
// I/O port address space whatsoever. This 
impl <const PORT: u16> SerialPort<PORT> {
    /// RX/TX buffer
    const DATA:     IoPort = IoPort::new(PORT + 0);
    /// Interrupt enable
    const INTR_EN:  IoPort = IoPort::new(PORT + 1);

    /// Baud divisor (least-significant byte)
    const BAUD_LSB: IoPort = IoPort::new(PORT + 0);
    /// Baud divisor (most-significant byte)
    const BAUD_MSB: IoPort = IoPort::new(PORT + 1);

    /// Interrupt Identifier
    const INTR_ID:  IoPort = IoPort::new(PORT + 2);
    /// FIFO control
    const FIFO_CTL: IoPort = IoPort::new(PORT + 2);

    /// Line control
    const LINE_CTL: IoPort = IoPort::new(PORT + 3);
    /// Modem control
    const MODM_CTL: IoPort = IoPort::new(PORT + 4);
    /// Line status
    const LINE_STS: IoPort = IoPort::new(PORT + 5);
    /// Modem status
    const MODM_STS: IoPort = IoPort::new(PORT + 6);
    /// Scratch register
    const SCRATCH:  IoPort = IoPort::new(PORT + 7);

    const fn new() -> Self { 
        Self { initialized: false, }
    }
}

impl <const PORT: u16> SerialPort<PORT> {
    unsafe fn set_baud_divisor(&mut self, val: u16) {
        let lsb = (val & 0x00_ff) as u8;
        let msb = ((val & 0xff_00) >> 8) as u8;

        let line_ctl = Self::LINE_CTL.in8();
        Self::LINE_CTL.out8(line_ctl | 0b1000_0000);
        Self::BAUD_LSB.out8(lsb);
        Self::BAUD_MSB.out8(msb);
        Self::LINE_CTL.out8(line_ctl & 0b0111_1111);
    }

    unsafe fn disable_interrupts(&mut self) { 
        Self::INTR_EN.out8(0b0000_0000);
    }
}

impl <const PORT: u16> core::fmt::Write for SerialPort<PORT> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        unsafe { 
            self.send_bytes(s.as_bytes());
        }
        Ok(())
    }
}

impl <const PORT: u16> SerialPort<PORT> {
    pub fn is_initialized(&self) -> bool { 
        self.initialized
    }

    pub unsafe fn send_byte(&mut self, byte: u8) {
        // Wait for the TX buffer to drain
        while (Self::LINE_STS.in8() & 0b0010_0000) == 0 {}
        Self::DATA.out8(byte);
    }

    pub unsafe fn send_bytes(&mut self, bytes: &[u8]) {
        for byte in bytes { 
            self.send_byte(*byte);
        }
    }

    /// Initialize this serial port. 
    /// NOTE: Do you really *need* to do this after passing thru UEFI?
    pub unsafe fn init(&mut self) { 
        self.disable_interrupts();
        self.set_baud_divisor(0x0003);

        // 8 bits, no parity, one stop bit
        Self::LINE_CTL.out8(0b0000_0011);

        // IRQ disabled, RTS/DSR set
        Self::MODM_CTL.out8(0b0000_0011);

        // Clear RX/TX FIFO, disable FIFOs
        Self::FIFO_CTL.out8(0b0000_0110);

        // Loopback test
        // RTS, OUT1, OUT2 (IRQ enable), LOOP
        Self::MODM_CTL.out8(0b0001_1110);
        Self::DATA.out8(0x42);
        if Self::DATA.in8() == 0x42 { 
            // DTR, RTS, OUT1, OUT2 (IRQ enable)
            Self::MODM_CTL.out8(0b0000_1111);
            self.initialized = true;
        } 
        else { 
            self.initialized = false;
        }
    }

}

/// Serial port COM1
pub static COM1: spin::Mutex<SerialPort<0x3f8>> = {
    spin::Mutex::new(SerialPort::new())
};

