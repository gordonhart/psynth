# `psynth` Hardware

Here lives the hardware half of `psynth`.


## Requirements

- Audio out:
    - Requirements:
        - 1/8" connector (required)
        - Stereo RCA connectors (required)
        - Bluetooth (optional)
    - Selection: [JustBoom DAC HAT](https://www.sparkfun.com/products/14319)

- Platform: single-board computer with sufficient connectors for hi-fidelity output, sufficient
  compute to not worry about it, low friction (runs a normal Linux distribution)
    - Selection: found a [Jetson TX2 Dev Kit](https://developer.nvidia.com/embedded/jetson-tx2-developer-kit)
      lying around

- I/O: I2C for all controls, I2S via the above linked HAT for output


## Jetson Notes

- Exact model: Nvidia 945-82771-0000-000 Jetson TX2 Development Kit
- [User manual](https://developer.download.nvidia.com/embedded/L4T/r32-3-1_Release_v1.0/jetson_tx2_developer_kit_user_guide.pdf?agakKkvf7ZXZII2hdSOffwlHtg7iYFQ1dO2YIc48TRrAgS1XBEDrY5NkGjdwQmIH_rzmycKozqHYcKbU4WWx7HmyAb7ixxP1Myv1TDODQ0uI1Tgvaj0Jc3CXaZzb2M6ksKrQoK7uqOTk-nPI4uNGGYFg_PGBEi8BHJ8V3Ein93kUJqtjiqu1lA)
    - Based on peripheral connectors, looks to be either revision "B02" or "B04" referenced in this
      manual
- GPIO pinout is identical to Raspberry PI, can use HATs made for the latter
    - Main I2C bus exposed via these headers is bus 1 at `/dev/i2c-1`

- Getting the DAC hat running:
    - Install `device-tree-compiler` for `fdtdump`:
    ```
    # fdtdump /boot/dtb/tegra186-quill-p3310-1000-c03-00-base.dtb
    ```
