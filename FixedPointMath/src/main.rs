#![no_std]
#![no_main]

use psx::constants::*;
use psx::gpu::primitives::{PolyF3};
use psx::gpu::{Color, link_list, Packet, TexCoord, Vertex, VideoMode};
use psx::{dma, dprintln, Framebuffer};
use psx::sys::gamepad::{Gamepad, Button};
use psx::math::{f16, rotate_z, Rad};

// Following the GCC based PsyQ SDK / PSn00bSDK tutorial at:
// http://lameguy64.net/tutorials/pstutorials/chapter1/5-fixedpoint.html
// Builds on the Controllers example ....


const NTSC: bool = false;  // toggle between NTSC and PAL modes and texture

const ANG: Rad = Rad(512);  // Angle in radians to rotate by each keypress


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

    // Location of the player 
    let mut pos_x = f16::from_int(60);
    let mut pos_y = f16::from_int(60);
    let mut angle = Rad(0);

    let mut gamepad = Gamepad::new();

    let player_tri = [
        [0, -20, 0],
        [10, 20, 0],
        [-10, 20, 0]
    ].map(|v| v.map(|e| f16::from_int(e)));

    // Main loop
    loop {
        let gp = gamepad.poll_p1();
        if gp.pressed(Button::Right) {
            angle += ANG;
        } else if gp.pressed(Button::Left) {
            angle -= ANG;
        }
        if gp.pressed(Button::Up) {
            pos_y -= f16::ONE;
        } else if gp.pressed(Button::Down) {
            pos_y += f16::ONE;
        }

        let (a, b) = ot.split_at_mut(8);
        let (display, draw) = if db == 1 { (a, b) } else { (b, a) };
        gpu_dma.send_list_and(display, || {
            let rotated_tri =
                player_tri.map(|v| rotate_z(v, angle));
            draw[0]
                .contents.set_vertices(rotated_tri.map(|[x,y,z]| Vertex((x + pos_x).to_int_lossy(), (y + pos_y).to_int_lossy())))
                .set_color(YELLOW);
        });

        // Display fixed point f16, and Rad values on screen:
        dprintln!(txt, "POS_X={:#x?} ({})", pos_x, pos_x.to_int_lossy());
        dprintln!(txt, "POS_Y={:#x?} ({})", pos_y, pos_y.to_int_lossy());
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
