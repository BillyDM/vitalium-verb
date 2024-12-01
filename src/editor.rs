use std::sync::Arc;

use nih_plug::editor::Editor;
use nih_plug::params::Param;
use nih_plug::prelude::Plugin;
use nih_plug_vizia::vizia::prelude::*;
use nih_plug_vizia::widgets::{ParamSlider, ParamSliderExt, ParamSliderStyle};
use nih_plug_vizia::{assets, create_vizia_editor, ViziaState, ViziaTheming};

use crate::{VitaliumVerb, VitaliumVerbParams};

#[derive(Lens, Clone)]
pub(crate) struct Data {
    pub params: Arc<VitaliumVerbParams>,
}

impl Model for Data {}

pub(crate) fn default_state() -> Arc<ViziaState> {
    ViziaState::new(|| (730, 390))
}

pub fn create(
    params: Arc<VitaliumVerbParams>,
    editor_state: Arc<ViziaState>,
) -> Option<Box<dyn Editor>> {
    create_vizia_editor(editor_state, ViziaTheming::Custom, move |cx, _| {
        cx.add_stylesheet(include_style!("src/styles.css"))
            .expect("failed to read stylesheet");
        assets::register_noto_sans_regular(cx);

        Data {
            params: params.clone(),
        }
        .build(cx);

        VStack::new(cx, |cx| {
            build_gui(cx);
        })
        .class("background");
    })
}

fn build_gui(cx: &mut Context) {
    VStack::new(cx, |cx| {
        HStack::new(cx, |cx| {
            Label::new(cx, "VitaliumVerb")
                .font_family(vec![FamilyOwned::Name(String::from(assets::NOTO_SANS))])
                .font_weight(FontWeightKeyword::Regular)
                .font_size(24.0);
            Label::new(cx, VitaliumVerb::VERSION)
                .font_family(vec![FamilyOwned::Name(String::from(assets::NOTO_SANS))])
                .font_weight(FontWeightKeyword::Regular)
                .class("version_text")
                .top(Stretch(1.0))
                .bottom(Pixels(2.0))
                .left(Pixels(11.0));
        })
        .size(Auto);
    })
    .height(Pixels(30.0))
    .right(Pixels(17.0))
    // Somehow this overrides the 'row-between' value now
    .bottom(Pixels(0.0))
    .left(Pixels(17.0))
    .top(Pixels(10.0))
    // This contains the editor mode buttom all the way on the left, and the plugin's name all the way on the right
    .col_between(Stretch(1.0));

    HStack::new(cx, |cx| {
        make_column(cx, "Main", |cx| {
            VStack::new(cx, |cx| {
                create_slider(cx, "Mix", Data::params, false, |params| &params.main.mix);
                create_slider(cx, "Size", Data::params, false, |params| &params.main.size);
                create_slider(cx, "Decay", Data::params, false, |params| {
                    &params.main.decay
                });
                create_slider(cx, "Delay", Data::params, false, |params| {
                    &params.main.delay
                });
                create_slider(cx, "Width", Data::params, true, |params| &params.main.width);
            })
            .top(Pixels(20.0))
            .bottom(Pixels(15.0))
            .width(Auto)
            .row_between(Pixels(6.0));
        });

        make_column(cx, "Post EQ", |cx| {
            VStack::new(cx, |cx| {
                create_slider(cx, "LS Freq", Data::params, false, |params| {
                    &params.post_eq.low_shelf_cut
                });
                create_slider(cx, "LS Gain", Data::params, false, |params| {
                    &params.post_eq.low_shelf_gain
                });
                create_slider(cx, "HS Freq", Data::params, false, |params| {
                    &params.post_eq.high_shelf_cut
                });
                create_slider(cx, "HS Gain", Data::params, false, |params| {
                    &params.post_eq.high_shelf_gain
                });
            })
            .top(Pixels(20.0))
            .bottom(Pixels(15.0))
            .width(Auto)
            .row_between(Pixels(6.0));
        });
    })
    .col_between(Pixels(28.0));

    HStack::new(cx, |cx| {
        make_column(cx, "Chorus", |cx| {
            VStack::new(cx, |cx| {
                create_slider(cx, "Freq", Data::params, false, |params| {
                    &params.chorus.chorus_freq
                });
                create_slider(cx, "Amount", Data::params, false, |params| {
                    &params.chorus.chorus_amount
                });
            })
            .top(Pixels(20.0))
            .bottom(Pixels(15.0))
            .width(Auto)
            .row_between(Pixels(6.0));
        });

        make_column(cx, "Pre EQ", |cx| {
            VStack::new(cx, |cx| {
                create_slider(cx, "Low Cut", Data::params, false, |params| {
                    &params.pre_eq.pre_low_cut
                });
                create_slider(cx, "High Cut", Data::params, false, |params| {
                    &params.pre_eq.pre_high_cut
                });
            })
            .top(Pixels(20.0))
            .bottom(Pixels(15.0))
            .width(Auto)
            .row_between(Pixels(6.0));
        });
    })
    .top(Pixels(65.0))
    .col_between(Pixels(28.0));
}

fn make_column(cx: &mut Context, title: &str, contents: impl FnOnce(&mut Context)) {
    VStack::new(cx, |cx| {
        Label::new(cx, title)
            .font_family(vec![FamilyOwned::Name(String::from(assets::NOTO_SANS))])
            .font_weight(FontWeightKeyword::Regular)
            .font_size(21.0)
            .left(Stretch(1.0))
            // This should align nicely with the right edge of the slider
            .right(Pixels(-15.0))
            .bottom(Pixels(-10.0))
            .text_align(TextAlign::Right);

        contents(cx);
    })
    .width(Pixels(300.0));
}

#[allow(clippy::too_many_arguments)]
pub fn create_slider<L, Params, P, FMap>(
    cx: &mut Context,
    name: &str,
    params: L,
    from_center: bool,
    f: FMap,
) where
    L: Lens<Target = Params> + Clone,
    Params: 'static,
    P: Param + 'static,
    FMap: Fn(&Params) -> &P + Copy + 'static,
{
    HStack::new(cx, |cx| {
        Label::new(cx, name)
            .width(Pixels(80.0))
            .height(Pixels(20.0))
            .right(Pixels(6.0))
            .top(Pixels(5.5))
            .font_family(vec![FamilyOwned::Name(String::from(assets::NOTO_SANS))])
            .font_weight(FontWeightKeyword::Regular)
            .font_size(15.0)
            .text_align(TextAlign::Right);

        ParamSlider::new(cx, params, f)
            .height(Pixels(28.0))
            .width(Pixels(230.0))
            .set_style(if from_center {
                ParamSliderStyle::Centered
            } else {
                ParamSliderStyle::FromLeft
            });
    })
    .size(Auto);
}
