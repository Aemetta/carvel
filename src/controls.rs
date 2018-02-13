
use input::Button;
use input::Button::{Keyboard, Mouse};
use input::Key;
use input::mouse::MouseButton;

use tool;
use tool::InteractionState;

struct Control {
    button: Button,
    pressed: bool,
}

impl Control {
    fn new(button: Button) -> Control {
        Control {
            button,
            pressed: false,
        }
    }

    fn flip(&self, cmp: Button, on: bool) -> bool {
        if cmp != self.button { false } else {
        if on == self.pressed { false } else {
            true
        }}
    }
    fn flop(&mut self) {
        self.pressed = !self.pressed;
    }
}

pub struct PlayerController {
    move_forward: Control,
    move_backward: Control,
    strafe_left: Control,
    strafe_right: Control,
    jump: Control,
    crawl: Control,
    booster: Control,
    break_block: Control,
    place_block: Control,
    drop_player: Control,
    drop_camera: Control,
    mouse_sensitivity_horizontal: f32,
    mouse_sensitivity_vertical: f32,
}

impl PlayerController
{
    pub fn keyboard_wasd() -> PlayerController {

        PlayerController {
            move_forward:    Control::new(Keyboard(Key::W)),
            move_backward:   Control::new(Keyboard(Key::S)),
            strafe_left:     Control::new(Keyboard(Key::A)),
            strafe_right:    Control::new(Keyboard(Key::D)),
            jump:            Control::new(Keyboard(Key::Space)),
            crawl:           Control::new(Keyboard(Key::LShift)),
            booster:         Control::new(Keyboard(Key::LCtrl)),
            break_block:     Control::new(Mouse(MouseButton::Left)),
            place_block:     Control::new(Mouse(MouseButton::Right)),
            drop_player:     Control::new(Keyboard(Key::F7)),
            drop_camera:     Control::new(Keyboard(Key::F8)),

            mouse_sensitivity_horizontal: 1.0,
            mouse_sensitivity_vertical:   1.0,
        }
    }

    pub fn keyboard_wars() -> PlayerController {
        use input::Button::{Keyboard};
        use input::Key;

        let mut wars = PlayerController::keyboard_wasd();

        wars.move_forward =  Control::new(Keyboard(Key::W));
        wars.move_backward = Control::new(Keyboard(Key::R));
        wars.strafe_left =   Control::new(Keyboard(Key::A));
        wars.strafe_right =  Control::new(Keyboard(Key::S));

        wars
    }
}

use player::{Player, CrawlState};

const PI:    f32 = 3.14159265358979323846264338327950288;
const SQRT2: f32 = 1.41421356237309504880168872420969808;

impl PlayerController {
    pub fn mouse_movement(&self, dx: f32, dy: f32, player: &mut Player) {

        let dx = dx * self.mouse_sensitivity_horizontal;
        let dy = dy * self.mouse_sensitivity_vertical;

        player.yaw = (player.yaw - dx / 360.0 * PI / 4.0) % (2.0 * PI);
        player.pitch += dy / 360.0 * PI / 4.0;
        player.pitch = (player.pitch).min(PI / 2.0).max(-PI / 2.0);
    }

    pub fn input(&mut self, button: Button, on: bool, player: &mut Player, tool: &mut tool::Tool) {

        let &mut Player {
            ref mut dir,
            ref mut pos,
            ref mut cam,
            ref mut crawl,
            ref mut jump,
            ref mut noclip,
            ..
        } = player;

        let sgn = |x: f32| if x == 0.0 { 0.0 } else { x.signum() };
        let (dx, dy, dz) = (sgn(dir[0]), dir[1], sgn(dir[2]));
        let mut set = |x: f32, y: f32, z: f32| {
            let (x, z) = if x != 0.0 && z != 0.0 {
                (x / SQRT2, z / SQRT2)
            } else {
                (x, z)
            };
            *dir = [x, y, z];
        };
        match button {
            x if self.move_forward.flip(x, on) => { self.move_forward.flop();
                set(if on {-1.0+dx} else {1.0+dx}, dy, dz) },
            x if self.move_backward.flip(x, on) => { self.move_backward.flop();
                set(if on {1.0+dx} else {-1.0+dx}, dy, dz) },
            x if self.strafe_left.flip(x, on) => { self.strafe_left.flop();
                set(dx, dy, if on {1.0+dz} else {-1.0+dz}) },
            x if self.strafe_right.flip(x, on) => { self.strafe_right.flop();
                set(dx, dy, if on {-1.0+dz} else {1.0+dz}) },
            x if self.jump.flip(x, on) => { self.jump.flop();
                set(dx, if on {*jump = true; 1.0+dy} else {*jump = false;-1.0+dy}, dz)},
            x if self.crawl.flip(x, on) => { self.crawl.flop();
                set(dx, if on {-1.0+dy} else {1.0+dy}, dz);
                if !*noclip {
                    if on { *crawl = CrawlState::Crawl; }
                    else  { *crawl = CrawlState::Wait; }
                }},
            x if self.booster.flip(x, on) => { self.booster.flop(); },
            x if self.break_block.flip(x, on) => { self.break_block.flop();
                if on { tool.state = InteractionState::Mining;
                        if tool.clock < 0.0 { tool.clock = 0.0; } }
                else  { tool.state = InteractionState::Idle; }
            },
            x if self.place_block.flip(x, on) => { self.place_block.flop();
                if on { tool.state = InteractionState::Placing;
                        if tool.clock < 0.0 { tool.clock = 0.0; } }
                else  { tool.state = InteractionState::Idle; }
            },
            x if self.drop_player.flip(x, on) => { self.drop_player.flop(); if on {
                if *noclip { *noclip = false; *pos = cam.clone(); }
                else { *pos = cam.clone(); }
            }},
            x if self.drop_camera.flip(x, on) => { self.drop_camera.flop(); if on {
                if *noclip { *noclip = false; *cam = pos.clone(); }
                else { *noclip = true; *cam = pos.clone(); }
            }},
            _ => {}
        }
    }
}