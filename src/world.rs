use gen::Gen;

use gfx_voxel::cube;
use std::collections::HashMap;
use std::cell::RefCell;

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

#[derive(Clone,Debug)]
pub struct Block {
    color: [f32;4],
    textrans: [[[i32;4];2];6],
    vertices: RefCell<Vec<Vertex>>,
}

impl Block {
    pub fn new(rng: usize, c: [f32;4]) -> Block {
        Block {
            color: c,
            textrans: [TRANS[(rng>>00)%8], TRANS[(rng>>03)%8], TRANS[(rng>>06)%8], 
                       TRANS[(rng>>09)%8], TRANS[(rng>>12)%8], TRANS[(rng>>15)%8], ],
            vertices: RefCell::new(Vec::new()),
        }
    }
    pub fn get_vertex_data(&self) -> &RefCell<Vec<Vertex>> {
        &self.vertices
    }
    pub fn update_surface(&self, x: i32, y: i32, z: i32, w: &InfiniteWorld, shiny: f32) {
        let mut vertices = Vec::new();

        for f in 0..6 {
            let face = cube::Face::from_usize(f).unwrap();
            let d = face.direction();
            if let Some(&Empty) = w.at((d[0] + x), (d[1] + y), (d[2] + z)){

                let v = vertices_int(f, [x, y, z]);
                for i in 0..4{
                    vertices.push(Vertex::new(
                        [v[i][0] as f32, v[i][1] as f32, v[i][2] as f32],
                            [self.textrans[f][0][i] as f32 / TEXWIDTH,
                             self.textrans[f][1][i] as f32 / TEXWIDTH],
                        self.color,
                        shiny * get_light(f, get_surroundings(v[i], w))
                    ));
                }
            }
        }
        *self.vertices.borrow_mut() = vertices;
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

fn get_surroundings(pos: [i32;3], w: &InfiniteWorld) -> u8{
    let (x,y,z) = (pos[0], pos[1], pos[2]);
    let mut total = 0;
    if let Some(&Empty) = w.at(x,y,z) { total += 1; }
    if let Some(&Empty) = w.at(x,y,z-1) { total += 1; }
    if let Some(&Empty) = w.at(x,y-1,z) { total += 1; }
    if let Some(&Empty) = w.at(x,y-1,z-1) { total += 1; }
    if let Some(&Empty) = w.at(x-1,y,z) { total += 1; }
    if let Some(&Empty) = w.at(x-1,y,z-1) { total += 1; }
    if let Some(&Empty) = w.at(x-1,y-1,z) { total += 1; }
    if let Some(&Empty) = w.at(x-1,y-1,z-1) { total += 1; }
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
    pub fn is_rich(&self) -> bool {
        if let &Empty = self { false }
        else if let &Full = self { false }
        else { true }
    }
    pub fn unwrap_mut(&mut self) -> &mut Box<Block> {
        if let &mut Rich(ref mut b) = self { b }
        else {panic!("Unwrapped a Spot with a non-rich value");}
    }
}

const SIZE_I: i32 = 16;
const SIZE_U: usize = SIZE_I as usize;
const POT: i32 = 4;

#[derive(Debug)]
pub struct Chunk {
    pub bigpos: [i32;3],
    small: [[[Spot;SIZE_U];SIZE_U];SIZE_U],
    filled: bool,
    pub request: RefCell<bool>,
}

impl Chunk {
    fn new_full(x: i32, y: i32, z: i32) -> Chunk {
        Chunk{
            bigpos: [x, y, z],
            small: Default::default(),
            filled: true,
            request: RefCell::new(true),
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
    fn put(&mut self, x: usize, y: usize, z: usize, b: Block) -> &mut Block {
        assert!(x < SIZE_U && y < SIZE_U && z < SIZE_U);
        self.small[x][y][z] = Rich(Box::new(b));
        self.at_mut(x, y, z).unwrap_mut()
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
    fn update(&self) {
        *self.request.borrow_mut() = true;
    }
    fn build_surface(&self) -> Vec<Vertex> {
        let mut vertices = Vec::<Vertex>::new();

        for x in 0..SIZE_I { for y in 0..SIZE_I { for z in 0..SIZE_I {
            if let &Rich(ref b) = self.at(x as usize, y as usize, z as usize) {
                vertices.extend_from_slice(
                    b.get_vertex_data()
                    .borrow().as_slice());
            }
        }}}
        vertices
    }
}

pub struct InfiniteWorld {
    chunks: HashMap<(i32, i32, i32), Chunk>,
}
use std::collections::hash_map::ValuesMut;

impl InfiniteWorld {
    pub fn new_full() -> InfiniteWorld{
        InfiniteWorld{
            chunks: HashMap::new(),
        }
    }
    fn get_chunk(&self, x: i32, y: i32, z: i32) -> Option<&Chunk>{
        self.chunks.get(&(x, y, z))
    }
    pub fn get_chunk_mut(&mut self, x: i32, y: i32, z: i32) -> &mut Chunk{
        self.chunks.entry((x, y, z)).or_insert(Chunk::new_full(x, y, z))
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
        c.update();
        (c, sx, sy, sz)
    }
    pub fn at(&self, x: i32, y: i32, z: i32) -> Option<&Spot>{
        match self.splice(x, y, z){
            Some((c, sx, sy, sz)) => Some(&c.at(sx, sy, sz)),
            None => None
        }
    }
    pub fn at_update(&self, x: i32, y: i32, z: i32) -> Option<&Spot>{
        match self.splice(x, y, z){
            Some((c, sx, sy, sz)) => {c.update(); Some(&c.at(sx, sy, sz))},
            None => None
        }
    }
    pub fn chunks(&mut self) -> ValuesMut<(i32, i32, i32), Chunk>{
        self.chunks.values_mut()
    }
}

pub struct Milieu {
    pub world: InfiniteWorld,
    surfacecache: HashMap<(i32, i32, i32), Vec<Vertex>>,
    gen: Gen,
    shiny: Vec<(i32, i32, i32, f32)>,
}

impl Milieu {
    pub fn new_full(seed: usize) -> Milieu{
        Milieu{
            world: InfiniteWorld::new_full(),
            surfacecache: HashMap::new(),
            gen: Gen::new(seed),
            shiny: Vec::new(),
        }
    }
    pub fn put(&mut self, rx: i32, ry: i32, rz: i32, b: Block){
        b.update_surface(rx, ry, rz, &self.world, 1.0);
        let (c, x, y, z) = self.world.splice_mut(rx, ry, rz);
        c.put(x, y, z, b);
    }
    pub fn yank(&mut self, x: i32, y: i32, z: i32) -> Option<Block>{
        let (c, sx, sy, sz) = self.world.splice_mut(x, y, z);
        c.yank(sx, sy, sz)
    }
    pub fn pull(&mut self, x: i32, y: i32, z: i32) -> Option<Block>{
        let ret = self.yank(x,y,z);
        for face in cube::FaceIterator::new() {
            let d = face.direction();
            let (dx, dy, dz) = (x + d[0], y + d[1], z + d[2]);
            let (c, ux, uy, uz) = self.world.splice_mut(dx,dy,dz);
            if let b @ &mut Full = c.at_mut(ux, uy, uz){
                let block = Rich(Box::new(self.gen.at(dx,dy,dz)));
                *b = block;
            }
        }
        for dx in x-1..x+2 { for dy in y-1..y+2 { for dz in z-1..z+2 {
            if let Some(s) = self.world.at_update(dx,dy,dz) {
            if let &Rich(ref b) = s {
                b.update_surface(dx, dy, dz, &self.world, 1.0);
            }}
        }}}
        ret
    }
    pub fn viewcast(&self, pos: [f32;3], dir: [f32;3])
            -> (Option<(i32, i32, i32)>, Option<(i32, i32, i32)>){
        let mut full = None;
        let mut empty = None;

        use line_drawing::{ VoxelOrigin, WalkVoxels };
        use vecmath;
        let dir = vecmath::vec3_neg(dir);
        let dir = vecmath::vec3_scale(dir, 10.0);
        let end = vecmath::vec3_add(pos, dir);

        let mut temp = None;
        for (_, (x, y, z)) in WalkVoxels::<f32, i32>::new(
                        (pos[0], pos[1], pos[2]),
                        (end[0], end[1], end[2]),
                        &VoxelOrigin::Center)
                        .enumerate() {

            if let Some(b) = self.world.at(x, y, z){
                if b.is_rich() {
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
    pub fn set_shiny(&mut self, x: i32, y: i32, z: i32, shine: f32) {
        if let Some(s) = self.world.at_update(x,y,z) {
        if let &Rich(ref b) = s {
            b.update_surface(x, y, z, &self.world, shine);
        }}
        self.shiny.push((x,y,z,shine));
    }
    pub fn clear_shiny(&mut self) {
        for i in 0..self.shiny.len() {
            let (x, y, z, _) = self.shiny[i];
            if let Some(s) = self.world.at_update(x,y,z) {
            if let &Rich(ref b) = s {
                b.update_surface(x, y, z, &self.world, 1.0);
            }}
        }
        self.shiny = Vec::new();
    }
    pub fn get_vertex_data(&mut self) -> (Vec<Vertex>, Vec<u32>){
        let mut vertex_data = Vec::new();
        let mut updates = Vec::new();
        for c in self.world.chunks() {
            if *c.request.borrow() {
                updates.push((c.bigpos, c.build_surface()));
                *c.request.borrow_mut() = false;
            }
        };

        for u in updates.into_iter(){
            let (pos, vertices) = u;
            let (x, y, z) = (pos[0], pos[1], pos[2]);
            self.surfacecache.insert((x,y,z), vertices);
        }

        for v in self.surfacecache.values_mut() {
            vertex_data.extend_from_slice(v.as_slice());
        }

        let mut index_data = Vec::new();
        for i in 0..(vertex_data.len() / 4) {
            let l = (i*4) as u32;
            index_data.extend_from_slice(&[l+0,l+1,l+2,l+0,l+2,l+3]);
        }
        self.clear_shiny();

        (vertex_data, index_data)
    }
    /*pub fn refresh(&mut self){
        for v in self.chunks.values_mut(){
            v.build_surface();
        }
    }*/
}