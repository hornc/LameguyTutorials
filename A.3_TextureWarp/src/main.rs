#![no_std]
#![no_main]
#![feature(asm_experimental_arch)]

use psx::constants::*;
use psx::include_tim;
use psx::gpu::primitives::{PolyF4, PolyFT4};
use psx::gpu::{Bpp, Color, DrawEnv, link_list, Packet, TexCoord, Vertex, VideoMode};
use psx::hw::gpu::GP0Command;
use psx::{dma, dprintln, Framebuffer};
use psx::hw::{cop0, Register};
use psx::hw::cop0::{IntSrc};
use psx::hw::gte::{VXY0, VZ0, OFX, OFY, OTZ, VZ2, H, FLAG, IR1, IR2, IR3};

const ENABLE_GTE: u32 = 1 << 30;
const SQR_OP: u32 = 0x0a00428;

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
    let mut fb = Framebuffer::new((0, 0), (0, 240), (320, 240), VideoMode::NTSC, Some(Color::new(70,70,70))).unwrap();

    // --- Select exactly one texture bit-depth feature ---
    #[cfg(feature = "bpp4")]
    let texture_tim = include_tim!("../crate4bit.tim");
    #[cfg(feature = "bpp8")]
    let texture_tim = include_tim!("../crate8bit.tim");
    #[cfg(feature = "bpp15")]
    // Note: stored as 16bpp on disk: 5-5-5 RGB + 1 STP (special transparency processing) bit
    // The GPU's texture-page colour-depth *setting* for this mode is "15bit"
    let texture_tim = include_tim!("../crate16bit.tim");
    #[cfg(feature = "bpp24")]
    let texture_tim = include_tim!("../crate24bit.tim");
    // ----------------------------------------------------

    let bpp = texture_tim.bpp;
    let clt = texture_tim.clut.size;
    let offset = texture_tim.bmp.offset;

    let mut gpu_dma = dma::GPU::new();

    let mut db = 0;  // display buffer 0 or 1

    // Set up 2 x ordering tables in a 2x8 array
    // using the multi-primitive  approach suggested by psx-sdk-rs monkey example
    // .. this is still not a primitive buffer tho
    let mut ot = [const { Packet::new(PolyF { flat: PolyF4::new() }) }; 16];

    link_list(&mut ot[0..8]);
    link_list(&mut ot[8..16]);
    let loaded_font = fb.load_default_font();
    let mut txt = loaded_font.new_text_box((0, 8), (100, 50));

    let loaded_tim = fb.load_tim(texture_tim);

    // Location and Dimensions of the square
    let (x, y) = (132, 132);
    let (h, w) = (64, 64);
    // Location of the sprite
    let (sx, sy) = (148, 148);
    // Texture coordinates for the sprite
    let tex_coords = [(0, 0 + 48), (0, 64 + 48), (64, 0+48), (64, 64+48)].map(|(x, y)| TexCoord { x, y });

    let mut status = cop0::Status::new();
    status.assign(ENABLE_GTE).store();
    let otz = OTZ::new();
    let mut hpos = H::new();
    hpos.assign(123).store();
    let cop2r56 = OFX::new();

    let cop2r1 = VZ0::new();

    let cop2r57 = OFY::new();
    let cop2r0 = VXY0::new();
    hpos.assign(200);  // assign(), but don't .store()
    let cop2r5 = VZ2::new();

    let pos = H::new();

    // SQR, set up input vector
    let mut ir1 = IR1::new();
    ir1.assign(1).store();
    let mut ir2 = IR2::new();
    ir2.assign(3).store();
    let mut ir3 = IR3::new();
    ir3.assign(5).store();

    // SQR send cop2 
    unsafe {
        core::arch::asm! {
            "nop",
            ".long {} & 0x1ffffff | 37 << 25 # cop2",
            const SQR_OP,
            options(nomem, nostack)
        }
    }

    // re-read ir1,2,3 for output vector
    let out_ir1 = IR1::new();
    let out_ir2 = IR2::new();
    let out_ir3 = IR3::new();

    let flag = FLAG::new();

    // Main loop
    loop {
        let (a, b) = ot.split_at_mut(8);
        let (display, draw) = if db == 1 { (a, b) } else { (b, a) };
        dprintln!(txt, "CLUT size:{:?}", clt);
        dprintln!(txt, "GTE: {:?}", status.gte_enabled());
        dprintln!(txt, "OFX: {}", cop2r56.to_bits());
        dprintln!(txt, "VZ0: {}", cop2r1.to_bits());
        dprintln!(txt, "OFY: {}", cop2r57.to_bits());
        dprintln!(txt, "VXY0: {}", cop2r0.to_bits());
        dprintln!(txt, "VZ2: {}", cop2r5.to_bits());
        dprintln!(txt, "HPOS: {}", hpos.to_bits());
        dprintln!(txt, "POS: {}", pos.to_bits());

        dprintln!(txt, "OTZ: {}", otz.to_bits());
        dprintln!(txt, "SQR In X,Y,Z: <{},{},{}>", ir1.to_bits(), ir2.to_bits(), ir3.to_bits());
        dprintln!(txt, "SQR Out X,Y,Z: <{},{},{}>", out_ir1.to_bits(), out_ir2.to_bits(), out_ir3.to_bits());
        dprintln!(txt, "FLAG: {:b}", flag.to_bits());
        txt.reset();
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
                .set_color(Color::new(23, 24, 78));
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
