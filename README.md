# Lameguy PSX Tutorials ... but in Rust
Following the excellent PSXdev tutorials by Lameguy64 at:
http://lameguy64.net/tutorials/pstutorials/
[[archived URL]](https://web.archive.org/web/20240822185326/http://lameguy64.net/tutorials/pstutorials/)

... but using a Rust SDK / toolchain:
https://github.com/ayrtonm/psx-sdk-rs

In the 4 to 6 months since I started this learning / conversion project (mid-2024 to early 2025), the original tutorials have sadly dropped off the web :(
I am now linking to the archived pages at the [Internet Archive's Wayback Machine](https://web.archive.org/).

* [1.1](https://web.archive.org/web/20240822185308/http://lameguy64.net/tutorials/pstutorials/chapter1/1-display.html) Setting up Graphics and [Hello World](./1.1_HelloWorld/)
* [1.2](https://web.archive.org/web/20240916202225/http://lameguy64.net/tutorials/pstutorials/chapter1/2-graphics.html) Drawing Graphics Primitives [Yellow Square](./1.2_YellowSquare/)
* [1.3](https://web.archive.org/web/20240916202224/http://lameguy64.net/tutorials/pstutorials/chapter1/3-textures.html) Textures, TPages and CLUTs [Textures](./1.3_Textures/)
* [1.4](https://web.archive.org/web/20240916202225/http://lameguy64.net/tutorials/pstutorials/chapter1/4-controllers.html) Initializing / Parsing Controller Data [Controllers](./1.4_Controllers/)
* [1.5](https://web.archive.org/web/20240916202225/http://lameguy64.net/tutorials/pstutorials/chapter1/5-fixedpoint.html) Rotating and moving polys using [Fixed Point Math](./1.5_FixedPointMath)
* [1.6](https://web.archive.org/web/20240916202225/http://lameguy64.net/tutorials/pstutorials/chapter1/6-cdreading.html) [Using the CD-ROM](./1.6_UsingCdRom)

## To run the examples:
1. Install the latest cargo-psx from [psk-sdk-rs](https://github.com/ayrtonm/psx-sdk-rs), ideally from source, following the simple instructions in the README. (`cd cargo-psx; cargo install --path .`)
2. Install [Mednafen](https://mednafen.github.io/) (Playstation and other console runner / emulator) for your system. You will need to follow the [PSX BIOS instructions](https://mednafen.github.io/documentation/psx.html), this may be slightly complicated, but if you are doing PSX dev / emulation, you'll need to figure this out and obtain appropraite BIOS for the systems you need to emulate.
3. Run a tutorial example from this repo, e.g.:
```
cd 1.1_HelloWorld
cargo psx run
```

With cargo-psx and Mednafen correctly installed, you should see an blue/indigo screen with `HELLO, WORLD!` in white text.

All the original tutorial examples assume US / NA (North American) NTSC modes, so that is what I use here by default.

In the Mednafen config file, that's:

    ;Default region to use.
    psx.region_default na

I hope these examples are useful to someone. The best way to use them would be to read the original tutorials to understand how the PSX works, then cross-reference these examples with the psx-sdk-rs psx API documentation.

*C.Horn (Jan 2025)*
