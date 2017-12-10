extern crate piston_window;
extern crate vecmath;
extern crate camera_controllers;
#[macro_use]
extern crate gfx;
extern crate gfx_voxel;
extern crate shader_version;
extern crate find_folder;
extern crate rand;

#[macro_use]
extern crate bitflags;
extern crate input;

mod player;
use player::{
    FirstPersonSettings,
    FirstPerson
};

mod world;
use world::{
    Block, Spot, Chunk, Milieu
};

use piston_window::*;
use gfx::traits::*;
use shader_version::Shaders;
use shader_version::glsl::GLSL;
use camera_controllers::{
    CameraPerspective,
    model_view_projection
};
use rand::Rng;
use gfx_voxel::texture;

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

//----------------------------------------

fn main() {

    let mut rng = rand::thread_rng();

    let mut m = Milieu::new_empty();
    m.put(5,1,6,Block::new(rng.gen::<usize>(), [1.0, 0.0, 0.0, 1.0]));
    m.put(5,2,7,Block::new(rng.gen::<usize>(), [0.0, 1.0, 0.0, 1.0]));
    m.put(5,3,8,Block::new(rng.gen::<usize>(), [0.0, 0.0, 1.0, 1.0]));
    m.put(5,4,9,Block::new(rng.gen::<usize>(), [1.0, 1.0, 1.0, 1.0]));
    for x in 0..32 { for z in 0..32 {
        m.put(x,0,z,Block::new(rng.gen::<usize>(), [x as f32 * (1.0/32.0), 0.0, z as f32 * (1.0/32.0), 1.0]));
    }}

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

    let mut first_person_settings = FirstPersonSettings::keyboard_wars();
    first_person_settings.speed_horizontal = 10.0;
    first_person_settings.speed_vertical = 10.0;
    first_person_settings.gravity = 100.0;
    first_person_settings.jump_force = 25.0;
    let mut player = FirstPerson::new(
        [8.0, 1.0, 12.0],
        first_person_settings
    );

    let mut data = pipe::Data {
        vbuf: factory.create_vertex_buffer(&[]),
        u_model_view_proj: [[0.0; 4]; 4],
        t_color: (texture.view, factory.create_sampler(sinfo)),
        out_color: window.output_color.clone(),
        out_depth: window.output_stencil.clone(),
    };

    let model = vecmath::mat4_id();
    let mut projection = get_projection(&window);

    window.set_capture_cursor(true);

    while let Some(e) = window.next() {
        player.event(&e, &mut m);

        window.draw_3d(&e, |window| {
            let args = e.render_args().unwrap();

            window.encoder.clear(&window.output_color, [0.3, 0.3, 0.3, 1.0]);
            window.encoder.clear_depth(&window.output_stencil, 1.0);
            
            //m.refresh();
            let (vertex_data, index_data) = m.get_vertex_data();

            let (vbuf, slice) = factory.create_vertex_buffer_with_slice
                (&vertex_data, index_data.as_slice());

            data.vbuf = vbuf.clone();

            data.u_model_view_proj = model_view_projection(
                model,
                player.camera().orthogonal(),
                projection
            );

            window.encoder.draw(&slice, &pso, &data);
        });

        window.draw_2d(&e, |c, g| {
            Image::new().draw(&crosshair, &draw_state, c.transform.trans(reticule.0, reticule.1), g);
        });

        if let Some(_) = e.resize_args() {
            projection = get_projection(&window);
            data.out_color = window.output_color.clone();
            data.out_depth = window.output_stencil.clone();

            reticule = ((window.draw_size().width/2 - crosshair.get_size().0 / 2) as f64,
                        (window.draw_size().height/2 - crosshair.get_size().1 / 2) as f64);
        }
    }
}