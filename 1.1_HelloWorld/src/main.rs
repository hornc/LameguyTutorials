#![no_std]
#![no_main]

use psx::gpu::{Color, VideoMode};
use psx::{dma, dprintln, Framebuffer};

// Following the tutorial at:
//   http://lameguy64.net/tutorials/pstutorials/chapter1/1-display.html
// but using a Rust SDK / toolchain:
//   https://github.com/ayrtonm/psx-sdk-rs


#[no_mangle]
fn main() {

    // Init graphics and stuff
    let mut fb = Framebuffer::new((0, 0), (0, 240), (320, 240), VideoMode::NTSC, Some(Color::new(63, 0, 127))).unwrap();
    // The suggested PAL resolution produces an InvalidY error. TODO: Investigate what are the
    // correct PAL resolutions to use.
    //let mut fb = Framebuffer::new((0, 0), (0, 256), (320, 256), VideoMode::PAL, Some(Color::new(63, 0, 127))).unwrap();

    let mut txt = fb.load_default_font().new_text_box((0, 8), (320, 224));

    let mut gpu_dma = dma::GPU::new();

    // Main loop
    loop {
        dprintln!(txt, "HELLO, WORLD!");
        txt.reset();

        // Wait for GPU to finish drawing and V-Blank
        fb.draw_sync();
        fb.wait_vblank();

        // Flip buffers and display
        fb.dma_swap(&mut gpu_dma);
    }
}
