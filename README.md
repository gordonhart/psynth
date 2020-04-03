# `psynth`

Digital synthesizer project for personal entertainment and education.

## Development

TODOs:

- [ ] Generalize stream pattern to N-channel audio
    - [x] Implement `StereoConsumer`
- [ ] Implement `WavWriter` to save waveform to file
- [ ] Interface with hardware inputs (e.g. `MeatSpacePot` real-world `Pot` implementor)
- [ ] Figure out sampling/looping scheme -- how should this be implemented?
- [ ] Research and implement more filters
    - [ ] Define different rooms or parameterize `filters::reverb`
    - [ ] Envelope filters?
    - [ ] Band-pass filters?
- [ ] Research and implement more controls
- [x] Implement metronome
- [ ] Explore possibility of integration of 3rd-party effects (e.g. VST instruments)
- ~~[ ] Sort out the `'static` situation (shouldn't be a requirement for `Generator`, `Filter` types)~~
    - Currently prioritizing ease of use and functionality over correctness -- `'static` as a
      requirement for `Generator` has not proven to be a roadblock in any way, and it may be
      preferable to code littered with `<'a>` explicit lifetimes
