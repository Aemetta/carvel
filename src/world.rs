
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
    small: [[[Spot;16];16];16],
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
        assert!(x < 16 && y < 16 && z < 16);
        &self.small[x][y][z]
    }
    fn at_mut(&mut self, x: usize, y: usize, z: usize) -> &mut Spot {
        assert!(x < 16 && y < 16 && z < 16);
        &mut self.small[x][y][z]
    }
    fn try_at(&self, x: usize, y: usize, z: usize) -> Option<&Spot> {
        if x < 16 && y < 16 && z < 16 {
            Some(&self.small[x][y][z])
        } else {None}
    }
    fn try_at_mut(&mut self, x: usize, y: usize, z: usize) -> Option<&mut Spot> {
        if x < 16 && y < 16 && z < 16 {
            Some(&mut self.small[x][y][z])
        } else {None}
    }
    fn nearly_at<'a>(&'a self, m: &'a Milieu, x: i32, y: i32, z: i32) -> Option<&'a Spot> {
        match self.try_at(x as usize, y as usize, z as usize){
            Some(s) => {Some(s)}
            None => {m.at(self.bigpos[0]+x, self.bigpos[1]+y, self.bigpos[2]+z)}
        }
    }
    fn nearly_at_mut<'a>(&'a mut self, m: &'a mut Milieu, x: i32, y: i32, z: i32, big: [i32;3]) -> &'a mut Spot {
        match self.try_at_mut(x as usize, y as usize, z as usize){
            Some(s) => {s}
            None => {m.at_mut(big[0]+x, big[1]+y, big[2]+z)}
        }
    }
    fn nearly_at_or_full<'a>(&'a self, m: &'a Milieu, x: i32, y: i32, z: i32) -> &'a Spot {
        match self.try_at(x as usize, y as usize, z as usize){
            Some(s) => {s}
            None => {m.at_or_full(self.bigpos[0]+x, self.bigpos[1]+y, self.bigpos[2]+z)}
        }
    }
    fn put(&mut self, x: usize, y: usize, z: usize, b: Block) {
        assert!(x < 16 && y < 16 && z < 16);
        self.small[x][y][z] = Rich(Box::new(b));
        self.update();
    }
    fn pull(&mut self, x: usize, y: usize, z: usize)
     -> (Option<Block>, Option<((i32, i32, i32), Block)>) {
        assert!(x < 16 && y < 16 && z < 16);
        let block = if let Rich(b) = self.small[x][y][z].clone(){
            self.small[x][y][z] = Empty;
            self.update();
            Some(*b)
        } else {
            None
        };

        let mut reacherblock = None;

        let (x, y, z) = (x as i32, y as i32, z as i32);
        for face in cube::FaceIterator::new() {
            let d = face.direction();
            let (dx, dy, dz) = (x + d[0], y + d[1], z + d[2]);
            let big = self.bigpos.clone();
            match self.try_at_mut(dx as usize, dy as usize, dz as usize){
                Some(b @ &mut Full) => {*b = Rich(Box::new(Block::new(0, [0.3, 0.3, 0.3, 1.0])));},
                None => {reacherblock = Some(((dx+big[0], dy+big[1], dz+big[2]), Block::new(0, [0.3, 0.3, 0.3, 1.0])));},
                _ => {},
            }
        }
        self.update();

        (block, reacherblock)
    }
    fn yank(&mut self, x: usize, y: usize, z: usize) -> Option<Block> {
        assert!(x < 16 && y < 16 && z < 16);
        if let Rich(b) = self.small[x][y][z].clone(){
            self.small[x][y][z] = Empty;
            self.update();
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

        for x in 0..16 { for y in 0..16 { for z in 0..16 {
        if let &Rich(ref b) = self.at(x,y,z) {
        for f in 0..6 {
            let face = cube::Face::from_usize(f).unwrap();
            let d = face.direction();
            if let &Empty = self.nearly_at_or_full(m,
                        (d[0] + x as i32), (d[1] + y as i32), (d[2] + z as i32)){

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
        vertices
    }
}

pub struct Milieu {
    big: HashMap<(i32, i32, i32), Chunk>,
    cache: HashMap<(i32, i32, i32), Vec<Vertex>>,
    filled: bool,
}

impl Milieu {
    pub fn new_full() -> Milieu{
        Milieu{
            big: HashMap::new(),
            cache: HashMap::new(),
            filled: true,
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
        let (bx, by, bz) = (x>>4, y>>4, z>>4);
        let (sx, sy, sz) = ((x & 15) as usize,
                            (y & 15) as usize,
                            (z & 15) as usize);
        match self.get_chunk(bx, by, bz){
            Some(c) => Some((c, sx, sy, sz)),
            None => None
        }
    }
    fn splice_mut(&mut self, x: i32, y: i32, z: i32)
    -> (&mut Chunk, usize, usize, usize){
        let (bx, by, bz) = (x>>4, y>>4, z>>4);
        let (sx, sy, sz) = ((x & 15) as usize,
                            (y & 15) as usize,
                            (z & 15) as usize);
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
    pub fn at_or_full(&self, x: i32, y: i32, z: i32) -> &Spot{
        match self.splice(x, y, z){
            Some((c, sx, sy, sz)) => &c.at(sx, sy, sz),
            None => &Empty
        }
    }
    pub fn put(&mut self, x: i32, y: i32, z: i32, b: Block){
        let (c, x, y, z) = self.splice_mut(x, y, z);
        c.put(x, y, z, b);
    }
    pub fn yank(&mut self, x: i32, y: i32, z: i32) -> Option<Block>{
        let (c, sx, sy, sz) = self.splice_mut(x, y, z);
        c.yank(sx, sy, sz)
    }
    pub fn pull(&mut self, x: i32, y: i32, z: i32) -> Option<Block>{
        let mut reacher = None;
        let mut b;
        {
            let (c, sx, sy, sz) = self.splice_mut(x, y, z);
            let (block, reacherblock) = c.pull(sx, sy, sz);
            reacher = reacherblock.clone();
            b = block;
        }
        if let Some(((x,y,z), b)) = reacher {
            self.put(x, y, z, b); }
        b
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