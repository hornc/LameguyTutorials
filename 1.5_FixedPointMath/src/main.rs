#![no_std]
#![no_main]

use psx::constants::*;
use psx::gpu::primitives::{PolyF3};
use psx::gpu::{Color, link_list, Packet, TexCoord, Vertex, VideoMode};
use psx::{dma, dprintln, Framebuffer};
use psx::sys::gamepad::{Gamepad, Button};
use psx::math::{f16, rotate_z, Rad, sin, cos};

// Following the GCC based PsyQ SDK / PSn00bSDK tutorial at:
// http://lameguy64.net/tutorials/pstutorials/chapter1/5-fixedpoint.html
// Builds on the Controllers example ....


const NTSC: bool = true;  // toggle between NTSC and PAL modes and texture

const ANG: Rad = Rad(512);  // Angle in radians to rotate by each keypress
const SPEED: i8 = 2;        // Movement speed multiplier
const W: i16 = 320;
const H: i16 = 240;
const X_WRAP: i8 = 107;
const Y_WRAP: i8 = 80;


#[no_mangle]
fn main() {
    // Init graphics and stuff
    let mut fb = if NTSC {
        Framebuffer::new((0, 0), (0, H), (W, H), VideoMode::NTSC, Some(INDIGO)).unwrap()
    } else {   // PAL
        Framebuffer::new((0, 0), (0, 256), (W, 256), VideoMode::PAL, Some(INDIGO)).unwrap()
    };

    let mut txt = fb.load_default_font().new_text_box((0, 8), (W, H));
    let mut gpu_dma = dma::GPU::new();
    let mut db = 0;  // display buffer 0 or 1

    // Set up 2 x ordering tables in a 2x8 array
    // using the multi-primitive  approach suggested by psx-sdk-rs monkey example
    // .. this is still not a primitive buffer tho
    let mut ot = [const { Packet::new(PolyF3::new()) }; 16];

    link_list(&mut ot[0..8]);
    link_list(&mut ot[8..16]);

    // Location of the player 
    let mut pos_x = f16::from_int(0);
    let mut pos_y = f16::from_int(0);
    let mut vel_x = f16::from_int(0);
    let mut vel_y = f16::from_int(0);
    let mut angle = Rad(0);

    let mut gamepad = Gamepad::new();

    let player_tri = [
        [0, -10, 0],
        [5, 10, 0],
        [-5, 10, 0]
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
            //pos_x += sin(angle) * SPEED;
            //pos_y -= cos(angle) * SPEED;
            vel_x += sin(angle) / 8;
            vel_y -= cos(angle) / 8;
        } else if gp.pressed(Button::Down) {
            //pos_x -= sin(angle) * SPEED;
            //pos_y += cos(angle) * SPEED;
            vel_x -= sin(angle) / 8;
            vel_y += cos(angle) / 8;
        }

        // Accumulate player coordinates by velocity
        pos_x += vel_x;
        pos_y += vel_y;

        // Wrap player coordinates
        if pos_x.to_int_lossy() > X_WRAP as i16 {
            pos_x = pos_x.fract() - f16::from_int(X_WRAP);
        } else if pos_x.to_int_lossy() < -X_WRAP as i16 {
            pos_x = pos_x.fract() + f16::from_int(X_WRAP);
        }

        if pos_y.to_int_lossy() > Y_WRAP as i16 {
            pos_y = pos_y.fract() - f16::from_int(Y_WRAP);
        } else if pos_y.to_int_lossy() < -Y_WRAP as i16 {
            pos_y = pos_y.fract() + f16::from_int(Y_WRAP);
        }

        let (a, b) = ot.split_at_mut(8);
        let (display, draw) = if db == 1 { (a, b) } else { (b, a) };
        gpu_dma.send_list_and(display, || {
            let rotated_tri =
                player_tri.map(|v| rotate_z(v, angle));
            draw[0]
                .contents.set_vertices(rotated_tri.map(|[x,y,z]| Vertex((x + pos_x).to_int_lossy() * 3 / 2 + W / 2, (y + pos_y).to_int_lossy() * 3 / 2 + H / 2)))
                .set_color(YELLOW);
        });

        // Display fixed point f16, and Radian values
        dprintln!(txt, "POS_X={:#06x} ({}.{:04})", pos_x.0, pos_x.to_int_lossy(), pos_x.fract().0 * 39);
        dprintln!(txt, "POS_Y={:#06x} ({}.{:04})", pos_y.0, pos_y.to_int_lossy(), pos_y.fract().0 * 39);
        dprintln!(txt, "VEL_X={:#06x} ({}.{:04})", vel_x.0, vel_x.to_int_lossy(), vel_x.fract().0 * 39);
        dprintln!(txt, "VEL_Y={:#06x} ({}.{:04})", vel_y.0, vel_y.to_int_lossy(), vel_y.fract().0 * 39);
        dprintln!(txt, "ANGLE={}", angle.0 as i16);

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
