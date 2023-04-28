use crate::texture::CHUNK_SIZE;
use num_traits::PrimInt;

pub fn convert32x32<N: PrimInt>(array: &mut [N]) {
    // Swap every other pair of columns
    // (I read nibbles opposite of the C++ program)
    // (no I won't just read nibbles in the same way)
    for i in (0..array.len()).step_by(2) {
        array.swap(i, i + 1);
    }

    let mut temp_array = [N::zero(); 32 * 32];

    // Rearrange 16 sets of 64 contiguous pixels
    // Original order is 0 8 1 9 2 10 3 11 4 12 5 13 6 14 7 15
    // Needs to be rearranged to 0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15
    for i in 0..16 {
        for j in 0..64 {
            let index = if i % 2 == 0 {
                i / 2 * 64 + j
            } else {
                i / 2 * 64 + j + 32 * 16
            };
            temp_array[index] = array[i * 64 + j];
        }
    }

    // Load temp_array back into array1
    for i in 0..32 * 32 {
        array[i] = temp_array[i];
    }

    // Swap every other pair of pixels with the pair of pixels in the following row
    for i in 0..16 {
        for j in 0..32 {
            if (i / 2 % 2 == 0 && (j % 4 == 1 || j % 4 == 2))
                || (i / 2 % 2 == 1 && (j % 4 == 0 || j % 4 == 3))
            {
                let index1 = (i * 2 + 1) * 32 + j;
                let index2 = i * 2 * 32 + j;
                array.swap(index1, index2);
            }
        }
    }

    // Put every 8th pixel in a 32-pixel row next to each other
    for i in 0..32 {
        for j in 0..32 {
            temp_array[i * 32 + j] = array[i * 32 + (j % 4) * 8 + j / 4];
        }
    }

    for i in 0..32 * 32 {
        array[i] = temp_array[i];
    }

    // "Unweave" adjacent 4-pixel-wide columns
    for i in 0..16 {
        for j in 0..4 {
            for k in 0..4 {
                let index1 = (i * 2 + 1) * 32 + j * 8 + 4 + k;
                let index2 = i * 2 * 32 + j * 8 + k;
                array.swap(index1, index2);
            }
        }
    }

    // Switch odd pairs of columns
    for i in 0..32 {
        for j in 0..4 {
            for k in 0..4 {
                if j % 2 == 1 {
                    let index1 = i * 32 + j * 8 + 4 + k;
                    let index2 = i * 32 + j * 8 + k;
                    array.swap(index1, index2);
                }
            }
        }
    }

    //one more pass to swap every other pair of rows
    for i in 0..8 {
        for j in 0..32 {
            let index1 = i * 128 + 32 + j;
            let index2 = i * 128 + 64 + j;
            array.swap(index1, index2);
        }
    }
}

pub fn convert64x64_4bit<N: PrimInt>(array: &mut [N]) {
    // Swap every other pair of columns
    // (I read nibbles opposite of the C++ program)
    // (no I won't just read nibbles in the same way)
    for i in (0..array.len()).step_by(2) {
        array.swap(i, i + 1);
    }

    const HEIGHT: usize = 64;
    const WIDTH: usize = 64;
    for row in 0..HEIGHT {
        if row / 8 % 2 == 1 {
            for j in 0..WIDTH / 4 {
                let temp = array[row * WIDTH + j * 2 + 1];
                array[row * WIDTH + j * 2 + 1] = array[row * WIDTH + WIDTH / 2 + j * 2];
                array[row * WIDTH + WIDTH / 2 + j * 2] =
                    array[row * WIDTH + WIDTH / 2 + j * 2 + 1];
                array[row * WIDTH + WIDTH / 2 + j * 2 + 1] = temp;
            }
        } else {
            for j in 0..WIDTH / 4 {
                let temp = array[row * WIDTH + WIDTH / 2 + j * 2];
                array[row * WIDTH + WIDTH / 2 + j * 2] = array[row * WIDTH + j * 2 + 1];
                array[row * WIDTH + j * 2 + 1] = array[row * WIDTH + j * 2];
                array[row * WIDTH + j * 2] = temp;
            }
        }
    }
    let mut chunk_array = vec![N::zero(); array.len()];
    for w in 0..WIDTH / CHUNK_SIZE {
        for h in 0..HEIGHT / CHUNK_SIZE {
            for c in 0..CHUNK_SIZE {
                for d in 0..CHUNK_SIZE {
                    chunk_array[c * CHUNK_SIZE + d] =
                        array[h * WIDTH * CHUNK_SIZE + c * WIDTH + w * CHUNK_SIZE + d];
                }
            }
            convert_4bit(&mut chunk_array);
            for c in 0..CHUNK_SIZE {
                for d in 0..CHUNK_SIZE {
                    array[h * WIDTH * CHUNK_SIZE + c * WIDTH + w * CHUNK_SIZE + d] =
                        chunk_array[c * CHUNK_SIZE + d];
                }
            }
        }
    }
    let mut new_array = vec![N::zero(); array.len()];
    for i in 0..64 {
        for j in 0..32 {
            if i % 2 == 1 {
                new_array[(i - 1) * 2 * 32 + j + 32] = array
                    [i % 16 / 4 * 16 * 32 + i / 16 * 4 * 32 + (1 - i / 2 % 2) * 32 + 32 * 64 + j];
                new_array[(i - 1) * 2 * 32 + j + 32 + 64] = array[i % 16 / 4 * 16 * 32
                    + i / 16 * 4 * 32
                    + (1 - i / 2 % 2) * 32
                    + 32 * 64
                    + j
                    + 64];
            } else {
                new_array[i * 2 * 32 + j] =
                    array[i % 16 / 4 * 16 * 32 + i / 16 * 4 * 32 + (1 - i / 2 % 2) * 32 + j];
                new_array[i * 2 * 32 + j + 64] =
                    array[i % 16 / 4 * 16 * 32 + i / 16 * 4 * 32 + (1 - i / 2 % 2) * 32 + j + 64];
            }
        }
    }
    array.copy_from_slice(&new_array);
}

// unscramble 4-bit image data
fn convert_4bit<N: PrimInt>(array: &mut [N]) {
    let mut temp_array = vec![N::zero(); array.len()];
    // put every 8th pixel in a 32-pixel row next to each other
    for i in 0..32 {
        for j in 0..32 {
            temp_array[i * 32 + j] = array[i * 32 + (j % 4) * 8 + j / 4];
        }
    }
    // interweave every other row
    for i in 0..32 {
        for j in 0..32 {
            if i % 2 == 0 {
                array[i * 32 + j] = temp_array[i / 8 * 8 * 32 + i % 8 / 2 * 32 + j];
            } else {
                array[i * 32 + j] = temp_array[i / 8 * 8 * 32 + i % 8 / 2 * 32 + 4 * 32 + j];
            }
        }
    }
}
