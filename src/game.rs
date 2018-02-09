use input::GenericEvent;

use world::*;
use player::*;
use controls::*;
use rand::{self, Rng};

pub struct Game {
    pub milieu: Milieu,
    pub player: Player,
    pub controls: PlayerController,
}

impl Game {
    pub fn new() -> Game {

        let mut rng = rand::thread_rng();
        let mut m = Milieu::new_full(rng.gen::<usize>());
        m.pull(1,0,0); //the first pulled block never actually gets pulled
        for x in -6..6 { for y in 0..7 { for z in -6..6 {
            m.pull(x,y,z);
        }}}

        let p = Player::new(
            [0.0, 0.0, 3.0],
        );

        Game {
            milieu: m,
            player: p,
            controls: PlayerController::keyboard_wars(),
        }
    }

    pub fn event<E: GenericEvent>(&mut self, e: &E) {

        e.mouse_relative(|dx, dy| {
            self.controls.mouse_movement(dx as f32, dy as f32, &mut self.player);
        });

        e.press(|button| {
            self.controls.input(button, true, &mut self.player);
        });
        e.release(|button| {
            self.controls.input(button, false, &mut self.player);
        });

        e.update(|args| {

            let dt = args.dt as f32;
            
            self.player.update(dt, &mut self.milieu);
        });
    }
}