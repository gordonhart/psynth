# JustBoom DAC on Jetson TX2

Not quite working yet.

Product: [JustBoom DAC HAT for Raspberry PI](https://shop.justboom.co/products/justboom-dac-hat)

## Development

Install `device-tree-compiler` for `fdtdump`, `dtc` other useful programs.

Unpack the `.dtbo` compiled device tree overlay for the JustBoom DAC on the RPi
([raspberrypi/firmware source](https://github.com/raspberrypi/firmware/blob/master/boot/overlays/justboom-dac.dtbo)
to plaintext:

```bash
$ fdtdump justboom-dac.dtbo
```

Dump the current device tree (currently located at `../tegra-device-tree.dts`):

```bash
$ dtc -I fs /sys/firmware/devicetree/base
```

Alternatively:

```
# fdtdump /boot/dtb/tegra186-quill-p3310-1000-c03-00-base.dtb
```

### Unstructured Notes

- Developer forum: [How to update device tree on TX2](https://forums.developer.nvidia.com/t/how-to-update-device-tree-on-tx2/53506/17)
    - No definitive answer, especially not for 28.2.1
- Looks like setting up an Ubuntu 16/18.04 machine as JetPack host will be necessary, because some
  sort of re-flashing will be requried to update the compiled+signed `.dtb`
    - If this is the case, might as well update to latest L4T version
      ([32.3.1](https://developer.nvidia.com/embedded/linux-tegra))
