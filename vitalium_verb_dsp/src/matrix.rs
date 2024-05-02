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

use crate::poly_utils;
use std::simd::{f32x4, simd_swizzle};

const V_1: f32x4 = f32x4::from_array([1.0; f32x4::LEN]);
const V_2: f32x4 = f32x4::from_array([2.0; f32x4::LEN]);
const V_3: f32x4 = f32x4::from_array([3.0; f32x4::LEN]);
const V_4: f32x4 = f32x4::from_array([4.0; f32x4::LEN]);
const V_5: f32x4 = f32x4::from_array([5.0; f32x4::LEN]);
const V_HALF: f32x4 = f32x4::from_array([0.5; f32x4::LEN]);

#[derive(Default, Debug, Clone, Copy)]
pub struct Matrix {
    pub rows: [f32x4; 4],
}

impl Matrix {
    #[inline(always)]
    pub fn polynomial_interpolation_matrix(t_from: f32x4) -> Self {
        const V_MULT_PREV: f32x4 = f32x4::from_array([-1.0 / 6.0; f32x4::LEN]);
        const V_MULT_FROM: f32x4 = f32x4::from_array([1.0 / 2.0; f32x4::LEN]);
        const V_MULT_TO: f32x4 = f32x4::from_array([-1.0 / 2.0; f32x4::LEN]);
        const V_MULT_NEXT: f32x4 = f32x4::from_array([1.0 / 6.0; f32x4::LEN]);

        let t_prev = t_from + V_1;
        let t_to = t_from - V_1;
        let t_next = t_from - V_2;

        let t_prev_from = t_prev * t_from;
        let t_to_next = t_to * t_next;

        return Self {
            rows: [
                t_from * t_to_next * V_MULT_PREV,
                t_prev * t_to_next * V_MULT_FROM,
                t_prev_from * t_next * V_MULT_TO,
                t_prev_from * t_to * V_MULT_NEXT,
            ],
        };
    }

    #[inline(always)]
    pub fn catmull_interpolation_matrix(t: f32x4) -> Self {
        let half_t = t * V_HALF;
        let half_t2 = t * half_t;
        let half_t3 = half_t2 * t;
        let half_three_t3 = half_t3 * V_3;

        return Self {
            rows: [
                half_t2 * V_2 - half_t3 - half_t,
                poly_utils::mul_sub_f32(half_three_t3, half_t2, V_5) + V_1,
                poly_utils::mul_add_f32(half_t, half_t2, V_4) - half_three_t3,
                half_t3 - half_t2,
            ],
        };
    }

    #[inline(always)]
    pub fn transpose(&mut self) {
        let low0 = simd_swizzle!(self.rows[0], self.rows[1], [0, 4, 1, 5]);
        let low1 = simd_swizzle!(self.rows[2], self.rows[3], [0, 4, 1, 5]);
        let high0 = simd_swizzle!(self.rows[0], self.rows[1], [2, 6, 3, 7]);
        let high1 = simd_swizzle!(self.rows[2], self.rows[3], [2, 6, 3, 7]);

        self.rows[0] = simd_swizzle!(low0, low1, [0, 1, 4, 5]);
        self.rows[1] = simd_swizzle!(low0, low1, [2, 3, 6, 7]);
        self.rows[2] = simd_swizzle!(high0, high1, [0, 1, 4, 5]);
        self.rows[3] = simd_swizzle!(high0, high1, [2, 3, 6, 7]);
    }

    #[inline(always)]
    pub fn multiply_and_sum_rows(&self, other: &Matrix) -> f32x4 {
        let row01 =
            poly_utils::mul_add_f32(self.rows[0] * other.rows[0], self.rows[1], other.rows[1]);
        let row012 = poly_utils::mul_add_f32(row01, self.rows[2], other.rows[2]);
        poly_utils::mul_add_f32(row012, self.rows[3], other.rows[3])
    }

    #[inline(always)]
    pub fn sum_rows(&self) -> f32x4 {
        self.rows[0] + self.rows[1] + self.rows[2] + self.rows[3]
    }
}
