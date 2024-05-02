/* Copyright 2024 Billy Messenger
*
* vitalium-verb is free software: you can redistribute it and/or modify
* it under the terms of the GNU General Public License as published by
* the Free Software Foundation, either version 3 of the License, or
* (at your option) any later version.
*
* vitalium-verb is distributed in the hope that it will be useful,
* but WITHOUT ANY WARRANTY; without even the implied warranty of
* MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
* GNU General Public License for more details.
*
* You should have received a copy of the GNU General Public License
* along with vitalium-verb.  If not, see <http://www.gnu.org/licenses/>.
*/

use std::simd::{
    f32x4, i32x4,
    num::{SimdFloat, SimdInt},
};

use crate::matrix::Matrix;

pub struct StereoMemory {
    left: Vec<f32>,
    right: Vec<f32>,

    size: i32,
    bitmask: i32,
    bitmask_v: i32x4,
    offset: i32,
}

impl StereoMemory {
    pub fn new(size: u32) -> Self {
        let size = size.next_power_of_two() as i32;
        let bitmask = size - 1;

        Self {
            left: vec![0.0; 2 * size as usize],
            right: vec![0.0; 2 * size as usize],

            size,
            bitmask,
            bitmask_v: i32x4::splat(bitmask),
            offset: 0,
        }
    }

    pub fn push(&mut self, sample: f32x4) {
        self.offset = (self.offset + 1) & self.bitmask;

        let sample_array = sample.as_array();

        // SAFETY:
        // The bitmask ensures that the indices are within bounds.
        unsafe {
            *self.left.get_unchecked_mut(self.offset as usize) = sample_array[0];
            *self
                .left
                .get_unchecked_mut((self.offset + self.size) as usize) = sample_array[0];

            *self.right.get_unchecked_mut(self.offset as usize) = sample_array[1];
            *self
                .right
                .get_unchecked_mut((self.offset + self.size) as usize) = sample_array[1];
        }

        /*
        self.left[self.offset as usize] = sample_array[0];
        self.left[(self.offset + self.size) as usize] = sample_array[0];
        self.right[self.offset as usize] = sample_array[1];
        self.right[(self.offset + self.size) as usize] = sample_array[1];
        */

        debug_assert!(sample_array[0].is_finite());
        debug_assert!(sample_array[1].is_finite());
    }

    pub fn clear(&mut self) {
        self.left.fill(0.0);
        self.right.fill(0.0);
    }

    /// # Safety
    ///
    /// The value `past` must:
    ///
    /// * Not be NaN
    /// * Not be infinite
    /// * Be representable as an `i32x4`, after truncating off its fractional part
    #[inline(always)]
    pub unsafe fn get_interpolated(&self, past: f32x4) -> f32x4 {
        const VF32_0: f32x4 = f32x4::from_array([0.0; f32x4::LEN]);
        const VF32_1: f32x4 = f32x4::from_array([1.0; f32x4::LEN]);
        const VI32_2: i32x4 = i32x4::from_array([2; i32x4::LEN]);

        let past_index: i32x4 = past.to_int_unchecked();
        let past_truncated: f32x4 = past_index.cast();

        let t = past_truncated - past + VF32_1;
        let interpolation_matrix = Matrix::catmull_interpolation_matrix(t);

        let indices = (i32x4::splat(self.offset) - past_index - VI32_2) & self.bitmask_v;
        let indices = indices.as_array();

        // SAFETY:
        // The bitmask ensures that the indices are within bounds.
        let (row0_slice, row1_slice) = unsafe {
            (
                std::slice::from_raw_parts(self.left.as_ptr().add(indices[0] as usize), 4),
                std::slice::from_raw_parts(self.right.as_ptr().add(indices[1] as usize), 4),
            )
        };

        // TODO: Make sure the internal check in `f32x4::from_slice` is being
        // properly elided (the check is to see if the length of the slice is
        // at least 4).
        let mut value_matrix = Matrix {
            rows: [
                f32x4::from_slice(row0_slice),
                f32x4::from_slice(row1_slice),
                VF32_0,
                VF32_0,
            ],
        };
        value_matrix.transpose();

        interpolation_matrix.multiply_and_sum_rows(&value_matrix)
    }
}
