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

use std::f32::consts::{PI, TAU};
use std::simd::num::SimdFloat;
use std::simd::{f32x4, i32x4};

use crate::matrix::Matrix;
use crate::one_pole_filter::OnePoleFilter;
use crate::params::ReverbParams;
use crate::stereo_memory::StereoMemory;
use crate::{poly_utils, utils};

pub const MAX_BLOCK_SIZE: usize = 128;

// ------------------------------------------------------------------------------------------
// Private constants

const T60_AMPLITUDE: f32 = 0.001;
const ALLPASS_FEEDBACK: f32 = 0.6;
const MIN_DELAY: f32 = 3.0;

const SAMPLE_DELAY_MULTIPLIER: f32 = 0.05;
const SAMPLE_INCREMENT_MULTIPLIER: f32 = 0.05;

const BASE_SAMPLE_RATE: f32 = 44_100.0;
const MAX_SAMPLE_RATE: f32 = 192_000.0;

const MAX_CHORUS_DRIFT: f32 = 2500.0;

const NETWORK_SIZE: usize = 16;
const NETWORK_CONTAINERS: usize = NETWORK_SIZE / f32x4::LEN;

const BASE_FEEDBACK_BITS: i32 = 14;
const EXTRA_LOOKUP_SAMPLE: i32 = 1;
const BASE_ALLPASS_BITS: i32 = 10;

const MIN_SIZE_POWER: i32 = -3;
const MAX_SIZE_POWER: i32 = 1;
const SIZE_POWER_RANGE: f32 = (MAX_SIZE_POWER - MIN_SIZE_POWER) as f32;

const FEED_FORWARD_SCALE: f32 = 0.125;

const ALLPASS_DELAYS: [i32x4; NETWORK_CONTAINERS] = [
    i32x4::from_array([1001, 799, 933, 876]),
    i32x4::from_array([895, 807, 907, 853]),
    i32x4::from_array([957, 1019, 711, 567]),
    i32x4::from_array([833, 779, 663, 997]),
];

const FEEDBACK_DELAYS: [f32x4; NETWORK_CONTAINERS] = [
    f32x4::from_array([6753.2, 9278.4, 7704.5, 11328.5]),
    f32x4::from_array([9701.12, 5512.5, 8480.45, 5638.65]),
    f32x4::from_array([3120.73, 3429.5, 3626.37, 7713.52]),
    f32x4::from_array([4521.54, 6518.97, 5265.56, 5630.25]),
];

const NETWORK_OFFSET: f32 = 2.0 * PI / NETWORK_SIZE as f32;

const V_0: f32x4 = f32x4::from_array([0.0; f32x4::LEN]);
const V_INPUT_SCALE: f32x4 = f32x4::from_array([0.25; f32x4::LEN]);
const V_ALLPASS_FEEDBACK: f32x4 = f32x4::from_array([ALLPASS_FEEDBACK; f32x4::LEN]);
const V_NEG_ONE_HALF: f32x4 = f32x4::from_array([-0.5; f32x4::LEN]);
const V_FEED_FORWARD_SCALE: f32x4 = f32x4::from_array([FEED_FORWARD_SCALE; f32x4::LEN]);
const V_DELAY_OFFSET: i32x4 = i32x4::from_array([0, -1, -2, -3]);
const V_POLY_LEN_I32: i32x4 = i32x4::from_array([f32x4::LEN as i32; f32x4::LEN]);
const V_POLY_LEN_F32: f32x4 = f32x4::from_array([f32x4::LEN as f32; f32x4::LEN]);
const V_CHORUS_PHASE_OFFSET: f32x4 = f32x4::from_array([0.0, 1.0, 2.0, 3.0]);
const V_8: f32x4 = f32x4::from_array([8.0; f32x4::LEN]);
const V_MIN_DELAY: f32x4 = f32x4::from_array([MIN_DELAY; f32x4::LEN]);
const V_MAX_SAMPLE_RATE: f32x4 = f32x4::from_array([MAX_SAMPLE_RATE; f32x4::LEN]);
const V_NETWORK_OFFSET: f32x4 = f32x4::from_array([NETWORK_OFFSET; f32x4::LEN]);
const V_SAMPLE_INCREMENT_MULTIPLIER: f32x4 =
    f32x4::from_array([SAMPLE_INCREMENT_MULTIPLIER; f32x4::LEN]);
const V_SAMPLE_DELAY_MULTIPLIER: f32x4 = f32x4::from_array([SAMPLE_DELAY_MULTIPLIER; f32x4::LEN]);
const V_TAU: f32x4 = f32x4::from_array([TAU; f32x4::LEN]);

// ------------------------------------------------------------------------------------------
// Reverb struct

/// A reverb module based on the reverb module from the
/// [Vital](https://github.com/mtytel/vital)/[Vitalium](https://github.com/DISTRHO/DISTRHO-Ports/tree/5c55f9445ee6ff75d53c7f8601fc341d200aa4a0/ports-juce6.0/vitalium)
/// synthesizer.
///
/// The reverb must be initialized by calling `Reverb::init()` before processing.
pub struct Reverb {
    stereo_memory: StereoMemory,

    allpass_memories: [Vec<f32>; NETWORK_CONTAINERS],
    feedback_memories: [[Vec<f32>; f32x4::LEN]; NETWORK_CONTAINERS],
    decays: [f32x4; NETWORK_CONTAINERS],

    pre_low_filter: OnePoleFilter,
    pre_high_filter: OnePoleFilter,

    low_shelf_filters: [OnePoleFilter; NETWORK_CONTAINERS],
    high_shelf_filters: [OnePoleFilter; NETWORK_CONTAINERS],

    pre_low_coeff: f32x4,
    pre_high_coeff: f32x4,
    low_shelf_coeff: f32x4,
    high_shelf_coeff: f32x4,
    low_shelf_amp: f32x4,
    high_shelf_amp: f32x4,

    chorus_phase: f32,
    chorus_amount: f32x4,
    sample_delay: f32x4,
    sample_delay_increment: f32x4,
    dry_amp: f32x4,
    wet_amp: f32x4,

    width_coeff: f32,

    write_index: i32,
    max_feedback_size: usize,
    feedback_mask: i32,
    feedback_mask_v: i32x4,
    allpass_mask: i32,
    allpass_mask_v: i32x4,
    delay_offset_v: i32x4,
    allpass_offsets: [i32x4; NETWORK_CONTAINERS],
    delays: [f32x4; NETWORK_CONTAINERS],

    prev_pre_low_cut_hz: f32,
    prev_pre_high_cut_hz: f32,
    prev_low_shelf_cut_hz: f32,
    prev_high_shelf_cut_hz: f32,
    prev_size_val: f32,
    prev_decay_val: f32,
    prev_chorus_freq_hz: f32,
    prev_mix_val: f32,
    prev_low_shelf_gain_db: f32,
    prev_high_shelf_gain_db: f32,

    size_mult_v: f32x4,
    chorus_increment_real_v: f32x4,
    chorus_increment_imaginary_v: f32x4,

    sample_rate: f32,
    sample_rate_recip: f32,
    sample_rate_recip_v: f32x4,
    sample_rate_ratio: f32,
    sample_rate_ratio_v: f32x4,
    buffer_scale: i32,

    did_init: bool,
}

impl Default for Reverb {
    fn default() -> Self {
        Self {
            stereo_memory: StereoMemory::new(MAX_SAMPLE_RATE as u32),

            allpass_memories: Default::default(),
            feedback_memories: Default::default(),
            decays: Default::default(),

            pre_low_filter: OnePoleFilter::new(),
            pre_high_filter: OnePoleFilter::new(),

            low_shelf_filters: [OnePoleFilter::new(); NETWORK_CONTAINERS],
            high_shelf_filters: [OnePoleFilter::new(); NETWORK_CONTAINERS],

            pre_low_coeff: f32x4::splat(0.1),
            pre_high_coeff: f32x4::splat(0.1),
            low_shelf_coeff: f32x4::splat(0.1),
            high_shelf_coeff: f32x4::splat(0.1),

            low_shelf_amp: V_0,
            high_shelf_amp: V_0,

            chorus_phase: 0.0,
            chorus_amount: V_0,
            sample_delay: f32x4::splat(MIN_DELAY),
            sample_delay_increment: V_0,

            dry_amp: V_0,
            wet_amp: V_0,

            width_coeff: 0.5,

            write_index: 0,
            max_feedback_size: 0,
            feedback_mask: 0,
            feedback_mask_v: i32x4::splat(0),
            allpass_mask: 0,
            allpass_mask_v: i32x4::splat(0),
            delay_offset_v: i32x4::splat(0),
            allpass_offsets: [i32x4::splat(0); NETWORK_CONTAINERS],
            delays: [V_0; NETWORK_CONTAINERS],

            prev_pre_low_cut_hz: 0.0,
            prev_pre_high_cut_hz: 0.0,
            prev_low_shelf_cut_hz: 0.0,
            prev_high_shelf_cut_hz: 0.0,
            prev_size_val: -1.0,
            prev_decay_val: -1.0,
            prev_chorus_freq_hz: -1.0,
            prev_mix_val: -1.0,
            prev_low_shelf_gain_db: -1000.0,
            prev_high_shelf_gain_db: -1000.0,

            size_mult_v: V_0,
            chorus_increment_real_v: V_0,
            chorus_increment_imaginary_v: V_0,

            sample_rate: 0.0,
            sample_rate_ratio: 0.0,
            sample_rate_recip: 0.0,
            sample_rate_recip_v: V_0,
            sample_rate_ratio_v: V_0,
            buffer_scale: 0,

            did_init: false,
        }
    }
}

impl Reverb {
    /// Initialize the reverb with the given sample rate.
    pub fn init(&mut self, sample_rate: f32) {
        *self = Self::default();

        self.sample_rate = sample_rate;
        self.sample_rate_recip = sample_rate.recip();
        self.sample_rate_recip_v = f32x4::splat(self.sample_rate_recip);
        self.sample_rate_ratio = get_sample_rate_ratio(sample_rate);
        self.sample_rate_ratio_v = f32x4::splat(self.sample_rate_ratio);

        // ----------------------------------------------------------------------------------
        // Calculate the needed size for feedback state buffers

        self.buffer_scale = get_buffer_scale(sample_rate);
        let buffer_scale_v = i32x4::splat(self.buffer_scale);
        self.max_feedback_size =
            (self.buffer_scale * (1 << (BASE_FEEDBACK_BITS + MAX_SIZE_POWER))) as usize;
        self.feedback_mask = (self.max_feedback_size as i32) - 1;
        self.feedback_mask_v = i32x4::splat(self.feedback_mask);

        self.delay_offset_v = {
            let mut delay_offset = V_DELAY_OFFSET;
            if self.buffer_scale != 0 {
                delay_offset = delay_offset + V_POLY_LEN_I32;
            }
            delay_offset
        };

        // ----------------------------------------------------------------------------------
        // Allocate capacity for the feedback state buffers

        for memory_v in self.feedback_memories.iter_mut() {
            for memory in memory_v.iter_mut() {
                *memory = vec![
                    0.0;
                    self.max_feedback_size
                        + ((EXTRA_LOOKUP_SAMPLE as usize) * f32x4::LEN)
                ];
            }
        }

        // ----------------------------------------------------------------------------------
        // Calculate the needed size for allpass state buffers

        let max_allpass_size = self.buffer_scale * (1 << BASE_ALLPASS_BITS) * f32x4::LEN as i32;
        self.allpass_mask = max_allpass_size - 1;
        self.allpass_mask_v = i32x4::splat(self.allpass_mask);

        self.allpass_offsets = [
            poly_utils::swap_stereo_x4(
                ALLPASS_DELAYS[0] * buffer_scale_v * V_POLY_LEN_I32 + self.delay_offset_v,
            ),
            poly_utils::swap_stereo_x4(
                ALLPASS_DELAYS[1] * buffer_scale_v * V_POLY_LEN_I32 + self.delay_offset_v,
            ),
            poly_utils::swap_stereo_x4(
                ALLPASS_DELAYS[2] * buffer_scale_v * V_POLY_LEN_I32 + self.delay_offset_v,
            ),
            poly_utils::swap_stereo_x4(
                ALLPASS_DELAYS[3] * buffer_scale_v * V_POLY_LEN_I32 + self.delay_offset_v,
            ),
        ];

        // ----------------------------------------------------------------------------------
        // Allocate capacity for the allpass state buffers

        for memory in self.allpass_memories.iter_mut() {
            *memory = vec![0.0; max_allpass_size as usize];
        }

        self.write_index &= self.feedback_mask;

        self.did_init = true;
    }

    /// Returns the estimated length of the reverb tail in units of samples.
    pub fn tail_samples(&self, decay_seconds: f32) -> u32 {
        // TODO: Be more exact instead of giving an estimate?
        (decay_seconds * 2.0 * self.sample_rate).ceil() as u32
    }

    /// Process the given buffers with the given parameters.
    ///
    /// Note, parameters are only linearly smoothed over a maximum 128 frame period.
    /// If you want more smoothing than that, call this method multiple times in
    /// chunks of 128 frames.
    ///
    /// # Panics
    ///
    /// This will panic if:
    /// * The `left` and `right` buffers are not the same length
    /// * `Reverb::init()` has not been called at-least once
    pub fn process(&mut self, left: &mut [f32], right: &mut [f32], params: &ReverbParams) {
        assert!(self.did_init);

        // TODO: Smooth parameters over a longer period.

        let total_frames = left.len();
        let right = &mut right[0..total_frames];

        // Process in blocks
        let mut processed_frames = 0;
        while processed_frames < total_frames {
            let frames = (total_frames - processed_frames).min(MAX_BLOCK_SIZE);

            self.process_block(
                &mut left[processed_frames..processed_frames + frames],
                &mut right[processed_frames..processed_frames + frames],
                params,
            );

            processed_frames += frames;
        }
    }

    fn process_block(&mut self, left: &mut [f32], right: &mut [f32], params: &ReverbParams) {
        // ----------------------------------------------------------------------------------
        // Prepare constants

        let frames = left.len();

        let tick_increment = 1.0 / frames as f32;
        let tick_increment_v = f32x4::splat(tick_increment);

        // ----------------------------------------------------------------------------------
        // Wrap feedback memory buffers

        for feedback_memory_v in self.feedback_memories.iter_mut() {
            for buffer in feedback_memory_v.iter_mut() {
                // SAFETY:
                // The `init()` function has ensured that these buffers have the correct
                // length, and we have asserted that the user called the init function
                // at least once.
                unsafe {
                    *buffer.get_unchecked_mut(0) = *buffer.get_unchecked(self.max_feedback_size);
                    *buffer.get_unchecked_mut(self.max_feedback_size + 1) =
                        *buffer.get_unchecked(1);
                    *buffer.get_unchecked_mut(self.max_feedback_size + 2) =
                        *buffer.get_unchecked(2);
                    *buffer.get_unchecked_mut(self.max_feedback_size + 3) =
                        *buffer.get_unchecked(3);
                }
            }
        }

        // ----------------------------------------------------------------------------------
        // Prepare filter cutoff parameters

        let prepare_filter_param = |new_cut: f32,
                                    prev_cut: &mut f32,
                                    coeff: &mut f32x4|
         -> (f32x4, f32x4) {
            let curr_coeff = *coeff;
            let new_cut =
                new_cut.clamp(ReverbParams::MIN_CUTOFF_FREQ, ReverbParams::MAX_CUTOFF_FREQ);

            // Only recompute the coefficients if the cutoff has changed.
            // The original Vitalium code did not do this.
            if *prev_cut != new_cut {
                *prev_cut = new_cut;
                *coeff =
                    OnePoleFilter::compute_coeff(f32x4::splat(new_cut), self.sample_rate_recip_v);

                (curr_coeff, (*coeff - curr_coeff) * tick_increment_v)
            } else {
                (curr_coeff, V_0)
            }
        };

        let (mut current_pre_low_coeff, delta_pre_low_coeff) = prepare_filter_param(
            params.pre_low_cut_hz,
            &mut self.prev_pre_low_cut_hz,
            &mut self.pre_low_coeff,
        );
        let (mut current_pre_high_coeff, delta_pre_high_coeff) = prepare_filter_param(
            params.pre_high_cut_hz,
            &mut self.prev_pre_high_cut_hz,
            &mut self.pre_high_coeff,
        );

        let (mut current_low_shelf_coeff, delta_low_shelf_coeff) = prepare_filter_param(
            params.low_shelf_cut_hz,
            &mut self.prev_low_shelf_cut_hz,
            &mut self.low_shelf_coeff,
        );
        let (mut current_high_shelf_coeff, delta_high_shelf_coeff) = prepare_filter_param(
            params.high_shelf_cut_hz,
            &mut self.prev_high_shelf_cut_hz,
            &mut self.high_shelf_coeff,
        );

        // ----------------------------------------------------------------------------------
        // Prepare mix parameter

        let mut current_dry_amp = self.dry_amp;
        let mut current_wet_amp = self.wet_amp;

        let mix_val = params.mix.clamp(0.0, 1.0);

        // Only recompute amps if mix has changed.
        let (delta_dry_amp, delta_wet_amp) = if self.prev_mix_val != mix_val {
            self.prev_mix_val = mix_val;

            self.dry_amp = f32x4::splat(utils::equal_power_fade(mix_val));
            self.wet_amp = f32x4::splat(utils::equal_power_fade_inverse(mix_val));

            (
                (self.dry_amp - current_dry_amp) * tick_increment_v,
                (self.wet_amp - current_wet_amp) * tick_increment_v,
            )
        } else {
            (V_0, V_0)
        };

        // ----------------------------------------------------------------------------------
        // Prepare shelf gain parameters

        let low_shelf_gain_db = params.low_shelf_gain_db.clamp(
            ReverbParams::MIN_SHELF_GAIN_DB,
            ReverbParams::MAX_SHELF_GAIN_DB,
        );
        let high_shelf_gain_db = params.high_shelf_gain_db.clamp(
            ReverbParams::MIN_SHELF_GAIN_DB,
            ReverbParams::MAX_SHELF_GAIN_DB,
        );

        let mut current_low_shelf_amp = self.low_shelf_amp;
        let mut current_high_shelf_amp = self.high_shelf_amp;

        // Only recompute amplitudes if parameters have changed.
        let delta_low_shelf_amp = if self.prev_low_shelf_gain_db != low_shelf_gain_db {
            self.prev_low_shelf_gain_db = low_shelf_gain_db;

            self.low_shelf_amp = f32x4::splat(1.0 - utils::db_to_amplitude(low_shelf_gain_db));

            (self.low_shelf_amp - current_low_shelf_amp) * tick_increment_v
        } else {
            V_0
        };
        let delta_high_shelf_amp = if self.prev_high_shelf_gain_db != high_shelf_gain_db {
            self.prev_high_shelf_gain_db = high_shelf_gain_db;

            self.high_shelf_amp = f32x4::splat(utils::db_to_amplitude(high_shelf_gain_db));

            (self.high_shelf_amp - current_high_shelf_amp) * tick_increment_v
        } else {
            V_0
        };

        // ----------------------------------------------------------------------------------
        // Prepare width parameter

        let mut current_width_coeff = self.width_coeff;
        self.width_coeff = (params.width.clamp(-1.0, 1.0) + 1.0) * 0.5;
        let delta_width_coeff = (self.width_coeff - current_width_coeff) * tick_increment;

        // ----------------------------------------------------------------------------------
        // Prepare size/decay parameters

        let mut current_decays = self.decays;

        let size_val = params.size.clamp(0.0, 1.0);
        let decay_val = params.decay.clamp(
            ReverbParams::MIN_DECAY_SECONDS,
            ReverbParams::MAX_DECAY_SECONDS,
        );

        // Only recompute size_mult, decay, and delays if the parameters have changed.
        let delta_decays = if self.prev_size_val != size_val || self.prev_decay_val != decay_val {
            self.prev_decay_val = decay_val;

            if self.prev_size_val != size_val {
                self.prev_size_val = size_val;

                // In the original Vitalium code, this power function was implemented as
                // a complex series of SIMD methods. But since every value in this vector
                // is the same, I've opted to use the much simpler scalar method.
                self.size_mult_v =
                    f32x4::splat(2.0f32.powf(size_val * SIZE_POWER_RANGE + MIN_SIZE_POWER as f32));
            }

            let decay_samples = f32x4::splat(decay_val * BASE_SAMPLE_RATE);
            let decay_period = self.size_mult_v / decay_samples;

            for (decay, feedback_delay) in self.decays.iter_mut().zip(FEEDBACK_DELAYS) {
                *decay = feedback_delay * decay_period;
                for e in decay.as_mut_array().iter_mut() {
                    *e = T60_AMPLITUDE.powf(*e);
                }
            }

            self.delays = [
                self.size_mult_v * FEEDBACK_DELAYS[0] * self.sample_rate_ratio_v,
                self.size_mult_v * FEEDBACK_DELAYS[1] * self.sample_rate_ratio_v,
                self.size_mult_v * FEEDBACK_DELAYS[2] * self.sample_rate_ratio_v,
                self.size_mult_v * FEEDBACK_DELAYS[3] * self.sample_rate_ratio_v,
            ];

            [
                (self.decays[0] - current_decays[0]) * tick_increment_v,
                (self.decays[1] - current_decays[1]) * tick_increment_v,
                (self.decays[2] - current_decays[2]) * tick_increment_v,
                (self.decays[3] - current_decays[3]) * tick_increment_v,
            ]
        } else {
            [V_0; NETWORK_CONTAINERS]
        };

        // ----------------------------------------------------------------------------------
        // Prepare chorus parameters

        let chorus_freq = params
            .chorus_freq_hz
            .clamp(ReverbParams::MIN_CHORUS_FREQ, ReverbParams::MAX_CHORUS_FREQ);
        let chorus_phase_increment = chorus_freq * self.sample_rate_recip;

        // Only recompute chorus increments if the chorus frequency has changed.
        if self.prev_chorus_freq_hz != chorus_freq {
            self.prev_chorus_freq_hz = chorus_freq;

            self.chorus_increment_real_v = f32x4::splat((chorus_phase_increment * TAU).cos());
            self.chorus_increment_imaginary_v = f32x4::splat((chorus_phase_increment * TAU).sin());
        }

        let phase_offset = V_CHORUS_PHASE_OFFSET * V_NETWORK_OFFSET;
        let container_phase = phase_offset + f32x4::splat(self.chorus_phase) * V_TAU;
        self.chorus_phase += frames as f32 * chorus_phase_increment;
        self.chorus_phase -= self.chorus_phase.floor();

        let mut current_chorus_real = {
            let mut p = container_phase.clone();
            for phase in p.as_mut_array().iter_mut() {
                *phase = phase.cos();
            }
            p
        };
        let mut current_chorus_imaginary = {
            let mut p = container_phase.clone();
            for phase in p.as_mut_array().iter_mut() {
                *phase = phase.sin();
            }
            p
        };

        let mut current_chorus_amount = self.chorus_amount;
        self.chorus_amount = f32x4::splat(
            params.chorus_amount.clamp(0.0, 1.0) * MAX_CHORUS_DRIFT * self.sample_rate_ratio,
        );
        self.chorus_amount = self
            .chorus_amount
            .simd_min(self.delays[0] - V_8 * V_POLY_LEN_F32);
        self.chorus_amount = self
            .chorus_amount
            .simd_min(self.delays[1] - V_8 * V_POLY_LEN_F32);
        self.chorus_amount = self
            .chorus_amount
            .simd_min(self.delays[2] - V_8 * V_POLY_LEN_F32);
        self.chorus_amount = self
            .chorus_amount
            .simd_min(self.delays[3] - V_8 * V_POLY_LEN_F32);
        let delta_chorus_amount = (self.chorus_amount - current_chorus_amount) * tick_increment_v;

        // ----------------------------------------------------------------------------------
        // Prepare delay parameter

        let mut current_sample_delay = self.sample_delay;
        let mut current_delay_increment = self.sample_delay_increment;
        let end_target =
            current_sample_delay + current_delay_increment * f32x4::splat(frames as f32);
        let target_delay = {
            let target_delay = (params.delay * self.sample_rate).clamp(MIN_DELAY, MAX_SAMPLE_RATE);
            poly_utils::interpolate_f32(
                self.sample_delay,
                f32x4::splat(target_delay),
                V_SAMPLE_DELAY_MULTIPLIER,
            )
        };
        let makeup_delay = target_delay - end_target;
        let delta_delay_increment = makeup_delay
            / f32x4::splat(0.5 * frames as f32 * frames as f32)
            * V_SAMPLE_INCREMENT_MULTIPLIER;

        // ----------------------------------------------------------------------------------
        // Process loop

        // Hint to the compiler to optimize loop.
        let right = &mut right[0..frames];

        for (l, r) in left.iter_mut().zip(right.iter_mut()) {
            // ------------------------------------------------------------------------------
            // Tick chorus

            current_chorus_amount += delta_chorus_amount;
            current_chorus_real = current_chorus_real * self.chorus_increment_real_v
                - current_chorus_imaginary * self.chorus_increment_imaginary_v;
            current_chorus_imaginary = current_chorus_imaginary * self.chorus_increment_real_v
                + current_chorus_real * self.chorus_increment_imaginary_v;

            // ------------------------------------------------------------------------------
            // Apply chorus by offsetting the feedback offsets

            let feedback_offsets = [
                self.delays[0] + current_chorus_real * current_chorus_amount,
                self.delays[1] - current_chorus_real * current_chorus_amount,
                self.delays[2] + current_chorus_imaginary * current_chorus_amount,
                self.delays[3] - current_chorus_imaginary * current_chorus_amount,
            ];

            // ------------------------------------------------------------------------------
            // Read from the feedback memory

            let feedback_reads = [
                self.read_feedback_interpolated(&self.feedback_memories[0], feedback_offsets[0]),
                self.read_feedback_interpolated(&self.feedback_memories[1], feedback_offsets[1]),
                self.read_feedback_interpolated(&self.feedback_memories[2], feedback_offsets[2]),
                self.read_feedback_interpolated(&self.feedback_memories[3], feedback_offsets[3]),
            ];

            // ------------------------------------------------------------------------------
            // Get audio input

            let input = f32x4::from_array([*l, *r, *l, *r]);

            // ------------------------------------------------------------------------------
            // Apply pre-filters to input

            let filtered_input = self.pre_high_filter.tick(input, current_pre_high_coeff);
            let filtered_input =
                self.pre_low_filter.tick(input, current_pre_low_coeff) - filtered_input;
            let scaled_input = filtered_input * V_INPUT_SCALE;

            // ------------------------------------------------------------------------------
            // Read the current state of allpass filters

            let allpass_reads = [
                self.read_allpass(&self.allpass_memories[0], self.allpass_offsets[0]),
                self.read_allpass(&self.allpass_memories[1], self.allpass_offsets[1]),
                self.read_allpass(&self.allpass_memories[2], self.allpass_offsets[2]),
                self.read_allpass(&self.allpass_memories[3], self.allpass_offsets[3]),
            ];

            // ------------------------------------------------------------------------------
            // Tick the allpass filters

            let allpass_delay_inputs = [
                feedback_reads[0] - allpass_reads[0] * V_ALLPASS_FEEDBACK,
                feedback_reads[1] - allpass_reads[1] * V_ALLPASS_FEEDBACK,
                feedback_reads[2] - allpass_reads[2] * V_ALLPASS_FEEDBACK,
                feedback_reads[3] - allpass_reads[3] * V_ALLPASS_FEEDBACK,
            ];

            // ------------------------------------------------------------------------------
            // Store the new state into the allpass memory

            let allpass_write_index =
                ((self.write_index * f32x4::LEN as i32) & self.allpass_mask) as usize;
            for (allpass_memory, delay_input) in
                self.allpass_memories.iter_mut().zip(allpass_delay_inputs)
            {
                let s = scaled_input + delay_input;

                // SAFETY:
                // The bitmask ensures that the index is within bounds.
                let memory_slice = unsafe {
                    std::slice::from_raw_parts_mut(
                        allpass_memory.as_mut_ptr().add(allpass_write_index),
                        4,
                    )
                };

                // TODO: Make sure the internal check in `f32x4::copy_to_slice` is being
                // properly elided (the check is to see if the length of the slice is
                // at least 4).
                s.copy_to_slice(memory_slice);
            }

            // ------------------------------------------------------------------------------
            // Apply the first set of allpass filters

            let mut allpass_outputs = Matrix {
                rows: [
                    allpass_reads[0] + allpass_delay_inputs[0] * V_ALLPASS_FEEDBACK,
                    allpass_reads[1] + allpass_delay_inputs[1] * V_ALLPASS_FEEDBACK,
                    allpass_reads[2] + allpass_delay_inputs[2] * V_ALLPASS_FEEDBACK,
                    allpass_reads[3] + allpass_delay_inputs[3] * V_ALLPASS_FEEDBACK,
                ],
            };

            let total_rows = allpass_outputs.sum_rows();
            let other_feedback = poly_utils::mul_add_f32(
                f32x4::splat(total_rows.reduce_sum() * 0.25),
                total_rows,
                V_NEG_ONE_HALF,
            );

            let mut writes = Matrix {
                rows: [
                    other_feedback + allpass_outputs.rows[0],
                    other_feedback + allpass_outputs.rows[1],
                    other_feedback + allpass_outputs.rows[2],
                    other_feedback + allpass_outputs.rows[3],
                ],
            };

            allpass_outputs.transpose();
            let adjacent_feedback = (allpass_outputs.rows[0]
                + allpass_outputs.rows[1]
                + allpass_outputs.rows[2]
                + allpass_outputs.rows[3])
                * V_NEG_ONE_HALF;

            writes.rows[0] += f32x4::splat(adjacent_feedback[0]);
            writes.rows[1] += f32x4::splat(adjacent_feedback[1]);
            writes.rows[2] += f32x4::splat(adjacent_feedback[2]);
            writes.rows[3] += f32x4::splat(adjacent_feedback[3]);

            // ------------------------------------------------------------------------------
            // Apply the high and low shelf filters to the feedback signal

            let high_filtered_vals = [
                self.high_shelf_filters[0].tick(writes.rows[0], current_high_shelf_coeff),
                self.high_shelf_filters[1].tick(writes.rows[1], current_high_shelf_coeff),
                self.high_shelf_filters[2].tick(writes.rows[2], current_high_shelf_coeff),
                self.high_shelf_filters[3].tick(writes.rows[3], current_high_shelf_coeff),
            ];

            writes.rows[0] = high_filtered_vals[0]
                + current_high_shelf_amp * (writes.rows[0] - high_filtered_vals[0]);
            writes.rows[1] = high_filtered_vals[1]
                + current_high_shelf_amp * (writes.rows[1] - high_filtered_vals[1]);
            writes.rows[2] = high_filtered_vals[2]
                + current_high_shelf_amp * (writes.rows[2] - high_filtered_vals[2]);
            writes.rows[3] = high_filtered_vals[3]
                + current_high_shelf_amp * (writes.rows[3] - high_filtered_vals[3]);

            let low_filtered_vals = [
                self.low_shelf_filters[0].tick(writes.rows[0], current_low_shelf_coeff),
                self.low_shelf_filters[1].tick(writes.rows[1], current_low_shelf_coeff),
                self.low_shelf_filters[2].tick(writes.rows[2], current_low_shelf_coeff),
                self.low_shelf_filters[3].tick(writes.rows[3], current_low_shelf_coeff),
            ];

            writes.rows[0] -= low_filtered_vals[0] * current_low_shelf_amp;
            writes.rows[1] -= low_filtered_vals[1] * current_low_shelf_amp;
            writes.rows[2] -= low_filtered_vals[2] * current_low_shelf_amp;
            writes.rows[3] -= low_filtered_vals[3] * current_low_shelf_amp;

            // ------------------------------------------------------------------------------
            // Increment the decay parameter

            current_decays[0] += delta_decays[0];
            current_decays[1] += delta_decays[1];
            current_decays[2] += delta_decays[2];
            current_decays[3] += delta_decays[3];

            // ------------------------------------------------------------------------------
            // Store the signal in the feedback memory

            let mut stores = Matrix {
                rows: [
                    current_decays[0] * writes.rows[0],
                    current_decays[1] * writes.rows[1],
                    current_decays[2] * writes.rows[2],
                    current_decays[3] * writes.rows[3],
                ],
            };

            let feedback_write_index = (self.write_index + EXTRA_LOOKUP_SAMPLE) as usize;
            for (feedback_memory_v, store_v) in self.feedback_memories.iter_mut().zip(stores.rows) {
                let store_array = store_v.as_array();
                for (feedback_memory, store) in feedback_memory_v.iter_mut().zip(store_array) {
                    // SAFETY:
                    // The bitmask ensures that `self.write_index` is within bounds.
                    unsafe {
                        *feedback_memory.get_unchecked_mut(feedback_write_index) = *store;
                    }
                }
            }

            // ------------------------------------------------------------------------------
            // Apply next set of allpass filters

            let total_allpass = stores.sum_rows();

            let other_feedback_allpass = poly_utils::mul_add_f32(
                f32x4::splat(total_allpass.reduce_sum() * 0.25),
                total_allpass,
                V_NEG_ONE_HALF,
            );

            let mut feed_forward_vals = [
                other_feedback_allpass + stores.rows[0],
                other_feedback_allpass + stores.rows[1],
                other_feedback_allpass + stores.rows[2],
                other_feedback_allpass + stores.rows[3],
            ];

            stores.transpose();
            let adjacent_feedback_allpass = stores.sum_rows() * V_NEG_ONE_HALF;

            feed_forward_vals[0] += f32x4::splat(adjacent_feedback_allpass[0]);
            feed_forward_vals[1] += f32x4::splat(adjacent_feedback_allpass[1]);
            feed_forward_vals[2] += f32x4::splat(adjacent_feedback_allpass[2]);
            feed_forward_vals[3] += f32x4::splat(adjacent_feedback_allpass[3]);

            let mut total = writes.sum_rows();
            total += (feed_forward_vals[0] * current_decays[0]
                + feed_forward_vals[1] * current_decays[1]
                + feed_forward_vals[2] * current_decays[2]
                + feed_forward_vals[3] * current_decays[3])
                * V_FEED_FORWARD_SCALE;

            // ------------------------------------------------------------------------------
            // Push the output into the delay ring buffer

            self.stereo_memory
                .push(total + poly_utils::swap_voices_x4(total));

            // ------------------------------------------------------------------------------
            // Read the data from the delay ring buffer

            // SAFETY:
            // Our algorithm never causes `current_sample_delay` to be NaN or Infinity,
            // and it never generates any values that are too large to fit in an i32.
            let wet = unsafe { self.stereo_memory.get_interpolated(current_sample_delay) };

            let wet = wet.as_array();

            // ------------------------------------------------------------------------------
            // Apply stereo width control to the wet output

            let mid = (wet[0] + wet[1]) * 0.5;
            let side = (wet[1] - wet[0]) * current_width_coeff;

            let wet_left = mid - side;
            let wet_right = mid + side;

            let final_wet = f32x4::from_array([wet_left, wet_right, 0.0, 0.0]);

            // ------------------------------------------------------------------------------
            // Get the final output by mixing the wet and dry signals

            let final_output = (current_wet_amp * final_wet) + (current_dry_amp * input);
            let final_output = final_output.as_array();

            // ------------------------------------------------------------------------------
            // Write the final output to the audio buffer

            *l = final_output[0];
            *r = final_output[1];

            // ------------------------------------------------------------------------------
            // Increment the write index for the next frame

            self.write_index = (self.write_index + 1) & self.feedback_mask;

            // ------------------------------------------------------------------------------
            // Increment parameters

            current_width_coeff += delta_width_coeff;

            current_delay_increment += delta_delay_increment;
            current_sample_delay += current_delay_increment;
            current_sample_delay = current_sample_delay.simd_clamp(V_MIN_DELAY, V_MAX_SAMPLE_RATE);

            current_dry_amp += delta_dry_amp;
            current_wet_amp += delta_wet_amp;
            // The original Vitalium code forgot to increment low_shelf_amp.
            current_low_shelf_amp += delta_low_shelf_amp;
            current_high_shelf_amp += delta_high_shelf_amp;

            // The original Vitalium code forgot to increment pre_low_coeff, pre_high_coeff,
            // and low_shelf_coeff.
            current_pre_low_coeff += delta_pre_low_coeff;
            current_pre_high_coeff += delta_pre_high_coeff;
            current_low_shelf_coeff += delta_low_shelf_coeff;
            current_high_shelf_coeff += delta_high_shelf_coeff;
        }

        // ----------------------------------------------------------------------------------
        // Store the state of the delay parameter for the next call to process

        self.sample_delay_increment = current_delay_increment;
        self.sample_delay = current_sample_delay;
    }

    /// Resets all buffers.
    pub fn reset(&mut self) {
        self.pre_low_filter.reset();
        self.pre_high_filter.reset();

        for f in self.low_shelf_filters.iter_mut() {
            f.reset();
        }
        for f in self.high_shelf_filters.iter_mut() {
            f.reset();
        }

        for memory_v in self.feedback_memories.iter_mut() {
            for memory in memory_v.iter_mut() {
                memory.fill(0.0);
            }
        }

        for memory in self.allpass_memories.iter_mut() {
            memory.fill(0.0);
        }

        self.stereo_memory.clear();
    }

    #[inline(always)]
    /// Gets an interpolated value from the feedback memory.
    fn read_feedback_interpolated(&self, memories: &[Vec<f32>; 4], offset: f32x4) -> f32x4 {
        let write_offset = f32x4::splat(self.write_index as f32) - offset;

        // SAFETY:
        // Our algorithm never causes `offset` to be NaN or Infinity, and it
        // never generates any values that are too large to fit in an i32.
        let (floored_offset, floored_offset_i32) = unsafe {
            let floored_offset = poly_utils::simd_floor_f32x4_unchecked(write_offset);
            (floored_offset, floored_offset.to_int_unchecked())
        };

        let t = write_offset - floored_offset;
        let interpolation_matrix = Matrix::polynomial_interpolation_matrix(t);

        let indices = floored_offset_i32 & self.feedback_mask_v;
        let indices = indices.as_array();

        // SAFETY:
        // The bitmask ensures that the indices are within bounds.
        let (row0_slice, row1_slice, row2_slice, row3_slice) = unsafe {
            (
                std::slice::from_raw_parts(
                    memories[0]
                        .as_ptr()
                        .add((indices[0] + EXTRA_LOOKUP_SAMPLE) as usize),
                    4,
                ),
                std::slice::from_raw_parts(
                    memories[1]
                        .as_ptr()
                        .add((indices[1] + EXTRA_LOOKUP_SAMPLE) as usize),
                    4,
                ),
                std::slice::from_raw_parts(
                    memories[2]
                        .as_ptr()
                        .add((indices[2] + EXTRA_LOOKUP_SAMPLE) as usize),
                    4,
                ),
                std::slice::from_raw_parts(
                    memories[3]
                        .as_ptr()
                        .add((indices[3] + EXTRA_LOOKUP_SAMPLE) as usize),
                    4,
                ),
            )
        };

        // TODO: Make sure the internal check in `f32x4::from_slice` is being
        // properly elided (the check is to see if the length of the slice is
        // at least 4).
        let mut value_matrix = Matrix {
            rows: [
                f32x4::from_slice(row0_slice),
                f32x4::from_slice(row1_slice),
                f32x4::from_slice(row2_slice),
                f32x4::from_slice(row3_slice),
            ],
        };

        value_matrix.transpose();

        interpolation_matrix.multiply_and_sum_rows(&value_matrix)
    }

    #[inline(always)]
    /// Gets a value from the allpass memory.
    fn read_allpass(&self, memories: &[f32], offset: i32x4) -> f32x4 {
        let indices =
            (i32x4::splat(self.write_index * f32x4::LEN as i32) - offset) & self.allpass_mask_v;
        let indices = indices.as_array();

        // SAFETY:
        // The bitmask ensures that the indices are within bounds.
        let memory_array = unsafe {
            [
                *memories.get_unchecked(indices[0] as usize),
                *memories.get_unchecked(indices[1] as usize),
                *memories.get_unchecked(indices[2] as usize),
                *memories.get_unchecked(indices[3] as usize),
            ]
        };

        // TODO: Use unchecked indexing once I'm confident the code is sound.
        f32x4::from_array(memory_array)
    }
}

fn get_sample_rate_ratio(sample_rate: f32) -> f32 {
    sample_rate / BASE_SAMPLE_RATE
}

fn get_buffer_scale(sample_rate: f32) -> i32 {
    let mut scale = 1;
    let ratio = get_sample_rate_ratio(sample_rate);

    while (scale as f32) < ratio {
        scale *= 2;
    }

    scale
}
