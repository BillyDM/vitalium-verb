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

use nih_plug::prelude::*;
use std::sync::Arc;

use vitalium_verb_dsp::{Reverb, ReverbParams, MAX_BLOCK_SIZE};

#[derive(Params)]
struct VitaliumVerbParams {
    #[id = "mix"]
    pub mix: FloatParam,

    #[id = "size"]
    pub size: FloatParam,
    #[id = "decay"]
    pub decay: FloatParam,

    #[id = "delay"]
    pub delay: FloatParam,

    #[id = "width"]
    pub width: FloatParam,

    #[id = "chorus_freq"]
    pub chorus_freq: FloatParam,
    #[id = "chorus_amount"]
    pub chorus_amount: FloatParam,

    #[id = "pre_low_cut"]
    pub pre_low_cut: FloatParam,
    #[id = "pre_high_cut"]
    pub pre_high_cut: FloatParam,

    #[id = "low_shelf_cut"]
    pub low_shelf_cut: FloatParam,
    #[id = "low_shelf_gain"]
    pub low_shelf_gain: FloatParam,

    #[id = "high_shelf_cut"]
    pub high_shelf_cut: FloatParam,
    #[id = "high_shelf_gain"]
    pub high_shelf_gain: FloatParam,
}

impl Default for VitaliumVerbParams {
    fn default() -> Self {
        let cutoff_freq_range = FloatRange::Skewed {
            min: ReverbParams::MIN_CUTOFF_FREQ,
            max: ReverbParams::MAX_CUTOFF_FREQ,
            factor: FloatRange::skew_factor(-2.0),
        };

        let shelf_gain_range = FloatRange::Skewed {
            min: ReverbParams::MIN_SHELF_GAIN_DB,
            max: ReverbParams::MAX_SHELF_GAIN_DB,
            factor: FloatRange::skew_factor(0.5),
        };

        Self {
            mix: FloatParam::new(
                "Mix",
                ReverbParams::DEFAULT_DRY_WET_MIX * 100.0,
                FloatRange::Linear {
                    min: 0.0,
                    max: 100.0,
                },
            )
            .with_smoother(SmoothingStyle::Linear(50.0))
            .with_unit(" %"),

            size: FloatParam::new(
                "Size",
                ReverbParams::DEFAULT_REVERB_SIZE * 100.0,
                FloatRange::Linear {
                    min: 0.0,
                    max: 100.0,
                },
            ),
            decay: FloatParam::new(
                "Decay",
                decay_seconds_to_normal(ReverbParams::DEFAULT_DECAY_SECONDS),
                FloatRange::Linear { min: 0.0, max: 1.0 },
            )
            .with_value_to_string(Arc::new(|normal: f32| -> String {
                format!("{:.3} secs", decay_normal_to_seconds(normal))
            }))
            .with_string_to_value(Arc::new(|s: &str| -> Option<f32> {
                if let Ok(seconds) = s.parse::<f32>() {
                    Some(decay_seconds_to_normal(seconds))
                } else {
                    None
                }
            })),

            delay: FloatParam::new(
                "Delay",
                ReverbParams::DEFAULT_DELAY_SECONDS * 1_000.0,
                FloatRange::Skewed {
                    min: ReverbParams::MIN_DELAY_SECONDS * 1_000.0,
                    max: ReverbParams::MAX_DELAY_SECONDS * 1_000.0,
                    factor: 0.3,
                },
            )
            .with_unit(" ms"),

            width: FloatParam::new(
                "Width",
                ReverbParams::DEFAULT_WIDTH * 100.0,
                FloatRange::SymmetricalSkewed {
                    min: -100.0,
                    max: 100.0,
                    factor: 0.6,
                    center: 0.0,
                },
            )
            .with_smoother(SmoothingStyle::Linear(50.0))
            .with_unit(" %"),

            chorus_freq: FloatParam::new(
                "Chorus Freq",
                ReverbParams::DEFAULT_CHORUS_FREQ,
                FloatRange::SymmetricalSkewed {
                    min: ReverbParams::MIN_CHORUS_FREQ,
                    max: ReverbParams::MAX_CHORUS_FREQ,
                    factor: 0.5,
                    center: ReverbParams::DEFAULT_CHORUS_FREQ,
                },
            )
            .with_smoother(SmoothingStyle::Linear(50.0))
            .with_value_to_string(formatters::v2s_f32_hz_then_khz(3))
            .with_string_to_value(formatters::s2v_f32_hz_then_khz()),
            chorus_amount: FloatParam::new(
                "Chorus Amt",
                ReverbParams::DEFAULT_CHORUS_AMOUNT * 100.0,
                FloatRange::Skewed {
                    min: 0.0,
                    max: 100.0,
                    factor: 0.23,
                },
            )
            .with_smoother(SmoothingStyle::Linear(50.0))
            .with_unit(" %"),

            pre_low_cut: FloatParam::new(
                "Pre Low Cut",
                ReverbParams::DEFAULT_PRE_LOW_CUTOFF,
                cutoff_freq_range.clone(),
            )
            .with_smoother(SmoothingStyle::Linear(50.0))
            .with_value_to_string(formatters::v2s_f32_hz_then_khz(0))
            .with_string_to_value(formatters::s2v_f32_hz_then_khz()),
            pre_high_cut: FloatParam::new(
                "Pre High Cut",
                ReverbParams::DEFAULT_PRE_HIGH_CUTOFF,
                cutoff_freq_range.clone(),
            )
            .with_smoother(SmoothingStyle::Linear(50.0))
            .with_value_to_string(formatters::v2s_f32_hz_then_khz(0))
            .with_string_to_value(formatters::s2v_f32_hz_then_khz()),

            low_shelf_cut: FloatParam::new(
                "Low Shelf Cut",
                ReverbParams::DEFAULT_LOW_SHELF_CUTOFF,
                cutoff_freq_range.clone(),
            )
            .with_smoother(SmoothingStyle::Linear(50.0))
            .with_value_to_string(formatters::v2s_f32_hz_then_khz(0))
            .with_string_to_value(formatters::s2v_f32_hz_then_khz()),
            low_shelf_gain: FloatParam::new(
                "Low Shelf Gain",
                ReverbParams::DEFAULT_LOW_SHELF_GAIN_DB,
                shelf_gain_range.clone(),
            )
            .with_smoother(SmoothingStyle::Linear(50.0))
            .with_unit(" dB"),

            high_shelf_cut: FloatParam::new(
                "High Shelf Cut",
                ReverbParams::DEFAULT_HIGH_SHELF_CUTOFF,
                cutoff_freq_range.clone(),
            )
            .with_smoother(SmoothingStyle::Linear(50.0))
            .with_value_to_string(formatters::v2s_f32_hz_then_khz(0))
            .with_string_to_value(formatters::s2v_f32_hz_then_khz()),
            high_shelf_gain: FloatParam::new(
                "High Shelf Gain",
                ReverbParams::DEFAULT_HIGH_SHELF_GAIN_DB,
                shelf_gain_range.clone(),
            )
            .with_smoother(SmoothingStyle::Linear(50.0))
            .with_unit(" dB"),
        }
    }
}

struct VitaliumVerb {
    params: Arc<VitaliumVerbParams>,
    reverb: Reverb,
}

impl Default for VitaliumVerb {
    fn default() -> Self {
        Self {
            params: Arc::new(VitaliumVerbParams::default()),
            reverb: Reverb::default(),
        }
    }
}

impl Plugin for VitaliumVerb {
    const NAME: &'static str = "Vitalium Verb";
    const VENDOR: &'static str = "Billy Messenger";
    const URL: &'static str = env!("CARGO_PKG_HOMEPAGE");
    const EMAIL: &'static str = "60663878+BillyDM@users.noreply.github.com";

    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: NonZeroU32::new(2),
        main_output_channels: NonZeroU32::new(2),

        aux_input_ports: &[],
        aux_output_ports: &[],

        names: PortNames::const_default(),
    }];

    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::None;

    const SAMPLE_ACCURATE_AUTOMATION: bool = false;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        self.reverb.init(buffer_config.sample_rate);
        true
    }

    fn reset(&mut self) {
        self.reverb.reset();
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let mut max_decay_seconds: f32 = 0.0;

        for (_, block) in buffer.iter_blocks(MAX_BLOCK_SIZE) {
            let mut block_channels = block.into_iter();

            let out_l = block_channels.next().unwrap();
            let out_r = block_channels.next().unwrap();

            let frames = out_l.len();

            let decay_seconds = decay_normal_to_seconds(self.params.decay.value());
            max_decay_seconds = max_decay_seconds.max(decay_seconds);

            let params = ReverbParams {
                mix: self.params.mix.smoothed.next_step(frames as u32) * 0.01,

                size: self.params.size.value() * 0.01,
                decay: decay_seconds,

                delay: self.params.delay.value() * 0.001,

                width: self.params.width.smoothed.next_step(frames as u32) * 0.01,

                chorus_freq_hz: self.params.chorus_freq.smoothed.next_step(frames as u32),
                chorus_amount: self.params.chorus_amount.smoothed.next_step(frames as u32) * 0.01,

                pre_low_cut_hz: self.params.pre_low_cut.smoothed.next_step(frames as u32),
                pre_high_cut_hz: self.params.pre_high_cut.smoothed.next_step(frames as u32),

                low_shelf_cut_hz: self.params.low_shelf_cut.smoothed.next_step(frames as u32),
                low_shelf_gain_db: self.params.low_shelf_gain.smoothed.next_step(frames as u32),

                high_shelf_cut_hz: self.params.high_shelf_cut.smoothed.next_step(frames as u32),
                high_shelf_gain_db: self
                    .params
                    .high_shelf_gain
                    .smoothed
                    .next_step(frames as u32),
            };

            self.reverb.process(out_l, out_r, &params);
        }

        ProcessStatus::Tail(self.reverb.tail_samples(max_decay_seconds))
    }
}

impl ClapPlugin for VitaliumVerb {
    const CLAP_ID: &'static str = "com.github.billydm.vitalium-verb";
    const CLAP_DESCRIPTION: Option<&'static str> =
        Some("A port of the reverb module from Vitalium");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;

    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::AudioEffect,
        ClapFeature::Stereo,
        ClapFeature::Reverb,
    ];
}

impl Vst3Plugin for VitaliumVerb {
    const VST3_CLASS_ID: [u8; 16] = *b"bdm-vitaliumverb";

    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Reverb];
}

// ----------------------------------------------------------------------------------
// Use a piece-wise function as the mapping for the decay parameter.
// The lower part is linear, while the higher part is logarithmic.

const DECAY_NORMAL_STOP: f32 = 0.8;
const DECAY_SECONDS_STOP: f32 = 5.0;

fn decay_normal_to_seconds(normal: f32) -> f32 {
    let normal = normal.clamp(0.0, 1.0);

    // Dedicate the majority of the range to small values.
    if normal <= DECAY_NORMAL_STOP {
        ReverbParams::MIN_DECAY_SECONDS
            + (normal
                * (1.0 / DECAY_NORMAL_STOP)
                * (DECAY_SECONDS_STOP - ReverbParams::MIN_DECAY_SECONDS))
    } else {
        let n1 = (normal - DECAY_NORMAL_STOP) * (1.0 / (1.0 - DECAY_NORMAL_STOP));
        DECAY_SECONDS_STOP + (n1 * n1 * (ReverbParams::MAX_DECAY_SECONDS - DECAY_SECONDS_STOP))
    }
}

fn decay_seconds_to_normal(seconds: f32) -> f32 {
    let seconds = seconds.clamp(
        ReverbParams::MIN_DECAY_SECONDS,
        ReverbParams::MAX_DECAY_SECONDS,
    );

    if seconds <= DECAY_SECONDS_STOP {
        (seconds - ReverbParams::MIN_DECAY_SECONDS)
            * (1.0 / (DECAY_SECONDS_STOP - ReverbParams::MIN_DECAY_SECONDS))
            * DECAY_NORMAL_STOP
    } else {
        let n1 = (seconds - DECAY_SECONDS_STOP)
            * (1.0 / (ReverbParams::MAX_DECAY_SECONDS - DECAY_SECONDS_STOP));
        DECAY_NORMAL_STOP + (n1.sqrt() * (1.0 - DECAY_NORMAL_STOP))
    }
}

// ----------------------------------------------------------------------------------

nih_export_clap!(VitaliumVerb);
nih_export_vst3!(VitaliumVerb);
