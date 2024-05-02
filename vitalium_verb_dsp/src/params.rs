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

/// The parameters of the reverb.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ReverbParams {
    /// The wet/dry mix, in the range `[0.0, 1.0]`
    ///
    /// By default this is set to `0.25`
    pub mix: f32,

    /// The size of the reverb, in the range `[0.0, 1.0]`
    ///
    /// By default this is set to `0.5`
    pub size: f32,
    /// The decay of the reverb in seconds, in the range `[0.1, 64.0]`
    ///
    /// By default this is set to `1.0`
    pub decay: f32,

    /// The pre-delay of the reverb in seconds, in the range `[0.0, 0.3]`
    ///
    /// By default this is set to `0.004`
    pub delay: f32,

    /// The stereo width adjustment of the wet signal, in the range
    /// `[-1.0, 1.0]`, where:
    /// * `0.0` is no change to stereo width
    /// * `-1.0` reduces the stereo width to mono
    /// * `1.0` widens the stereo width to the maximum amount
    ///
    /// By default this is set to `-0.05`
    pub width: f32,

    /// The frequency of the chorus applied to the feedback, in the range
    /// `[0.003, 8.0]`
    ///
    /// By default this is set to `0.25`
    pub chorus_freq_hz: f32,
    /// The amount of chorus applied to the feedback, in the range
    /// `[0.0, 1.0]`
    ///
    /// By default this is set to `0.046`
    pub chorus_amount: f32,

    /// The cutoff of the highpass filter applied to the input before it
    /// is sent to the reverb tank, in the range `[20.0, 20,000.0]`
    ///
    /// By default this is set to `20.0`
    pub pre_low_cut_hz: f32,
    /// The cutoff of the lowpass filter applied to the input before it
    /// is sent to the reverb tank, in the range `[20.0, 20,000.0]`
    ///
    /// By default this is set to `4,700.0`
    pub pre_high_cut_hz: f32,

    /// The cutoff of the low-shelf filter applied to the feedback, in
    /// the range `[20.0, 20,000.0]`
    ///
    /// By default this is set to `20.0`
    pub low_shelf_cut_hz: f32,
    /// The gain of the low-shelf filter applied to the feedback in
    /// decibels, in the range `[-6.0, 0.0]`
    ///
    /// By default this is set to `0.0`
    pub low_shelf_gain_db: f32,

    /// The cutoff of the high-shelf filter applied to the feedback, in
    /// the range `[20.0, 20,000.0]`
    ///
    /// By default this is set to `1,480.0`
    pub high_shelf_cut_hz: f32,
    /// The gain of the high-shelf filter applied to the feedback in
    /// decibels, in the range `[-6.0, 0.0]`
    ///
    /// By default this is set to `-1.0`
    pub high_shelf_gain_db: f32,
}

impl ReverbParams {
    pub const MIN_CUTOFF_FREQ: f32 = 20.0;
    pub const MAX_CUTOFF_FREQ: f32 = 20_000.0;

    pub const MIN_SHELF_GAIN_DB: f32 = -6.0;
    pub const MAX_SHELF_GAIN_DB: f32 = 0.0;

    pub const MIN_DELAY_SECONDS: f32 = 0.0;
    pub const MAX_DELAY_SECONDS: f32 = 0.3;

    pub const MIN_DECAY_SECONDS: f32 = 0.1;
    pub const MAX_DECAY_SECONDS: f32 = 64.0;

    pub const MIN_CHORUS_FREQ: f32 = 0.003;
    pub const MAX_CHORUS_FREQ: f32 = 8.0;

    pub const DEFAULT_PRE_LOW_CUTOFF: f32 = Self::MIN_CUTOFF_FREQ;
    pub const DEFAULT_PRE_HIGH_CUTOFF: f32 = 4_700.0;
    pub const DEFAULT_LOW_SHELF_CUTOFF: f32 = Self::MIN_CUTOFF_FREQ;
    pub const DEFAULT_LOW_SHELF_GAIN_DB: f32 = Self::MAX_SHELF_GAIN_DB;
    pub const DEFAULT_HIGH_SHELF_CUTOFF: f32 = 1_480.0;
    pub const DEFAULT_HIGH_SHELF_GAIN_DB: f32 = -1.0;
    pub const DEFAULT_DRY_WET_MIX: f32 = 0.25;
    pub const DEFAULT_DELAY_SECONDS: f32 = 0.004;
    pub const DEFAULT_DECAY_SECONDS: f32 = 1.0;
    pub const DEFAULT_REVERB_SIZE: f32 = 0.5;
    pub const DEFAULT_WIDTH: f32 = -0.05;
    pub const DEFAULT_CHORUS_AMOUNT: f32 = 0.046;
    pub const DEFAULT_CHORUS_FREQ: f32 = 0.25;
}

impl Default for ReverbParams {
    fn default() -> Self {
        Self {
            mix: Self::DEFAULT_DRY_WET_MIX,

            size: Self::DEFAULT_REVERB_SIZE,
            decay: Self::DEFAULT_DECAY_SECONDS,

            delay: Self::DEFAULT_DELAY_SECONDS,

            width: Self::DEFAULT_WIDTH,

            chorus_freq_hz: Self::DEFAULT_CHORUS_FREQ,
            chorus_amount: Self::DEFAULT_CHORUS_AMOUNT,

            pre_low_cut_hz: Self::DEFAULT_PRE_LOW_CUTOFF,
            pre_high_cut_hz: Self::DEFAULT_PRE_HIGH_CUTOFF,

            low_shelf_cut_hz: Self::DEFAULT_LOW_SHELF_CUTOFF,
            low_shelf_gain_db: Self::DEFAULT_LOW_SHELF_GAIN_DB,

            high_shelf_cut_hz: Self::DEFAULT_HIGH_SHELF_CUTOFF,
            high_shelf_gain_db: Self::DEFAULT_HIGH_SHELF_GAIN_DB,
        }
    }
}
