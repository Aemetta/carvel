
use gfx_voxel::cube;
use std::collections::HashMap;

use super::Vertex;

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

#[derive(Debug)]
pub enum Spot {
    Empty,
    Full,
    Rich(Box<Block>),
}
use Spot::{Empty, Full, Rich};
impl Default for Spot {
    fn default() -> Spot { Empty }
}

impl Spot {
    pub fn isEmpty(&self) -> bool {
        if let &Empty = self { true }
        else { false }
    }
}

#[derive(Debug)]
pub struct Chunk {
    bigpos: [i32;3],
    small: [[[Spot;16];16];16],
    filled: bool,
    pub vertex_data: Vec<Vertex>,
}

impl Chunk {
    /*pub fn new_full(x: i32, y: i32, z: i32) -> Chunk {
        Chunk{
            bigpos: [x, y, z],
            small: [[[Full;16];16];16],
            filled: true,
            vertex_data: Vec::new(),
        }
    }*/
    pub fn new_empty(x: i32, y: i32, z: i32) -> Chunk {
        Chunk{
            bigpos: [x, y, z],
            small: Default::default(),
            filled: false,
            vertex_data: Vec::new(),
        }
    }
    pub fn at(&self, x: usize, y: usize, z: usize) -> &Spot {
        assert!(x < 16 && y < 16 && z < 16);
        &self.small[x][y][z]
    }
    pub fn try_at(&self, x: usize, y: usize, z: usize) -> Option<&Spot> {
        if x < 16 && y < 16 && z < 16 {
            Some(&self.small[x][y][z])
        } else {None}
    }
    pub fn nearly_at(&self, x: i32, y: i32, z: i32) -> &Spot {
        match self.try_at(x as usize, y as usize, z as usize){
            Some(s) => {s}
            None => {&Empty}
        }
    }
    pub fn put(&mut self, x: usize, y: usize, z: usize, b: Block) {
        assert!(x < 16 && y < 16 && z < 16);
        self.small[x][y][z] = Rich(Box::new(b));
        self.update();
    }
    /*pub fn yank(&mut self, x: usize, y: usize, z: usize) -> Option<Block> {
        assert!(x < 16 && y < 16 && z < 16);
        if let Rich(b) = self.small[x][y][z]{
            self.small[x][y][z] = Empty;
            self.update();
            Some(*b)
        } else {
            None
        }
    }*/
    fn update(&mut self) {
        let mut vertices = Vec::new();

        for x in 0..16 { for y in 0..16 { for z in 0..16 {
        if let &Rich(ref b) = self.at(x,y,z) {
        for f in 0..6 {
            let face = cube::Face::from_usize(f).unwrap();
            if let &Empty = self.nearly_at((face.direction()[0] + x as i32),
                                          (face.direction()[1] + y as i32),
                                          (face.direction()[2] + z as i32)){

                let (rx, ry, rz) = (self.bigpos[0]*16+x as i32,
                                    self.bigpos[1]*16+y as i32,
                                    self.bigpos[2]*16+z as i32);
                let v = face.vertices([rx as f32, ry as f32, rz as f32], [1f32,1f32,1f32]);
                for i in 0..4{
                    vertices.push(Vertex::new([v[i][0], v[i][1], v[i][2]],
                                                [b.textrans[f][0][i] as f32 / TEXWIDTH,
                                                 b.textrans[f][1][i] as f32 / TEXWIDTH],
                                                 b.color));
                }
            }
        }}
        }}}
        self.vertex_data.clear();
        self.vertex_data.append(&mut vertices);
    }
}

pub struct Milieu {
    big: HashMap<(i32, i32, i32), Chunk>,
    filled: bool,
}

impl Milieu {
    pub fn new_full() -> Milieu{
        Milieu{
            big: HashMap::new(),
            filled: true,
        }
    }
    pub fn new_empty() -> Milieu{
        Milieu{
            big: HashMap::new(),
            filled: false,
        }
    }
    fn get_chunk(&self, x: i32, y: i32, z: i32) -> Option<&Chunk>{
        self.big.get(&(x, y, z))
    }
    pub fn get_chunk_mut(&mut self, x: i32, y: i32, z: i32) -> &mut Chunk{
        self.big.entry((x, y, z)).or_insert(Chunk::new_empty(x, y, z))
    }
    fn splice(&self, x: i32, y: i32, z: i32)
    -> Option<(&Chunk, usize, usize, usize)>{
        let (bx, by, bz) = (x/16, y/16, z/16);
        let (sx, sy, sz) = (x%16, y%16, z%16);
        match self.get_chunk(bx, by, bz){
            Some(c) => Some((c, sx as usize, sy as usize, sz as usize)),
            None => None
        }
    }
    fn splice_mut(&mut self, x: i32, y: i32, z: i32)
    -> (&mut Chunk, usize, usize, usize){
        let (bx, by, bz) = (x/16, y/16, z/16);
        let (sx, sy, sz) = (x%16, y%16, z%16);
        let c = self.get_chunk_mut(bx, by, bz);
        (c, sx as usize, sy as usize, sz as usize)
    }
    pub fn at(&self, x: i32, y: i32, z: i32) -> Option<&Spot>{
        match self.splice(x, y, z){
            Some((c, sx, sy, sz)) => Some(&c.at(sx, sy, sz)),
            None => None
        }
    }
    pub fn put(&mut self, x: i32, y: i32, z: i32, b: Block){
        let (c, x, y, z) = self.splice_mut(x, y, z);
        c.put(x, y, z, b);
    }
    /*pub fn yank(&mut self, x: i32, y: i32, z: i32) -> Option<Block>{
        let (c, sx, sy, sz) = self.splice_mut(x, y, z);
        c.yank(sx, sy, sz)
    }*/
    pub fn get_vertex_data(&self) -> (Vec<Vertex>, Vec<u32>){
        let mut vertex_data = Vec::new();
        for v in self.big.values() {
            vertex_data.extend_from_slice(v.vertex_data.as_slice());
        };

        let mut index_data = Vec::new();
        for i in 0..(vertex_data.len() / 4) {
            let l = (i*4) as u32;
            index_data.extend_from_slice(&[l+0,l+1,l+2,l+0,l+2,l+3]);
        }

        (vertex_data, index_data)
    }
    pub fn refresh(&mut self){
        for v in self.big.values_mut(){
            v.update();
        }
    }
}