use x86_64::instructions::port::Port;
use spin::Mutex;
use lazy_static::lazy_static;

lazy_static! {
    static ref KEYBOARD_BUFFER: Mutex<[u8; 16]> = Mutex::new([0; 16]);
    static ref BUFFER_START: Mutex<usize> = Mutex::new(0);
    static ref BUFFER_END: Mutex<usize> = Mutex::new(0);
}

// Scancode to ASCII mapping for common keys
const SCANCODE_TO_CHAR: [Option<char>; 128] = {
    let mut map = [None; 128];
    
    // Numbers
    map[0x02] = Some('1'); map[0x03] = Some('2'); map[0x04] = Some('3');
    map[0x05] = Some('4'); map[0x06] = Some('5'); map[0x07] = Some('6');
    map[0x08] = Some('7'); map[0x09] = Some('8'); map[0x0A] = Some('9');
    map[0x0B] = Some('0');
    
    // Letters
    map[0x10] = Some('q'); map[0x11] = Some('w'); map[0x12] = Some('e');
    map[0x13] = Some('r'); map[0x14] = Some('t'); map[0x15] = Some('y');
    map[0x16] = Some('u'); map[0x17] = Some('i'); map[0x18] = Some('o');
    map[0x19] = Some('p');
    
    map[0x1E] = Some('a'); map[0x1F] = Some('s'); map[0x20] = Some('d');
    map[0x21] = Some('f'); map[0x22] = Some('g'); map[0x23] = Some('h');
    map[0x24] = Some('j'); map[0x25] = Some('k'); map[0x26] = Some('l');
    
    map[0x2C] = Some('z'); map[0x2D] = Some('x'); map[0x2E] = Some('c');
    map[0x2F] = Some('v'); map[0x30] = Some('b'); map[0x31] = Some('n');
    map[0x32] = Some('m');
    
    // Special keys
    map[0x39] = Some(' '); // Space
    map[0x1C] = Some('\n'); // Enter
    
    map
};

fn keyboard_status_ready() -> bool {
    let mut port = Port::new(0x64);
    let status: u8 = unsafe { port.read() };
    (status & 0x01) != 0
}

fn read_keyboard_data() -> u8 {
    let mut port = Port::new(0x60);
    unsafe { port.read() }
}

/// Poll keyboard hardware, return Some(char) if a key was pressed, else None
pub fn poll_keyboard() -> Option<char> {
    if !keyboard_status_ready() {
        return None;
    }
    
    let scancode = read_keyboard_data();
    
    // Ignore key releases (high bit set)
    if scancode & 0x80 != 0 {
        return None;
    }
    
    SCANCODE_TO_CHAR.get(scancode as usize).copied().flatten()
}

/// Try reading a char from keyboard without blocking
pub fn try_read_char() -> Option<char> {
    poll_keyboard()
}

/// Blocking call to read a char from keyboard
pub fn read_char_blocking() -> char {
    loop {
        if let Some(ch) = poll_keyboard() {
            return ch;
        }
        for _ in 0..1000 {
            x86_64::instructions::nop();
        }
    }
}

pub fn init() {

}