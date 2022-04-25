extern crate cgmath;

use std::{mem};
use std::os::raw::c_void;
use std::ptr;

use cgmath::*;
use csv::{StringRecord, Trim};

use crate::gl;
use crate::gl_helper::gl_matrix4;
use crate::gl_helper::texture::{create_texture_jpg, create_texture_png};
use crate::scenery::Scenery;
use crate::gl_helper::model::Model;

static mut TEXTURE_LOADED: i32 = -1;

pub struct LandscapeInstance {
    pub id: u128,
    pub matrix: Matrix4<f32>,
}

impl PartialEq for LandscapeInstance {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

pub const SQUARE_SIZE: f32 = 0.2;
pub const SQUARE_ROWS: usize = 32;
pub const SQUARE_COLUMNS: usize = 32;
//pub const MAX: f32 = 0.12;
pub const IMAGE_SCALE_FACTOR: f32 = 256.0;

#[derive(Clone)]
pub struct AtCell {
    pub(crate) height: f32,
}

#[derive(Clone)]
pub struct LandscapeObject {
    pub vertices: Vec<Vector3<f32>>,
    pub description: String,
}

pub struct Landscape {
    //id:u128,
    texture: u32,
    vao: u32,
    height_map: Vec<Vec<AtCell>>,
    pub xyz: Vector3<f32>,
    vertices_count: usize,
    pub filename: String,
    pub landscape_objects: Vec<LandscapeObject>,
    pub scenery_instances: Vec<Scenery>,
}


//const ROUND_Y: f32 = 10000.0;
//pub const MAX_HEIGHT: f32 = 2.5;

impl Landscape {
    pub fn new(gl: &gl::Gl, image_file: &str, xyz: Vector3<f32>, name: String, height_map: &mut Vec<Vec<AtCell>>,tree_model:&Model,house_model:&Model,office1_model:&Model) -> Landscape {
        let filename = format!("resources/road_{}.txt", name);
        let mut landscape_objects: Vec<LandscapeObject> = vec![];
        let mut scenery_instances: Vec<Scenery> = vec![];

        let split_up = 4;
        let grass_min=0.75;
        let grass = SQUARE_SIZE * SQUARE_ROWS as f32 / split_up as f32 ;
        let (_vbo, vao, texture, vertices_count) = unsafe {

            let mut vertices: Vec<f32> = vec![
            ];
            for x in 0..split_up {
                for z in 0..split_up {
                    let minx= x as f32* grass;
                    let minz= z as f32* grass;
                    let mut add:Vec<f32> = vec![
                    minx,       -0.01,minz,    grass_min,grass_min,
                    minx+grass, -0.01,minz+grass,    0.99,0.99,
                    minx+grass, -0.01,minz,    0.99,grass_min,
                    minx,       -0.01,minz,    grass_min,grass_min,
                    minx,       -0.01,minz+grass,    grass_min,0.99,
                    minx+grass, -0.01,minz+grass,    0.99,0.99,
                    ];
                    vertices.append(&mut add);

                }
            }


            println!("-----------------------    {}       {}/{}", name, xyz.x, xyz.z);
            let mut rows: Vec<StringRecord> = vec![];
            let reader = csv::ReaderBuilder::new().has_headers(false).flexible(true)
                .comment(Some(b'#')).trim(Trim::All).from_path(&filename).expect(&filename);
            let mut landscape_object: LandscapeObject = LandscapeObject {
                vertices: vec![],
                description: String::new(),
            };
            for record in reader.into_records() {
                if record.is_ok() {
                    let record = record.unwrap();
                    if (&record[0]).starts_with("o") {
                        let what = &record[1];
                        let x = &record[2].parse::<f32>().unwrap() * SQUARE_SIZE - SQUARE_SIZE * SQUARE_COLUMNS as f32 / 2.0;
                        let y = &record[3].parse::<f32>().unwrap() * SQUARE_SIZE ;
                        let z = &record[4].parse::<f32>().unwrap() * SQUARE_SIZE - SQUARE_SIZE * SQUARE_COLUMNS as f32 / 2.0;
                        match what {
                            "tree" => {
                                let s = Scenery::new_tree(&gl,vec3(x,y,z),&tree_model);
                                scenery_instances.push(s)
                            }
                            "house" => {
                                let s = Scenery::new_house(&gl,vec3(x,y,z),&house_model);
                                scenery_instances.push(s)
                            }
                            "office1" => {
                                let s = Scenery::new_office1(&gl,vec3(x,y,z),&office1_model);
                                scenery_instances.push(s)
                            }
                            _ => println!("Ain't special"),
                        }
                    } else if (&record[0]).starts_with("d") {
                        println!("Description {}", &record[1]);
                        landscape_object.description = String::from(&record[1]);
                    } else if !(&record[0]).starts_with("s") {
                        rows.push(record.clone());
                        if rows.len() == 4 {
                            landscape_object.vertices.push(Landscape::push_record(&mut vertices, &rows[2], None));
                            landscape_object.vertices.push(Landscape::push_record(&mut vertices, &record, None));
                            landscape_object.vertices.push(Landscape::push_record(&mut vertices, &rows[0], None));
                        } else {
                            landscape_object.vertices.push(Landscape::push_record(&mut vertices, &record, None));
                        }
                    } else {
                        if &record[1] == "1" { Landscape::create_side(&mut vertices, &mut rows, 0, 1); }
                        if &record[2] == "1" { Landscape::create_side(&mut vertices, &mut rows, 1, 2); }
                        if &record[3] == "1" { Landscape::create_side(&mut vertices, &mut rows, 2, 3); }
                        if &record[4] == "1" { Landscape::create_side(&mut vertices, &mut rows, 3, 0); }
                        landscape_objects.push(landscape_object.clone());
                        landscape_object.vertices.clear();
                        landscape_object.description.clear();
                        rows.clear();
                    }
                }
            }

            //println!("VERTICES {}", vertices.len());

            let (mut vbo, mut vao) = (0, 0);
            if vertices.len() > 0 {
                gl.GenVertexArrays(1, &mut vao);
                gl.GenBuffers(1, &mut vbo);

                gl.BindVertexArray(vao);

                gl.BindBuffer(gl::ARRAY_BUFFER, vbo);
                gl.BufferData(gl::ARRAY_BUFFER,
                              (vertices.len() * mem::size_of::<gl::types::GLfloat>()) as gl::types::GLsizeiptr,
                              &vertices[0] as *const f32 as *const c_void,
                              gl::STATIC_DRAW);

                let stride = 5 * mem::size_of::<gl::types::GLfloat>() as gl::types::GLsizei;
                gl.VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, stride, ptr::null());
                gl.EnableVertexAttribArray(0);
                gl.VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, stride, (3 * mem::size_of::<gl::types::GLfloat>()) as *const c_void);
                gl.EnableVertexAttribArray(1);
            }

            let texture = if TEXTURE_LOADED == -1 {
                if image_file.ends_with(".png") {
                    TEXTURE_LOADED = create_texture_png(&gl, image_file) as i32;
                } else {
                    TEXTURE_LOADED = create_texture_jpg(&gl, image_file) as i32;
                };
                TEXTURE_LOADED as u32
            } else {
                TEXTURE_LOADED as u32
            };

            (vbo, vao, texture, (vertices.len() as f32 * 0.2) as usize)
        };


        Landscape {
            //id:id,
            texture,
            vao,
            height_map: height_map.clone(),
            xyz,
            vertices_count,
            filename,
            landscape_objects,
            scenery_instances,
        }
    }

    // https://stackoverflow.com/questions/22521982/check-if-point-is-inside-a-polygon
    fn polygon_contains_x_z(x: f32, z: f32, p_xyz: &Vec<Vector3<f32>>) -> bool {
        let mut j = p_xyz.len() - 1;
        let mut does_contain = false;

        for i in 0..p_xyz.len() {
            if (p_xyz[i].z < z && p_xyz[j].z >= z || p_xyz[j].z < z && p_xyz[i].z >= z)
                && (p_xyz[i].x <= x || p_xyz[j].x <= x) {
                does_contain ^= (p_xyz[i].x + (z - p_xyz[i].z) * (p_xyz[j].x - p_xyz[i].x) / (p_xyz[j].z - p_xyz[i].z)) < x;
            }
            j = i;
        }
        return does_contain;
    }
    fn area_x_z(vertices: &Vec<Vector3<f32>>) -> f32
    {
        let mut sum: f32 = 0.0;
        for i in 0..vertices.len() {
            if i == 0 {
                sum += vertices[i].x * (vertices[i + 1].z - vertices[vertices.len() - 1].z);
            } else if i == vertices.len() - 1 {
                sum += vertices[i].x * (vertices[0].z - vertices[i - 1].z);
            } else {
                sum += vertices[i].x * (vertices[i + 1].z - vertices[i - 1].z);
            }
        }
        return 0.5 * sum.abs();
    }

    pub fn object_at(&self, x: f32, z: f32) -> Option<&LandscapeObject> {
        let mut found: Option<&LandscapeObject> = None;
        let mut area: f32 = 99999999.0;
        for landscape_object in self.landscape_objects.iter() {
            let area_for_this_one_pick_the_smallest_one = Landscape::area_x_z(&landscape_object.vertices);
            if area_for_this_one_pick_the_smallest_one < area {
                let does_contain = Landscape::polygon_contains_x_z(x - self.xyz.x, z - self.xyz.z, &landscape_object.vertices);
                if does_contain {
                    found = Some(landscape_object);
                    area = area_for_this_one_pick_the_smallest_one;
                }
            }
        }
        return found;
    }
    pub fn scenery_at(&self, x: f32, z: f32) -> Option<&Scenery> {
        let mut found: Option<&Scenery> = None;
        let xyz = vec3(x,0.0,z);
        for scenery in self.scenery_instances.iter() {
            if (self.xyz + scenery.position).distance(xyz) < scenery.collision_radius {
                found = Some(scenery)
            }
        }
        return found;
    }

    fn create_side(mut vertices: &mut Vec<f32>, rows: &mut Vec<StringRecord>, side: usize, side2: usize) {
        Landscape::push_record(&mut vertices, &rows[side], None);
        Landscape::push_record(&mut vertices, &rows[side2], None);
        Landscape::push_record(&mut vertices, &rows[side2], Some(0.0));

        Landscape::push_record(&mut vertices, &rows[side2], Some(0.0));
        Landscape::push_record(&mut vertices, &rows[side], Some(0.0));
        Landscape::push_record(&mut vertices, &rows[side], None);
    }

    fn push_record(vertices: &mut Vec<f32>, record: &StringRecord, height: Option<f32>) -> Vector3<f32> {
        //println!("RECORD {} {} {}", &record[0], &record[1], &record[2], );
        let offset = 3.2; //SQUARE_SIZE * SQUARE_COLUMNS as f32 /2.0;

        let x = (&record[0]).parse::<f32>().unwrap() * SQUARE_SIZE - offset;
        vertices.push(x);
        let y = if height.is_none() {
            (&record[1]).parse::<f32>().unwrap() * SQUARE_SIZE
        } else {
            height.unwrap() * SQUARE_SIZE
        };
        vertices.push(y);
        let z = (&record[2]).parse::<f32>().unwrap() * SQUARE_SIZE - offset;
        vertices.push(z);
        vertices.push((&record[3]).parse::<f32>().unwrap() / IMAGE_SCALE_FACTOR);
        vertices.push((IMAGE_SCALE_FACTOR - (&record[4]).parse::<f32>().unwrap()) / IMAGE_SCALE_FACTOR);
        return Vector3::new(x, y, z);
    }

    pub fn position_height(&self, x: f32, z: f32) -> f32 {
        let col = ((x - self.xyz.x) / SQUARE_SIZE) + SQUARE_COLUMNS as f32 / 2.0;
        let row = ((z - self.xyz.z) / SQUARE_SIZE) + SQUARE_ROWS as f32 / 2.0;


        let height = if col as usize >= SQUARE_COLUMNS || col < 0.0 || row as usize >= SQUARE_ROWS || row < 0.0 {
            0.0
        } else {
            let height = self.height_map[row as usize][col as usize].height;
            height
        };
        //println!("x,z={},{}   col={} row={}    height={} me={},{}", x, z, col as usize, row as usize, height, self.xyz.x, self.xyz.z);

        return height;
    }

    pub fn render(&mut self, gl: &gl::Gl, view: &Matrix4<f32>, projection: &Matrix4<f32>, here: Matrix4<f32>, _wrapped_position: Vector3<f32>, our_shader: u32) {

        for s in self.scenery_instances.iter_mut() {
            s.render(gl,view,projection,our_shader,_wrapped_position)
        }
        unsafe {
            //gl.UseProgram(our_shader);
            gl.ActiveTexture(gl::TEXTURE0);
            gl.BindTexture(gl::TEXTURE_2D, self.texture);
            gl.BindVertexArray(self.vao);

            gl_matrix4(gl, our_shader, here, "model");
            gl_matrix4(gl, our_shader, *view, "view");
            gl_matrix4(gl, our_shader, *projection, "projection");
            gl.DrawArrays(gl::TRIANGLES, 0, self.vertices_count as i32);
        }
    }
}

