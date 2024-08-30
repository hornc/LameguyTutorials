#![no_std]
#![no_main]

use psx::gpu::{Color, VideoMode};
use psx::{dma, Framebuffer};

// Following the tutorial at:
//   http://lameguy64.net/tutorials/pstutorials/chapter1/1-display.html
// but using a Rust SDK / toolchain:
//   https://github.com/ayrtonm/psx-sdk-rs


#[no_mangle]
fn main() {
    let mut fb = Framebuffer::new((0, 0), (0, 240), (320, 240), VideoMode::NTSC, Some(Color::new(63, 0, 127))).unwrap();
    // The suggested PAL resolution produces an InvalidY error:
    //let mut fb = Framebuffer::new((0, 0), (0, 256), (320, 256), VideoMode::PAL, Some(Color::new(63, 0, 127))).unwrap();
    let mut gpu_dma = dma::GPU::new();
    loop {
        fb.draw_sync();
        fb.wait_vblank();
        fb.dma_swap(&mut gpu_dma);
    }
}

