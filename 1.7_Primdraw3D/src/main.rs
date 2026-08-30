#![no_std]
#![no_main]
#![feature(asm_experimental_arch)]

use psx::{dma, dprintln, Framebuffer};
use psx::constants::*;
use psx::gpu::primitives::{PolyF3};
use psx::gpu::{Color, link_list, Packet, TexCoord, Vertex, VideoMode};
use psx::hw::{cop0, gte, Register};
use psx::math::{sin, cos, Rad};
use psx::sys::gamepad::{Gamepad, Button};

const NTSC: bool = true;  // toggle between NTSC and PAL modes and texture
                          //
// Attempt to convert Lameguy64 / Meido-Tek primdraw.c 3D example from C to Rust..

/*	primdraw.c by Lameguy64
	2014 Meido-Tek Productions.
	
	Demonstrates:
		- Using a primitive OT to draw triangles without libgs.
		- Using the GTE to rotate, translate, and project 3D primitives.
	
	Controls:
		Start							- Toggle interactive/non-interactive mode.
		Select							- Reset object's position and angles.
		L1/L2							- Move object closer/farther.
		L2/R2							- Rotate object (XY).
		Up/Down/Left/Right				- Rotate object (XZ/YZ).
		Triangle/Cross/Square/Circle	- Move object up/down/left/right.
		
*/

pub struct Trans {
    pub x: i16,
    pub y: i16,
    pub z: i16,
}

pub struct Rotate {
    pub x: i16,
    pub y: i16,
    pub z: i16,
}

const CENTERX: i16 = 320;
const CENTERY: i16 = 120;
const OT_LENGTH: usize = 48;  // breaks at 64, 50?  works at 32, 40, 48, 49
const ENABLE_GTE: u32 = 1 << 30;
const RTPS: u32 = 0x0180001;
const ONE: i16 = 1<<12;


#[inline(always)]
pub fn delay() {
    unsafe {
        core::arch::asm!(
            "nop",
         //   "nop",
        );
    }
}

/// Convert Euler rotation vector into fixed point rotation matrix
fn rot_matrix(rotate: &Rotate, matrix: &mut [[i16; 3]; 3]) {
    // Test: lets just do roll, about z axis:
    let cos_z = cos(Rad(rotate.z as u16)).to_bits() as i16;
    let sin_z = sin(Rad(rotate.z as u16)).to_bits() as i16;
    matrix[0][0] = cos_z;
    matrix[1][0] = -sin_z;
    matrix[0][1] = sin_z;
    matrix[1][1] = cos_z;
    matrix[2][2] = ONE;

}

/// Sends matrix to the GTE
fn set_rot_matrix(matrix: [[i16; 3]; 3]) {
    delay();
    let mut r1 = gte::RT11_12::new();
    let v1 = (ONE | matrix[0][0]) as u32 | ((matrix[0][1] as u32 ) << 28);
    delay();
    r1.assign(v1).store();
    delay();
    let mut r2 = gte::RT13_21::new();
    let v2 = (matrix[0][2]) as u32 | ((matrix[1][0] as u32 ) << 28);
    delay();
    r2.assign(v2).store();
    delay();
    let mut r3 = gte::RT22_23::new();
    let v3 = (ONE | matrix[1][1]) as u32 | ((matrix[1][2] as u32 ) << 28);
    delay();
    r3.assign(v3).store();
    delay();
    let mut r4 = gte::RT31_32::new();
    let v4 = (matrix[2][0]) as u32 | ((matrix[2][1] as u32 ) << 28);
    delay();
    r4.assign(v4).store();
    delay();
    let mut r5 = gte::RT33::new();
    let v5 = ONE | matrix[2][2];
    delay();
    r5.assign(v5).store();
    delay();
    delay();
    delay(); // 3
}


fn rot_trans_pers(v0: (i16, i16, i16), sxy: &mut Vertex, p: &u32, flag: &mut u32) -> i32 {
    let flag_reg = gte::FLAG::new();
    let z_reg = gte::IR0::new();
    //let z_reg = gte::SZ0::new();
    let mut vxy0 = gte::VXY0::new();
    let mut vz0 = gte::VZ0::new();
    
    // Send vector v0 to GTE:
    vxy0.assign((v0.1 as u32) << 16 | (v0.0) as u32).store();
    //delay();
    vz0.assign(v0.2).store();
    //delay();

    // Send RTPS GTE command to cop2
    unsafe {
        core::arch::asm! {
            ".long {} & 0x1ffffff | 37 << 25 # cop2",
            const RTPS,
            options(nomem, nostack)
        }
    }

    *flag = flag_reg.to_bits();
    //let sx = gte::MAC1::new().to_bits() as i16;
    let sx = gte::IR1::new().to_bits() as i16;
    delay();
    //let sy = gte::MAC2::new().to_bits() as i16;
    let sy = gte::IR2::new().to_bits() as i16;
    delay();
    
    //assert!(sxa == sx);
    //assert!(sya == sy);   // These asserts added just enough delay to allow this code to work ... without them, sxy is not correctly set!
                          // TODO: determine WHERE the correct delay needs to be added!
    *sxy = Vertex(sx, sy);
    z_reg.to_bits() as i32  // screen Z
}


#[no_mangle]
fn main() {
    let mut gpu_dma = dma::GPU::new();

    // Full Hi-res is 640 x 480 as used in primdraw.c,
    // can't get a double buffer with that y res
               //Framebuffer::new((0, 0), (0, 240), (320, 240), VideoMode::NTSC, Some(INDIGO)).unwrap()
    let mut fb = Framebuffer::new((0, 0), (0, 240), (640, 240), VideoMode::NTSC, Some(BLUE)).unwrap();

    let mut txt = fb.load_default_font().new_text_box((0, 0), (520, 200));
    let mut db = 0;  // display buffer 0 or 1
    let mut autorotate = 0;
    let mut tpressed = 0;
    let mut p = 0;   // idk what this is for, used by RTPS?
    let mut flag = 0;  // GTE status flag

    // Enable GTE:
    let mut status = cop0::Status::new();
    status.assign(ENABLE_GTE).store();

    // If gamepad init occurs before GTE enable, gamepad doesn't work ...
    let mut gamepad = Gamepad::new();

    // Set up 2 x ordering tables in a OT_LENGTH array
    let mut polys_a = [const { Packet::new(PolyF3::new()) }; OT_LENGTH];
    let mut polys_b = [const { Packet::new(PolyF3::new()) }; OT_LENGTH];
    let mut my_prims: [PolyF3; 1024] = [Default::default(); 1024];
    link_list(&mut polys_a);
    link_list(&mut polys_b);

    // Location and Dimensions of the object 
    let mut trans = Trans{ x: CENTERX, y: CENTERY, z: 0 };
    let mut rotate = Rotate{ x: 0, y: 0, z: 0 };

    let (h, w) = (64, 64);
    let vec_mesh = [  // list of triangle vertices in 3d space
        (50, 50, 10), (25, 25, 10), (50, 10, 10),
        //(10, 10, -1), (15, 5, -1), (20, 10, -1),
        //(250, 150, 0), (225, 125, 0), (225, 110, 0),  
        //(50, 50, 0), (25, 25, 0), (50, 10, 0),
       
    ];
    // OFX, OFY, and H don't appear to be having an effect on screen position, but does on the
    // reported coords?
    gte::OFX::new().assign((CENTERX as u32) << 16 ).store();  // TODO: should OFX and OFY be i16? NO!, 1 sign, 15bit int, 16 frac
    gte::OFY::new().assign((CENTERY as u32) << 16 ).store();
    //    SetGeomScreen(CENTERX);
    gte::H::new().assign(CENTERX).store();


    let mut matrix: [[i16; 3]; 3] = [
        [ONE, 0, 0],
        [0, ONE, 0],
        [0, 0, ONE],
    ];

    set_rot_matrix(matrix);

    // Main loop
    loop {
        dprintln!(txt, "after a");
        dprintln!(txt, " SIMPLE 3D EXAMPLE BY LAMEGUY64");
		dprintln!(txt, " 2014 MEIDO-TEK PRODUCTIONS");
		dprintln!(txt, " WWW.PSXDEV.NET");
		dprintln!(txt, "autorotate: {}", autorotate);
        //dprintln!(txt, "DEBUG: {:b}", r5.to_bits());

        let gp = gamepad.poll_p1();
        if autorotate == 0 {
            if gp.pressed(Button::L1) {
                trans.z -= 4;
            } else if gp.pressed(Button::L2) {
                trans.z += 4;
            }
            if gp.pressed(Button::R1) {
                rotate.z -= 8;
            } else if gp.pressed(Button::R2) {
                rotate.z += 8;
            }

            if gp.pressed(Button::Triangle) {
                trans.y -= 2;
            } else if gp.pressed(Button::Cross) {
                trans.y += 2;
            }
            if gp.pressed(Button::Square) {
                trans.x -= 4;
            } else if gp.pressed(Button::Circle) {
                trans.x += 4;
            }

            if gp.pressed(Button::Up) {
                rotate.y -= 8;
            } else if gp.pressed(Button::Down) {
                rotate.y += 8;
            }
            if gp.pressed(Button::Left) {
                rotate.x -= 8;
            } else if gp.pressed(Button::Right) {
                rotate.x += 8;
            }

            if gp.pressed(Button::Select) {
                rotate = Rotate{x: 0, y: 0, z: 0};
                trans = Trans{x: CENTERX, y: CENTERY, z: 0};
            }
        } else {  // Autorotate
			//rotate.y += 8;	// Pan
			//rotate.x += 8;	// Tilt
			rotate.z += 8;  // Roll
        }

        if gp.pressed(Button::Start) {
            if tpressed == 0 {
                autorotate = (autorotate + 1) & 1;
                rotate = Rotate{x: 0, y: 0, z: 0};
                trans = Trans{x: CENTERX, y: CENTERY, z: 0};
            }
            tpressed = 1;
        } else {
            tpressed = 0;
        }

        // TODO: Use trig fns to covnert pitch / yaw / roll / tilt &c to rotation matrix....
        set_rot_matrix(matrix);

        /// Set Translation vector:
        delay();
        gte::TRX::new().assign(trans.x as i32).store();
        delay();
        gte::TRY::new().assign(trans.y as i32).store();
        delay();
        gte::TRZ::new().assign(trans.z as i32).store();
        delay();
        delay();  // appears critical

        rot_matrix(&rotate, &mut matrix);

        db = 1 - db;

        let (display, draw) = if db == 1 { (&mut polys_a, &mut polys_b) } else { (&mut polys_b, &mut polys_a) }; 
        let mut debug_screen_vertex: [Vertex; 3] = [Vertex(0, 0); 3];

        gpu_dma.send_list_and(display, || {
            let mut my_prim = 0;
            for i in (0..vec_mesh.len()).step_by(3) {
                let mut vertices: [Vertex; 3] = my_prims[my_prim].get_vertices();
                let mut ot_z = rot_trans_pers(vec_mesh[i], &mut vertices[0], &p, &mut flag);
                ot_z += rot_trans_pers(vec_mesh[i+1], &mut vertices[1], &p, &mut flag);
                ot_z += rot_trans_pers(vec_mesh[i+2], &mut vertices[2], &p, &mut flag);
                ot_z /= 3;
                //dprintln!(txt, "debug: {:?}", flag);
                my_prims[my_prim].set_vertices(vertices);  // Do I need my_prims here? just an empty Poly3F?
                debug_screen_vertex = vertices; 

                draw[my_prim]
                    .contents.set_vertices(vertices)
                    .set_color(YELLOW);
                my_prim += 1;
            }
        });
        dprintln!(txt, "last prim: {:?}", debug_screen_vertex);
		dprintln!(txt, "FLAG: {:b}", flag);
        txt.reset();

/*
		// Render the sample vector model
		t=0;
		for (i=0; i<vec.len; i++) {
			
			// Initialize the primitive and set its color values
			SetPolyF3(&myPrims[ActivePage][myPrimNum]);
			setRGB0(&myPrims[ActivePage][myPrimNum], vec_color[i].r, vec_color[i].g, vec_color[i].b);
			
			// Rotate, translate, and project the vectors and output the results into a primitive
			OTz = RotTransPers(&vec_mesh[t], (long*)&myPrims[ActivePage][myPrimNum].x0, &p, &Flag);
			OTz += RotTransPers(&vec_mesh[t+1], (long*)&myPrims[ActivePage][myPrimNum].x1, &p, &Flag);
			OTz += RotTransPers(&vec_mesh[t+2], (long*)&myPrims[ActivePage][myPrimNum].x2, &p, &Flag);
			
			// Sort the primitive into the OT
			OTz /= 3;
			if ((OTz > 0) && (OTz < OT_LENGTH))
				AddPrim(&myOT[ActivePage][OTz-2], &myPrims[ActivePage][myPrimNum]);
			
			myPrimNum++;
			t+=3;
		}
*/
        // Wait for GPU to finish drawing and V-Blank
        fb.draw_sync();
        fb.wait_vblank();

        // Flip buffers and display
        fb.dma_swap(&mut gpu_dma);
    }
}
