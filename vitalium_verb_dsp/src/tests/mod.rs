use crate::{Reverb, ReverbParams};

#[test]
fn sine_wave() {
    const AMPLITUDE: f32 = 0.25;
    const FREQ_HZ: f32 = 440.0;
    const BUFFER_LEN: usize = 256;
    const SAMPLE_RATE: f32 = 48_000.0;
    const ITERATIONS: usize = 16;

    let mut phasor = 0.0;
    let phasor_inc = FREQ_HZ / SAMPLE_RATE;
    let input: Vec<f32> = (0..BUFFER_LEN)
        .map(|_| {
            let s = (phasor * std::f32::consts::TAU).sin() * AMPLITUDE;
            phasor = (phasor + phasor_inc).fract();
            s
        })
        .collect();

    let mut reverb = Reverb::default();
    reverb.init(SAMPLE_RATE);

    let params = ReverbParams {
        delay: 0.0,
        ..Default::default()
    };

    for _ in 0..ITERATIONS {
        println!("-------------------------------------------------");

        let mut out_l = input.clone();
        let mut out_r = input.clone();

        reverb.process(&mut out_l, &mut out_r, &params);

        // Make sure there is nothing obviously wrong with the samples.
        for s in out_l.iter().chain(out_r.iter()) {
            assert!(s.is_finite());
            assert!(!s.is_nan());
            assert!(!s.is_subnormal());
            assert!(s.abs() <= 1.0);
        }
    }
}
