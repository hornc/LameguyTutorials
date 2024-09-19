#![no_std]
#![no_main]

use psx::constants::*;
use psx::include_tim;
use psx::gpu::primitives::{PolyF4, PolyFT4};
use psx::gpu::{Color, link_list, Packet, TexCoord, Vertex, VideoMode};
use psx::hw::gpu::GP0Command;
use psx::{dma, Framebuffer};
use psx::sys::gamepad::{Gamepad, Button};


// Following the GCC based PsyQ SDK / PSn00bSDK tutorial at:
// http://lameguy64.net/tutorials/pstutorials/chapter1/4-controllers.html
// Builds on the Textures example, and uses the controller to move the textured square.


const NTSC: bool = false;  // toggle between NTSC and PAL modes and texture


#[repr(C)]
union PolyF {
    flat: PolyF4,
    text: PolyFT4,
}


pub trait SafeUnionAccess {
    fn as_flat(&mut self) -> &mut PolyF4;
    fn as_text(&mut self) -> &mut PolyFT4;
}
impl SafeUnionAccess for Packet<PolyF> { // taken from ayrtonm/psx-sdk-rs/tree/main/examples/monkey/src/main.rs
    fn as_flat(&mut self) -> &mut PolyF4 {
        // SAFETY: We resize the packet to hold a PolyF4 and reset the polygon's command
        // to ensure that the union's PolyF4 is in a valid state when we access it
        unsafe { self.resize::<PolyF4>().contents.flat.reset_cmd() }
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
        Framebuffer::new((0, 0), (0, 240), (320, 240), VideoMode::NTSC, Some(INDIGO)).unwrap()
    } else {   // PAL
        Framebuffer::new((0, 0), (0, 256), (320, 256), VideoMode::PAL, Some(INDIGO)).unwrap()
    };
    let texture_tim = if NTSC {
        include_tim!("../../Textures/texture64_320x240-NTSC.tim")
    } else {
        include_tim!("../../Textures/texture64_320x256-PAL.tim")
    };

    let mut gpu_dma = dma::GPU::new();

    let mut db = 0;  // display buffer 0 or 1

    // Set up 2 x ordering tables in a 2x8 array
    // using the multi-primitive  approach suggested by psx-sdk-rs monkey example
    // .. this is still not a primitive buffer tho
    let mut ot = [const { Packet::new(PolyF { flat: PolyF4::new() }) }; 16];

    link_list(&mut ot[0..8]);
    link_list(&mut ot[8..16]);

    let loaded_tim = fb.load_tim(texture_tim);  // contains the TexPage and CLUT (if any)

    // Location and Dimensions of the square
    let (x, y) = (32, 32);
    let (h, w) = (64, 64);
    // Location of the sprite
    let (mut sx, mut sy) = (48, 48);
    // Texture coordinates for the sprite
    let tex_coords = [(0, 0), (0, 64), (64, 0), (64, 64)].map(|(x, y)| TexCoord { x, y });

    let mut gamepad = Gamepad::new();

    // Main loop
    loop {
        let mut gp = gamepad.poll_p1();
        if gp.pressed(Button::Right) {
            sx += 1;
        } else if gp.pressed(Button::Left) {
            sx -= 1;
        }
        if gp.pressed(Button::Up) {
            sy -= 1;
        } else if gp.pressed(Button::Down) {
            sy += 1;
        }

        let (a, b) = ot.split_at_mut(8);
        let (display, draw) = if db == 1 { (a, b) } else { (b, a) };
        gpu_dma.send_list_and(display, || {
            draw[1]
                // TODO: be clear about Vertex and TexCoord ordering!
                .as_text().set_vertices([(sx, sy), (sx, sy+h), (sx+w, sy), (sx+w, sy+h)].map(|v| Vertex::new(v)))
                .set_color(Color::new(255, 255, 255))
                .set_tex_page(loaded_tim.tex_page)
                .set_tex_coords(tex_coords)
                .set_clut(loaded_tim.clut.unwrap());
            draw[0]
                .as_flat().set_vertices([Vertex(x, y), Vertex(x+w, y), Vertex(x, y+h), Vertex(x+w, y+h)])
                .set_color(YELLOW);
        });

        // Wait for GPU to finish drawing and V-Blank
        fb.draw_sync();
        fb.wait_vblank();

        // Flip buffers and display
        fb.dma_swap(&mut gpu_dma);
        // switch display / draw ot lists
        db = 1 - db;
    }
}
