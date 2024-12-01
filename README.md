# VitaliumVerb

![screenshot](assets/screenshot.png)

A [Rust](https://www.rust-lang.org/) port of the reverb module from the [Vital](https://github.com/mtytel/vital)/[Vitalium] synthesizer, allowing it to be used as an effect plugin. There are also a couple of minor improvements and optimizations added:
* A stereo width parameter applied to the wet signal
* Runtime-evaluated constants like filter coefficients, gain amplitudes, chorus phase increments, and allpass matrices are only recalculated when their respective parameters have changed (the original recalculated these every process cycle).

## Download

You can download pre-built binaries for Linux, Windows, and MacOS from the [Releases](https://github.com/BillyDM/vitalium-verb/releases) tab.

## Building and Installing

After installing [Rust](https://rustup.rs/) and the [nightly toolchain](https://rust-lang.github.io/rustup/concepts/channels.html) (`rustup toolchain install nightly`), you can compile VitaliumVerb as follows:

```shell
cargo +nightly xtask bundle vitalium_verb --release
```

Then copy `/target/bundled/VitaliumVerb.clap` and/or `/target/bundled/VitaliumVerb.vst3` to the corresponding plugin directories for your OS.

On macOS you may need to [disable Gatekeeper](https://disable-gatekeeper.github.io/) as Apple has recently made it more difficult to run unsigned code on macOS.

[Vitalium]: https://github.com/DISTRHO/DISTRHO-Ports/tree/5c55f9445ee6ff75d53c7f8601fc341d200aa4a0/ports-juce6.0/vitalium
