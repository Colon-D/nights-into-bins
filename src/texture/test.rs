use crate::texture::{Color, Palette, Texture};
use byteorder::*;
use std::fs::{self, File};
use std::io::{self, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::Path;

use super::palette_texture::PaletteTexture;

pub enum Test {
    _8Bit256Colors32x32,
    _8Bit4096Colors64x64,
    _8Bit16384Colors128x128,
    _8Bit1024Colors32x32,
    _4Bit2048Colors32x64,
}

pub fn palette(test_type: Test) -> Palette {
    Palette(match test_type {
        Test::_8Bit256Colors32x32 => {
            let mut palette = vec![
                Color {
                    r: 0xFF,
                    g: 0xFF,
                    b: 0xFF,
                    a: 0xFF,
                };
                8 * 8
            ];
            for row in 0..8 {
                for col in 0..8 {
                    palette[(row * 8 + col) as usize] = Color {
                        r: (row * 32) as _,
                        g: (col * 32) as _,
                        b: (row * 32) as _,
                        a: 0xFF,
                    };
                }
            }
            palette
        }
        Test::_8Bit4096Colors64x64 => {
            let mut palette = vec![
                Color {
                    r: 0xFF,
                    g: 0xFF,
                    b: 0xFF,
                    a: 0xFF,
                };
                64 * 64
            ];
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
        Test::_8Bit16384Colors128x128 => {
            let mut palette = vec![
                Color {
                    r: 0xFF,
                    g: 0xFF,
                    b: 0xFF,
                    a: 0xFF,
                };
                128 * 128
            ];
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
        Test::_8Bit1024Colors32x32 => {
            let mut palette = vec![
                Color {
                    r: 0xFF,
                    g: 0xFF,
                    b: 0xFF,
                    a: 0xFF,
                };
                32 * 32
            ];
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
        Test::_4Bit2048Colors32x64 => {
            let mut palette = vec![
                Color {
                    r: 0xFF,
                    g: 0xFF,
                    b: 0xFF,
                    a: 0xFF,
                };
                32 * 64
            ];
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
    })
}

pub fn texture(test_type: Test) -> PaletteTexture {
    match test_type {
        Test::_8Bit256Colors32x32 => {
            let mut palette_tex = PaletteTexture {
                data: vec![0; 32 * 32],
                width: 32,
            };
            for row in 0..32 {
                for col in 0..32 {
                    for sub_row in 0..4 {
                        for sub_col in 0..4 {
                            palette_tex.data
                                [((row * 4 + sub_row) * 32 + col * 4 + sub_col) as usize] =
                                (row * 8 + col) as _;
                        }
                    }
                }
            }
            palette_tex
        }
        Test::_8Bit4096Colors64x64 => {
            let mut palette_tex = PaletteTexture {
                data: vec![0; 64 * 64],
                width: 64,
            };
            for row in 0..64 {
                for col in 0..64 {
                    palette_tex.data[(row * 64 + col) as usize] = (row * 64 + col) as _;
                }
            }
            palette_tex
        }
        Test::_8Bit16384Colors128x128 => {
            let mut palette_tex = PaletteTexture {
                data: vec![0; 128 * 128],
                width: 128,
            };
            for row in 0..128 {
                for col in 0..128 {
                    palette_tex.data[(row * 128 + col) as usize] = (row * 128 + col) as _;
                }
            }
            palette_tex
        }
        Test::_8Bit1024Colors32x32 => {
            let mut palette_tex = PaletteTexture {
                data: vec![0; 32 * 32],
                width: 32,
            };
            for row in 0..32 {
                for col in 0..32 {
                    palette_tex.data[(row * 32 + col) as usize] = (row * 32 + col) as _;
                }
            }
            palette_tex
        }
        Test::_4Bit2048Colors32x64 => {
            let mut palette_tex = PaletteTexture {
                data: vec![0; 32 * 64],
                width: 32,
            };
            for row in 0..64 {
                for col in 0..32 {
                    palette_tex.data[(row * 32 + col) as usize] = (row * 32 + col) as _;
                }
            }
            palette_tex
        }
    }
}
