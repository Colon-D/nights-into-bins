use ndarray::{s, Array2, ArrayView2, Axis};

// there is probably a better way to do this, that does not involve swapping
// middle quarters. I barely found this solution as it is though...
pub fn decode<T: Default + Copy>(array: &Array2<T>) -> Array2<T> {
    let mut out = array.clone();

    // should already be decoded?
    if out.ncols() == 8 {
        return out;
    }

    // deswizzle(?) 32x4 chunks
    for y in (0..out.nrows()).step_by(4) {
        for x in (0..out.ncols()).step_by(32) {
            let x_max = 32.min(out.ncols());
            let chunk = out.slice(s![y..y + 4, x..x + x_max]);

            let (even, odd) = split_by_column_parity(chunk.view());
            let chunk = join_vertically(even.view(), odd.view());

            let (even, odd) = split_by_column_parity(chunk.view());
            let chunk = join_horizontally(even.view(), odd.view());

            let (even, odd) = split_by_row_parity(chunk.view());
            let chunk = join_horizontally(even.view(), odd.view());

            out.slice_mut(s![y..y + 4, x..x + x_max]).assign(&chunk);
        }
    }

    // swap middle quarters around, from smallest to biggest
    if out.ncols() == 16 {
        out = swap_middle_quarters(&out, 16);
    } else if out.ncols() >= 64 {
        let mut col_size = 64;
        while col_size <= out.ncols() {
            out = swap_middle_quarters(&out, col_size);
            col_size *= 2;
        }
    }

    // swap chunks
    swap_4x4_chunks(&out)
}

pub fn encode<T: Default + Copy>(array: &Array2<T>) -> Array2<T> {
    let mut out = array.clone();

    // should already be encoded?
    if out.ncols() == 8 {
        return out;
    }

    // swap chunks
    out = swap_4x4_chunks(&out);

    // swap middle quarters around, from biggest to smallest
    if out.ncols() == 16 {
        out = swap_middle_quarters(&out, 16);
    } else if out.ncols() >= 64 {
        let mut col_size = out.ncols();
        while col_size >= 64 {
            out = swap_middle_quarters(&out, col_size);
            col_size /= 2;
        }
    }

    // swizzle(?) 32x4 chunks
    for y in (0..out.nrows()).step_by(4) {
        for x in (0..out.ncols()).step_by(32) {
            let x_max = 32.min(out.ncols());
            let chunk = out.slice(s![y..y + 4, x..x + x_max]);

            let (left, right) = split_vertically(chunk);
            let chunk = join_by_row_parity(left, right);

            let (left, right) = split_vertically(chunk.view());
            let chunk = join_by_column_parity(left, right);

            let (top, bottom) = split_horizontally(chunk.view());
            let chunk = join_by_column_parity(top, bottom);
            out.slice_mut(s![y..y + 4, x..x + x_max]).assign(&chunk);
        }
    }
    out
}

pub fn swap_4x4_chunks<T: Default + Copy>(array: &Array2<T>) -> Array2<T> {
    let mut out = array.clone();
    for y in (2..array.nrows()).step_by(8) {
        for y in y..y + 4 {
            for x in (0..array.ncols()).step_by(8) {
                for x in x..x + 4 {
                    out[[y, x]] = array[[y, x + 4]];
                    out[[y, x + 4]] = array[[y, x]];
                }
            }
        }
    }
    out
}

pub fn swap_middle_quarters<T: Default + Copy>(array: &Array2<T>, columns: usize) -> Array2<T> {
    let quarter = columns / 4;
    let mut out = array.clone();
    for y in 0..array.nrows() {
        for x in (quarter..array.ncols()).step_by(columns) {
            for x in x..x + quarter {
                out[[y, x]] = array[[y, x + quarter]];
                out[[y, x + quarter]] = array[[y, x]];
            }
        }
    }
    out
}

pub fn split_by_column_parity<T: Default + Copy>(array: ArrayView2<T>) -> (Array2<T>, Array2<T>) {
    let even = array.slice(s![.., ..;2]).to_owned();
    let odd = array.slice(s![.., 1..;2]).to_owned();
    (even, odd)
}

pub fn join_by_column_parity<T: Default + Copy>(
    even: ArrayView2<T>,
    odd: ArrayView2<T>,
) -> Array2<T> {
    let mut out = Array2::default((even.nrows(), even.ncols() + odd.ncols()));
    out.slice_mut(s![.., ..;2]).assign(&even);
    out.slice_mut(s![.., 1..;2]).assign(&odd);
    out
}

pub fn split_by_row_parity<T: Default + Copy>(array: ArrayView2<T>) -> (Array2<T>, Array2<T>) {
    let even = array.slice(s![..;2, ..]).to_owned();
    let odd = array.slice(s![1..;2, ..]).to_owned();
    (even, odd)
}

pub fn join_by_row_parity<T: Default + Copy>(
    even: ArrayView2<T>,
    odd: ArrayView2<T>,
) -> Array2<T> {
    let mut out = Array2::default((even.nrows() + odd.nrows(), even.ncols()));
    out.slice_mut(s![..;2, ..]).assign(&even);
    out.slice_mut(s![1..;2, ..]).assign(&odd);
    out
}

pub fn join_horizontally<'a, T: Default + Copy>(
    lhs: ArrayView2<'a, T>,
    rhs: ArrayView2<'a, T>,
) -> Array2<T> {
    ndarray::concatenate(Axis(1), &[lhs, rhs]).unwrap()
}

/// returns (lhs, rhs)
pub fn split_vertically<T: Default + Copy>(
    array: ArrayView2<T>,
) -> (ArrayView2<T>, ArrayView2<T>) {
    array.split_at(Axis(1), array.ncols() / 2)
}

pub fn join_vertically<'a, T: Default + Copy>(
    top: ArrayView2<'a, T>,
    bottom: ArrayView2<'a, T>,
) -> Array2<T> {
    ndarray::concatenate(Axis(0), &[top, bottom]).unwrap()
}

/// returns (top, bottom)
pub fn split_horizontally<T: Default + Copy>(
    array: ArrayView2<T>,
) -> (ArrayView2<T>, ArrayView2<T>) {
    array.split_at(Axis(0), array.nrows() / 2)
}
