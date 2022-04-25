#[macro_use]
extern crate lazy_static;

use std::time::Instant;

use cgmath::{Matrix4, Point3, Vector3, Vector4};

use crate::game::Runtime;

//use std::time::Instant;

mod game;
mod gl;
mod cube;
mod gl_helper;
mod flying_camera;
mod handle_javascript;
mod landscape;
//mod shadow_shaders;
//mod openglshadow;
mod ground;
mod special_effects;
mod sound;
mod car_main_player;
mod map_display;
mod scenery;
mod passengers;

pub const WIDTH: u32 = 800;
pub const HEIGHT: u32 = 600;

static mut GLOBAL_ID: u128 = 1;

pub fn get_next_id() -> u128 {
    unsafe {
        let next = GLOBAL_ID;
        GLOBAL_ID = GLOBAL_ID + 1;
        next
    }
}

pub fn get_start_time() -> Instant {
    let start = Instant::now();
    return start;
}

pub fn output_elapsed(start: Instant, msg: &str) {
    let duration = start.elapsed();
    println!("*********** {} {:?}", msg, duration);
}

pub fn vec2point(vector: Vector3<f32>) -> Point3<f32> {
    let p = Point3::new(vector.x, vector.y, vector.z);
    return p;
}

pub fn point2vec(point: Point3<f32>) -> Vector3<f32> {
    let v = Vector3::new(point.x, point.y, point.z);
    return v;
}

pub fn print_matrix(m: Matrix4<f32>) {
    println!("x= {}", get_vector4_as_string(m.x));
    println!("y= {}", get_vector4_as_string(m.y));
    println!("z= {}", get_vector4_as_string(m.z));
    println!("w= {}", get_vector4_as_string(m.w));
}

pub fn get_vector4_as_string(v: Vector4<f32>) -> String {
    let s = format!("{},{},{},{}", v.x, v.y, v.z, v.w);
    return s;
}


fn main() {
    let runtime = Runtime::new();

    emscripten_main_loop::run(runtime);
}

pub static mut TICKPRINTCOUNTER: i128 = 0;
pub static mut TICKDEBUG: bool = true;

#[macro_export]
    macro_rules! tickprintln {
    ($($args:expr),*) => {{
        unsafe {
            tickprintcounter=tickprintcounter+1;
            if tickdebug && tickprintcounter%17 == 0 {
                $(
                    print!("{}", $args);
                )*
                println!();
                if tickprintcounter > 999999 {
                    tickprintcounter = tickprintcounter - 999999;
                }
            }
        }
    }}
}
