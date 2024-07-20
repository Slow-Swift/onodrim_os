use core::arch::asm;

pub struct Port {
    port_number: u16,
}

impl Port {

    /// ## Safety
    /// 
    /// The port specified must be value and must be used in a way supported by
    /// the hardware.
    pub const unsafe fn new(port_number: u16) -> Port {
        Port { port_number }
    }

    pub unsafe fn out_u8(&self, value: u8) {
        asm!("out dx, al", in("dx") self.port_number, in("al") value, options(nomem, nostack, preserves_flags));
    }

    pub unsafe fn in_u8(&self) -> u8 {
        let output: u8;
        asm!("in al, dx", in("dx") self.port_number, out("al") output, options(nomem, nostack, preserves_flags));

        output
    }
}