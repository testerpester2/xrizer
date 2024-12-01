# xrizer - XR-ize your OpenVR games

xrizer is a reimplementation of OpenVR on top of OpenXR. This enables you to run OpenVR games through any OpenXR runtime without running SteamVR.

Note that xrizer is currently immature. Many things are likely broken, so please open bugs! For a more mature solution, check out [OpenComposite](https://gitlab.com/znixian/OpenComposite).
# Building
```
# dev build
cargo xbuild
# release build
cargo xbuild --release
```

# Usage

In order to use xrizer, you must change where OpenVR games search for the runtime. There are two ways to accomplish this:

1. Set the `VR_OVERRIDE` environment variable when launching a game: `VR_OVERRIDE=/path/to/xrizer <command>` (steam launch options: `VR_OVERRIDE=/path/to/xrizer %command%`)
2. Add the path to xrizer to `$XDG_CONFIG_HOME/openvr/openvrpaths.vrpath`

After building, the output directory can be used for either of these paths. If you built the dev build, this will be `<path to xrizer repo>/target/debug`, and for the release build this is `<path to xrizer repo>/target/release`.

## openvrpaths.vrpath

This file is a JSON file read by OpenVR games to determine where the runtime is located. If you have SteamVR installed and have run it before, you can simply add the path to xrizer to the "runtime" section.
If you haven't, here is a sample file that can be placed in `$XDG_CONFIG_HOME/openvr/openvrpaths.vrpath`:
```json
{
    "version": 1,
    "runtime": [
        "<path to xrizer>/target/debug"
    ]
}
```

## Steam Linux Runtime

When running games through the Steam Linux Runtime - which is all of them when using Proton 6 or newer - filesystem paths will not work the same way since it's a container. Some things to keep in mind are:
- Any path in your home directory will always be available
- Paths under `/usr` will be available at `/run/host/usr`
- Some paths will be completely unavailable.

You can make paths available within the container using the `PRESSURE_VESSEL_FILESYSTEM_RW` env var. An example of what you might put in your game's steam launch option if you want to use xrizer with Monado installed systemwide is:
```
XR_RUNTIME_JSON=/run/host/usr/share/openxr/1/openxr_monado.json VR_OVERRIDE=<path to xrizer>/target/debug PRESSURE_VESSEL_FILESYSTEM_RW=$XDG_RUNTIME_DIR/monado_comp_ipc
```
For more info on the container, see [Valve's docs on Pressure Vessel](https://gitlab.steamos.cloud/steamrt/steam-runtime-tools/-/blob/main/pressure-vessel/wrap.1.md).

# Contributing

All contributions welcome. If submitting pull requests, please consider writing a test if possible - OpenVR is a large API surface and games are fickle, so ensuring things are well tested prevents future unintentional breakage.

# See also

- [OpenComposite](https://gitlab.com/znixian/OpenOVR) - The original OpenVR/OpenXR implementation, much more mature than xrizer. Some of the code in this repo was rewritten based on OpenComposite.
- [OpenVR](https://github.com/ValveSoftware/openvr)
- [Monado](https://gitlab.freedesktop.org/monado/monado) - An open source OpenXR runtime
- [WiVRn](https://github.com/WiVRn/WiVRn) - An OpenXR streaming runtime for standalone headsets
