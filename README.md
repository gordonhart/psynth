# `psynth`

Digital synthesizer project for personal entertainment and education.


## Development

### Dependencies

On top of those listed in the cargo manifest, the following system packages are required (exact
package name will depend on your distro):

- `pkg-config`
- `libzmq` ≥ 4.1
- `alsa` if compiling on Linux (see `cpal` docs for more details)

Hardware device drivers are gated behind the `hardware` feature flag and will only work on Linux.


### TODOs

- [ ] Generalize stream pattern to N-channel audio
    - [x] Implement `StereoConsumer`
- [ ] Implement `WavWriter` to save waveform to file
- [ ] Interface with hardware inputs (e.g. `MeatSpacePot` real-world `Pot` implementor)
- [ ] Figure out sampling/looping scheme -- how should this be implemented?
- [ ] Research and implement more filters
    - [ ] Define different rooms or parameterize `filters::reverb`
    - [ ] Envelope filters?
    - [ ] Band-pass filters?
- [ ] Research and implement a few instruments
    - [ ] At the very least, a good-sounding digital keyboard and some sort of drumkit
    - [ ] Explore possibility of integration of 3rd-party effects (e.g. VST instruments)
- [ ] Implement music handling
    - [x] Notes and operations on notes
    - [ ] Scales and operations on scales
    - [ ] Feeding notes/scales into controls
- [ ] Research and implement more controls
    - [ ] Consider what a `Keyboard` might be -- how are the buttons mapped to notes, or to sounds?
      How is the sound from a keypress fed into a `Consumer`?
- [x] Implement metronome
- [ ] ~~Sort out the `'static` situation (shouldn't be a requirement for `Generator`, `Filter` types)~~
    - Currently prioritizing ease of use and functionality over correctness -- `'static` as a
      requirement for `Generator` has not proven to be a roadblock in any way, and it may be
      preferable to code littered with `<'a>` explicit lifetimes
- [ ] Implement some form of CLI for `psynth-play` such that doing new things doesn't always
  involve modifying the `bin/main.rs` and recompiling
