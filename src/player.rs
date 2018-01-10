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
    pub speed_horizontal: f32,
    pub speed_vertical: f32,
    pub mouse_sensitivity_horizontal: f32,
    pub mouse_sensitivity_vertical: f32,
    pub gravity: f32,
    pub jump_force: f32,
    pub head_offset: f32,
    pub head_offset_crawl: f32,
    pub hitbox_radius: f64,
    pub hitbox_height: f64,
    pub hitbox_height_crawl: f64,
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
            place_button: Keyboard(Key::E),
            drop_player_button: Keyboard(Key::F7),
            drop_camera_button: Keyboard(Key::F8),
            speed_horizontal: 1.0,
            speed_vertical: 1.0,
            gravity: 1.0,
            jump_force: 1.0,
            mouse_sensitivity_horizontal: 1.0,
            mouse_sensitivity_vertical: 1.0,
            head_offset: 2.4,
            head_offset_crawl: 0.8,
            hitbox_radius: 0.7,
            hitbox_height: 2.8,
            hitbox_height_crawl: 0.9,
        }
    }

    pub fn keyboard_wars() -> FirstPersonSettings {
        use input::Button::{Keyboard, Mouse};
        use input::Key;
        use input::mouse::MouseButton;

        FirstPersonSettings {
            move_forward_button: Keyboard(Key::W),
            move_backward_button: Keyboard(Key::R),
            strafe_left_button: Keyboard(Key::A),
            strafe_right_button: Keyboard(Key::S),
            jump_button: Keyboard(Key::Space),
            crawl_button: Keyboard(Key::LShift),
            booster_button: Keyboard(Key::LCtrl),
            break_button: Mouse(MouseButton::Left),
            place_button: Mouse(MouseButton::Right),
            drop_player_button: Keyboard(Key::F7),
            drop_camera_button: Keyboard(Key::F8),
            speed_horizontal: 1.0,
            speed_vertical: 1.0,
            gravity: 1.0,
            jump_force: 1.0,
            mouse_sensitivity_horizontal: 1.0,
            mouse_sensitivity_vertical: 1.0,
            head_offset: 2.4,
            head_offset_crawl: 0.8,
            hitbox_radius: 0.7,
            hitbox_height: 2.8,
            hitbox_height_crawl: 0.9,
        }
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
    pub vel: [f32; 3],
    keys: Keys,
    pub on_ground: bool,
    pub noclip: bool,
    pub force: f32,
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
            force: 1.0,
            vel: [0.0, 0.0, 0.0],
            on_ground: true,
            noclip: false,
        }
    }

    pub fn camera(&self) -> Camera<f32> {
        let yoffset = if self.keys.contains(Keys::CRAWL) && !self.noclip
        {self.settings.head_offset_crawl} else {self.settings.head_offset};
        let mut c = Camera::new([self.pos[0] as f32,
                                 self.pos[1] as f32 + yoffset,
                                 self.pos[2] as f32]);
        c.set_yaw_pitch(self.yaw, self.pitch);
        c
    }

    pub fn event<E>(&mut self, e: &E, m: &mut world::Milieu) where E: GenericEvent {

        let c = self.camera();
        let (point_full, point_empty) = select(c.position, c.forward, m);
        m.set_cursor(point_full);
        let &mut FirstPerson {
            ref mut yaw,
            ref mut pitch,
            ref mut keys,
            ref mut dir,
            ref mut vel,
            ref mut pos,
            ref mut force,
            ref mut on_ground,
            ref mut state,
            ref mut noclip,
            ref settings,
            ..
        } = self;

        let pi: f32 = Radians::_180();
        let sqrt2: f32 = 1.41421356237309504880168872420969808;

        e.update(|args| {
            let dt = args.dt as f32;
            let (dx, dy, dz) = (dir[0], dir[1], dir[2]);
            let (s, c) = (yaw.sin(), yaw.cos());

            let dh = *force * settings.speed_horizontal;
            let (xo, zo) = ((s * dx - c * dz) * dh,
                            (s * dz + c * dx) * dh);
            if *on_ground {
                vel[0] = xo;
                vel[1] = dy * settings.speed_vertical;
                vel[2] = zo;
            } else {
                vel[0] += xo * dt * 5.0;
                vel[1] += -settings.gravity * dt;
                vel[2] += zo * dt * 5.0;
            }

            pos[0] += (vel[0] * dt) as f64;
            pos[1] += (vel[1] * dt) as f64;
            pos[2] += (vel[2] * dt) as f64;


            //COLLISION DETECTION

            *on_ground = false;

            let pos_i = [pos[0] as i32, pos[1] as i32, pos[2] as i32];
            let (r, h) = (settings.hitbox_radius,
                            if keys.contains(Keys::CRAWL)
                            {settings.hitbox_height_crawl} else 
                            {settings.hitbox_height});
            let (ri, hi) = (r.ceil() as i32, h.ceil() as i32);

            let frac = [(pos[0]%1.0+1.0)%1.0, (pos[1]%1.0+1.0)%1.0, (pos[2]%1.0+1.0)%1.0];

            let mut bounds = [
                [if frac[0] < r {-ri} else {-ri+1}, if frac[0] > 1.0-r {ri+1} else {ri}],
                [0,                                if frac[1] > 1.0-h {hi+1} else {hi}],
                [if frac[2] < r {-ri} else {-ri+1}, if frac[2] > 1.0-r {ri+1} else {ri}],
            ];
            if pos[0] < 0.0 {bounds[0][0] -= 1; bounds[0][1] -= 1;}
            if pos[1] < 0.0 {bounds[1][0] -= 1; bounds[1][1] -= 1;}
            if pos[2] < 0.0 {bounds[2][0] -= 1; bounds[2][1] -= 1;}

            for x in bounds[0][0]..bounds[0][1] {
            for y in bounds[1][0]..bounds[1][1] {
            for z in bounds[2][0]..bounds[2][1] {
                if let Some(b) = m.world.at(pos_i[0]+x, pos_i[1]+y, pos_i[2]+z){
                if !b.is_empty() {

                    if y == bounds[1][0] { if vel[1] < 0.0 {
                        pos[1] = pos[1].ceil() + y as f64;
                        if keys.contains(Keys::JUMP) {
                            vel[1] *= -1.0;
                        } else {
                            vel[1] = 0.0;
                            *on_ground = true;
                        }
                    }} else if y == bounds[1][1] { if vel[1] > 0.0 {
                        vel[1] = 0.0;
                        pos[1] = (pos_i[1]-y) as f64 + h;
                    }} else {
                        if x == bounds[0][0] && vel[0] < 0.0 {
                            vel[0] = 0.0;
                            pos[0] = pos_i[0] as f64 - (1.0 - r);
                        }
                        if x+1 == bounds[0][1] && vel[0] > 0.0 {
                            vel[0] = 0.0;
                            pos[0] = pos_i[0] as f64 + (1.0 - r);
                        }
                        if z == bounds[2][0] && vel[2] < 0.0 {
                            vel[2] = 0.0;
                            pos[2] = pos_i[2] as f64 - (1.0 - r);
                        }
                        if z+1 == bounds[2][1] && vel[2] > 0.0 {
                            vel[2] = 0.0;
                            pos[2] = pos_i[2] as f64 + (1.0 - r);
                        }
                    }
                }}
            }}}


            //BLOCK INTERACTION

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
                x if x == settings.jump_button => {
                    set(Keys::JUMP, dx, 0.0, dz);
                    if *on_ground {
                        *on_ground = false;
                        vel[1] += settings.jump_force;
                    }},
                x if x == settings.crawl_button => {
                    set(Keys::CRAWL, dx, 0.0, dz);
                    *force = 1.0 / 2.0;},
                x if x == settings.booster_button => {},
                x if x == settings.break_button => {
                    *state = Mining;
                },
                x if x == settings.place_button => {
                    *state = Placing;
                },
                x if x == settings.drop_player_button => {*noclip = false;},
                x if x == settings.drop_camera_button => {*noclip = true;},
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
                x if x == settings.jump_button => { release(Keys::JUMP, Keys::CRAWL, 0.0); },
                x if x == settings.crawl_button => {
                    release(Keys::CRAWL, Keys::JUMP, 0.0);
                    *force = 1.0;},
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