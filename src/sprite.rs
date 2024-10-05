use crate::{
    wasm4::{self, BLIT_1BPP, BLIT_2BPP},
    wasm4_mmio,
};

pub trait Texture {}

pub struct Texture2Color<'a> {
    pub data: &'a [u8],
    pub stride: u32,
}

impl Texture for Texture2Color<'_> {}

pub struct Texture4Color<'a> {
    pub data: &'a [u8],
    pub stride: u32,
}

impl Texture for Texture4Color<'_> {}

pub struct Sprite<'a, T: Texture> {
    pub texture: &'a T,
    pub width: u32,
    pub height: u32,
    pub src_x: u32,
    pub src_y: u32,
    pub draw_colors: u16,
}

impl<'a> Sprite<'a, Texture2Color<'a>> {
    pub fn draw(&self, x: i32, y: i32, flags: u32) {
        wasm4_mmio::DRAW_COLORS.write(self.draw_colors);
        wasm4::blit_sub(
            self.texture.data,
            x,
            y,
            self.width,
            self.height,
            self.src_x,
            self.src_y,
            self.texture.stride,
            flags | BLIT_1BPP,
        );
    }
}

impl<'a> Sprite<'a, Texture4Color<'a>> {
    pub fn draw(&self, x: i32, y: i32, flags: u32) {
        wasm4_mmio::DRAW_COLORS.write(self.draw_colors);
        wasm4::blit_sub(
            self.texture.data,
            x,
            y,
            self.width,
            self.height,
            self.src_x,
            self.src_y,
            self.texture.stride,
            flags | BLIT_2BPP,
        );
    }
}
