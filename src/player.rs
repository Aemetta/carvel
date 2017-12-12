#![allow(dead_code)]

//! A first person camera.
/// Stolen and modified from the camera_controllers crate to work as a player object instead

use input::{ Button, GenericEvent };
use vecmath;
use vecmath::traits::{ Float, Radians };

use camera_controllers::Camera;

use world;

bitflags!(pub struct Keys: u8 {
    const MOVE_FORWARD  = 0b00000001;
    const MOVE_BACKWARD = 0b00000010;
    const STRAFE_LEFT   = 0b00000100;
    const STRAFE_RIGHT  = 0b00001000;
    const JUMP          = 0b00010000;
    const CRAWL         = 0b00100000;
    const BOOSTER       = 0b00100000;
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
    pub speed_horizontal: f32,
    pub speed_vertical: f32,
    pub mouse_sensitivity_horizontal: f32,
    pub mouse_sensitivity_vertical: f32,
    pub gravity: f32,
    pub jump_force: f32,
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
            speed_horizontal: 1.0,
            speed_vertical: 1.0,
            gravity: 1.0,
            jump_force: 1.0,
            mouse_sensitivity_horizontal: 1.0,
            mouse_sensitivity_vertical: 1.0,
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
            speed_horizontal: 1.0,
            speed_vertical: 1.0,
            gravity: 1.0,
            jump_force: 1.0,
            mouse_sensitivity_horizontal: 1.0,
            mouse_sensitivity_vertical: 1.0,
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
    pub direction: [f32; 3],
    pub position: [f64; 3],
    pub velocity: [f32; 3],
    keys: Keys,
    pub on_ground: bool,
    pub noclip: bool,
    pub force: f32,
    pub head_offset: f32,
    pub head_offset_crawl: f32,
}

impl FirstPerson {
    pub fn new(
        position: [f64; 3],
        settings: FirstPersonSettings
    ) -> FirstPerson {
        FirstPerson {
            settings: settings,
            state: Idle,
            yaw: 0.0,
            pitch: 0.0,
            keys: Keys::empty(),
            direction: [0.0, 0.0, 0.0],
            position: position,
            force: 1.0,
            velocity: [0.0, 0.0, 0.0],
            on_ground: true,
            noclip: true,
            head_offset: 2.4,
            head_offset_crawl: 0.8,
        }
    }

    pub fn camera(&self) -> Camera<f32> {
        let yoffset = if self.keys.contains(Keys::CRAWL) && !self.noclip
        {self.head_offset_crawl} else {self.head_offset};
        let mut c = Camera::new([self.position[0] as f32,
                                 self.position[1] as f32 + yoffset,
                                 self.position[2] as f32]);
        c.set_yaw_pitch(self.yaw, self.pitch);
        c
    }

    pub fn event<E>(&mut self, e: &E, m: &mut world::Milieu) where E: GenericEvent {

        let c = self.camera();
        let (point_full, point_empty) = select(c.position, c.forward, m);
        let &mut FirstPerson {
            ref mut yaw,
            ref mut pitch,
            ref mut keys,
            ref mut direction,
            ref mut velocity,
            ref mut position,
            ref mut force,
            ref mut on_ground,
            ref mut state,
            ref noclip,
            ref settings,
            ..
        } = self;

        let pi: f32 = Radians::_180();
        let sqrt2: f32 = 1.41421356237309504880168872420969808;

        e.update(|args| {
            let dt = args.dt as f32;
            let (dx, dy, dz) = (direction[0], direction[1], direction[2]);
            let (s, c) = (yaw.sin(), yaw.cos());

            let dh = *force * settings.speed_horizontal;
            let (xo, zo) = ((s * dx - c * dz) * dh,
                            (s * dz + c * dx) * dh);
            if *on_ground {
                velocity[0] = xo;
                velocity[1] = dy * settings.speed_vertical;
                velocity[2] = zo;
            } else {
                velocity[0] += xo * dt * 5.0;
                velocity[1] += -settings.gravity * dt;
                velocity[2] += zo * dt * 5.0;
            }

            position[0] += (velocity[0] * dt) as f64;
            position[1] += (velocity[1] * dt) as f64;
            position[2] += (velocity[2] * dt) as f64;

            if !*on_ground
                if let Some(b) = m.at(position[0] as i32,
                                      position[1] as i32,
                                      position[2] as i32){
                    if !b.is_empty() {
                        position[1] = position[1].ceil();
                        *on_ground = true;
            }}}

            match *state {
                Idle => {},
                Mining => {
                    if let Some((x,y,z)) = point_full {
                        m.pull(x,y,z);
                    }
                    //*state = Idle;
                },
                Placing => {
                    if let Some((x,y,z)) = point_empty {
                        m.put(x,y,z, world::Block::new(
                            0, [1.0, 1.0, 1.0, 1.0]
                        ));
                    }
                    //*state = Idle;
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
            let (dx, dy, dz) = (direction[0], direction[1], direction[2]);
            let sgn = |x: f32| if x == 0.0 { 0.0 } else { x.signum() };
            let mut set = |k, x: f32, y: f32, z: f32| {
                let (x, z) = (sgn(x), sgn(z));
                let (x, z) = if x != 0.0 && z != 0.0 {
                    (x / sqrt2, z / sqrt2)
                } else {
                    (x, z)
                };
                *direction = [x, y, z];
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
                    if *noclip { set(Keys::JUMP, dx, 1.0, dz) }
                    else { set(Keys::JUMP, dx, 0.0, dz);
                           *on_ground = false;
                           velocity[1] += settings.jump_force;},
                x if x == settings.crawl_button =>
                    if *noclip { set(Keys::CRAWL, dx, -1.0, dz) }
                    else { set(Keys::CRAWL, dx, 0.0, dz);
                           *force = 1.0 / 2.0;},
                x if x == settings.booster_button => {},
                x if x == settings.break_button => {
                    *state = Mining;
                },
                x if x == settings.place_button => {
                    *state = Placing;
                },
                _ => {}
            }
        });
        e.release(|button| {
            let (dx, dy, dz) = (direction[0], direction[1], direction[2]);
            let sgn = |x: f32| if x == 0.0 { 0.0 } else { x.signum() };
            let mut set = |x: f32, y: f32, z: f32| {
                let (x, z) = (sgn(x), sgn(z));
                let (x, z) = if x != 0.0 && z != 0.0 {
                    (x / sqrt2, z / sqrt2)
                } else {
                    (x, z)
                };
                *direction = [x, y, z];
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
                x if x == settings.jump_button =>
                    if *noclip { set(dx, release(Keys::JUMP, Keys::CRAWL, -1.0), dz) }
                    else { release(Keys::JUMP, Keys::CRAWL, 0.0); },
                x if x == settings.crawl_button =>
                    if *noclip { set(dx, release(Keys::CRAWL, Keys::JUMP, 1.0), dz) }
                    else { release(Keys::CRAWL, Keys::JUMP, 0.0);
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

    let pos = [pos[0]-0.5, pos[1]-0.5, pos[2]-0.5];

    use line_drawing::WalkVoxels;
    let dir = vecmath::vec3_neg(dir);
    let dir = vecmath::vec3_scale(dir, 10.0);
    let end = vecmath::vec3_add(pos, dir);

    let mut temp = None;
    for (i, (x, y, z)) in WalkVoxels::<f32, i32>::new(
                                (pos[0], pos[1], pos[2]),
                                (end[0], end[1], end[2])).enumerate() {

        if let Some(b) = m.at(x, y, z){
            if !b.is_empty() {
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