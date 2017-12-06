extern crate piston_window;
extern crate vecmath;
extern crate camera_controllers;
#[macro_use]
extern crate gfx;
extern crate gfx_voxel;
extern crate shader_version;
extern crate find_folder;
extern crate rand;


use piston_window::*;
use gfx::traits::*;
use gfx_voxel::{array, cube, texture};
use shader_version::Shaders;
use shader_version::glsl::GLSL;
use camera_controllers::{
    FirstPersonSettings,
    FirstPerson,
    CameraPerspective,
    model_view_projection
};
use std::collections::HashMap;
use rand::Rng;

//----------------------------------------
// Cube associated data

gfx_vertex_struct!( Vertex {
    a_pos: [f32; 3] = "a_pos",
    a_tex_coord: [f32; 2] = "a_tex_coord",
    a_color: [f32; 4] = "a_color",
});

impl Vertex {
    fn new(pos: [f32; 3], tc: [f32; 2], col: [f32; 4]) -> Vertex {
        Vertex {
            a_pos: pos,
            a_tex_coord: [tc[0], tc[1]],
            a_color: col,
        }
    }
}

gfx_pipeline!( pipe {
    vbuf: gfx::VertexBuffer<Vertex> = (),
    u_model_view_proj: gfx::Global<[[f32; 4]; 4]> = "u_model_view_proj",
    t_color: gfx::TextureSampler<[f32; 4]> = "t_color",
    out_color: gfx::RenderTarget<::gfx::format::Srgba8> = "o_Color",
    out_depth: gfx::DepthTarget<::gfx::format::DepthStencil> =
        gfx::preset::depth::LESS_EQUAL_WRITE,
});

const TEXWIDTH:f32 = 4.0;
const TRANS:[[[i32;4];2];8] = [
    [[1,0,0,1,],[1,1,0,0,]],
    [[0,0,1,1,],[1,0,0,1,]],
    [[0,1,1,0,],[0,0,1,1,]],
    [[1,1,0,0,],[0,1,1,0,]],
    [[0,1,1,0,],[1,1,0,0,]],
    [[1,1,0,0,],[1,0,0,1,]],
    [[1,0,0,1,],[0,0,1,1,]],
    [[0,0,1,1,],[0,1,1,0,]],
];

#[derive(Copy,Clone)]
struct Block {
    color: [f32;4],
    textrans:[[[i32;4];2];6],
}

impl Block {
    fn new(rng: usize, c: [f32;4]) -> Block {
        Block {
            color: c,
            textrans: [TRANS[(rng>>00)%8], TRANS[(rng>>03)%8], TRANS[(rng>>06)%8], 
                       TRANS[(rng>>09)%8], TRANS[(rng>>12)%8], TRANS[(rng>>15)%8], ]
        }
    }
}

#[derive(Copy,Clone)]
enum Spot {
    Empty,
    Full,
    Rich(Block),
}
use Spot::{Empty, Full, Rich};

struct Chunk {
    bigpos: [i32;3],
    small: [[[Spot;16];16];16],
    filled: bool,
    vertex_data: Vec<Vertex>,
    index_data: Vec<u32>,
}

impl Chunk {
    pub fn new_full(x: i32, y: i32, z: i32) -> Chunk {
        Chunk{
            bigpos: [x, y, z],
            small: [[[Full;16];16];16],
            filled: true,
            vertex_data: Vec::new(),
            index_data: Vec::new(),
        }
    }
    pub fn new_empty(x: i32, y: i32, z: i32) -> Chunk {
        Chunk{
            bigpos: [x, y, z],
            small: [[[Empty;16];16];16],
            filled: false,
            vertex_data: Vec::new(),
            index_data: Vec::new(),
        }
    }
    pub fn at(&self, x: usize, y: usize, z: usize) -> Spot {
        assert!(x < 16 && y < 16 && z < 16);
        self.small[x][y][z]
    }
    pub fn try_at(&self, x: usize, y: usize, z: usize) -> Option<Spot> {
        if(x < 16 && y < 16 && z < 16){
            Some(self.small[x][y][z])
        } else {None}
    }
    pub fn nearly_at(&self, x: i32, y: i32, z:i32) -> Spot {
        match self.try_at(x as usize, y as usize, z as usize){
            Some(s) => {s}
            None => {Empty}
        }
    }
    pub fn put(&mut self, x: usize, y: usize, z: usize, b: Block) {
        assert!(x < 16 && y < 16 && z < 16);
        self.small[x][y][z] = Rich(b);
        self.update();
    }
    pub fn yank(&mut self, x: usize, y: usize, z: usize) -> Option<Block> {
        assert!(x < 16 && y < 16 && z < 16);
        if let Rich(b) = self.small[x][y][z]{
            self.small[x][y][z] = Empty;
            self.update();
            Some(b)
        } else {
            None
        }
    }
    fn update(&mut self) {
        self.vertex_data = Vec::new();
        self.index_data = Vec::new();

        for x in 0..16 { for y in 0..16 { for z in 0..16 {
        if let Rich(b) = self.at(x,y,z) {
        for f in 0..6 {
            let face = cube::Face::from_usize(f).unwrap();
            if let Empty = self.nearly_at((face.direction()[0] + x as i32),
                                          (face.direction()[1] + y as i32),
                                          (face.direction()[2] + z as i32)){
                let l = self.vertex_data.len() as u32;
                self.index_data.extend_from_slice(&[l+0,l+1,l+2,l+0,l+2,l+3]);

                let (rx, ry, rz) = (self.bigpos[0]*16+x as i32,
                                    self.bigpos[1]*16+y as i32,
                                    self.bigpos[2]*16+z as i32);
                let v = face.vertices([rx as f32, ry as f32, rz as f32], [1f32,1f32,1f32]);
                for i in 0..4{
                    self.vertex_data.push(Vertex::new([v[i][0], v[i][1], v[i][2]],
                                                [b.textrans[f][0][i] as f32 / TEXWIDTH,
                                                 b.textrans[f][1][i] as f32 / TEXWIDTH],
                                                 b.color));
                }
            }
        }}
        }}}
    }
}

struct Milieu {
    big: HashMap<(i32, i32, i32), Chunk>
}

impl Milieu {
    fn get_chunk(){

    }
    fn at(&self, x: i32, y: i32, z: i32){
        let (bx, by, bz) = (x/16, y/16, z/16);
        let (sx, sy, sz) = (x%16, y%16, z%16);

    }
}

//----------------------------------------

fn main() {

    let opengl = OpenGL::V3_2;

    let mut window: PistonWindow = PistonWindow::new(opengl, 0,
        WindowSettings::new("piston: cube", [640, 480])
        .exit_on_esc(true)
        .samples(4)
        .opengl(opengl)
        .srgb(false)
        .build()
        .unwrap());

    let ref mut factory = window.factory.clone();

    let mut rng = rand::thread_rng();

    let mut c = Chunk::new_empty(0,0,0);
    c.put(5,1,6,Block::new(rng.gen::<usize>(), [1.0, 0.0, 0.0, 1.0]));
    c.put(5,2,7,Block::new(rng.gen::<usize>(), [0.0, 1.0, 0.0, 1.0]));
    c.put(5,3,8,Block::new(rng.gen::<usize>(), [0.0, 0.0, 1.0, 1.0]));
    c.put(5,4,9,Block::new(rng.gen::<usize>(), [1.0, 1.0, 1.0, 1.0]));
    for x in 0..16 { for z in 0..16 {
        c.put(x,0,z,Block::new(rng.gen::<usize>(), [0.0, 0.0, 0.0, 1.0]));
    }}

    let (vbuf, slice) = factory.create_vertex_buffer_with_slice
        (&c.vertex_data, c.index_data.as_slice());

    let assets = find_folder::Search::ParentsThenKids(3, 3)
        .for_folder("assets").unwrap();
    let crosshair = assets.join("crosshair.png");
    let crosshair: G2dTexture = Texture::from_path(
            &mut window.factory,
            &crosshair,
            Flip::None,
            &TextureSettings::new()
        ).unwrap();
    
    let mut reticule: (f64, f64) = (0f64, 0f64);
    let draw_state = piston_window::DrawState::new_alpha();//.blend(piston_window::draw_state::Blend::Invert);

    let mut atlas = texture::AtlasBuilder::new(assets.join("blocks"), 256, 256);
    let offset = atlas.load("ground");
    let texture = atlas.complete(factory);

    let sinfo = gfx::texture::SamplerInfo::new(
        gfx::texture::FilterMethod::Bilinear,
        gfx::texture::WrapMode::Clamp);

    let glsl = opengl.to_glsl();
    let pso = factory.create_pipeline_simple(
            Shaders::new()
                .set(GLSL::V1_20, include_str!("../assets/cube_120.glslv"))
                .set(GLSL::V1_50, include_str!("../assets/cube_150.glslv"))
                .get(glsl).unwrap().as_bytes(),
            Shaders::new()
                .set(GLSL::V1_20, include_str!("../assets/cube_120.glslf"))
                .set(GLSL::V1_50, include_str!("../assets/cube_150.glslf"))
                .get(glsl).unwrap().as_bytes(),
            pipe::new()
        ).unwrap();

    let get_projection = |w: &PistonWindow| {
        let draw_size = w.window.draw_size();
        CameraPerspective {
            fov: 90.0, near_clip: 0.1, far_clip: 1000.0,
            aspect_ratio: (draw_size.width as f32) / (draw_size.height as f32)
        }.projection()
    };

    let mut first_person_settings = FirstPersonSettings::keyboard_wasd();
    first_person_settings.speed_horizontal = 5.0;
    first_person_settings.speed_vertical = 5.0;
    first_person_settings.move_forward_button = Button::from(Key::W);
    first_person_settings.move_backward_button = Button::from(Key::R);
    first_person_settings.strafe_left_button = Button::from(Key::A);
    first_person_settings.strafe_right_button = Button::from(Key::S);

    let model = vecmath::mat4_id();
    let mut projection = get_projection(&window);
    let mut first_person = FirstPerson::new(
        [8.0, 4.0, 12.0],
        first_person_settings
    );

    let mut data = pipe::Data {
            vbuf: vbuf.clone(),
            u_model_view_proj: [[0.0; 4]; 4],
            t_color: (texture.view, factory.create_sampler(sinfo)),
            out_color: window.output_color.clone(),
            out_depth: window.output_stencil.clone(),
        };

    window.set_capture_cursor(true);

    while let Some(e) = window.next() {
        first_person.event(&e);

        window.draw_3d(&e, |window| {
            let args = e.render_args().unwrap();

            window.encoder.clear(&window.output_color, [0.3, 0.3, 0.3, 1.0]);
            window.encoder.clear_depth(&window.output_stencil, 1.0);

            data.u_model_view_proj = model_view_projection(
                model,
                first_person.camera(args.ext_dt).orthogonal(),
                projection
            );
            //let (vbuf, slice) = factory.create_vertex_buffer_with_slice
            //                            (&vertex_data, index_data);

            window.encoder.draw(&slice, &pso, &data);
        });

        window.draw_2d(&e, |c, g| {
            Image::new().draw(&crosshair, &draw_state, c.transform.trans(reticule.0, reticule.1), g);
        });

        //swindow.set_capture_cursor(true);

        if let Some(_) = e.resize_args() {
            projection = get_projection(&window);
            data.out_color = window.output_color.clone();
            data.out_depth = window.output_stencil.clone();

            reticule = ((window.draw_size().width/2 - crosshair.get_size().0 / 2) as f64,
                        (window.draw_size().height/2 - crosshair.get_size().1 / 2) as f64);
        }
    }
}
