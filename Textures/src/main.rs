#![no_std]
#![no_main]

use psx::constants::*;
use psx::include_tim;
use psx::gpu::primitives::{PolyFT4};
use psx::gpu::{Color, link_list, Packet, TexCoord, Vertex, VideoMode};
use psx::{dma, Framebuffer};


// Following the GCC based PsyQ SDK / PSn00bSDK tutorial at:
// http://lameguy64.net/tutorials/pstutorials/chapter1/3-textures.html
// Builds on the Yellow Square example, but loads a TIM and textures a second primitive.


const NTSC: bool = false;  // toggle between NTSC and PAL modes and texture


#[no_mangle]
fn main() {
    // Init graphics and stuff
    let mut fb = if NTSC {
        Framebuffer::new((0, 0), (0, 240), (320, 240), VideoMode::NTSC, Some(INDIGO)).unwrap()
    } else {   // PAL
        Framebuffer::new((0, 0), (0, 256), (320, 256), VideoMode::PAL, Some(INDIGO)).unwrap()
    };
    let texture_tim = if NTSC {
        include_tim!("../texture64_320x240-NTSC.tim")
    } else {
        include_tim!("../texture64_320x256-PAL.tim")
    };

    let mut gpu_dma = dma::GPU::new();

    let mut db = 0;  // display buffer 0 or 1

    // Set up 2 x ordering tables in a 2x8 array
    // TODO: set up a primitive buffer as mentioned in the tutorial so we aren't stuck with
    // an ordering table per primitive type...
    let mut ot = [const { Packet::new(PolyFT4::new())}; 16];
    
    let ot = &mut ot;
    link_list(&mut ot[0..8]);
    link_list(&mut ot[8..16]);

    let loaded_tim = fb.load_tim(texture_tim);  // contains the TexPage and CLUT (if any)

    // Location and Dimensions of the square
    let (x, y) = (32, 32);
    let (h, w) = (64, 64);
    // Location of the sprite
    let (sx, sy) = (48, 48);
    // Texture coordinates for the sprite
    let tex_coords = [(0, 0), (0, 64), (64, 0), (64, 64)].map(|(x, y)| TexCoord { x, y });

    // Main loop
    loop {
        let (a, b) = ot.split_at_mut(8);
        let (display, draw) = if db == 1 { (a, b) } else { (b, a) };
        gpu_dma.send_list_and(display, || {
            draw[1]
                // TODO: be clear about Vertex and TexCoord ordering!
                //.contents.set_vertices([Vertex(sx, sy), Vertex(sx, sy+h), Vertex(sx+w, sy), Vertex(sx+w, sy+h)])
                .contents.set_vertices([(sx, sy), (sx, sy+h), (sx+w, sy), (sx+w, sy+h)].map(|v| Vertex::new(v)))
                //.set_color(Color::new(128, 128, 128))
                .set_color(Color::new(255, 255, 255))
                //.set_color(WHITE)
                .set_tex_page(loaded_tim.tex_page)
                .set_tex_coords(tex_coords)
                .set_clut(loaded_tim.clut.unwrap());
            draw[0]  // TODO: this should be a PolyF4 (without texture!)
                .contents.set_vertices([Vertex(x, y), Vertex(x+w, y), Vertex(x, y+h), Vertex(x+w, y+h)])
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
