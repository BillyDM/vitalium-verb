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
use params::VitaliumVerbParams;
use std::sync::Arc;

use vitalium_verb_dsp::{Reverb, ReverbParams, MAX_BLOCK_SIZE};

mod editor;
mod params;

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
    const NAME: &'static str = "VitaliumVerb";
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

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        editor::create(
            Arc::clone(&self.params),
            Arc::clone(&self.params.editor_state),
        )
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

            let decay_seconds =
                crate::params::decay_normal_to_seconds(self.params.main.decay.value());
            max_decay_seconds = max_decay_seconds.max(decay_seconds);

            let params = ReverbParams {
                mix: self.params.main.mix.smoothed.next_step(frames as u32) * 0.01,

                size: self.params.main.size.value() * 0.01,
                decay: decay_seconds,

                delay: self.params.main.delay.value() * 0.001,

                width: self.params.main.width.smoothed.next_step(frames as u32) * 0.01,

                chorus_freq_hz: self
                    .params
                    .chorus
                    .chorus_freq
                    .smoothed
                    .next_step(frames as u32),
                chorus_amount: self
                    .params
                    .chorus
                    .chorus_amount
                    .smoothed
                    .next_step(frames as u32)
                    * 0.01,

                pre_low_cut_hz: self
                    .params
                    .pre_eq
                    .pre_low_cut
                    .smoothed
                    .next_step(frames as u32),
                pre_high_cut_hz: self
                    .params
                    .pre_eq
                    .pre_high_cut
                    .smoothed
                    .next_step(frames as u32),

                low_shelf_cut_hz: self
                    .params
                    .post_eq
                    .low_shelf_cut
                    .smoothed
                    .next_step(frames as u32),
                low_shelf_gain_db: self
                    .params
                    .post_eq
                    .low_shelf_gain
                    .smoothed
                    .next_step(frames as u32),

                high_shelf_cut_hz: self
                    .params
                    .post_eq
                    .high_shelf_cut
                    .smoothed
                    .next_step(frames as u32),
                high_shelf_gain_db: self
                    .params
                    .post_eq
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

nih_export_clap!(VitaliumVerb);
nih_export_vst3!(VitaliumVerb);
