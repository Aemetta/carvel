extern crate piston_window;
extern crate vecmath;
extern crate camera_controllers;
#[macro_use]
extern crate gfx;
extern crate gfx_voxel;
extern crate gfx_text;
extern crate gfx_debug_draw;
extern crate shader_version;
extern crate find_folder;
extern crate rand;
extern crate input;
extern crate line_drawing;
extern crate noise;
extern crate fps_counter;
extern crate collada;
extern crate skeletal_animation;

mod game;
use game::*;

mod player;

mod world;
use world::{
    Vertex, Block, Spot
};

mod controls;

mod gen;

use piston_window::*;
use gfx::traits::*;
use shader_version::Shaders;
use shader_version::glsl::GLSL;
use camera_controllers::{
    CameraPerspective,
    model_view_projection
};
use gfx_voxel::texture;

//----------------------------------------
// Cube associated data

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

    let loadstatus = |window: &mut PistonWindow, glyphs: &mut Glyphs, message: &str| {
        if let Some(e) = window.next() {
            window.draw_2d(&e, |c, g| {
            clear([0.0, 0.0, 0.0, 1.0], g);
            text::Text::new_color([1.0, 1.0, 1.0, 1.0], 18).draw(
                message,
                glyphs,
                &c.draw_state,
                c.transform.trans(5.0, 10.0),
                g
            ).unwrap();
        });
        } else { panic!("Could not display load status"); }
    };

    let opengl = OpenGL::V3_2;

    let mut window: PistonWindow = PistonWindow::new(opengl, 0,
        if let Ok(w) = WindowSettings::new("CARVEL", [768, 432])
            .opengl(opengl).build() {
            w
        } else {
            WindowSettings::new("CARVEL", [768, 432])
            .opengl(opengl).srgb(false).build().unwrap()
        });

    let ref mut factory = window.factory.clone();

    let assets = find_folder::Search::ParentsThenKids(3, 3)
        .for_folder("assets").unwrap();
    
    let ref font = assets.join("VeraMono.ttf");
    let mut glyphs = Glyphs::new(font, window.factory.clone(), TextureSettings::new()).unwrap();

    loadstatus(&mut window, &mut glyphs, "Loading Textures");

    let crosshair = assets.join("crosshair.png");
    let crosshair: G2dTexture = Texture::from_path(
            &mut window.factory,
            &crosshair,
            Flip::None,
            &TextureSettings::new()
        ).unwrap();
    let mut reticule: (f64, f64) = (0f64, 0f64);
    let draw_state = piston_window::DrawState::new_alpha();

    let mut atlas = texture::AtlasBuilder::new(assets.join("blocks"), 256, 256);
    let _offset = atlas.load("ground");
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
            fov: 110.0, near_clip: 0.05, far_clip: 1000.0,
            aspect_ratio: (draw_size.width as f32) / (draw_size.height as f32)
        }.projection()
    };

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
    let mut fps_counter = fps_counter::FPSCounter::new();

    loadstatus(&mut window, &mut glyphs, "Loading Character");

    use collada::document::ColladaDocument;
    use skeletal_animation::*;
    use std::rc::Rc;
    use std::path::Path;
    use vecmath::Matrix4;
    use gfx_debug_draw::DebugRenderer;

    let mut debug_renderer = {
        let text_renderer = {
            gfx_text::new(window.factory.clone()).unwrap()
        };
        DebugRenderer::new(window.factory.clone(), text_renderer, 64).ok().unwrap()
    };

    let collada_document = ColladaDocument::
    from_path(&Path::new("assets/vug/character.dae")).unwrap();

    let skeleton = {
        let skeleton_set = collada_document.get_skeletons().unwrap();
        Skeleton::from_collada(&skeleton_set[0])
    };

    let skeleton = Rc::new(skeleton);

    let mut asset_manager = AssetManager::<QVTransform>::new();

    asset_manager.load_assets("assets/vug/animation.json");

    let controller_def = asset_manager.controller_defs["vug"].clone();

    let mut controller = AnimationController::
    new(controller_def, skeleton.clone(), &asset_manager.animation_clips);

    let mut skinned_renderer = SkinnedRenderer::<_, Matrix4<f32>>::
    from_collada(factory, collada_document, vec!["assets/vug/char.png"]).unwrap();
    

    loadstatus(&mut window, &mut glyphs, "Loading World");

    let mut game = Game::new();


    while let Some(e) = window.next() {
        game.event(&e);

        window.draw_3d(&e, |window| {
            window.encoder.clear(&window.output_color, [0.0, 1.0, 1.0, 1.0]);
            window.encoder.clear_depth(&window.output_stencil, 1.0);
            
            let (vertex_data, index_data) = game.milieu.get_vertex_data();

            let (vbuf, slice) = factory.create_vertex_buffer_with_slice
                (&vertex_data, index_data.as_slice());

            data.vbuf = vbuf.clone();
            let camera = game.player.camera().orthogonal();

            data.u_model_view_proj = model_view_projection(model, camera, projection);

            window.encoder.draw(&slice, &pso, &data);

            controller.update(0.02);
            let mut global_poses = [ Matrix4::<f32>::identity(); 64 ];
            controller.get_output_pose(0.02, &mut global_poses[0 .. skeleton.joints.len()]);
            let camera_projection = model_view_projection(
                model, camera, projection
            );
            skinned_renderer.render(&mut window.encoder, &window.output_color, &window.output_stencil,
                                    camera, camera_projection, &global_poses);
            skeleton.draw(&global_poses, &mut debug_renderer, true);
        });

        window.draw_2d(&e, |c, g| {
            Image::new().draw(&crosshair, &draw_state, c.transform.trans(reticule.0, reticule.1), g);
            text::Text::new_color([0.0, 1.0, 0.0, 1.0], 18).draw(
                &format!("{}", fps_counter.tick()),
                &mut glyphs,
                &c.draw_state,
                c.transform.trans(5.0, 20.0),
                g
            ).unwrap();
            for row in 0..3 {
            for column in 0..3 {
                text::Text::new_color([1.0, 1.0, 1.0, 1.0], 14).draw(
                    &game.player.debug_info[row][column],
                    &mut glyphs,
                    &c.draw_state,
                    c.transform.trans(5.0+column as f64*80.0, 40.0+row as f64*20.0),
                    g
                ).unwrap();
            }}
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
