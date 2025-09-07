# wl-starfield

An AI-slop starfield animation written in Rust using [`winit`](https://github.com/rust-windowing/winit) and [`pixels`](https://github.com/parasyte/pixels).
Built for fun as a **desktop background effect** for myself on Wayland / Hyprland.

> [!WARNING] Do not expect support. Works on my machine™.
> Pull requests welcome.

---

## Features
- Twinkling stars
- Occasional shooting stars with trails
- Fullscreen window, intended for compositor background layers
- Wayland + Hyprland tested

---

## Future Improvements
- Config file to tune constants (number of stars, speeds, colors, etc.)

---

## Development

This project uses [Nix](https://nixos.org/) for shell / packaging.

To enter a dev shell:

```sh
nix develop
```

To have nix build application and run:

```sh
nix build
result/bin/wl-starfield
```

Or just directly with Cargo if you don't use nix:

```sh
cargo run --release
```
