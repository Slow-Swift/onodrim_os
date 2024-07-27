use r_efi;
use r_efi::protocols::simple_text_output;

pub struct SimpleTextOutputProtocol {
    output_ptr: *mut simple_text_output::Protocol,
}

impl SimpleTextOutputProtocol {
    pub fn new(output_ptr: *mut simple_text_output::Protocol) -> SimpleTextOutputProtocol {
        SimpleTextOutputProtocol { output_ptr }
    }
}