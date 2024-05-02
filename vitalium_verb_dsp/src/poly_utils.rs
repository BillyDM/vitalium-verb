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
    cmp::SimdPartialOrd,
    f32x4,
    num::{SimdFloat, SimdInt},
    simd_swizzle, LaneCount, Simd, SimdElement, SupportedLaneCount,
};

#[inline(always)]
pub fn mul_add_f32<const N: usize>(
    a: Simd<f32, N>,
    b: Simd<f32, N>,
    c: Simd<f32, N>,
) -> Simd<f32, N>
where
    LaneCount<N>: SupportedLaneCount,
{
    // TODO: Use fmadd if `portable_simd` gets that functionality.
    a + (b * c)
}

#[inline(always)]
pub fn mul_sub_f32<const N: usize>(
    a: Simd<f32, N>,
    b: Simd<f32, N>,
    c: Simd<f32, N>,
) -> Simd<f32, N>
where
    LaneCount<N>: SupportedLaneCount,
{
    // TODO: Use fmsub if `portable_simd` gets that functionality.
    a - (b * c)
}

#[inline(always)]
pub fn interpolate_f32<const N: usize>(
    from: Simd<f32, N>,
    to: Simd<f32, N>,
    t: Simd<f32, N>,
) -> Simd<f32, N>
where
    LaneCount<N>: SupportedLaneCount,
{
    mul_add_f32(from, to - from, t)
}

#[inline(always)]
pub fn swap_voices_x4<T: SimdElement>(a: Simd<T, 4>) -> Simd<T, 4> {
    simd_swizzle!(a, [2, 3, 0, 1])
}

#[inline(always)]
pub fn swap_stereo_x4<T: SimdElement>(a: Simd<T, 4>) -> Simd<T, 4> {
    simd_swizzle!(a, [1, 0, 3, 2])
}

/// # Quickly rounds an f32 vector towards zero.
///
/// # Safety
///
/// The value must:
///
/// * Not be NaN
/// * Not be infinite
/// * Be representable as an `i32x4`, after truncating off its fractional part
#[inline(always)]
pub unsafe fn simd_trunc_f32_unchecked<const N: usize>(a: Simd<f32, N>) -> Simd<f32, N>
where
    LaneCount<N>: SupportedLaneCount,
{
    let a: Simd<i32, N> = a.to_int_unchecked();
    a.cast()
}

/// # Quickly floors an f32 towards zero.
///
/// # Safety
///
/// The value must:
///
/// * Not be NaN
/// * Not be infinite
/// * Be representable as an `i32x4`, after truncating off its fractional part
#[inline(always)]
pub unsafe fn simd_floor_f32x4_unchecked(a: f32x4) -> f32x4 {
    const V_ZERO: f32x4 = Simd::from_array([0.0; f32x4::LEN]);
    const V_NEG_1: f32x4 = Simd::from_array([-1.0; f32x4::LEN]);

    let truncated = simd_trunc_f32_unchecked(a);

    truncated + truncated.simd_gt(a).select(V_NEG_1, V_ZERO)
}
