#![no_std]
#![no_main]

use psx::constants::*;
use psx::gpu::primitives::PolyF4;
use psx::gpu::{link_list, Packet, Vertex, VideoMode};
use psx::{dma, Framebuffer};


// Following the GCC based PsyQ SDK / PSn00bSDK tutorial at:
// http://lameguy64.net/tutorials/pstutorials/chapter1/2-graphics.html

#[no_mangle]
fn main() {
    // Init graphics and stuff
    let mut fb = Framebuffer::new((0, 0), (0, 240), (320, 240), VideoMode::NTSC, Some(INDIGO)).unwrap();
    //let mut fb = Framebuffer::new((0, 0), (0, 256), (320, 256), VideoMode::PAL, Some(INDIGO)).unwrap();

    let mut gpu_dma = dma::GPU::new();

    let mut db = 0;  // display buffer 0 or 1

    // Set up 2 x ordering tables in a 2x8 array
    let mut ot = [const { Packet::new(PolyF4::new()) }; 16];
    
    let ot = &mut ot;
    link_list(&mut ot[0..8]);
    link_list(&mut ot[8..16]);

    // Location and Dimensions of the square
    let (x, y) = (32, 32);
    let (h, w) = (64, 64);


    // Main loop
    loop {
        let (a, b) = ot.split_at_mut(8);
        let (display, draw) = if db == 1 { (a, b) } else { (b, a) };
        gpu_dma.send_list_and(display, || {
            draw[0]
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
