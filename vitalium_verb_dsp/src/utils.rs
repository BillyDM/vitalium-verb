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

use std::f32::consts::FRAC_PI_2;

#[inline]
/// Convert decibels to amplitude.
pub fn db_to_amplitude(dbs: f32) -> f32 {
    10.0f32.powf(dbs * 0.05)
}

#[inline]
pub fn equal_power_fade(normal: f32) -> f32 {
    (normal * FRAC_PI_2).cos()
}

#[inline]
pub fn equal_power_fade_inverse(normal: f32) -> f32 {
    ((normal - 1.0) * FRAC_PI_2).cos()
}
