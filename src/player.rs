#![allow(dead_code)]

use vecmath;

use camera_controllers::Camera;

use world;

const SPEED_HORIZONTAL:        f32 = 2.8;
const SPEED_HORIZONTAL_CRAWL:  f32 = 1.3;
const SPEED_VERTICAL:          f32 = 3.0;
const GRAVITY:                 f32 = 0.2;
const JUMP_FORCE:              f32 = 10.3;

const FRICTION_GROUND:         f32 = 0.5;
const FRICTION_AIR:            f32 = 0.002;
const GRIP_GROUND:             f32 = 1.0;
const GRIP_AIR:                f32 = 0.06;
const STATIC_FRICTION_CUTOFF:  f32 = 1.5;

const HEAD_OFFSET:             f32 = 2.15;
const HEAD_OFFSET_CRAWL:       f32 = 0.6;
const HITBOX_RADIUS:           f64 = 0.7;
const HITBOX_HEIGHT:           f64 = 2.8;
const HITBOX_HEIGHT_CRAWL:     f64 = 0.9;

pub enum CrawlState {
    Stand,
    Crawl,
    Wait,
}

impl CrawlState {
    #[inline]
    pub fn is_crawling(&self) -> bool {
        if let &CrawlState::Stand = self
        { false } else { true }
    }
}

pub struct Player {
    pub yaw: f32,
    pub pitch: f32,
    pub dir: [f32; 3],
    pub pos: [f64; 3],
    pub cam: [f64; 3],
    pub vel: [f32; 3],
    pub crawl: CrawlState,
    pub jump: bool,
    pub on_ground: bool,
    pub noclip: bool,
    pub debug_info: [[String; 3]; 3],
}

impl Player {
    pub fn new(
        pos: [f64; 3],
    ) -> Player {
        Player {
            yaw: 0.0,
            pitch: 0.0,
            dir: [0.0, 0.0, 0.0],
            pos: pos,
            cam: pos,
            vel: [0.0, 0.0, 0.0],
            crawl: CrawlState::Stand,
            jump: false,
            on_ground: true,
            noclip: false,
            debug_info: Default::default(),
        }
    }

    pub fn camera(&self) -> Camera<f32> {
        let p = if self.noclip { self.cam } else { self.pos };
        let yoffset = if self.crawl.is_crawling() && !self.noclip
        { HEAD_OFFSET_CRAWL } else { HEAD_OFFSET };
        let mut c = Camera::new([p[0] as f32,
                                 p[1] as f32 + yoffset,
                                 p[2] as f32]);
        c.set_yaw_pitch(self.yaw, self.pitch);
        c
    }

    pub fn update(&mut self, dt: f32, m: &mut world::Milieu) {

        let &mut Player {
            ref mut yaw,
            ref mut dir,
            ref mut vel,
            ref mut pos,
            ref mut cam,

            ref mut crawl,
            ref mut jump,
            ref mut on_ground,
            ref mut noclip,
            ref mut debug_info,
            ..
        } = self;

        //MOVEMENT

        let (dx, dy, dz) = (dir[0], dir[1], dir[2]);
        let (s, c) = (yaw.sin(), yaw.cos());

        let dh =  if crawl.is_crawling() && !*noclip { SPEED_HORIZONTAL_CRAWL } else { SPEED_HORIZONTAL };
        let (mut xo, yo, mut zo) = 
                ((s * dx - c * dz) * dh,
                dy * SPEED_VERTICAL,
                (s * dz + c * dx) * dh);
        
        if *noclip {
            cam[0] += (xo * 4.0 * dt) as f64;
            cam[1] += (yo * 4.0 * dt) as f64;
            cam[2] += (zo * 4.0 * dt) as f64;
            xo = 0.0; zo = 0.0;
        }

        let (grip, friction) = if *on_ground {
                (GRIP_GROUND, FRICTION_GROUND)
            } else {
                (GRIP_AIR, FRICTION_AIR)
            };

        let (xo, zo) = (xo * grip, zo * grip);
        let mut accel = [xo, -GRAVITY, zo];

        let speed = vecmath::vec3_len(*vel);
        if speed <= STATIC_FRICTION_CUTOFF && *on_ground {
            vel[0] = 0.0; vel[1] = 0.0; vel[2] = 0.0;
        } else if speed != 0.0 {
            let dir = vecmath::vec3_normalized(*vel);
            let ndir = vecmath::vec3_neg(dir);
            let friction = friction * speed;
            let force = vecmath::vec3_scale(ndir, friction);
            accel = vecmath::vec3_add(accel, force);
        }

        let (a, b) = (accel[0], accel[2]);
        let max_move_speed = dh / FRICTION_GROUND;
        let proposed = vecmath::vec2_len([vel[0] + a, vel[2] + b]);
        if max_move_speed < proposed {
            let softened_move = vecmath::vec2_scale([vel[0] + a, vel[2] + b],
                                                    max_move_speed / proposed);
            accel[0] = softened_move[0] - vel[0];
            accel[2] = softened_move[1] - vel[2];
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

        let (r, h) = (HITBOX_RADIUS,
                        if crawl.is_crawling()
                        {HITBOX_HEIGHT_CRAWL} else 
                        {HITBOX_HEIGHT});

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
                    if *jump {
                        vel[1] = JUMP_FORCE;
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

        if let CrawlState::Wait = *crawl {
            let by1 = by2;
            let (_, by2) = bound_v(pos[1], HITBOX_HEIGHT);
            *crawl = CrawlState::Stand;
    'crawl: for y in by1..by2+1 {
            for x in bx1..bx2+1 {
            for z in bx1..bx2+1 {
                let (bx, by, bz) = (mov[0] as i32+x, mov[1] as i32+y, mov[2] as i32+z);
                if let Some(b) = m.world.at(bx, by, bz){
                if !b.is_empty() {
                    *crawl = CrawlState::Wait;
                    break 'crawl;
                }}
            }}}
        }        
    }
}
