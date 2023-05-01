use super::palette_texture::PaletteTexture;
use crate::texture::{Color, Palette};
use ndarray::Array2;

pub enum Test {
    _4096Colors64x64,
    _16384Colors128x128,
    _65536Colors256x256,
    _1024Colors32x32,
    _256Colors16x16,
    _2048Colors32x64,
}

pub fn palette(test_type: Test) -> Palette {
    Palette(match test_type {
        Test::_4096Colors64x64 => {
            let mut palette = vec![Color::default(); 64 * 64];
            for row in 0..64 {
                for col in 0..64 {
                    palette[(row * 64 + col) as usize] = Color {
                        r: (row * 4) as _,
                        g: (col * 4) as _,
                        b: (row * 4) as _,
                        a: 0xFF,
                    };
                }
            }
            palette
        }
        Test::_16384Colors128x128 => {
            let mut palette = vec![Color::default(); 128 * 128];
            for row in 0..128 {
                for col in 0..128 {
                    palette[(row * 128 + col) as usize] = Color {
                        r: (row * 2) as _,
                        g: (col * 2) as _,
                        b: (row * 2) as _,
                        a: 0xFF,
                    };
                }
            }
            palette
        }
        Test::_1024Colors32x32 => {
            let mut palette = vec![Color::default(); 32 * 32];
            for row in 0..32 {
                for col in 0..32 {
                    palette[(row * 32 + col) as usize] = Color {
                        r: (row * 8) as _,
                        g: (col * 8) as _,
                        b: (row * 8) as _,
                        a: 0xFF,
                    };
                }
            }
            palette
        }
        Test::_256Colors16x16 => {
            let mut palette = vec![Color::default(); 16 * 16];
            for row in 0..16 {
                for col in 0..16 {
                    palette[(row * 16 + col) as usize] = Color {
                        r: (row * 16) as _,
                        g: (col * 16) as _,
                        b: (row * 16) as _,
                        a: 0xFF,
                    };
                }
            }
            palette
        }
        Test::_2048Colors32x64 => {
            let mut palette = vec![Color::default(); 32 * 64];
            for row in 0..64 {
                for col in 0..32 {
                    palette[(row * 32 + col) as usize] = Color {
                        r: (row * 4) as _,
                        g: (col * 8) as _,
                        b: (row * 4) as _,
                        a: 0xFF,
                    };
                }
            }
            palette
        }
        Test::_65536Colors256x256 => {
            let mut palette = vec![Color::default(); 256 * 256];
            for row in 0..256 {
                for col in 0..256 {
                    palette[(row * 256 + col) as usize] = Color {
                        r: row as _,
                        g: col as _,
                        b: row as _,
                        a: 0xFF,
                    };
                }
            }
            palette
        }
    })
}

pub fn texture(test_type: Test) -> PaletteTexture {
    let mut palette_tex = Array2::default(match test_type {
        Test::_4096Colors64x64 => (64, 64),
        Test::_16384Colors128x128 => (128, 128),
        Test::_1024Colors32x32 => (32, 32),
        Test::_256Colors16x16 => (16, 16),
        Test::_2048Colors32x64 => (64, 32),
        Test::_65536Colors256x256 => (256, 256),
    });
    for row in 0..palette_tex.nrows() {
        for col in 0..palette_tex.ncols() {
            palette_tex[[row, col]] = (row * palette_tex.ncols() + col) as _;
        }
    }
    PaletteTexture(palette_tex)
}
