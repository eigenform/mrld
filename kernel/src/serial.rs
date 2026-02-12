//! Interfaces to the serial ports exposed in x86 I/O port space.

use mrld::x86::io::*;
use spin;

/// Serial port COM2
/// NOTE: This may not be correct on hardware..?
pub static COM2: spin::Mutex<SerialPort<0x2f8>> = {
    spin::Mutex::new(SerialPort::new())
};

/// Representing a serial port in the x86 I/O port address space. 
pub struct SerialPort<const PORT: u16>;
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

    const fn new() -> Self { Self }
}

impl <const PORT: u16> SerialPort<PORT> {

    #[inline(always)]
    pub unsafe fn set_baud_divisor(&mut self, val: u16) {
        let lsb = (val & 0xff) as u8;
        let msb = ((val & 0xff00) >> 8) as u8;
        let line_ctl = Self::LINE_CTL.in8();
        Self::LINE_CTL.out8(line_ctl | 0b1000_0000);
        Self::BAUD_LSB.out8(lsb);
        Self::BAUD_MSB.out8(msb);
        Self::LINE_CTL.out8(line_ctl & 0b0111_1111);
    }

    #[inline(always)]
    pub unsafe fn disable_interrupts(&mut self) {
        Self::INTR_EN.out8(0b0000_0000);
    }

    /// Send a byte to this port
    pub unsafe fn send_byte(&mut self, byte: u8) {
        // Wait for the TX buffer to drain
        while (Self::LINE_STS.in8() & 0b0010_0000) == 0 {}
        Self::DATA.out8(byte);
    }

    /// Send a slice of bytes to this port
    pub unsafe fn send_bytes(&mut self, bytes: &[u8]) {
        for byte in bytes { 
            self.send_byte(*byte);
        }
    }

    /// Initialize this serial port. 
    pub unsafe fn init(&mut self) { 
        self.disable_interrupts();
        self.set_baud_divisor(1);

        // 8 bits, no parity, one stop bit
        Self::LINE_CTL.out8(0b0000_0011);

        // Clear/Enable FIFO, 14-byte threshold
        Self::FIFO_CTL.out8(0b1100_0111);

        // IRQs enabled, OUT#1/OUT#2 enabled
        Self::MODM_CTL.out8(0b0000_1111);
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


