# PSX Texture Warp

This example tries to re-create this texture warping demo video from https://www.david-colson.com/2021/11/30/ps1-style-renderer.html
(see mp4 under **Warped textures**), using the PSX GTE for some simple rotation about an axis.

The crate image here is one I made roughly myself from a license free 'wood' texture and is avaliable in three bit depths:

* `crate4bit.tim`
* `crate8bit.tim`
* `crate16bit.tim`

This example code has been used to debug and test the [GPU `cop2` additions in psx-sdk-rs](https://github.com/ayrtonm/psx-sdk-rs/pull/49) feature,
and for supporting [8bit CLUTs](https://github.com/ayrtonm/psx-sdk-rs/pull/50).

It will help test and confirm 16bit full color .TIM support too, which is currently unmplemtned in `psx-sdk-rs`.

### Cargo features, selectable at compile time:
* `bpp4`
* `bpp8`
* `bpp16`

Run (or build) with one of the following:

    cargo psx run --features bpp4

    cargo psx run --features bpp8

    cargo psx run --features bpp16

4 / 8 bit CLUT sizes will be displayed on screen, with the same basic wooden crate texture loaded from the appropriate image.

4bit: `CLUT size: Vertex(16, 1)` or 8bit: `CLUT size: Vertex(256, 1)`

There are also some `cop2` values displayed on screen, and a very simple test of squaring some numbers using `cop2` and reading the results:

```
SQR In  X, Y, Z: <1, 3, 5>
SQR Out X, Y, Z: <1, 9, 25>
```

### TODO:
* 16 bit .TIM / `15bpp` working in the sdk
* The actual movement and warping
