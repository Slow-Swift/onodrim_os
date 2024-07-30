use core::fmt::{self, Debug, Write};

use bootinfo::BootInfo;
use x86_64_hardware::devices::uart::COM1;

use crate::{font_renderer::FontRenderer, graphics_renderer::{Color, FrameBuffer}, layout_renderer::LayoutRenderer};

const MIN_SERIAL_LOG_LEVEL: LogLevel = LogLevel::Debug;
const MIN_DISPLAY_LOG_LEVEL: LogLevel = LogLevel::Debug;

const OUTPUT_SERIAL_COLORS: bool = true;
const OUTPUT_LOG_LEVEL: bool = false;

const SERIAL_COLOR_RESET: &str = "\x1b[0;0;0m";
const DEFAULT_DISPLAY_FOREGROUND: Color = Color::new(0x00FF00);
const DEFAULT_DISPLAY_BACKGROUND: Color = Color::new(0x0000000);

#[repr(u8)]
#[derive(Debug, PartialEq, PartialOrd)]
#[allow(dead_code)]
pub enum LogLevel {
    Debug = 0,
    Info = 1,
    Warn = 2,
    Error = 3,
    Critical = 4,
}

impl LogLevel {
    pub fn get_serial_color(&self) -> &str {
        match self {
            LogLevel::Debug => "\x1b[2;37m",
            LogLevel::Info => "\x1b[37m",
            LogLevel::Warn => "\x1b[1;33m",
            LogLevel::Error => "\x1b[1;31m",
            LogLevel::Critical => "\x1b[1;37;41m",
        }
    }

    pub fn get_display_color(&self) -> (Color, Color) {
        match self {
            LogLevel::Debug => (DEFAULT_DISPLAY_FOREGROUND / 2, DEFAULT_DISPLAY_BACKGROUND),
            LogLevel::Info => (DEFAULT_DISPLAY_FOREGROUND, DEFAULT_DISPLAY_BACKGROUND),
            LogLevel::Warn => (Color::new(0xCCCC00), DEFAULT_DISPLAY_BACKGROUND),
            LogLevel::Error => (Color::new(0xCC0000), DEFAULT_DISPLAY_BACKGROUND),
            LogLevel::Critical => (Color::new(0xFFFFFF), Color::new(0xBB0000)),
        }
    }

    pub fn get_prefix(&self) -> &str {
        match self {
            LogLevel::Debug => "D",
            LogLevel::Info => "I",
            LogLevel::Warn => "W",
            LogLevel::Error => "E",
            LogLevel::Critical => "C",
        }
    }
}

static mut RENDERER: Option<LayoutRenderer> = None;

pub fn initialize_com1() {
    COM1.lock().initialize();
}

pub fn initialize_screen_output(bootinfo: &BootInfo) {
    let mut frame_buffer = FrameBuffer::from_boot_data(&bootinfo)
        .expect("Could not create frame buffer.");
    frame_buffer.fill(Color::new(0x000000));

    let font_renderer = FontRenderer::create(
        bootinfo.font_file_address.as_u64() as *mut u8, 
        bootinfo.font_file_size, 
        frame_buffer
    ).expect("Could not create font renderer");
    
    let mut layout_renderer = LayoutRenderer::new(font_renderer);
    layout_renderer.set_colors((DEFAULT_DISPLAY_FOREGROUND, DEFAULT_DISPLAY_BACKGROUND));

    unsafe { RENDERER = Some(layout_renderer) };
}

pub fn _print_fmt(args: fmt::Arguments) {
    COM1.lock().write_fmt(args).unwrap();

    unsafe {
        match RENDERER.as_mut() {
            Some(renderer) => { renderer.write_fmt(args).unwrap(); },
            None => {},
        }
    }
}

pub fn _log_fmt(level: LogLevel, args: fmt::Arguments) {
    if level >= MIN_SERIAL_LOG_LEVEL {
        let color_code = if OUTPUT_SERIAL_COLORS { level.get_serial_color() } else { SERIAL_COLOR_RESET };

        if OUTPUT_LOG_LEVEL {
            COM1.lock().write_fmt(format_args!(
                "{color_code}[{}] {args}{SERIAL_COLOR_RESET}\n", level.get_prefix()
            )).unwrap();
        } else {
            COM1.lock().write_fmt(format_args!("{color_code}{args}{SERIAL_COLOR_RESET}\n"))
                .unwrap();
        }
        
    }

    if level >= MIN_DISPLAY_LOG_LEVEL {
        unsafe {
            match RENDERER.as_mut() {
                Some(renderer) => { 
                    renderer.set_colors(level.get_display_color());
                    if OUTPUT_LOG_LEVEL {
                        renderer.write_fmt(format_args!("[{}] {args}\n", level.get_prefix())).unwrap(); 
                    } else {
                        renderer.write_fmt(format_args!("{args}\n")).unwrap();
                    }
                    renderer.set_colors((DEFAULT_DISPLAY_FOREGROUND, DEFAULT_DISPLAY_BACKGROUND)); 
                },
                None => {},
            }
        }
    }
}

pub fn _log_module_fmt(level: LogLevel, module: &str, args: fmt::Arguments) {
    _log_fmt(level, format_args!("[{module}] {args}"));
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::logger::_print_fmt(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::com1_print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! log {
    ($level:expr, $($arg:tt)*) => ($crate::logger::_log_fmt($level, format_args!($($arg)*)));
    ($level:expr, module:expr, $($arg:tt)*) => (
        $crate::logger::_log_module_fmt($level, module, format_args!($($arg)*))
    );
}

#[macro_export]
macro_rules! log_debug {
    ($module:literal, $($arg:tt)*) => (
        $crate::logger::_log_module_fmt(crate::logger::LogLevel::Debug, $module, format_args!($($arg)*))
    );
    ($($arg:tt)*) => ($crate::logger::_log_fmt(crate::logger::LogLevel::Debug, format_args!($($arg)*)));
}

#[macro_export]
macro_rules! log_info {
    ($module:expr, $($arg:tt)*) => (
        $crate::logger::_log_module_fmt(crate::logger::LogLevel::Info, $module, format_args!($($arg)*))
    );
    ($($arg:tt)*) => ($crate::logger::_log_fmt(crate::logger::LogLevel::Info, format_args!($($arg)*)));
}

#[macro_export]
macro_rules! log_warn {
    ($module:expr, $($arg:tt)*) => (
        $crate::logger::_log_module_fmt(crate::logger::LogLevel::Warn, $module, format_args!($($arg)*))
    );
    ($($arg:tt)*) => ($crate::logger::_log_fmt(crate::logger::LogLevel::Warn, format_args!($($arg)*)));
}

#[macro_export]
macro_rules! log_error {
    ($module:expr, $($arg:tt)*) => (
        $crate::logger::_log_module_fmt(crate::logger::LogLevel::Error, $module, format_args!($($arg)*))
    );
    ($($arg:tt)*) => ($crate::logger::_log_fmt(crate::logger::LogLevel::Error, format_args!($($arg)*)));
}

#[macro_export]
macro_rules! log_critical {
    ($module:expr, $($arg:tt)*) => (
        $crate::logger::_log_module_fmt(crate::logger::LogLevel::Critical, $module, format_args!($($arg)*))
    );
    ($($arg:tt)*) => ($crate::logger::_log_fmt(crate::logger::LogLevel::Critical, format_args!($($arg)*)));
}