# JustBoom DAC on Jetson TX2

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
