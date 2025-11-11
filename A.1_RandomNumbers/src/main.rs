#![feature(asm_experimental_arch)]
#![no_std]
#![no_main]

use psx::gpu::{Color, VideoMode};
use psx::sys::kernel::{psx_get_timer};
use psx::sys::rng::{Rng};
use psx::{dma, dprintln, Framebuffer};
use core::arch::asm;

/* Random number testing
 *
 *
 */

#[no_mangle]
fn main() {

    // Init graphics and stuff
    let mut fb = Framebuffer::new((0, 0), (0, 240), (320, 240), VideoMode::NTSC, Some(Color::new(63, 100, 127))).unwrap();
    //let mut fb = Framebuffer::new((0, 0), (0, 256), (320, 256), VideoMode::PAL, Some(Color::new(63, 0, 127))).unwrap();

    let mut txt = fb.load_default_font().new_text_box((0, 8), (320, 224));
    let mut number = fb.load_default_font().new_text_box((100, 100), (80, 40));

    let mut time: u32 = 0;
    
    let mut rng = Rng::new(1234);

    let mut gpu_dma = dma::GPU::new();

    let mut n: u8 = 0; // rng.rand();
    let mut i: u8 = 0;


    // TODO: Use button presses to init timer as seed, and get next random number ...

    // Main loop
    loop {
        dprintln!(txt, "Random Number:");
        dprintln!(number, "n: {}", n);
        dprintln!(number, "time/seed: {:?}", time);
        txt.reset();
        number.reset();
        
        // Wait for GPU to finish drawing and V-Blank
        fb.draw_sync();
        fb.wait_vblank();
        i += 1;
        if i == 50 {
            unsafe {
                psx_get_timer(0);
                asm!(
                    // Move the value from the v0 (r2) register into the output variable
                    "move {0}, $v0",
                    out(reg) time,
                    options(nomem, nostack, preserves_flags)
                );
            }
            rng.reseed(time);
            n = rng.rand();
        }
        // Flip buffers and display
        fb.dma_swap(&mut gpu_dma);
    }
}
