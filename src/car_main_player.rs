
use cgmath::{Angle, Deg, EuclideanSpace, InnerSpace, Matrix4, Point3, Transform, vec3, Vector3, Zero, MetricSpace};

use crate::{gl, point2vec, get_start_time, output_elapsed};
use crate::flying_camera::Flying_Camera;
use crate::game::MovementAndCollision;
use crate::gl_helper::instance_model::ModelInstance;
use crate::gl_helper::model::Model;
use crate::ground::{BY, Ground};
//use std::ops::{AddAssign, Add, Mul};
use crate::landscape::{LandscapeObject, SQUARE_COLUMNS, SQUARE_SIZE};
use crate::sound::{ play, WARNING};
//use crate::gl_helper::texture::create_texture;
//use std::ops::AddAssign;
use crate::scenery::Scenery;

pub struct CarMainPlayer {
    pub(crate) model_instances: Vec<ModelInstance>,
    pub(crate) movement_collision: MovementAndCollision,
    matrix: Matrix4<f32>,
    rotation_y_axis: Matrix4<f32>,
    steering: f32,
    angle:f32,
    pub rotation_y: f32,
    rotation_x_axis: Matrix4<f32>,
    pub rotation_x: f32,
    force: Matrix4<f32>,
    ahead_force: Matrix4<f32>,
    pub accelerator_pressed: f32,
    applied_rotation: Matrix4<f32>,
    gravity: f32,
    forward_reverse: f32,
    dir: Vector3<f32>,
    pub off_road:f32,
    pub msg: String,
    tick:f32,
}

const MODEL_HEIGHT: f32 = 0.007;
const GRAVITY_ADD: f32 = 0.05;
const GRAVITY_MAX: f32 = 0.05;
const SCALE: f32 = 0.013;
const GRAVITY: bool = true;

fn start_position() -> Vector3<f32> {
    vec3(0.0, 2.0, 0.0)
}

impl CarMainPlayer {
    pub fn new(gl: &gl::Gl) -> CarMainPlayer {
        let start = get_start_time();

        let mut model_instances:Vec<ModelInstance> = vec![];
        let model = Model::new(gl, "resources/models/bus.obj", "resources/models/bus.png");
        model_instances.push( ModelInstance::new(gl, model.clone(), SCALE, Some("resources/models/bus1.png" ) ) );
        model_instances.push( ModelInstance::new(gl, model.clone(), SCALE, Some("resources/models/bus2.png" ) ) );
        model_instances.push( ModelInstance::new(gl, model.clone(), SCALE, Some("resources/models/bus3.png" ) ) );
        model_instances.push( ModelInstance::new(gl, model.clone(), SCALE, Some("resources/models/bus4.png" ) ) );

        output_elapsed(start,"time elapsed for car_main_player new()");
        CarMainPlayer {
            model_instances,
            movement_collision: MovementAndCollision::new(MODEL_HEIGHT * 1.25, start_position()),
            matrix: Matrix4::from_translation(start_position()),
            rotation_y_axis: Matrix4::from_angle_y(Deg(0.0)),
            rotation_y: 0.0,
            rotation_x_axis: Matrix4::from_angle_x(Deg(0.0)),
            steering: 0.0,
            angle:0.0,
            rotation_x: 0.0,
            force: Matrix4::from_translation(vec3(0.0, 0.0, 0.0)),
            ahead_force: Matrix4::from_translation(vec3(0.0, 0.0, 0.0)),
            accelerator_pressed: 0.0,
            applied_rotation: Matrix4::from_translation(vec3(0.0, 0.0, 0.0)),
            forward_reverse: -1.0,
            dir: vec3(0.0, 0.00, 0.0),
            gravity: GRAVITY_ADD,
            off_road:0.0,
            msg: "".to_string(),
            tick:0.0,
        }
    }

    pub fn reset(&mut self) {
        //stop(ENGINE);
        self.movement_collision.position = start_position();
        self.applied_rotation = Matrix4::from_translation(vec3(0.0, 0.0, 0.0));
        self.force = Matrix4::from_translation(vec3(0.0, 0.0, 0.0));
        self.gravity = GRAVITY_ADD;
        self.dir = Vector3::<f32>::zero();
        self.accelerator_pressed = 0.0;
        self.off_road = 0.0;
    }
    pub fn off_road_too_much(&mut self) -> bool {
        if self.off_road > 40.0 {
            self.off_road = 0.0;
            self.reset();
            return true
        }
        false
    }
    pub fn go_forward(&mut self, change_by: f32, _ground: &Ground) {
        if self.accelerator_pressed == 0.0 {
            self.forward_reverse = -change_by;
        }
    }



    pub fn steer_rotation_y_constant(&mut self, change_by: f32) {
        let max =0.75;
        self.steering = self.steering + change_by;
        if self.steering > max { self.steering = max }
        if self.steering < -max { self.steering = -max }
    }
    pub fn accelerate(&mut self, forward_by: f32, _ground: &Ground) {
        self.accelerator_pressed = self.accelerator_pressed + forward_by;
        if self.accelerator_pressed < 0.0 {
            self.accelerator_pressed = 0.0;
        }
        if self.accelerator_pressed > 0.5 {
            self.accelerator_pressed = 0.5;
        }
    }
    pub fn update(&mut self, delta: f32, ground: &Ground, _camera: &Flying_Camera, tick: i128, ) {
        if self.accelerator_pressed > 0.01 {
            if self.angle > 0.0 && self.angle < 180.0 {
                self.tick = self.tick + self.accelerator_pressed ;
            } else {
                self.tick = self.tick - self.accelerator_pressed ;
            }
            if self.tick < 0.0 {
                self.tick = 100000.0;
            }
            self.rotation_y_axis = Matrix4::from_angle_y(Deg(-self.steering));
            self.angle = self.angle +self.steering;
            if self.angle >= 360.0 {
                self.angle = self.angle - 360.0;
            }
            if self.angle < 0.0 {
                self.angle = self.angle + 360.0;
            }
        } else {
            self.rotation_y_axis = Matrix4::from_angle_y(Deg(0.0));
        }
        let mut dir = vec3(0.0, 0.0, self.accelerator_pressed * self.forward_reverse);
        dir = self.applied_rotation.transform_vector(dir) * 0.1;
        self.dir = dir ; //self.dir + dir;

        self.applied_rotation = self.applied_rotation * self.rotation_x_axis * self.rotation_y_axis;

        let rotated = self.applied_rotation.transform_vector(vec3(0.0, 0.0, 1.0));
        let ang1 = rotated.angle(vec3(0.0, 1.0, 0.0)).sin_cos();
        let ang2 = rotated.angle(vec3(0.0, 0.0, 1.0)).sin_cos();

        self.rotation_x = (Deg::asin(ang1.1).0).round();
        self.rotation_y = (Deg::acos(ang2.1).0).round();

        self.matrix = Matrix4::from_translation(self.movement_collision.position) * self.applied_rotation;
        let original_matrix = self.matrix;

        self.matrix.w.x = self.matrix.w.x + self.dir.x;
        self.matrix.w.y = self.matrix.w.y + self.dir.y;
        self.matrix.w.z = self.matrix.w.z + self.dir.z;

        if GRAVITY {
            self.matrix.w.y = self.matrix.w.y - self.gravity;
        }
        if tick % 60 <= 1 {
            //self.dir = self.dir * 0.9;
            if self.gravity <= GRAVITY_MAX {
                self.gravity = self.gravity + GRAVITY_ADD;
            }
        }
        self.update_position();

        let half_width = (SQUARE_COLUMNS / 2) as f32 * SQUARE_SIZE * BY as f32;

        //println!("{} {} {} {}",self.thrust.w.x,self.thrust.w.y,self.thrust.w.z,self.thrust.w.w);

        if self.movement_collision.position.x < -half_width {
            println!("b4 x< 0 Reset x={},z={} {}", self.movement_collision.position.x, self.movement_collision.position.z, self.force.w.y);
            self.flip_reset_the_matrix(1.0, 0.0);
            println!("x< 0 Reset x={},z={}", self.movement_collision.position.x, self.movement_collision.position.z);
        } else if self.movement_collision.position.x > half_width {
            println!("b4 x> Reset x={},z={}  {}", self.movement_collision.position.x, self.movement_collision.position.z, self.force.w.y);
            self.flip_reset_the_matrix(-1.0, 0.0);
            println!("x> Reset x={},z={}", self.movement_collision.position.x, self.movement_collision.position.z);
        }
        if self.movement_collision.position.z <= -half_width {
            println!("b4 z<0 Reset x={},z={}  {}", self.movement_collision.position.x, self.movement_collision.position.z, self.force.w.y);
            self.flip_reset_the_matrix(0.0, 1.0);
            println!("z<0 Reset x={},z={}", self.movement_collision.position.x, self.movement_collision.position.z);
        } else if self.movement_collision.position.z >= half_width {
            println!("b4 z> Reset x={},z={}  {}", self.movement_collision.position.x, self.movement_collision.position.z, self.force.w.y);
            self.flip_reset_the_matrix(0.0, -1.0);
        }
        let ahead_matrix = self.matrix * self.rotation_y_axis * self.rotation_x_axis * self.ahead_force;
        let ahead = CarMainPlayer::position_ahead(ahead_matrix);

        let over = ground.object_at(self.movement_collision.position.x, self.movement_collision.position.z);
        /*
        fn over_one(landscape_object: &LandscapeObject) {
            println!("Over {}", landscape_object.description);
        }
         */
        if self.off_road > 0.0 {
            self.off_road = self.off_road - delta * 5.0;
        }
        if over.is_none() {
            play(WARNING);
        }
        self.msg = String::new();
        self.off_road = self.off_road + over.map_or(1.0 , |l:&LandscapeObject| {
            if l.description.contains("road") {
                self.msg = l.description.clone();
                0.0
            } else {
                play(WARNING);
                1.0
            }
        });
        let (hit_scenery,xyz) = ground.scenery_at(self.movement_collision.position.x, self.movement_collision.position.z);
        hit_scenery.map(|l:&Scenery| {
            //let distance = l.movement_collision.position.distance(self.movement_collision.position);
            let distance = (xyz + l.position).distance(self.movement_collision.position);
            if distance < l.movement_collision.radius {
                self.msg = format!("HIT!!!!!!! Over  {:?} {} {}",l.scenery_type, l.position.x,l.position.z);
                println!("**** HIT!!!!!!! Over  {} {:?} {} {}",distance,l.scenery_type, l.position.x,l.position.z);
                self.matrix = original_matrix * self.rotation_y_axis * self.rotation_x_axis;
                self.update_position();
            }
            println!("HIT!!!!!!! Over  {} {:?} {} {}",distance,l.scenery_type, l.position.x,l.position.z);
        });


        let ground_height = ground.position_height(self.movement_collision.position.x, self.movement_collision.position.z);
        let ground_height_ahead = ground.position_height(ahead.x, ahead.z);


        if ahead.y < ground_height_ahead + MODEL_HEIGHT {
            self.matrix = original_matrix * self.rotation_y_axis * self.rotation_x_axis;
            self.update_position();
            //println!("AHEAD ROLLBACK ground={} ahead.y={}", ground_height_ahead, ahead.y);
            //self.crashed(special_effects);
            self.gravity = 0.0;
        } else if self.movement_collision.position.y < ground_height + MODEL_HEIGHT {
            self.matrix = original_matrix * self.rotation_y_axis * self.rotation_x_axis;
            self.update_position();
            self.gravity = 0.0;
            println!("ROLLBACK height {} ", ground_height);
            //self.crashed(special_effects);
        }


        if self.accelerator_pressed > 0.0 && tick % 3 == 0 {
            let mut dir = vec3(0.0, MODEL_HEIGHT, 0.0);
            dir = self.applied_rotation.transform_vector(dir);
            /*
            for _i in 1..rng.gen_range(2, 5) {
                let pos = self.movement_collision.position - dir * rng.gen_range(0.95, 1.75);
                special_effects.thrust(pos, self.dir, delta);
            }
             */
        }
    }

    fn flip_reset_the_matrix(&mut self, x: f32, z: f32) {
        let width = (SQUARE_COLUMNS) as f32 * SQUARE_SIZE * BY as f32;

        if !x.is_zero() {
            self.matrix.w.x = self.matrix.w.x + (x * width);
        }
        if !z.is_zero() {
            self.matrix.w.z = self.matrix.w.z + (z * width);
        }
        self.update_position();
    }

    fn update_position(&mut self) {
        let mut point = Point3::from_vec(vec3(0.0, 0.0, 0.0));
        point = self.matrix.transform_point(point);
        self.movement_collision.position.x = point.x;
        self.movement_collision.position.y = point.y;
        self.movement_collision.position.z = point.z;
    }
    fn position_ahead(matrix: Matrix4<f32>) -> Vector3<f32> {
        let mut point = Point3::from_vec(vec3(0.0, 0.0, 0.0));
        point = matrix.transform_point(point);
        point2vec(point)
    }
    pub(crate) fn render(&mut self, gl: &gl::Gl, view: &Matrix4<f32>, projection: &Matrix4<f32>, our_shader: u32) {
        let alt = if self.tick% 2.0 < 1.25 {
            true
        } else {
            false
        };
        let instance = (self.tick % self.model_instances.len() as f32) as usize ;
        self.model_instances[instance].matrix = self.matrix;
        self.model_instances[instance].render(gl, &view, &projection, our_shader, alt);
    }
}