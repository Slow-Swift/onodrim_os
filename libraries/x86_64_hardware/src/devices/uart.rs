use core::fmt;

use spin::Mutex;

use super::ioport::Port;

#[allow(dead_code)]
pub struct SerialPort {
    data_reg: Port,
    inter_reg: Port,
    inter_ident_fifo_control_reg: Port,
    line_control_reg: Port,
    modem_control_reg: Port,
    line_status_reg: Port,
    modem_status_reg: Port,
    scratch_reg: Port,
}

impl SerialPort {
    /// Create a new serial port
    /// 
    /// ## Safety
    /// It must be ensured that a correct base port is specified
    /// so that this instance actually points to a serial port
    pub const unsafe fn new(base_port: u16) -> SerialPort {
        SerialPort {
            data_reg: Port::new(base_port),
            inter_reg: Port::new(base_port + 1),
            inter_ident_fifo_control_reg: Port::new(base_port + 2),
            line_control_reg: Port::new(base_port + 3),
            modem_control_reg: Port::new(base_port + 4),
            line_status_reg: Port::new(base_port + 5),
            modem_status_reg: Port::new(base_port + 6),
            scratch_reg: Port::new(base_port + 7),
        }
    }

    pub fn initialize(&self) {
        unsafe {
            self.disable_interrupts();
            self.set_baud_divisor(1);
            self.line_control_reg.out_u8(0x3); // 8-bit, 1 stop bit, no parity
            self.inter_ident_fifo_control_reg.out_u8(0x07); // Enabled, Clear Receive, Clear Transmit, DMA 0
            self.modem_control_reg.out_u8(0x0B);
        }
    }

    pub fn write_byte(&self, byte: u8) {
        unsafe {
            // Wait for transmit to clear
            while !self.is_transmit_empty() { } 
            // Send the byte
            self.data_reg.out_u8(byte);
        }
    }

    unsafe fn set_baud_divisor(&self, divisor: u16) {
        let least_significant_byte = (divisor & 0xFF) as u8;
        let most_significant_byte = (divisor >> 8 & 0xFF) as u8;
        self.set_dlab(true);
        self.data_reg.out_u8(least_significant_byte);
        self.inter_reg.out_u8(most_significant_byte);
        self.set_dlab(false);
    }

    unsafe fn disable_interrupts(&self) {
        self.inter_reg.out_u8(0x00);
    }

    unsafe fn set_dlab(&self, enable: bool) {
        let current_value = self.line_control_reg.in_u8();
        if enable {
            self.line_control_reg.out_u8(current_value | 0x80);
        } else {
            self.line_control_reg.out_u8(current_value & !0x80);
        }
    }

    unsafe fn is_transmit_empty(&self) -> bool {
        // Line status register bit 5 low indicates empty 
        (self.line_status_reg.in_u8() & 0x20) != 0
    }
}

impl fmt::Write for SerialPort {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            self.write_byte(byte);
        }
        Ok(())
    }
}

const COM1_BASE: u16 = 0x3F8;

pub static COM1: Mutex<SerialPort> = Mutex::new(unsafe {SerialPort::new(COM1_BASE)});

#[macro_export]
macro_rules! com1_print {
    ($($arg:tt)*) => ($crate::devices::uart::_com1_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! com1_println {
    () => ($crate::com1_print!("\n"));
    ($($arg:tt)*) => ($crate::com1_print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _com1_print(args: fmt::Arguments) {
    use core::fmt::Write;
    COM1.lock().write_fmt(args).unwrap();
}