use super::Block;
use rand::{self, Rng};
use noise::*;

pub struct Gen {
    red: Billow<f32>,
    green: Billow<f32>,
    blue: Billow<f32>,
}

impl Gen {
    pub fn new(seed: usize) -> Gen {
        let red = Billow::new().set_seed(seed+0);
        let green = Billow::new().set_seed(seed+1);
        let blue = Billow::new().set_seed(seed+2);
        Gen {red, green, blue}
    }
    pub fn at(&self, x: i32, y: i32, z: i32) -> Block {
        let (x,y,z) = (x as f32 / 100.0, y as f32 / 100.0, z as f32 / 100.0);
        let color =  [self.red.get([x,y,z]) + 0.8,
                    self.green.get([x,y,z]) + 0.8,
                     self.blue.get([x,y,z]) + 0.8,
                     1.0];
        let mut rng = rand::thread_rng();
        Block::new(rng.gen::<usize>(), color)
    }
}