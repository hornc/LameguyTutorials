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
    // let mut fb = Framebuffer::new((0, 0), (0, 240), (320, 240), VideoMode::NTSC, Some(Color::new(63, 0, 127))).unwrap();
    // x:  256, 320, 512, 640, 384
    const hres: [i16; 6] = [256, 320, 512, 640, 384, 368];
    const x: i16 = hres[2];
    const y: i16 = 240;
    //const y: i16 = 480;
    //const y: i16 = 256;

    //                             buf0,  buf1,  size
    let mut fb = Framebuffer::new((0, 0), (0, y), (x, y), VideoMode::NTSC, Some(Color::new(63, 0, 127))).unwrap();
    // Interlaced side-by-side disp/draw:
    //let mut fb = Framebuffer::new((0, 0), (x, 0), (x, y), VideoMode::NTSC, Some(Color::new(63, 0, 127))).unwrap();
    //let mut fb = Framebuffer::new((0, 0), (0, y), (x, y), VideoMode::PAL, Some(Color::new(63, 0, 127))).unwrap();
    //let mut fb = Framebuffer::new((0, 0), (0, 256), (320, 256), VideoMode::PAL, Some(Color::new(63, 0, 127))).unwrap();
    //let mut fb = Framebuffer::new((0, 0), (0, 240), (320, 240), VideoMode::PAL, Some(Color::new(63, 0, 127))).unwrap();

    let mut txt = fb.load_default_font().new_text_box((0, 8), (x, y));
    

    let mut gpu_dma = dma::GPU::new();

    // Main loop
    loop {
        dprintln!(txt, "0123456789a123456789b123456789c123456789d123456789e123456789f123456789g\nHELLO, \nWORLD!");
        txt.reset();
        
        // Wait for GPU to finish drawing and V-Blank
        fb.draw_sync();
        fb.wait_vblank();

        // Flip buffers and display
        fb.dma_swap(&mut gpu_dma);
    }
}
