#![allow(dead_code)]

//! A first person camera.
/// Stolen and modified from the camera_controllers crate to work as a player object instead

use input::{ Button, GenericEvent };
use vecmath;
use vecmath::traits::Radians;

use camera_controllers::Camera;

use world;

bitflags!(pub struct Keys: u8 {
    const MOVE_FORWARD  = 0b00000001;
    const MOVE_BACKWARD = 0b00000010;
    const STRAFE_LEFT   = 0b00000100;
    const STRAFE_RIGHT  = 0b00001000;
    const JUMP          = 0b00010000;
    const CRAWL         = 0b00100000;
    const BOOSTER       = 0b01000000;
});

pub struct FirstPersonSettings {
    pub move_forward_button: Button,
    pub move_backward_button: Button,
    pub strafe_left_button: Button,
    pub strafe_right_button: Button,
    pub jump_button: Button,
    pub crawl_button: Button,
    pub booster_button: Button,
    pub break_button: Button,
    pub place_button: Button,
    pub drop_player_button: Button,
    pub drop_camera_button: Button,
    pub mouse_sensitivity_horizontal: f32,
    pub mouse_sensitivity_vertical: f32,

    pub speed_horizontal: f32,
    pub speed_vertical: f32,
    pub gravity: f32,
    pub jump_force: f32,

    pub grip_ground: f32,
    pub grip_air: f32,
    pub friction_ground: f32,
    pub friction_air: f32,
    pub static_friction_cutoff: f32,

    pub head_offset: f32,
    pub head_offset_crawl: f32,
    pub hitbox_radius: f64,
    pub hitbox_height: f64,
    pub hitbox_height_crawl: f64,
    pub interaction_cooldown: f32,
}

impl FirstPersonSettings
{
    pub fn keyboard_wasd() -> FirstPersonSettings {
        use input::Button::{Keyboard, Mouse};
        use input::Key;
        use input::mouse::MouseButton;

        FirstPersonSettings {
            move_forward_button: Keyboard(Key::W),
            move_backward_button: Keyboard(Key::S),
            strafe_left_button: Keyboard(Key::A),
            strafe_right_button: Keyboard(Key::D),
            jump_button: Keyboard(Key::Space),
            crawl_button: Keyboard(Key::LShift),
            booster_button: Keyboard(Key::LCtrl),
            break_button: Mouse(MouseButton::Left),
            place_button: Mouse(MouseButton::Right),
            drop_player_button: Keyboard(Key::F7),
            drop_camera_button: Keyboard(Key::F8),
            speed_horizontal: 4.0,
            speed_vertical: 3.0,
            gravity: 0.2,
            jump_force: 10.3,
            friction_ground: 0.5,
            friction_air: 0.002,
            grip_ground: 1.0,
            grip_air: 0.06,
            static_friction_cutoff: 1.5,
            mouse_sensitivity_horizontal: 1.0,
            mouse_sensitivity_vertical: 1.0,
            head_offset: 2.4,
            head_offset_crawl: 0.8,
            hitbox_radius: 0.7,
            hitbox_height: 2.8,
            hitbox_height_crawl: 0.9,
            interaction_cooldown: 0.1,
        }
    }

    pub fn keyboard_wars() -> FirstPersonSettings {
        use input::Button::{Keyboard};
        use input::Key;

        let mut wars = FirstPersonSettings::keyboard_wasd();

        wars.move_forward_button = Keyboard(Key::W);
        wars.move_backward_button = Keyboard(Key::R);
        wars.strafe_left_button = Keyboard(Key::A);
        wars.strafe_right_button = Keyboard(Key::S);

        wars
    }
}

enum InteractionState {
    Idle,
    Mining,
    Placing,
}
use self::InteractionState::{Idle,Mining,Placing};

pub struct FirstPerson {
    pub settings: FirstPersonSettings,
    state: InteractionState,
    pub yaw: f32,
    pub pitch: f32,
    pub dir: [f32; 3],
    pub pos: [f64; 3],
    pub cam: [f64; 3],
    pub vel: [f32; 3],
    keys: Keys,
    pub crawling: bool,
    pub on_ground: bool,
    pub noclip: bool,
    pub clock: f32,
    pub debug_info: [[String; 3]; 3],
}

impl FirstPerson {
    pub fn new(
        pos: [f64; 3],
        settings: FirstPersonSettings
    ) -> FirstPerson {
        FirstPerson {
            settings: settings,
            state: Idle,
            yaw: 0.0,
            pitch: 0.0,
            keys: Keys::empty(),
            dir: [0.0, 0.0, 0.0],
            pos: pos,
            cam: pos,
            vel: [0.0, 0.0, 0.0],
            crawling: false,
            on_ground: true,
            noclip: false,
            clock: 0.0,
            debug_info: Default::default(),
        }
    }

    pub fn camera(&self) -> Camera<f32> {
        let p = if self.noclip { self.cam } else { self.pos };
        let yoffset = if self.crawling && !self.noclip
        {self.settings.head_offset_crawl} else {self.settings.head_offset};
        let mut c = Camera::new([p[0] as f32,
                                 p[1] as f32 + yoffset,
                                 p[2] as f32]);
        c.set_yaw_pitch(self.yaw, self.pitch);
        c
    }

    pub fn event<E>(&mut self, e: &E, m: &mut world::Milieu) where E: GenericEvent {

        let c = self.camera();
        let (point_full, point_empty) = select(c.position, c.forward, m);
        if let Some((a,b,c)) = point_full{
            m.set_shiny(a, b, c, 1.5);
        }
        let &mut FirstPerson {
            ref mut yaw,
            ref mut pitch,
            ref mut keys,
            ref mut dir,
            ref mut vel,
            ref mut pos,
            ref mut cam,

            ref mut crawling,
            ref mut on_ground,
            ref mut state,
            ref mut noclip,
            ref mut clock,
            ref settings,
            ref mut debug_info,
            ..
        } = self;

        let pi: f32 = Radians::_180();
        let sqrt2: f32 = 1.41421356237309504880168872420969808;

        e.update(|args| {

            let dt = args.dt as f32;

            //BLOCK INTERACTION

            *clock -= dt;
            if *clock <= 0.0 {
                *clock += settings.interaction_cooldown;
                match *state {
                    Idle => {},
                    Mining => {
                        if let Some((x,y,z)) = point_full {
                            m.pull(x,y,z);
                        }
                    },
                    Placing => {
                        if let Some((x,y,z)) = point_empty {
                            m.put(x,y,z, world::Block::new(
                                0, [1.0, 1.0, 1.0, 1.0]
                            ));
                        }
                    },
                }
            }

            //MOVEMENT

            let (dx, dy, dz) = (dir[0], dir[1], dir[2]);
            let (s, c) = (yaw.sin(), yaw.cos());

            let dh = settings.speed_horizontal * if *crawling && !*noclip { 0.5 } else { 1.0 };
            let (mut xo, yo, mut zo) = 
                    ((s * dx - c * dz) * dh,
                    dy * settings.speed_vertical,
                    (s * dz + c * dx) * dh);
            
            if *noclip {
                cam[0] += (xo * 4.0 * dt) as f64;
                cam[1] += (yo * 4.0 * dt) as f64;
                cam[2] += (zo * 4.0 * dt) as f64;
                xo = 0.0; zo = 0.0;
            }

            let (grip, friction) = if *on_ground {
                    (settings.grip_ground, settings.friction_ground)
                } else {
                    (settings.grip_air, settings.friction_air)
                };

            let (xo, zo) = (xo * grip, zo * grip);
            let mut accel = [xo, -settings.gravity, zo];

            let speed = vecmath::vec3_len(*vel);
            if speed <= settings.static_friction_cutoff && *on_ground {
                vel[0] = 0.0; vel[1] = 0.0; vel[2] = 0.0;
            } else if speed != 0.0 {
                let dir = vecmath::vec3_normalized(*vel);
                let ndir = vecmath::vec3_neg(dir);
                let friction = friction * speed;
                let force = vecmath::vec3_scale(ndir, friction);
                accel = vecmath::vec3_add(accel, force);
            }

            let (a, b) = (accel[0], accel[2]);
            if !*on_ground {
                let max_move_speed = dh / settings.friction_ground;
                let proposed = vecmath::vec2_len([vel[0] + a, vel[2] + b]);
                if max_move_speed < proposed {
                    let softened_move = vecmath::vec2_scale([vel[0] + a, vel[2] + b],
                                                            max_move_speed / proposed);
                    accel[0] = softened_move[0] - vel[0];
                    accel[2] = softened_move[1] - vel[2];
                }
            }

            vel[0] += accel[0];
            vel[1] += accel[1];
            vel[2] += accel[2];
            let mut mov = [
                pos[0] + (vel[0] * dt) as f64,
                pos[1] + (vel[1] * dt) as f64,
                pos[2] + (vel[2] * dt) as f64,
            ];


            //COLLISION DETECTION

            *on_ground = false;

            let (r, h) = (settings.hitbox_radius,
                            if *crawling
                            {settings.hitbox_height_crawl} else 
                            {settings.hitbox_height});

            let bound = |p: f64, r: f64| {
                let frac = (p%1.0+1.0)%1.0;
                let ri = r.ceil() as i32;
                let (b1, b2) = (if frac < r%1.0 {-ri} else {-ri+1}, if frac > 1.0-r%1.0 {ri} else {ri-1});
                if p < 0.0 && frac != 0.0 {(b1-1, b2-1)} else {(b1, b2)}
            };
            let bound_v = |p: f64, h: f64| {
                let frac = (p%1.0+1.0)%1.0;
                let hi = h.ceil() as i32;
                let (b1, b2) = (0, if frac > 1.0-h%1.0 {hi} else {hi-1});
                if p < 0.0 && frac != 0.0 {(b1-1, b2-1)} else {(b1, b2)}
            };

            let mut collision_debug = [None;3];

            let (bx1, bx2) = bound(pos[0], r);
            let (by1, by2) = bound_v(mov[1], h);
            let (bz1, bz2) = bound(pos[2], r);
            let safety_margin = (1.0 - r%1.0) - 0.000001;

            if vel[1] != 0.0 {
            let y = if vel[1] < 0.0 { by1 } else { by2 };
        'y: for x in bx1..bx2+1 {
            for z in bz1..bz2+1 {
                let (bx, by, bz) = (pos[0] as i32+x, mov[1] as i32+y, pos[2] as i32+z);
                if let Some(b) = m.world.at(bx, by, bz){/*
                        if let Some(spot) = m.world.at(pos_i[0]+x, pos_i[1]+y+1, pos_i[2]+z){
                        spot.is_empty() } else { false }*/
                if !b.is_empty() {
                    vel[1] = 0.0;
                    if y == by1 {
                        mov[1] = mov[1].ceil() as f64;
                        if keys.contains(Keys::JUMP) {
                            vel[1] = settings.jump_force;
                        } else {
                            *on_ground = true;
                        }
                    } else {
                        mov[1] = mov[1].floor() + (1.0 - h%1.0) - 0.000001;
                    }
                    collision_debug[1] = Some((bx, by, bz));
                    break 'y;
                }}
            }}}

            let (by1, by2) = bound_v(mov[1], h);
            let (bx1, bx2) = bound(mov[0], r);

            if vel[0] != 0.0 {
            let x = if vel[0] < 0.0 { bx1 } else { bx2 };
        'x: for y in by1..by2+1 {
            for z in bz1..bz2+1 {
                let (bx, by, bz) = (mov[0] as i32+x, mov[1] as i32+y, pos[2] as i32+z);
                if let Some(b) = m.world.at(bx, by, bz){
                if !b.is_empty() {
                    vel[0] = 0.0;
                    if x == bx1 {
                        mov[0] = mov[0].ceil() - safety_margin;
                    } else {
                        mov[0] = mov[0].floor() + safety_margin;
                    }
                    collision_debug[0] = Some((bx, by, bz));
                    break 'x;
                }}
            }}}

            let (bx1, bx2) = bound(mov[0], r);
            let (bz1, bz2) = bound(mov[2], r);

            if vel[2] != 0.0 {
            let z = if vel[2] < 0.0 { bz1 } else { bz2 };
        'z: for y in by1..by2+1 {
            for x in bx1..bx2+1 {
                let (bx, by, bz) = (mov[0] as i32+x, mov[1] as i32+y, mov[2] as i32+z);
                if let Some(b) = m.world.at(bx, by, bz){
                if !b.is_empty() {
                    vel[2] = 0.0;
                    if z == bz1 {
                        mov[2] = mov[2].ceil() - safety_margin;
                    } else {
                        mov[2] = mov[2].floor() + safety_margin;
                    }
                    collision_debug[2] = Some((bx, by, bz));
                    break 'z;
                }}
            }}}

            for i in 0..3 {
                debug_info[i][0] = format!("{:.4}", pos[i]);
                debug_info[i][1] = format!("{:.4}", vel[i]);
                debug_info[i][2] = format!("{:?}", collision_debug[i]);
            }

            pos[0] = mov[0];
            pos[1] = mov[1];
            pos[2] = mov[2];

            if *crawling && !keys.contains(Keys::CRAWL) {
                let by1 = by2;
                let (_, by2) = bound_v(pos[1], settings.hitbox_height);
                *crawling = false;
        'crawl: for y in by1..by2+1 {
                for x in bx1..bx2+1 {
                for z in bx1..bx2+1 {
                    let (bx, by, bz) = (mov[0] as i32+x, mov[1] as i32+y, mov[2] as i32+z);
                    if let Some(b) = m.world.at(bx, by, bz){
                    if !b.is_empty() {
                        *crawling = true;
                        break 'crawl;
                    }}
                }}}
            }
        });

        e.mouse_relative(|dx, dy| {

            let dx = dx as f32 * settings.mouse_sensitivity_horizontal;
            let dy = dy as f32 * settings.mouse_sensitivity_vertical;

            *yaw = (*yaw - dx / 360.0 * pi / 4.0) % (2.0 * pi);
            *pitch += dy / 360.0 * pi / 4.0;
            *pitch = (*pitch).min(pi / 2.0).max(-pi / 2.0);
        });

        e.press(|button| {
            let (dx, dy, dz) = (dir[0], dir[1], dir[2]);
            let sgn = |x: f32| if x == 0.0 { 0.0 } else { x.signum() };
            let mut set = |k, x: f32, y: f32, z: f32| {
                let (x, z) = (sgn(x), sgn(z));
                let (x, z) = if x != 0.0 && z != 0.0 {
                    (x / sqrt2, z / sqrt2)
                } else {
                    (x, z)
                };
                *dir = [x, y, z];
                keys.insert(k);
            };
            match button {
                x if x == settings.move_forward_button =>
                    set(Keys::MOVE_FORWARD, -1.0, dy, dz),
                x if x == settings.move_backward_button =>
                    set(Keys::MOVE_BACKWARD, 1.0, dy, dz),
                x if x == settings.strafe_left_button =>
                    set(Keys::STRAFE_LEFT, dx, dy, 1.0),
                x if x == settings.strafe_right_button =>
                    set(Keys::STRAFE_RIGHT, dx, dy, -1.0),
                x if x == settings.jump_button => 
                    set(Keys::JUMP, dx, 1.0, dz),
                x if x == settings.crawl_button => {
                    set(Keys::CRAWL, dx, -1.0, dz);
                    if !*noclip { *crawling = true; } },
                x if x == settings.booster_button => {},
                x if x == settings.break_button => {
                    *state = Mining;
                    *clock = 0.0;
                },
                x if x == settings.place_button => {
                    *state = Placing;
                    *clock = 0.0;
                },
                x if x == settings.drop_player_button => {
                    if *noclip { *noclip = false; *pos = cam.clone(); }
                    else { *pos = cam.clone(); }
                },
                x if x == settings.drop_camera_button => {
                    if *noclip { *noclip = false; *cam = pos.clone(); }
                    else { *noclip = true; *cam = pos.clone(); }
                },
                _ => {}
            }
        });
        e.release(|button| {
            let (dx, dy, dz) = (dir[0], dir[1], dir[2]);
            let sgn = |x: f32| if x == 0.0 { 0.0 } else { x.signum() };
            let mut set = |x: f32, y: f32, z: f32| {
                let (x, z) = (sgn(x), sgn(z));
                let (x, z) = if x != 0.0 && z != 0.0 {
                    (x / sqrt2, z / sqrt2)
                } else {
                    (x, z)
                };
                *dir = [x, y, z];
            };
            let mut release = |key, rev_key, rev_val| {
                keys.remove(key);
                if keys.contains(rev_key) { rev_val } else { 0.0 }
            };
            match button {
                x if x == settings.move_forward_button =>
                    set(release(Keys::MOVE_FORWARD, Keys::MOVE_BACKWARD, 1.0), dy, dz),
                x if x == settings.move_backward_button =>
                    set(release(Keys::MOVE_BACKWARD, Keys::MOVE_FORWARD, -1.0), dy, dz),
                x if x == settings.strafe_left_button =>
                    set(dx, dy, release(Keys::STRAFE_LEFT, Keys::STRAFE_RIGHT, -1.0)),
                x if x == settings.strafe_right_button =>
                    set(dx, dy, release(Keys::STRAFE_RIGHT, Keys::STRAFE_LEFT, 1.0)),
                x if x == settings.jump_button => {
                    set(dx, release(Keys::JUMP, Keys::CRAWL, 1.0), dz); },
                x if x == settings.crawl_button => {
                    set(dx, release(Keys::CRAWL, Keys::JUMP, 1.0), dz); },
                x if x == settings.booster_button => {},
                x if x == settings.break_button => {
                    *state = Idle;
                },
                x if x == settings.place_button => {
                    *state = Idle;
                },
                _ => {}
            }
        });
    }
}

fn select(pos: [f32;3], dir: [f32;3], m: &mut world::Milieu)
        -> (Option<(i32, i32, i32)>, Option<(i32, i32, i32)>){
    let mut full = None;
    let mut empty = None;

    //let pos = [pos[0]+0.5, pos[1]+0.5, pos[2]+0.5];

    use line_drawing::WalkVoxels;
    let dir = vecmath::vec3_neg(dir);
    let dir = vecmath::vec3_scale(dir, 10.0);
    let end = vecmath::vec3_add(pos, dir);

    let mut temp = None;
    for (_, (x, y, z)) in WalkVoxels::<f32, i32>::new(
                    (pos[0], pos[1], pos[2]),
                    (end[0], end[1], end[2]))
                    .enumerate() {

        if let Some(b) = m.world.at(x, y, z){
            if b.is_rich() {
                empty = temp;
                full = Some((x, y, z));
                break;
            } else {
                temp = Some((x, y, z));
            }
        } else {
            temp = Some((x, y, z));
        }
    }
    (full, empty)
}