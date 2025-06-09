use crate::trace;

use super::io::out8;

const PIT_CMD_PORT: u16 = 0x43;
const PIT_DATA_PORT: u16 = 0x40;

pub unsafe fn init_timer() {
    trace!("Initializing timer...");
    let clock_freq = 1193182usize;
    let expected_freq = 100usize;
    let k = (clock_freq / expected_freq) as u16;
    unsafe {
        out8(PIT_CMD_PORT, 0b00110110);
        out8(PIT_DATA_PORT + 0, k as u8);
        out8(PIT_DATA_PORT + 0, (k >> 8) as u8);
    }
}
