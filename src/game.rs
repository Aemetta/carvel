use input::{ Button, GenericEvent };

use world::*;
use player::*;
use rand::{self, Rng};

pub struct Game {
    pub milieu: Milieu,
    pub player: FirstPerson,
}

impl Game {
    pub fn new() -> Game {

        let mut rng = rand::thread_rng();
        let mut m = Milieu::new_full(rng.gen::<usize>());
        m.pull(1,0,0); //the first pulled block never actually gets pulled
        for x in 0..16 { for y in 0..7 { for z in 0..16 {
            m.pull(x,y,z);
        }}}

        let p = FirstPerson::new(
            [8.0, 0.0, 8.0],
            FirstPersonSettings::keyboard_wasd()
        );

        Game {
            milieu: m,
            player: p,
        }
    }

    pub fn update<E>(&mut self, e: &E) where E: GenericEvent {
        self.player.event(e, &mut self.milieu);
    }
}