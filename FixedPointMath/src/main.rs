#![no_std]
#![no_main]

use psx::constants::*;
use psx::gpu::primitives::{PolyF3};
use psx::gpu::{Color, link_list, Packet, TexCoord, Vertex, VideoMode};
use psx::{dma, dprintln, Framebuffer};
use psx::sys::gamepad::{Gamepad, Button};
use psx::math::{f16};

// Following the GCC based PsyQ SDK / PSn00bSDK tutorial at:
// http://lameguy64.net/tutorials/pstutorials/chapter1/5-fixedpoint.html
// Builds on the Controllers example ....


const NTSC: bool = false;  // toggle between NTSC and PAL modes and texture


#[no_mangle]
fn main() {
    // Init graphics and stuff
    let mut fb = if NTSC {
        Framebuffer::new((0, 0), (0, 240), (320, 240), VideoMode::NTSC, Some(INDIGO)).unwrap()
    } else {   // PAL
        Framebuffer::new((0, 0), (0, 256), (320, 256), VideoMode::PAL, Some(INDIGO)).unwrap()
    };

    let mut txt = fb.load_default_font().new_text_box((100, 8), (320, 224));
    let mut gpu_dma = dma::GPU::new();
    let mut db = 0;  // display buffer 0 or 1

    // Set up 2 x ordering tables in a 2x8 array
    // using the multi-primitive  approach suggested by psx-sdk-rs monkey example
    // .. this is still not a primitive buffer tho
    let mut ot = [const { Packet::new(PolyF3::new()) }; 16];

    link_list(&mut ot[0..8]);
    link_list(&mut ot[8..16]);

    // Location and Dimensions of the player object 
    let (x, y) = (32, 32);
    let (h, w): (i16, i16) = (64, 64);
    // Location of the player 
    //let (mut sx, mut sy) = (48, 48);
    let mut sx = f16::from_int(127);
    let mut sy = f16::from_int(127);
    let mut angle = f16::from_int(0);

    let mut gamepad = Gamepad::new();

    // Main loop
    loop {
        let mut gp = gamepad.poll_p1();
        if gp.pressed(Button::Right) {
            sx += f16::ONE;
        } else if gp.pressed(Button::Left) {
            sx -= f16::ONE;
        }
        if gp.pressed(Button::Up) {
            sy -= f16::ONE;
        } else if gp.pressed(Button::Down) {
            sy += f16::ONE;
        }

        let (a, b) = ot.split_at_mut(8);
        let (display, draw) = if db == 1 { (a, b) } else { (b, a) };
        gpu_dma.send_list_and(display, || {
            draw[0]
                .contents.set_vertices([Vertex(x, y), Vertex(x+w, y), Vertex(x, y+h)])
                .set_color(YELLOW);
        });

        // Display our fixed point values on screen:
        dprintln!(txt, "POS_X={:#x?}", sx);
        dprintln!(txt, "POS_Y={:#x?}", sy);
        dprintln!(txt, "ANGLE={:#x?}", angle);
        txt.reset();

        // Wait for GPU to finish drawing and V-Blank
        fb.draw_sync();
        fb.wait_vblank();

        // Flip buffers and display
        fb.dma_swap(&mut gpu_dma);
        // switch display / draw ot lists
        db = 1 - db;
    }
}
