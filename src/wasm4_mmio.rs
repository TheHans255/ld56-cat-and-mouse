use voladdress::VolBlock;
/// A set of VolAddress definitions that allow us to avoid using unsafe declarations
/// while pointing to memory everywhere
pub use voladdress::{Safe, VolAddress};

macro_rules! wasm4_mmio {
    ($addr:literal, $name:ident, $t:ty, $r:ty, $w:ty) => {
        pub const $name: VolAddress<$t, $r, $w> = unsafe { VolAddress::new($addr) };
    };
}

wasm4_mmio!(0x0004, PALETTE, [u32; 4], Safe, Safe);
wasm4_mmio!(0x0014, DRAW_COLORS, u16, Safe, Safe);
wasm4_mmio!(0x0016, GAMEPAD1, u8, Safe, ());
wasm4_mmio!(0x0017, GAMEPAD2, u8, Safe, ());
wasm4_mmio!(0x0018, GAMEPAD3, u8, Safe, ());
wasm4_mmio!(0x0019, GAMEPAD4, u8, Safe, ());
wasm4_mmio!(0x001a, MOUSE_X, i16, Safe, ());
wasm4_mmio!(0x001c, MOUSE_Y, i16, Safe, ());
wasm4_mmio!(0x001e, MOUSE_BUTTONS, u8, Safe, ());
wasm4_mmio!(0x001f, SYSTEM_FLAGS, u8, Safe, Safe);
wasm4_mmio!(0x0020, NETPLAY, u8, Safe, ());

pub const FRAMEBUFFER: VolBlock<u8, Safe, Safe, 6400> = unsafe { VolBlock::new(0x00a0) };