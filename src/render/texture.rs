use fnv::FnvHashMap;
use golem::{ColorFormat, Context, Texture};
use ref_thread_local::{ref_thread_local, RefThreadLocal};

use crate::static_map::StaticMap;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum TextureType {
    Main,
}

type TextureMap = FnvHashMap<TextureType, Texture>;

ref_thread_local!(
    pub static managed TEXTURES: StaticMap<TextureType, Texture, fn(&Context) -> TextureMap, &'static Context> = StaticMap::new(
        texture_map
    );
);

pub const MAIN_TEXTURE_WIDTH: usize = 1024;
pub const MAIN_TEXTURE_HEIGHT: usize = 1024;

pub const GLYPH_CACHE_OFFSET_X: usize = 0;
pub const GLYPH_CACHE_OFFSET_Y: usize = MAIN_TEXTURE_HEIGHT / 2;
pub const GLYPH_CACHE_WIDTH: usize = MAIN_TEXTURE_WIDTH;
pub const GLYPH_CACHE_HEIGHT: usize = MAIN_TEXTURE_HEIGHT - GLYPH_CACHE_OFFSET_Y;

fn texture_map(gl: &Context) -> TextureMap {
    vec![(TextureType::Main, {
        let mut tex = Texture::new(gl).unwrap();

        let img = vec![0u8; 4 * MAIN_TEXTURE_WIDTH * MAIN_TEXTURE_HEIGHT];
        tex.set_image(
            Some(&img),
            MAIN_TEXTURE_WIDTH as u32,
            MAIN_TEXTURE_HEIGHT as u32,
            ColorFormat::RGBA,
            false,
        );

        tex
    })]
    .into_iter()
    .collect()
}
