use gen::Gen;

use gfx_voxel::cube;
use std::collections::HashMap;

const SIZE_I: i32 = 4;
const SIZE_U: usize = SIZE_I as usize;
const POT: i32 = 2;

gfx_vertex_struct!( Vertex {
    a_pos: [f32; 3] = "a_pos",
    a_tex_coord: [f32; 2] = "a_tex_coord",
    a_color: [f32; 4] = "a_color",
    a_light: f32 = "a_light",
});

impl Vertex {
    fn new(pos: [f32; 3], tc: [f32; 2], col: [f32; 4], light: f32) -> Vertex {
        Vertex {
            a_pos: pos,
            a_tex_coord: [tc[0], tc[1]],
            a_color: col,
            a_light: light,
        }
    }
}

const TEXWIDTH:f32 = 4.0;
const TRANS:[[[i32;4];2];8] = [
    [[1,0,0,1,],[1,1,0,0,]],
    [[0,0,1,1,],[1,0,0,1,]],
    [[0,1,1,0,],[0,0,1,1,]],
    [[1,1,0,0,],[0,1,1,0,]],
    [[0,1,1,0,],[1,1,0,0,]],
    [[1,1,0,0,],[1,0,0,1,]],
    [[1,0,0,1,],[0,0,1,1,]],
    [[0,0,1,1,],[0,1,1,0,]],
];

#[derive(Copy,Clone,Debug)]
pub struct Block {
    color: [f32;4],
    textrans:[[[i32;4];2];6],
}

impl Block {
    pub fn new(rng: usize, c: [f32;4]) -> Block {
        Block {
            color: c,
            textrans: [TRANS[(rng>>00)%8], TRANS[(rng>>03)%8], TRANS[(rng>>06)%8], 
                       TRANS[(rng>>09)%8], TRANS[(rng>>12)%8], TRANS[(rng>>15)%8], ]
        }
    }
}

#[derive(Debug, Clone)]
pub enum Spot {
    Empty,
    Full,
    Rich(Box<Block>),
}
use Spot::{Empty, Full, Rich};
impl Default for Spot {
    fn default() -> Spot { Full }
}

impl Spot {
    pub fn is_empty(&self) -> bool {
        if let &Empty = self { true }
        else { false }
    }
}

#[derive(Debug)]
pub struct Chunk {
    pub bigpos: [i32;3],
    small: [[[Spot;SIZE_U];SIZE_U];SIZE_U],
    filled: bool,
    pub request: bool,
}

impl Chunk {
    fn new_full(x: i32, y: i32, z: i32) -> Chunk {
        Chunk{
            bigpos: [x, y, z],
            small: Default::default(),
            filled: true,
            request: false,
        }
    }
    /*pub fn new_empty(x: i32, y: i32, z: i32) -> Chunk {
        Chunk{
            bigpos: [x, y, z],
            small: Default::default(),
            filled: false,
            request: false,
            vertex_data: Vec::new(),
        }
    }*/
    fn at(&self, x: usize, y: usize, z: usize) -> &Spot {
        assert!(x < SIZE_U && y < SIZE_U && z < SIZE_U);
        &self.small[x][y][z]
    }
    fn at_mut(&mut self, x: usize, y: usize, z: usize) -> &mut Spot {
        assert!(x < SIZE_U && y < SIZE_U && z < SIZE_U);
        &mut self.small[x][y][z]
    }
    fn try_at(&self, x: usize, y: usize, z: usize) -> Option<&Spot> {
        if x < SIZE_U && y < SIZE_U && z < SIZE_U {
            Some(&self.small[x][y][z])
        } else {None}
    }
    fn try_at_mut(&mut self, x: usize, y: usize, z: usize) -> Option<&mut Spot> {
        if x < SIZE_U && y < SIZE_U && z < SIZE_U {
            Some(&mut self.small[x][y][z])
        } else {None}
    }
    fn nearly_at<'a>(&'a self, m: &'a Milieu, x: i32, y: i32, z: i32) -> Option<&'a Spot> {
        match self.try_at(x as usize, y as usize, z as usize){
            Some(s) => {Some(s)}
            None => {m.at(self.bigpos[0]+x, self.bigpos[1]+y, self.bigpos[2]+z)}
        }
    }
    fn put(&mut self, x: usize, y: usize, z: usize, b: Block) {
        assert!(x < SIZE_U && y < SIZE_U && z < SIZE_U);
        self.small[x][y][z] = Rich(Box::new(b));
    }
    fn yank(&mut self, x: usize, y: usize, z: usize) -> Option<Block> {
        assert!(x < SIZE_U && y < SIZE_U && z < SIZE_U);
        if let Rich(b) = self.small[x][y][z].clone(){
            self.small[x][y][z] = Empty;
            Some(*b)
        } else {
            None
        }
    }
    fn update(&mut self) {
        self.request = true;
    }
    fn build(&self, m: &Milieu) -> Vec<Vertex> {
        let mut vertices = Vec::new();

        for x in 0..SIZE_I { for y in 0..SIZE_I { for z in 0..SIZE_I {
        if let &Rich(ref b) = self.at(x as usize, y as usize, z as usize) {
        for f in 0..6 {
            let face = cube::Face::from_usize(f).unwrap();
            let d = face.direction();
            let (rx, ry, rz) = (self.bigpos[0]*SIZE_I+x,
                                    self.bigpos[1]*SIZE_I+y,
                                    self.bigpos[2]*SIZE_I+z);
            if let Some(&Empty) = m.at((d[0] + rx), (d[1] + ry), (d[2] + rz)){

                let v = vertices_int(f, [rx, ry, rz]);
                for i in 0..4{
                    vertices.push(Vertex::new(  [v[i][0] as f32, v[i][1] as f32, v[i][2] as f32],
                                                [b.textrans[f][0][i] as f32 / TEXWIDTH,
                                                 b.textrans[f][1][i] as f32 / TEXWIDTH],
                                                 b.color,
                                                 get_light(f, get_surroundings(v[i], m))
                                             ));
                }
            }
        }}
        }}}
        vertices
    }
}

fn get_light(face: usize, crowd: u8) -> f32{
    let a = match crowd {
        0 => 0.0,
        1 => 0.5,
        2 => 0.8,
        _ => 1.0,
    };
    match face {
        0 => 0.2 * a,
        1 => 0.8 * a,
        2 => 0.7 * a,
        3 => 0.3 * a,
        4 => 0.4 * a,
        5 => 0.6 * a,
        _ => 1.0 * a,
    }
}

fn get_surroundings(pos: [i32;3], m: &Milieu) -> u8{
    let (x,y,z) = (pos[0], pos[1], pos[2]);
    let mut total = 0;
    if let Some(&Empty) = m.at(x,y,z) { total += 1; }
    if let Some(&Empty) = m.at(x,y,z-1) { total += 1; }
    if let Some(&Empty) = m.at(x,y-1,z) { total += 1; }
    if let Some(&Empty) = m.at(x,y-1,z-1) { total += 1; }
    if let Some(&Empty) = m.at(x-1,y,z) { total += 1; }
    if let Some(&Empty) = m.at(x-1,y,z-1) { total += 1; }
    if let Some(&Empty) = m.at(x-1,y-1,z) { total += 1; }
    if let Some(&Empty) = m.at(x-1,y-1,z-1) { total += 1; }
    total
}

// Stolen/modified from gfx_voxel to use ints rather than floats
const CUBE_VERTICES: &'static [[i32;3]; 8] = &[
    [0, 0, 0], // 0
    [1, 0, 0], // 1
    [1, 1, 0], // 2
    [0, 1, 0], // 3
    [1, 0, 1], // 4
    [0, 0, 1], // 5
    [0, 1, 1], // 6
    [1, 1, 1]  // 7
];

// Stolen/modified from gfx_voxel to use ints rather than floats
fn vertices_int(face: usize, base: [i32;3]) -> [[i32;3]; 4] {
    use gfx_voxel::array::*;

    cube::QUADS[face].map(|i| CUBE_VERTICES[i]).map(|v| {
        [
            base[0] + v[0],
            base[1] + v[1],
            base[2] + v[2]
        ]
    })
}

pub struct Milieu {
    big: HashMap<(i32, i32, i32), Chunk>,
    cache: HashMap<(i32, i32, i32), Vec<Vertex>>,
    gen: Gen,
    filled: bool,
    special: Option<(i32, i32, i32)>,
}

impl Milieu {
    pub fn new_full() -> Milieu{
        Milieu{
            big: HashMap::new(),
            cache: HashMap::new(),
            gen: Gen::new(0),
            filled: true,
            special: None,
        }
    }
    /*pub fn new_empty() -> Milieu{
        Milieu{
            big: HashMap::new(),
            filled: false,
        }
    }*/
    fn get_chunk(&self, x: i32, y: i32, z: i32) -> Option<&Chunk>{
        self.big.get(&(x, y, z))
    }
    pub fn get_chunk_mut(&mut self, x: i32, y: i32, z: i32) -> &mut Chunk{
        self.big.entry((x, y, z)).or_insert(Chunk::new_full(x, y, z))
    }
    fn splice(&self, x: i32, y: i32, z: i32)
    -> Option<(&Chunk, usize, usize, usize)>{
        let (bx, by, bz) = (x>>POT, y>>POT, z>>POT);
        let (sx, sy, sz) = ((x & SIZE_I-1) as usize,
                            (y & SIZE_I-1) as usize,
                            (z & SIZE_I-1) as usize);
        match self.get_chunk(bx, by, bz){
            Some(c) => Some((c, sx, sy, sz)),
            None => None
        }
    }
    fn splice_mut(&mut self, x: i32, y: i32, z: i32)
    -> (&mut Chunk, usize, usize, usize){
        let (bx, by, bz) = (x>>POT, y>>POT, z>>POT);
        let (sx, sy, sz) = ((x & SIZE_I-1) as usize,
                            (y & SIZE_I-1) as usize,
                            (z & SIZE_I-1) as usize);
        let c = self.get_chunk_mut(bx, by, bz);
        (c, sx, sy, sz)
    }
    pub fn at(&self, x: i32, y: i32, z: i32) -> Option<&Spot>{
        match self.splice(x, y, z){
            Some((c, sx, sy, sz)) => Some(&c.at(sx, sy, sz)),
            None => None
        }
    }
    pub fn at_mut(&mut self, x: i32, y: i32, z: i32) -> &mut Spot{
        let (c, sx, sy, sz) = self.splice_mut(x, y, z);
        c.at_mut(sx, sy, sz)
    }
    pub fn put(&mut self, x: i32, y: i32, z: i32, b: Block){
        let (c, x, y, z) = self.splice_mut(x, y, z);
        c.update();
        c.put(x, y, z, b);
    }
    pub fn yank(&mut self, x: i32, y: i32, z: i32) -> Option<Block>{
        let (c, sx, sy, sz) = self.splice_mut(x, y, z);
        c.update();
        c.yank(sx, sy, sz)
    }
    pub fn pull(&mut self, x: i32, y: i32, z: i32) -> Option<Block>{
        for face in cube::FaceIterator::new() {
            let d = face.direction();
            let (dx, dy, dz) = (x + d[0], y + d[1], z + d[2]);
            let block = Rich(Box::new(self.gen.at(dx,dy,dz)));
            let (c, ux, uy, uz) = self.splice_mut(dx,dy,dz);
            if let b @ &mut Full = c.at_mut(ux, uy, uz){
                *b = block;
            }
            c.update();
        }
        self.yank(x,y,z)
    }
    pub fn get_vertex_data(&mut self) -> (Vec<Vertex>, Vec<u32>){
        let mut vertex_data = Vec::new();
        let mut updates = Vec::new();
        for v in self.big.values() {
            if v.request {
                updates.push((v.bigpos, v.build(&self)));
            }
        };
        for v in self.big.values_mut() { v.request = false; };

        for u in updates.into_iter(){
            let (pos, vertices) = u;
            let (x, y, z) = (pos[0], pos[1], pos[2]);
            self.cache.insert((x,y,z), vertices);
        }

        for v in self.cache.values_mut() {
            vertex_data.extend_from_slice(v.as_slice());
        }

        let mut index_data = Vec::new();
        for i in 0..(vertex_data.len() / 4) {
            let l = (i*4) as u32;
            index_data.extend_from_slice(&[l+0,l+1,l+2,l+0,l+2,l+3]);
        }

        (vertex_data, index_data)
    }
    /*pub fn refresh(&mut self){
        for v in self.big.values_mut(){
            v.build();
        }
    }*/
}