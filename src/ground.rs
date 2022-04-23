use std::collections::HashMap;
use std::time::Instant;

use cgmath::{Matrix4, vec2, vec3, Vector2, Vector3};
use rand::Rng;

use crate::{get_start_time, gl, output_elapsed};
//use crate::gl_helper::model::Model;
use crate::landscape::{AtCell, Landscape, LandscapeObject, SQUARE_COLUMNS, SQUARE_ROWS, SQUARE_SIZE};
use crate::passengers::Passenger;
use crate::scenery::Scenery;

pub struct Ground {
    pub land: Vec<Vec<Landscape>>,
    pub player_pos: Vector3<f32>,
    to_display: HashMap<Vector2<i32>, Vector2<i32>>,
}

pub(crate) const BY: usize = 5;

impl Ground {
    pub(crate) const MUL: f32 = SQUARE_SIZE * SQUARE_COLUMNS as f32;

    pub fn new(gl: &gl::Gl) -> Ground {
        let start = get_start_time();
        let mut land: Vec<Vec<Landscape>> = vec![vec![]];
        let mut height_map: Vec<Vec<AtCell>> = vec![vec![AtCell { height: 0.0 }; SQUARE_COLUMNS * BY]; SQUARE_COLUMNS * BY];

        let offset_x = BY as f32 * Ground::MUL * 0.5 - SQUARE_COLUMNS as f32 * SQUARE_SIZE * 0.5;
        let offset_z = BY as f32 * Ground::MUL * 0.5 - SQUARE_ROWS as f32 * SQUARE_SIZE * 0.5;

        let tree_model = Scenery::setup_tree(&gl);
        let house_model = Scenery::setup_house(&gl);
        let office1_model = Scenery::setup_office1(&gl);

        for y in 0..BY {
            land.push(vec![]);

            for x in 0..BY {
                let mut cell_height_map: Vec<Vec<AtCell>> = vec![vec![AtCell { height: 7.9 }; SQUARE_COLUMNS]; SQUARE_COLUMNS];
                for cell_y in 0..SQUARE_ROWS {
                    for cell_x in 0..SQUARE_COLUMNS {
                        let source_x = cell_x + x * SQUARE_COLUMNS;
                        let source_y = cell_y + y * SQUARE_ROWS;
                        let copy = height_map[source_y][source_x].clone();
                        cell_height_map[cell_y][cell_x] = copy;
                    }
                }
                let here = vec3(x as f32 * Ground::MUL - offset_x, 0.0, y as f32 * Ground::MUL - offset_z);
                let land_cell = Landscape::new(&gl, "resources/ground.png", here, format!("{}_{}", x, y), &mut cell_height_map,
                                               &tree_model, &house_model, &office1_model);

                land[y].push(land_cell);
                assert_eq!(land[y][x].xyz, here);
            }
        }

        output_elapsed(start, "Time elapsed in expensive_ground() is");

        Ground {
            land,
            player_pos: vec3(0.0, 0.0, 0.0),
            to_display: HashMap::new(),
        }
    }


    pub fn set_player_position(&mut self, x: f32, z: f32) {
        self.player_pos.x = x;
        self.player_pos.z = z;
    }
    pub fn currently_under_landscape(&self, x: f32, z: f32) -> &Landscape {
        let (xx, zz) = Ground::get_current_cell(x, z);
        &self.land[zz][xx]
    }
    pub fn object_at(&self, x: f32, z: f32) -> Option<&LandscapeObject> {
        let (xx, zz) = Ground::get_current_cell(x, z);
        return self.land[zz][xx].object_at(x, z);
    }
    pub fn scenery_at(&self, x: f32, z: f32) -> (Option<&Scenery>,&Vector3<f32>) {
        let (xx, zz) = Ground::get_current_cell(x, z);
        return (self.land[zz][xx].scenery_at(x, z),&self.land[zz][xx].xyz);
    }

    pub fn position_height(&self, x: f32, z: f32) -> f32 {
        let (xx, zz) = Ground::get_current_cell(x, z);
        let height = self.land[zz][xx].position_height(x, z);

        height
    }

    pub(crate) fn get_current_cell(x: f32, z: f32) -> (usize, usize) {
        let mut xx = ((x + Ground::MUL * BY as f32 * 0.5) / Ground::MUL) as usize;
        let mut zz = ((z + Ground::MUL * BY as f32 * 0.5) / Ground::MUL) as usize;
        if xx >= BY { xx = BY - 1 }
        if zz >= BY { zz = BY - 1 }
        (xx, zz)
    }
    pub fn update(&mut self, _gl: &gl::Gl, player_position: Vector3<f32>, _level: i32, camera_angle: f32, _delta: f32) {
        self.sort_out_what_to_display(player_position, camera_angle);
    }


    fn sort_out_what_to_display(&mut self, player_position: Vector3<f32>, camera_angle: f32) {
        self.to_display.clear();

        let (current_xx, current_zz) = Ground::get_current_cell(player_position.x, player_position.z);

        let xx = current_xx as f32;
        let zz = current_zz as f32;

        let v = vec2(xx as i32, zz as i32);
        self.to_display.insert(v, v);

        self.make_sure_everything_around_player_shown(xx, zz);

        // -1 so make sure when low we still show cell
        for going_away in -1..(BY as i32 * 2) {
            for a in (-120..130).step_by(5) {
                let apply = Vector3 {
                    x: (a as f32 - camera_angle).to_radians().sin() * going_away as f32,
                    y: 0.0,
                    z: (a as f32 - camera_angle).to_radians().cos() * -going_away as f32,
                };
                let v = vec2((xx + apply.x) as i32, (zz + apply.z) as i32);
                self.to_display.insert(v, v);
            }
        }
    }

    fn make_sure_everything_around_player_shown(&mut self, xx: f32, zz: f32) {
        let v = vec2(xx as i32 + 1, zz as i32);
        self.to_display.insert(v, v);
        let v = vec2(xx as i32 + 1, zz as i32 + 1);
        self.to_display.insert(v, v);
        let v = vec2(xx as i32, zz as i32 + 1);
        self.to_display.insert(v, v);
        let v = vec2(xx as i32 - 1, zz as i32);
        self.to_display.insert(v, v);
        let v = vec2(xx as i32, zz as i32 - 1);
        self.to_display.insert(v, v);
        let v = vec2(xx as i32 - 1, zz as i32 - 1);
        self.to_display.insert(v, v);
    }


    pub fn render(&mut self, gl: &gl::Gl, view: &Matrix4<f32>, projection: &Matrix4<f32>, _player_position: Vector3<f32>, _camera_angle: f32, our_shader: u32,
                  passengers: &mut Vec<Passenger>,tick:i128) {
        for xz in self.to_display.values() {
            let yyy = wrap_value(xz.y);
            let xxx = wrap_value(xz.x);
            let offset = BY as f32 * Ground::MUL / 2.0 - Ground::MUL / 2.0;
            let position = vec3(xz.x as f32 * (Ground::MUL) - offset, 0.0, xz.y as f32 * (Ground::MUL) - offset);
            let here = Matrix4::<f32>::from_translation(position);
            //print!("yyy={},xxx={}  here {},{}      {},{} ",yyy,xxx,position.x,position.z,self.land[yyy][xxx].xyz.x,self.land[yyy][xxx].xyz.z);
            self.land[yyy as usize][xxx as usize].render(gl, view, projection, here, position, our_shader);

            let land_xyz = self.land[yyy as usize][xxx as usize].xyz;

            let avatar_offset = position - land_xyz;
            for passenger in passengers.iter_mut() {
                let (xx, zz) = Ground::get_current_cell(passenger.movement_collision.position.x, passenger.movement_collision.position.z);
                if xx as i32 == xxx && zz as i32 == yyy {
                    passenger.render(gl, &view, &projection, our_shader, avatar_offset,tick);
                }
            }
        }
    }
}

fn wrap_value(v: i32) -> i32 {
    let mut r = v;
    if r >= BY as i32 {
        while r >= BY as i32 {
            r = r - BY as i32;
        }
    }
    if r < 0 {
        while r < 0 {
            r = r + BY as i32;
        }
    }
    if r < 0 { r = -99 };
    if r >= BY as i32 { r = -99 };
    return r;
}