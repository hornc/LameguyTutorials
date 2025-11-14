#![feature(asm_experimental_arch)]
#![no_std]
#![no_main]

use psx::gpu::{Color, VideoMode};
use psx::sys::gamepad::{Gamepad, Button};
use psx::sys::kernel::{psx_get_timer};
use psx::sys::rng::{Rng};
use psx::{dma, dprintln, Framebuffer};
use core::arch::asm;

/* Random number testing
 *
 *
 */

const DEBOUNCE: u8 = 5;  // Number of VBlank waits (i.e. loops) for button debounce

fn get_timer(t: u32) -> u32 {
    // t is one of: 0, 1, 2
    let time: u32;
    unsafe {
        psx_get_timer(t);
        asm!(
            // Move the value from the v0 (r2) register into the output variable
            "move {0}, $v0",
            out(reg) time,
            options(nomem, nostack, preserves_flags)
        );
    }
    time
}


#[no_mangle]
fn main() {
    // Init graphics and stuff
    let mut fb = Framebuffer::new((0, 0), (0, 240), (320, 240), VideoMode::NTSC, Some(Color::new(63, 100, 127))).unwrap();
    //let mut fb = Framebuffer::new((0, 0), (0, 256), (320, 256), VideoMode::PAL, Some(Color::new(63, 0, 127))).unwrap();

    let mut txt = fb.load_default_font().new_text_box((8, 58), (320, 224));
    let mut number = fb.load_default_font().new_text_box((100, 100), (80, 40));
    let mut debug: bool = false;
    let mut debounce: u8 = 0;  // debounce counter for gamepad
    let mut time: u32 = 0;
    
    let mut rng = Rng::new(0);  // We want to reseed this before use with time and user input

    let mut gpu_dma = dma::GPU::new();

    let mut n: u8 = 0;

    let mut gamepad = Gamepad::new();


    // Main loop
    loop {
        if debounce == 0 {
            let gp = gamepad.poll_p1();
            if gp.pressed(Button::Cross) {
                if time == 0 {  // Reseed the RNG on the first X press using kernel timer
                    time = get_timer(0);
                    rng.reseed(time);
                }
                n = rng.rand();  // Get a new random number
                debounce = DEBOUNCE;
            } else if gp.pressed(Button::Circle) {  // reseed. No real effect other than testing get_timer()
                time = get_timer(0);
                rng.reseed(time);
                debounce = DEBOUNCE;
            } else if gp.pressed(Button::Select) {  // Debug view of debounce counter and current seed
                debug = !debug;
                debounce = DEBOUNCE;
            }
        } else {
            debounce -= 1;
        }
        dprintln!(txt, "Press 'X' for a Random Number:");
        dprintln!(number, "n: {}", n);
        if debug {
            dprintln!(txt, "   (debounce: {})", debounce);
            dprintln!(number, "time/seed: {}", time);
        }

        txt.reset();
        number.reset();
        
        // Wait for GPU to finish drawing and V-Blank
        fb.draw_sync();
        fb.wait_vblank();
        // Flip buffers and display
        fb.dma_swap(&mut gpu_dma);
    }
}
