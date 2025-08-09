// src/vga_buffer.rs
use volatile::Volatile;
use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);

impl ColorCode {
    fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;
const HISTORY_SIZE: usize = 1000; // Lines of history to keep

#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

pub struct Writer {
    column_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
    history: [[ScreenChar; BUFFER_WIDTH]; HISTORY_SIZE],
    history_lines: usize,
    scroll_offset: usize, // How many lines scrolled up from bottom
}

impl Writer {
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;

                let color_code = self.color_code;
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code,
                });
                self.column_position += 1;
            }
        }
    }

    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                _ => self.write_byte(0xfe),
            }
        }
    }

    fn new_line(&mut self) {
        // Save current bottom line to history before scrolling
        if self.history_lines < HISTORY_SIZE {
            for col in 0..BUFFER_WIDTH {
                self.history[self.history_lines][col] = self.buffer.chars[BUFFER_HEIGHT - 1][col].read();
            }
            self.history_lines += 1;
        } else {
            // Shift history up and add new line at bottom
            for line in 1..HISTORY_SIZE {
                self.history[line - 1] = self.history[line];
            }
            for col in 0..BUFFER_WIDTH {
                self.history[HISTORY_SIZE - 1][col] = self.buffer.chars[BUFFER_HEIGHT - 1][col].read();
            }
        }

        // Scroll screen buffer up
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let character = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(character);
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }

    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }

    pub fn scroll_up(&mut self) {
        if self.scroll_offset < self.history_lines {
            self.scroll_offset += 1;
            self.refresh_screen();
        }
    }

    pub fn scroll_down(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
            self.refresh_screen();
        }
    }

    fn refresh_screen(&mut self) {
        if self.scroll_offset == 0 {
            // Show current screen - nothing to do, it's already there
            return;
        }

        // Calculate which history lines to show
        let history_start = if self.scroll_offset > BUFFER_HEIGHT {
            self.history_lines - self.scroll_offset
        } else {
            if self.history_lines >= self.scroll_offset {
                self.history_lines - self.scroll_offset
            } else {
                0
            }
        };

        // Fill screen with history
        for screen_row in 0..BUFFER_HEIGHT {
            let history_row = history_start + screen_row;
            
            if history_row < self.history_lines {
                // Show history line
                for col in 0..BUFFER_WIDTH {
                    self.buffer.chars[screen_row][col].write(self.history[history_row][col]);
                }
            } else {
                // Clear empty lines
                self.clear_row(screen_row);
            }
        }
    }

    pub fn clear_screen(&mut self) {
        for row in 0..BUFFER_HEIGHT {
            self.clear_row(row);
        }
        self.column_position = 0;
        self.scroll_offset = 0;
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::Yellow, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
        history: [[ScreenChar {
            ascii_character: b' ',
            color_code: ColorCode::new(Color::Yellow, Color::Black),
        }; BUFFER_WIDTH]; HISTORY_SIZE],
        history_lines: 0,
        scroll_offset: 0,
    });
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    interrupts::without_interrupts(|| {
        WRITER.lock().write_fmt(args).unwrap();
    });
}

// Public functions for scrolling
pub fn scroll_up() {
    use x86_64::instructions::interrupts;
    interrupts::without_interrupts(|| {
        WRITER.lock().scroll_up();
    });
}

pub fn scroll_down() {
    use x86_64::instructions::interrupts;
    interrupts::without_interrupts(|| {
        WRITER.lock().scroll_down();
    });
}

pub fn clear_screen() {
    use x86_64::instructions::interrupts;
    interrupts::without_interrupts(|| {
        WRITER.lock().clear_screen();
    });
}