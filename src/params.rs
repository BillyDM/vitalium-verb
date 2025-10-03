use std::sync::Arc;

use nih_plug::prelude::*;
use nih_plug_vizia::ViziaState;
use vitalium_verb_dsp::ReverbParams;

#[derive(Params)]
pub struct MainParams {
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
}

impl Default for MainParams {
    fn default() -> Self {
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
            .with_value_to_string(Arc::new(|val: f32| -> String { format!("{:.2}", val) }))
            .with_unit(" %"),

            size: FloatParam::new(
                "Size",
                ReverbParams::DEFAULT_REVERB_SIZE * 100.0,
                FloatRange::Linear {
                    min: 0.0,
                    max: 100.0,
                },
            )
            .with_value_to_string(Arc::new(|val: f32| -> String { format!("{:.2}", val) })),
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
            .with_value_to_string(Arc::new(|val: f32| -> String { format!("{:.2}", val) }))
            .with_unit(" ms"),

            width: FloatParam::new(
                "Width",
                100.0,
                FloatRange::SymmetricalSkewed {
                    min: 0.0,
                    max: 200.0,
                    factor: 0.6,
                    center: 100.0,
                },
            )
            .with_smoother(SmoothingStyle::Linear(50.0))
            .with_value_to_string(Arc::new(|val: f32| -> String { format!("{:.2}", val) }))
            .with_unit(" %"),
        }
    }
}

#[derive(Params)]
pub struct ChorusParams {
    #[id = "chorus_freq"]
    pub chorus_freq: FloatParam,
    #[id = "chorus_amount"]
    pub chorus_amount: FloatParam,
}

impl Default for ChorusParams {
    fn default() -> Self {
        Self {
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
            .with_value_to_string(Arc::new(|val: f32| -> String { format!("{:.2}", val) }))
            .with_unit(" %"),
        }
    }
}

#[derive(Params)]
pub struct PreFilterParams {
    #[id = "pre_low_cut"]
    pub pre_low_cut: FloatParam,
    #[id = "pre_high_cut"]
    pub pre_high_cut: FloatParam,
}

impl Default for PreFilterParams {
    fn default() -> Self {
        let cutoff_freq_range = FloatRange::Skewed {
            min: ReverbParams::MIN_CUTOFF_FREQ,
            max: ReverbParams::MAX_CUTOFF_FREQ,
            factor: FloatRange::skew_factor(-2.0),
        };

        Self {
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
        }
    }
}

#[derive(Params)]
pub struct HiLoDampingParams {
    #[id = "low_shelf_cut"]
    pub low_shelf_cut: FloatParam,
    #[id = "low_shelf_gain"]
    pub low_shelf_gain: FloatParam,

    #[id = "high_shelf_cut"]
    pub high_shelf_cut: FloatParam,
    #[id = "high_shelf_gain"]
    pub high_shelf_gain: FloatParam,
}

impl Default for HiLoDampingParams {
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
            low_shelf_cut: FloatParam::new(
                "Damping LS Cut",
                ReverbParams::DEFAULT_LOW_SHELF_CUTOFF,
                cutoff_freq_range.clone(),
            )
            .with_smoother(SmoothingStyle::Linear(50.0))
            .with_value_to_string(formatters::v2s_f32_hz_then_khz(0))
            .with_string_to_value(formatters::s2v_f32_hz_then_khz()),
            low_shelf_gain: FloatParam::new(
                "Damping LS Gain",
                ReverbParams::DEFAULT_LOW_SHELF_GAIN_DB,
                shelf_gain_range.clone(),
            )
            .with_smoother(SmoothingStyle::Linear(50.0))
            .with_value_to_string(Arc::new(|val: f32| -> String { format!("{:.2}", val) }))
            .with_unit(" dB"),

            high_shelf_cut: FloatParam::new(
                "Damping HS Cut",
                ReverbParams::DEFAULT_HIGH_SHELF_CUTOFF,
                cutoff_freq_range.clone(),
            )
            .with_smoother(SmoothingStyle::Linear(50.0))
            .with_value_to_string(formatters::v2s_f32_hz_then_khz(0))
            .with_string_to_value(formatters::s2v_f32_hz_then_khz()),
            high_shelf_gain: FloatParam::new(
                "Damping HS Gain",
                ReverbParams::DEFAULT_HIGH_SHELF_GAIN_DB,
                shelf_gain_range.clone(),
            )
            .with_smoother(SmoothingStyle::Linear(50.0))
            .with_value_to_string(Arc::new(|val: f32| -> String { format!("{:.2}", val) }))
            .with_unit(" dB"),
        }
    }
}

#[derive(Params)]
pub struct VitaliumVerbParams {
    /// The editor state, saved together with the parameter state so the custom scaling can be
    /// restored.
    #[persist = "editor-state"]
    pub editor_state: Arc<ViziaState>,

    #[nested(group = "main")]
    pub main: Arc<MainParams>,

    #[nested(group = "chorus")]
    pub chorus: Arc<ChorusParams>,

    #[nested(group = "pre_filter")]
    pub pre_filter: Arc<PreFilterParams>,

    #[nested(group = "hi_lo_damping")]
    pub hi_lo_damping: Arc<HiLoDampingParams>,
}

impl Default for VitaliumVerbParams {
    fn default() -> Self {
        Self {
            editor_state: crate::editor::default_state(),
            main: Arc::new(MainParams::default()),
            chorus: Arc::new(ChorusParams::default()),
            pre_filter: Arc::new(PreFilterParams::default()),
            hi_lo_damping: Arc::new(HiLoDampingParams::default()),
        }
    }
}

// ----------------------------------------------------------------------------------
// Use a piece-wise function as the mapping for the decay parameter.
// The lower part is linear, while the higher part is logarithmic.

const DECAY_NORMAL_STOP: f32 = 0.8;
const DECAY_SECONDS_STOP: f32 = 5.0;

pub fn decay_normal_to_seconds(normal: f32) -> f32 {
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
