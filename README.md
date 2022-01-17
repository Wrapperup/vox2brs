# vox2brs
[Download from the releases page](https://github.com/Wrapperup/vox2brs/releases)

<p align=center>
<img src=https://user-images.githubusercontent.com/7478134/149688794-7a39ba41-9187-4e80-86c7-4a9716e736d5.png>
</p>
<br>

vox2brs is a tool to convert your MagicaVoxel `.vox` files into Brickadia's `.brs` file format. It can turn voxels into bricks, plates, and microbricks in any size you want. Rampifier included!

## Getting Started
[Download vox2brs-gui or vox2brs-cli from here](https://github.com/Wrapperup/vox2brs/releases)

![image](https://user-images.githubusercontent.com/7478134/149688378-ec6761b4-d89c-41fd-a6bb-9e353f68b89a.png)

The GUI version is recommended if you are just getting started.
The CLI version is also available, see below for usage.

## vox2brs CLI Usage
See `vox2brs --help` for help.

```
vox2brs-cli
Convert MagicaVoxel models into a BRS file

USAGE:
    vox2brs.exe [OPTIONS] <INPUT> <OUTPUT> [ARGS]

ARGS:
    <INPUT>     Input path to .vox file
    <OUTPUT>    Output directory of the converted .brs file
    <MODE>      How voxels are interpreted [default: brick] [possible values: brick, plate,
                micro-brick]
    <WIDTH>     Width of the output brick
    <HEIGHT>    Height of the output brick

OPTIONS:
    -h, --help        Print help information
    -r, --rampify     Run rampifier?
    -s, --simplify    Should we run the simplifier?
```

Examples:
* `vox2brs my_tree.brs my_tree.vox micro-brick 1 1 --simplify`
* `vox2brs my_tree.brs my_tree.vox brick --rampify` NOTE: Rampify also implies simplify.
* `vox2brs my_tree.brs my_tree.vox plate`

## Media
<img src=https://user-images.githubusercontent.com/7478134/149688946-49d98267-9e4e-4165-a85d-5274d0623c31.png>
<img src=https://user-images.githubusercontent.com/7478134/149688242-f1afbf68-d0f5-4669-96f1-ce2f0a0ee614.png>
