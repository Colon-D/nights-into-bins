use crate::texture::CHUNK_SIZE;
use ndarray::{s, Array2, ArrayView2};
use num_traits::PrimInt;

// todo: test, and write inverse, + 4x4 chunk swap thing:
//   every eight rows, if row is 1 in table below, swap every pair of 4 elements in the row.
//   - 0
//   - 0
//   - 1
//   - 1
//   - 1
//   - 1
//   - 0
//   - 0
pub fn encode<T: Default + Copy>(array: &Array2<T>) -> Array2<T> {
    let mut out = Array2::<T>::default((array.nrows(), array.ncols()));
    for row in (0..array.nrows()).step_by(4) {
        let chunk = array.slice(s![row..row + 4, ..]);
        let (even, odd) = split_by_column_parity(&chunk);
        let joined = join_vertically(&even.view(), &odd.view());
        let (even, odd) = split_by_column_parity(&joined.view());
        let joined = join_horizontally(&even.view(), &odd.view());
        let (even, odd) = split_by_row_parity(&joined.view());
        let joined = join_horizontally(&even.view(), &odd.view());
        let mut out_chunk = out.slice_mut(s![row..row + 4, ..]);
        out_chunk.assign(&joined);
    }
    out
}

pub fn split_by_column_parity<T: Default + Copy>(array: &ArrayView2<T>) -> (Array2<T>, Array2<T>) {
    let mut even = Array2::<T>::default((array.nrows(), array.ncols() / 2));
    let mut odd = Array2::<T>::default((array.nrows(), array.ncols() / 2));
    for row in 0..array.nrows() {
        for col in 0..array.ncols() / 2 {
            even[(row, col)] = array[[row, col * 2]];
            odd[[row, col]] = array[[row, col * 2 + 1]];
        }
    }
    (even, odd)
}

pub fn split_by_row_parity<T: Default + Copy>(array: &ArrayView2<T>) -> (Array2<T>, Array2<T>) {
    let mut even = Array2::<T>::default((array.nrows(), array.ncols() / 2));
    let mut odd = Array2::<T>::default((array.nrows(), array.ncols() / 2));
    for row in 0..array.nrows() / 2 {
        for col in 0..array.ncols() {
            even[[row, col]] = array[[row * 2, col]];
            odd[[row, col]] = array[[row * 2 + 1, col]];
        }
    }
    (even, odd)
}

pub fn join_horizontally<T: Default + Copy>(
    lhs: &ArrayView2<T>,
    rhs: &ArrayView2<T>,
) -> Array2<T> {
    assert_eq!(lhs.nrows(), rhs.nrows());
    let mut out = Array2::<T>::default((lhs.nrows(), lhs.ncols() + rhs.ncols()));
    for row in 0..lhs.nrows() {
        for col in 0..lhs.ncols() {
            out[[row, col]] = lhs[[row, col]];
        }
    }
    for row in 0..rhs.nrows() {
        for col in 0..rhs.ncols() {
            out[[row, col + lhs.ncols()]] = rhs[[row, col]];
        }
    }
    out
}

pub fn join_vertically<T: Default + Copy>(
    top: &ArrayView2<T>,
    bottom: &ArrayView2<T>,
) -> Array2<T> {
    assert_eq!(top.ncols(), bottom.ncols());
    let mut out = Array2::<T>::default((top.nrows() + bottom.nrows(), top.ncols()));
    for row in 0..top.nrows() {
        for col in 0..top.ncols() {
            out[[row, col]] = top[[row, col]];
        }
    }
    for row in 0..bottom.nrows() {
        for col in 0..bottom.ncols() {
            out[[row + top.nrows(), col]] = bottom[[row, col]];
        }
    }
    out
}

pub fn convert_array<N: PrimInt>(num_rows: usize, num_chunks: usize, hex_array: &mut [N]) {
    let mut unscramble_array = vec![N::zero(); hex_array.len()];
    let mut out_array = vec![N::zero(); hex_array.len()];

    for i in 0..num_chunks {
        for j in (0..num_rows).step_by(2) {
            for k in 0..CHUNK_SIZE {
                if (j / 2) % 2 == 0 {
                    let unscramble_index = k * num_rows / 2 + j / 2;
                    let hex_index = j * num_chunks * CHUNK_SIZE + i * CHUNK_SIZE + k;
                    unscramble_array[unscramble_index] = hex_array[hex_index];
                    unscramble_array[unscramble_index + 1] =
                        hex_array[hex_index + num_chunks * CHUNK_SIZE];
                } else {
                    let unscramble_index = (k + 32) * num_rows / 2 + j / 2;
                    let hex_index = j * num_chunks * CHUNK_SIZE + i * CHUNK_SIZE + k;
                    unscramble_array[unscramble_index - 1] = hex_array[hex_index];
                    unscramble_array[unscramble_index] =
                        hex_array[hex_index + num_chunks * CHUNK_SIZE];
                }
            }
        }
        unscramble(num_rows, &mut unscramble_array);

        for j in 0..CHUNK_SIZE / 2 {
            for k in 0..num_rows * 2 {
                out_array[k * num_chunks * CHUNK_SIZE / 2 + i * CHUNK_SIZE / 2 + j] =
                    unscramble_array[j * num_rows * 2 + k];
            }
        }
    }
    hex_array.copy_from_slice(&out_array);
}

// unscramble 32-column chunk of 8-bit game file
fn unscramble<N: PrimInt>(num_rows: usize, array: &mut [N]) {
    let mut resolve_array: Vec<N> = vec![N::zero(); array.len()];
    // order that lines are stored in the chunk
    let order: [usize; 16] = [0, 4, 8, 12, 1, 5, 9, 13, 2, 6, 10, 14, 3, 7, 11, 15];
    let mut indices = [0usize; 4];
    let mut indices2 = [0usize; 4];
    for col in 0..CHUNK_SIZE / 2 {
        // save indices of matching quarter lines that make up each line index in order[]
        if order[col] % 2 == 0 {
            indices[0] = order[col] + 17;
            indices[1] = order[col];
            indices[2] = order[col] + 33;
            indices[3] = order[col] + 48;
            // need two indices arrays because reconstruction order depends on the column
            indices2[0] = order[col];
            indices2[1] = order[col] + 17;
            indices2[2] = order[col] + 48;
            indices2[3] = order[col] + 33;
        }
        // numbers are different depending on parity of index in order[]
        else {
            indices[0] = order[col] + 15;
            indices[1] = order[col];
            indices[2] = order[col] + 31;
            indices[3] = order[col] + 48;
            indices2[0] = order[col];
            indices2[1] = order[col] + 15;
            indices2[2] = order[col] + 48;
            indices2[3] = order[col] + 31;
        }
        for row in (0..num_rows / 2).step_by(2) {
            // k = which quarter line
            for k in 0..4 {
                // if col is 0-3 or 8-11, use indices2, else use indices
                let chunk_index = col * num_rows * 2 + row * 4 + k * 2;
                let array_index = if (col / 4) % 2 == 0 {
                    indices2
                } else {
                    indices
                }[k] * num_rows
                    / 2
                    + row;
                resolve_array[chunk_index] = array[array_index];
                resolve_array[chunk_index + 1] = array[array_index + 1];
            }
        }
    }
    array.clone_from_slice(&resolve_array);
}
