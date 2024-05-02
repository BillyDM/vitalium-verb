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

use std::f32::consts::PI;
use std::simd::f32x4;

#[derive(Clone, Copy)]
pub struct OnePoleFilter {
    current_state: f32x4,
    filter_state: f32x4,
}

impl OnePoleFilter {
    pub fn new() -> Self {
        Self {
            current_state: f32x4::splat(0.0),
            filter_state: f32x4::splat(0.0),
        }
    }

    pub fn reset(&mut self) {
        self.current_state = f32x4::splat(0.0);
        self.filter_state = f32x4::splat(0.0);
    }

    #[inline(always)]
    pub fn tick(&mut self, audio_in: f32x4, coefficient: f32x4) -> f32x4 {
        let delta = coefficient * (audio_in - self.filter_state);

        self.filter_state += delta;
        self.current_state = self.filter_state;
        self.filter_state += delta;

        self.current_state
    }

    pub fn compute_coeff(cutoff_frequency: f32x4, sample_rate_recip: f32x4) -> f32x4 {
        const V_PI: f32x4 = f32x4::from_array([PI; f32x4::LEN]);
        const V_1: f32x4 = f32x4::from_array([1.0; f32x4::LEN]);

        let delta_phase = cutoff_frequency * (V_PI * sample_rate_recip);
        let mut a = delta_phase / (delta_phase + V_1);

        for smp in a.as_mut_array().iter_mut() {
            *smp = smp.tan();
        }

        a
    }
}
