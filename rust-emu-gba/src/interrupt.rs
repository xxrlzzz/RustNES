
use rust_emu_common::types::*;
pub const INT_VBLANK: Byte = 1 << 0;
pub const INT_LCD: Byte = 1 << 1;
pub const INT_TIMER: Byte = 1 << 2;
pub const INT_SERIAL: Byte = 1 << 3;
pub const INT_JOYPAD: Byte = 1 << 4;