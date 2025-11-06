#![no_std]
#![no_main]

use psx::constants::*;
use psx::include_tim;
use psx::gpu::primitives::{PolyF3, PolyFT4};
use psx::gpu::{Color, link_list, Packet, TexCoord, Vertex, VideoMode};
use psx::hw::gpu::GP0Command;
use psx::{dma, dprintln, Framebuffer};
use psx::sys::gamepad::{Gamepad, Button};
use psx::math::{f16, rotate_z, Rad, sin, cos};

// Following the GCC based PsyQ SDK / PSn00bSDK tutorial at:
// https://web.archive.org/web/20240916202225/http://lameguy64.net/tutorials/pstutorials/chapter1/6-cdreading.html
// Builds on the Fixed Point Maths example ....


const NTSC: bool = true;  // toggle between NTSC and PAL modes and texture

const ANG: Rad = Rad(512);  // Angle in radians to rotate by each keypress
//const SPEED: i8 = 2;      // Movement speed multiplier
const W: i16 = 320;
const H: i16 = 240;
const X_WRAP: i8 = 107;
const Y_WRAP: i8 = 80;


#[repr(C)]
union PolyF {
    tri: PolyF3,
    text: PolyFT4,
}

pub trait SafeUnionAccess {
    fn as_tri(&mut self) -> &mut PolyF3;
    fn as_text(&mut self) -> &mut PolyFT4;
}
impl SafeUnionAccess for Packet<PolyF> { // taken from ayrtonm/psx-sdk-rs/tree/main/examples/monkey/src/main.rs
    fn as_tri(&mut self) -> &mut PolyF3 {
        unsafe { self.resize::<PolyF3>().contents.tri.reset_cmd() }
    }
    fn as_text(&mut self) -> &mut PolyFT4 {
        unsafe { self.resize::<PolyFT4>().contents.text.reset_cmd() }
    }
}

impl GP0Command for PolyF {}


#[no_mangle]
fn main() {
    // Init graphics and stuff
    let mut fb = if NTSC {
        Framebuffer::new((0, 0), (0, H), (W, H), VideoMode::NTSC, Some(INDIGO)).unwrap()
    } else {   // PAL
        Framebuffer::new((0, 0), (0, 256), (W, 256), VideoMode::PAL, Some(INDIGO)).unwrap()
    };

    let texture_tim = if NTSC {
        include_tim!("../../images/texture64_320x240_shift-NTSC.tim")  // shifted to not overwrite psx-sdk-rs font.tim
    } else {
        include_tim!("../../images/texture64_320x256-PAL.tim")
    };

    let mut txt = fb.load_default_font().new_text_box((0, 8), (W, H));
    let mut gpu_dma = dma::GPU::new();
    let mut db = 0;  // display buffer 0 or 1

    // Set up 2 x ordering tables in a 2x8 array
    let mut ot = [const { Packet::new(PolyF { tri: PolyF3::new() }) }; 16];

    link_list(&mut ot[0..8]);
    link_list(&mut ot[8..16]);

    let loaded_tim = fb.load_tim(texture_tim);
    let (h, w) = (64, 64);
    // Location of the sprite
    let (sx, sy) = (48, 48);
    // Texture coordinates for the sprite
    //let tex_coords = [(0, 0), (0, 64), (64, 0), (64, 64)].map(|(x, y)| TexCoord { x, y });
    let tex_coords = [(0, 0 + 48), (0, 64 + 48), (64, 0 + 48), (64, 64 + 48)].map(|(x, y)| TexCoord { x, y });

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
        let rotated_tri = player_tri.map(|v| rotate_z(v, angle));
        let rotated_sq = [(sx, sy), (sx, sy+h), (sx+w, sy), (sx+w, sy+h)].map(|v| rotate_z([f16::from_int(v.0), f16::from_int(v.1), f16::ZERO], angle));

        gpu_dma.send_list_and(display, || {
            draw[0]
                .as_text().set_vertices(rotated_sq.map(|[x,y,_z]| Vertex((x + pos_x).to_int_lossy() * 3 / 2 + W / 2, (y + pos_y).to_int_lossy() * 3 / 2 + H / 2)))
                .set_color(Color::new(255, 255, 255))
                .set_tex_page(loaded_tim.tex_page)
                .set_tex_coords(tex_coords)
                .set_clut(loaded_tim.clut.unwrap());
            draw[1]
                .as_tri().set_vertices(rotated_tri.map(|[x,y,_z]| Vertex((x + pos_x).to_int_lossy() * 3 / 2 + W / 2, (y + pos_y).to_int_lossy() * 3 / 2 + H / 2)))
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
