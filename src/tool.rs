
use world;
use player;

const INTERACTION_COOLDOWN:    f32 = 0.1;

pub enum InteractionState {
    Idle,
    Mining,
    Placing,
}

pub struct Tool {
    pub state: InteractionState,
    pub clock: f32,
}

impl Tool {
    pub fn new() -> Tool {
        Tool {
            state: InteractionState::Idle,
            clock: 0.0,
        }
    }

    pub fn update(&mut self, dt: f32, m: &mut world::Milieu, player: &player::Player) {
        let c = player.camera();
        let (point_full, point_empty) = m.viewcast(c.position, c.forward);
        if let Some((a,b,c)) = point_full{
            m.set_shiny(a, b, c, 1.5);
        }

        self.clock -= dt;
        if self.clock > 0.0 { return; }
        
        match self.state {
            InteractionState::Idle => {},
            InteractionState::Mining => {
                if let Some((x,y,z)) = point_full {
                    m.pull(x,y,z);
                    self.clock += INTERACTION_COOLDOWN;
                }
            },
            InteractionState::Placing => {
                if let Some((x,y,z)) = point_empty {
                    m.put(x,y,z, world::Block::new(
                        0, [1.0, 1.0, 1.0, 1.0]
                    ));
                    self.clock += INTERACTION_COOLDOWN;
                }
            },
        }

    }
}