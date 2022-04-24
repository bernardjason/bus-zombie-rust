extern crate cgmath;

use std::{fs, io, mem};
use std::borrow::Borrow;
use std::collections::HashMap;
use std::ops::{Index, Mul};
use std::os::raw::c_void;
use std::ptr;

use csv::{StringRecord, Trim};

use crate::{gl, get_start_time, output_elapsed};
use crate::gl_helper::{gl_matrix4, gl_vec3, gl_vec2};
use crate::gl_helper::shader::create_shader;
use crate::gl_helper::texture::{create_texture_jpg, create_texture_png};
use crate::ground::BY;
use crate::landscape::{IMAGE_SCALE_FACTOR, SQUARE_COLUMNS, SQUARE_ROWS, SQUARE_SIZE};

use self::cgmath::{Deg, frustum, Matrix4, ortho, perspective, vec3, Vector3, vec2};

//let position_map = vec3(0.750, 0.650, 0.0);

const FS: &str = "#version 300 es
precision mediump float;
out vec4 FragColor;
in vec2 TexCoord;
in vec3 use_colour;
in vec4 xyz;


uniform sampler2D texture0;

void main()
{
	vec4 t = texture(texture0, TexCoord) ;
	if ( (use_colour.x > 0.0 || use_colour.y > 0.0 || use_colour.z > 0.0 ) ) {
	    t = vec4(use_colour.x,use_colour.y,use_colour.z,1.0);
    }
    if ( xyz.x < 0.08 || xyz.x > 0.31 || xyz.y < 0.08 || xyz.y > 0.40 ) {
        discard;
    }
	FragColor = t;
}";
const VS:&str  = "#version 300 es
precision mediump float;

layout (location = 0) in vec3 aPos;
layout (location = 1) in vec2 aTexCoord;

out vec2 TexCoord;
out vec3 use_colour;
out vec4 xyz;

uniform mat4 model;
uniform vec3 colour;
uniform mat4 projection;
uniform vec2 screen;

void main()
{
	gl_Position = projection * model *  vec4(aPos.x,aPos.y * -1.0,aPos.z, 1.0f);
    xyz = gl_Position;
    gl_Position.x = gl_Position.x + screen.x;
    gl_Position.y = gl_Position.y + screen.y;

	TexCoord = vec2(aTexCoord.x, aTexCoord.y);
	use_colour = colour;

}
";


pub struct MapDisplay {
    pub vao: u32,
    pub shader: u32,
    pub texture: u32,
    pub vertices_count: usize,
    pub map_vao: u32,
    pub map_vertices_count: usize,
}

const SCALE: f32 = 0.125;

impl MapDisplay {
    const red_left:f32 = 0.99;
    const red_top:f32 = 0.01;
    const red_right:f32 = 1.00;

    const white_left:f32 = 0.98;
    const white_top:f32 = 0.01;
    const white_right:f32 = 0.985;

    const green_left:f32 = 0.96;
    const green_top:f32 = 0.01;
    const green_right:f32 = 0.965;

    const black_left:f32 = 0.95;
    const black_top:f32 = 0.01;
    const black_right:f32 = 0.955;

    const grey_left:f32 = 0.94;
    const grey_top:f32 = 0.01;
    const grey_right:f32 = 0.945;

    pub fn new(gl: &gl::Gl) -> MapDisplay {
        let start = get_start_time();
        let (our_shader, texture, vao, vertices_count) = unsafe {


            let background_z = -1.0;
            let mut vertices: Vec<f32> = vec![
                0.0, 0.0 , background_z,       MapDisplay::grey_left,0.0,
                99.0, 0.0, background_z,       MapDisplay::grey_right,0.0,
                99.0, 99.0, background_z,       MapDisplay::grey_right,MapDisplay::grey_top,

                0.0, 0.0,    background_z,    MapDisplay::grey_left,0.0,
                0.0, 99.0,    background_z,    MapDisplay::grey_left,MapDisplay::grey_top,
                99.0, 99.0,    background_z,    MapDisplay::grey_right,MapDisplay::grey_top,

            ];

            MapDisplay::create_vertices_of_ground_map(&mut vertices);

            println!("MAP VERTICES {}", vertices.len());

            let our_shader = create_shader(&gl, VS, FS, None);

            let vao = MapDisplay::bind_vertices(gl, &mut vertices);

            let image_file = "resources/ground.png";
            let texture = if image_file.ends_with(".png") {
                create_texture_png(&gl, image_file)
            } else {
                create_texture_jpg(&gl, image_file)
            };

            (our_shader, texture, vao, (vertices.len() as f32) as usize)
        };
        let mut map_triangle: Vec<f32> = vec![
            -0.08 * SCALE, -0.08 * SCALE, 0.0, 0.0, 0.0,
            0.08 * SCALE, -0.08 * SCALE, 0.0, 0.0, 0.0,
            0.0, 0.08 * SCALE, 0.0, 0.0, 0.0,
        ];

        let map_vao = unsafe { MapDisplay::bind_vertices(gl, &mut map_triangle) };

        output_elapsed(start,"MAP new() completed in ");

        MapDisplay {
            shader: our_shader,
            texture,
            vao,
            vertices_count,
            map_vao,
            map_vertices_count: map_triangle.len(),
        }
    }

    unsafe fn bind_vertices(gl: &gl::Gl, mut vertices: &mut Vec<f32>) -> u32 {
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
        vao
    }

    fn create_vertices_of_ground_map(mut vertices: &mut Vec<f32>) {
        for xx in 0..BY {
            for zz in 0..BY {
                let filename = format!("resources/road_{}_{}.txt", xx, zz);
                println!("MAP FILE -----------------------    {} ", filename);
                let mut rows: Vec<StringRecord> = vec![];
                let mut reader = csv::ReaderBuilder::new().has_headers(false).flexible(true)
                    .comment(Some(b'#')).trim(Trim::All).from_path(&filename).expect(&filename);
                for record in reader.into_records() {
                    if record.is_ok() {
                        let record = record.unwrap();
                        if (&record[0]).starts_with("o") {

                            let what = &record[1];
                            let x = &record[2].parse::<f32>().unwrap() * SCALE;
                            let z = &record[4].parse::<f32>().unwrap() * SCALE;
                            match what {
                                "house" => {
                                    MapDisplay::draw_house_on_map(&mut vertices, xx, zz, x, z);
                                }
                                _ => println!("Ain't special"),
                            }


                        } else if (&record[0]).starts_with("d") {
                            println!("Description {}", &record[1]);
                        } else if !(&record[0]).starts_with("s") {
                            rows.push(record.clone());
                            if rows.len() == 4 {
                                MapDisplay::push_record(&mut vertices, &rows[2], xx as f32, zz as f32);
                                MapDisplay::push_record(&mut vertices, &record, xx as f32, zz as f32);
                                MapDisplay::push_record(&mut vertices, &rows[0], xx as f32, zz as f32);
                            } else {
                                MapDisplay::push_record(&mut vertices, &record, xx as f32, zz as f32);
                            }
                        } else {
                            rows.clear();
                        }
                    }
                }
            }
        }
    }
    fn draw_house_on_map(vertices: &mut &mut Vec<f32>, xx: usize, zz: usize, x: f32, z: f32) {
        let size = 0.03;
        vertices.push(x + xx as f32 * SCALE);
        vertices.push(z + zz as f32 * SCALE);
        vertices.push(0.0);
        vertices.push(MapDisplay::white_left); vertices.push(0.0);
        vertices.push(x + xx as f32 * SCALE + size);
        vertices.push(z + zz as f32 * SCALE);
        vertices.push(0.0);
        vertices.push(MapDisplay::white_right); vertices.push(0.00);
        vertices.push(x + xx as f32 * SCALE + size);
        vertices.push(z + zz as f32 * SCALE + size);
        vertices.push(0.0);
        vertices.push(MapDisplay::white_right); vertices.push(MapDisplay::white_top);

        vertices.push(x + xx as f32 * SCALE);
        vertices.push(z + zz as f32 * SCALE);
        vertices.push(0.0);
        vertices.push(MapDisplay::white_left); vertices.push(0.0);
        vertices.push(x + xx as f32 * SCALE);
        vertices.push(z + zz as f32 * SCALE + size);
        vertices.push(0.0);
        vertices.push(MapDisplay::white_left); vertices.push(MapDisplay::white_top);
        vertices.push(x + xx as f32 * SCALE + size);
        vertices.push(z + zz as f32 * SCALE + size);
        vertices.push(0.0);
        vertices.push(MapDisplay::white_right); vertices.push(MapDisplay::white_top);
    }
    fn push_record(vertices: &mut Vec<f32>, record: &StringRecord, x: f32, y: f32) {
        //println!("RECORD {} {} {}", &record[0], &record[1], &record[2], );
        //let offset = 3.2; //SQUARE_SIZE * SQUARE_COLUMNS as f32 /2.0;

        MapDisplay::push_xyz(vertices, &record, x, y);

        //println!("MAP X={} ,Y={} ,Z={} ", x, y, z);

        vertices.push((&record[3]).parse::<f32>().unwrap() / IMAGE_SCALE_FACTOR);
        vertices.push((IMAGE_SCALE_FACTOR - (&record[4]).parse::<f32>().unwrap()) / IMAGE_SCALE_FACTOR);
    }

    fn push_xyz(vertices: &mut Vec<f32>, record: &&StringRecord, x: f32, y: f32) {
        let xx = x * SCALE;
        let yy = y * SCALE;

        let x = (&record[0]).parse::<f32>().unwrap() / SQUARE_COLUMNS as f32;
        vertices.push(x * SCALE + xx);
        let y = (&record[2]).parse::<f32>().unwrap() / SQUARE_ROWS as f32;
        vertices.push(y * SCALE + yy);
        let z = 0.0;
        vertices.push(z);
    }
    pub fn render(&mut self, gl: &gl::Gl, player_position: Vector3<f32>) {
        let position_map = vec3(0.4, 0.4, 0.0);
        /*  BY=5
         */
        let scale_xz = 0.0193;
        let centre_map_offset = vec3(-0.31, 0.31, 0.0);
        let map_offset = vec3(0.4, 0.4, 0.0);
        /*
        // BY 3
        let scale_xz = 0.020;
        let centre_map_offset = vec3(-0.193, 0.193, 0.0);
        let map_offset = vec3(0.62, 0.62, 0.0);
         */


        let colour = vec3(0.0, 0.0, 0.0);
        let player_model: Matrix4<f32> = Matrix4::from_translation(vec3(0.0, 0.0, 0.10) + position_map);

        let red = vec3(1.0, 0.0, 0.0);

        let projection: Matrix4<f32> =
            ortho(-2.0, 2.0, -2.0, 2.0, -1.0, 100.0);

        unsafe {
            gl.ActiveTexture(gl::TEXTURE0);
            gl.BindTexture(gl::TEXTURE_2D, self.texture);
            gl.UseProgram(self.shader);

            gl_vec3(gl, self.shader, colour, "colour"); //        shader.setVec3("lightPos", lightPos);
            gl_matrix4(gl, self.shader, projection, "projection");

            gl.ActiveTexture(gl::TEXTURE0);
            gl.BindTexture(gl::TEXTURE_2D, self.texture);
            gl.BindVertexArray(self.vao);

            for x in -1..2 {
                for y in -1..2 {
                    let this_offset = centre_map_offset + position_map - vec3(map_offset.x * x as f32, map_offset.y * y as f32, 0.0);

                    let scaled_player_position = vec3(player_position.x * -scale_xz + x as f32, player_position.z * scale_xz + y as f32, 0.0) +
                        this_offset;
                    let model: Matrix4<f32> = Matrix4::from_translation(scaled_player_position);
                    gl_matrix4(gl, self.shader, model, "model");
                    gl_vec2(gl, self.shader, vec2(0.670,0.570), "screen");
                    gl.DrawArrays(gl::TRIANGLES, 0, self.vertices_count as i32 /5);
                }
            }


            gl_vec2(gl, self.shader, vec2(0.670,0.570), "screen");
            gl_vec3(gl, self.shader, red, "colour");
            gl.BindVertexArray(self.map_vao);
            gl_matrix4(gl, self.shader, player_model, "model");
            gl_matrix4(gl, self.shader, projection, "projection");
            gl.DrawArrays(gl::TRIANGLES, 0, self.map_vertices_count as i32 /5);
        }
    }
}