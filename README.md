# psynth

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
