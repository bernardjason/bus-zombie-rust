
use cgmath::{Angle, Deg, EuclideanSpace, InnerSpace, Matrix4, MetricSpace, Point3, Rad, Transform, vec3, Vector3, Zero};
use rand::Rng;

use crate::{get_start_time, gl, output_elapsed, point2vec};
use crate::flying_camera::Flying_Camera;
use crate::game::MovementAndCollision;
use crate::gl_helper::instance_model::ModelInstance;
use crate::ground::{BY, Ground};
use crate::landscape::{ SQUARE_COLUMNS, SQUARE_SIZE};
use crate::special_effects::SpecialEffects;
//use crate::TICKDEBUG;
//use crate::TICKPRINTCOUNTER;
//use crate::tickprintln;

pub struct Passenger {
    pub(crate) model_instance: Vec<ModelInstance>,
    animate: f32,
    pub zombie: bool,
    zombie_countdown: f32,
    turning_zombie: i32,
    zombie_exploding: bool,
    pub(crate) movement_collision: MovementAndCollision,
    matrix: Matrix4<f32>,
    rotation_y_axis: Matrix4<f32>,
    angle_of_rotation: f32,
    rotation_angle: f32,
    pub rotation_y: f32,
    rotation_x_axis: Matrix4<f32>,
    pub rotation_x: f32,
    ahead_force: Matrix4<f32>,
    moves_since_last_change: i32,
    nsew: Vec<bool>,
    old_nsew: Vec<bool>,
    nsew_change_clicks: i128,
    target_angle: f32,
    previous_target_angle: f32,
    applied_rotation: Matrix4<f32>,
    gravity: f32,
    forward_reverse: f32,
    dir: Vector3<f32>,
    speed: f32,
    animate_speed: f32,
}

const MODEL_HEIGHT: f32 = 0.10;
const GRAVITY_ADD: f32 = 0.05;
const GRAVITY_MAX: f32 = 0.05;
pub const PASSENGER_SCALE: f32 = 0.004;
const ZOMBIE_SCALE: f32 = 0.006;
const ZOMBIE_SPEED: f32 = 0.25;
const GRAVITY: bool = true;
const ZOMBIE_DISTANCE_GOT_TO_BUS: f32 = 0.2;


const HUMAN_SEE_BUS: f32 = 4.0;
const ZOMBIE_SEE_BUS: f32 = 8.0;

impl Passenger {
    pub fn new(_gl: &gl::Gl, start_position: Vector3<f32>, instances: &Vec<ModelInstance>) -> Passenger {
        let start = get_start_time();

        let mut rng = rand::thread_rng();

        let speed = rng.gen_range(0.10, 0.15);
        let p = Passenger {
            model_instance: instances.clone(),
            animate: 0.0,
            zombie: false,
            zombie_countdown: 6.0,
            turning_zombie: 0,
            zombie_exploding: false,
            movement_collision: MovementAndCollision::new(MODEL_HEIGHT * 1.25, start_position),
            matrix: Matrix4::from_translation(start_position),
            rotation_y_axis: Matrix4::from_angle_y(Deg(0.0)),
            rotation_y: 0.0,
            rotation_x_axis: Matrix4::from_angle_x(Deg(0.0)),
            angle_of_rotation: 0.0,
            rotation_x: 0.0,
            ahead_force: Matrix4::from_translation(vec3(0.0, 0.0, 0.1)),
            moves_since_last_change: 0,
            target_angle: 180.0,
            previous_target_angle: 0.0,
            nsew: vec![false, false, false, false],
            old_nsew: vec![false, false, false, false],
            nsew_change_clicks: 100,
            rotation_angle: 180.0,
            applied_rotation: Matrix4::from_translation(vec3(0.0, 0.0, 0.0)),
            forward_reverse: 1.0,
            dir: vec3(0.0, 0.00, 0.0),
            gravity: GRAVITY_ADD,
            speed,
            animate_speed: speed * 84.0,
        };
        output_elapsed(start, "time elapsed for passenger new()");
        p
    }

    pub fn set_random_time_to_zombie(&mut self) {
        let mut rng = rand::thread_rng();
        self.zombie_countdown = rng.gen_range(40.0,120.0);
        //self.zombie_countdown = rng.gen_range(4.0,12.0);
    }


    pub fn update(&mut self, delta: f32, ground: &Ground, _camera: &Flying_Camera, tick: i128, special_effects: &mut SpecialEffects, chase_target: Vector3<f32>) -> (bool, bool, bool) {
        let mut finished = false;
        let mut add_score = false;
        let mut zombie_explode = false;
        let original_matrix = self.matrix;

        let old_pos = self.movement_collision.position.clone();
        self.zombie_countdown = self.zombie_countdown - delta;

        self.animate = self.animate + delta * self.animate_speed;
        if self.animate as usize >= self.model_instance.len() || self.zombie_exploding {
            self.animate = 0.0;
        }

        if !self.zombie_exploding {
            self.do_movement_updates(tick, special_effects,);
        } else {
            self.matrix.w.y = self.matrix.w.y - delta * 1.25;
            self.update_position();
        }

        let half_width = (SQUARE_COLUMNS / 2) as f32 * SQUARE_SIZE * BY as f32;

        //println!("{} {} {} {}",self.thrust.w.x,self.thrust.w.y,self.thrust.w.z,self.thrust.w.w);

        self.wrap_position_if_needed(half_width);

        let distance = chase_target.distance2(self.movement_collision.position);
        if distance < 0.06 && !self.zombie {
            finished = true;
            add_score = true;
        }
        if !self.zombie_exploding && distance < ZOMBIE_DISTANCE_GOT_TO_BUS && self.zombie {
            special_effects.explosion(self.movement_collision.position);
            //finished = true;
            zombie_explode = true;
            self.set_to_explode();
        }
        if self.zombie && self.zombie_countdown < -100.0 && ! self.zombie_exploding {
            self.set_to_explode();
        }
        if self.zombie_exploding && self.zombie_countdown <= 0.0 {
            finished = true;
        }

        self.workout_my_direction(ground, chase_target, old_pos, original_matrix, distance);

        self.turn_around_update(original_matrix);

        let ground_height = ground.position_height(self.movement_collision.position.x, self.movement_collision.position.z);
        let ground_height_ahead = ground.position_height(self.movement_collision.position.x, self.movement_collision.position.z);

        if !self.zombie_exploding {
            if self.movement_collision.position.y < ground_height_ahead + MODEL_HEIGHT {
                self.matrix = original_matrix * self.rotation_y_axis * self.rotation_x_axis;
                self.update_position();
                self.gravity = 0.0;
            } else if self.movement_collision.position.y < ground_height + MODEL_HEIGHT {
                self.matrix = original_matrix * self.rotation_y_axis * self.rotation_x_axis;
                self.update_position();
                self.gravity = 0.0;
                //println!("ROLLBACK height {} ", ground_height);
            }
        }

        return (finished, add_score, zombie_explode);
    }

    fn set_to_explode(&mut self) -> bool {
        self.zombie_exploding = true;
        self.zombie_countdown = 0.5;
        true
    }

    fn turn_around_update(&mut self, original_matrix: Matrix4<f32>) {
        let mut diff_angle2 = self.target_angle - self.rotation_angle;
        if diff_angle2 < 0.0 { diff_angle2 = diff_angle2 + 360.0; }

        if self.target_angle == self.rotation_angle {
            self.angle_of_rotation = 0.0;
            self.moves_since_last_change = self.moves_since_last_change + 1;
        } else {
            self.animate = 0.0;
            if diff_angle2 > 0.0 && diff_angle2 < 180.0 {
                //tickprintln!(format!("1 rotation_angle={} target_angle={} diff_angle1={} diff_angle2={}",self.rotation_angle,self.target_angle,diff_angle1,diff_angle2));
                self.matrix = original_matrix * self.rotation_y_axis * self.rotation_x_axis;
                self.update_position();
                self.angle_of_rotation = 1.0;
                self.moves_since_last_change = 0;
            } else {
                //tickprintln!(format!("2 rotation_angle={} target_angle={} diff_angle1={} diff_angle2={}",self.rotation_angle,self.target_angle,diff_angle1,diff_angle2));
                self.matrix = original_matrix * self.rotation_y_axis * self.rotation_x_axis;
                self.update_position();
                self.angle_of_rotation = -1.0;
                self.moves_since_last_change = 0;
            }
        }
    }

    fn workout_my_direction(&mut self, ground: &Ground, chase_target: Vector3<f32>, old_pos: Vector3<f32>, original_matrix: Matrix4<f32>, distance: f32) {
        if (self.zombie && distance < ZOMBIE_SEE_BUS || distance < HUMAN_SEE_BUS) && self.moves_since_last_change > 60 {
            let my_degrees = Rad::atan2(old_pos.z - chase_target.z, old_pos.x - chase_target.x);
            let mut angle_degrees = Deg::from(my_degrees).0.round() - 90.0;
            if angle_degrees < 0.0 { angle_degrees = angle_degrees + 360.0; }
            if angle_degrees >= 360.0 { angle_degrees = angle_degrees - 360.0; }

            let rotation = Matrix4::from_angle_y(Deg(angle_degrees));
            let mut off_road = false;
            for few_steps in 1..3 {
                let mut here = Matrix4::from_translation(vec3(0.0, 0.0, 0.1 * few_steps as f32)) * rotation;
                //let add = self.ahead_force * rotation ;
                //let mut here = add  ;//+ Matrix4::from_translation(self.movement_collision.position);
                here.w.x = self.movement_collision.position.x;
                here.w.z = self.movement_collision.position.z;

                let ahead = self.see_if_road_ahead(ground, here);
                if ahead == false {
                    off_road = true;
                }
            }
            if off_road == false {
                self.target_angle = angle_degrees;
                //println!("Close by {}",angle_degrees);
            } else {
                self.do_some_street_smarts(ground, chase_target, original_matrix, old_pos);
                //println!("failed close by check {}",angle_degrees);
            }
        } else {
            self.do_some_street_smarts(ground, chase_target, original_matrix, old_pos);
        }
    }

    fn wrap_position_if_needed(&mut self, half_width: f32) {
        if self.movement_collision.position.x < -half_width {
            //println!("b4 x< 0 Reset x={},z={} {}", self.movement_collision.position.x, self.movement_collision.position.z, self.force.w.y);
            self.flip_reset_the_matrix(1.0, 0.0);
            //println!("x< 0 Reset x={},z={}", self.movement_collision.position.x, self.movement_collision.position.z);
        } else if self.movement_collision.position.x > half_width {
            //println!("b4 x> Reset x={},z={}  {}", self.movement_collision.position.x, self.movement_collision.position.z, self.force.w.y);
            self.flip_reset_the_matrix(-1.0, 0.0);
            //println!("x> Reset x={},z={}", self.movement_collision.position.x, self.movement_collision.position.z);
        }
        if self.movement_collision.position.z <= -half_width {
            //println!("b4 z<0 Reset x={},z={}  {}", self.movement_collision.position.x, self.movement_collision.position.z, self.force.w.y);
            self.flip_reset_the_matrix(0.0, 1.0);
            //println!("z<0 Reset x={},z={}", self.movement_collision.position.x, self.movement_collision.position.z);
        } else if self.movement_collision.position.z >= half_width {
            //println!("b4 z> Reset x={},z={}  {}", self.movement_collision.position.x, self.movement_collision.position.z, self.force.w.y);
            self.flip_reset_the_matrix(0.0, -1.0);
        }
    }

    fn do_movement_updates(&mut self, tick: i128, special_effects: &mut SpecialEffects, ) {
        self.rotation_y_axis = Matrix4::from_angle_y(Deg(-self.angle_of_rotation));
        let mut dir = vec3(0.0, 0.0, self.forward_reverse * self.speed);
        dir = self.applied_rotation.transform_vector(dir) * 0.1;
        self.dir = dir; //self.dir + dir;

        if ! self.zombie && self.zombie_countdown < 0.0 {
            self.zombie_countdown = -1.0;
            if self.turning_zombie > 100 {
                self.zombie = true;
                self.speed = ZOMBIE_SPEED;
            } else {
                self.dir.set_zero();
                self.animate = 0.0;
                self.turning_zombie = self.turning_zombie + 1;
                if self.turning_zombie % 5 == 0 {
                    special_effects.zombie(self.movement_collision.position);
                }
            }
        }

        self.rotation_angle = self.rotation_angle + self.angle_of_rotation;
        if self.rotation_angle < 0.0 {
            self.rotation_angle = self.rotation_angle + 360.0;
        }
        if self.rotation_angle >= 360.0 {
            self.rotation_angle = self.rotation_angle - 360.0;
        }

        self.applied_rotation = self.applied_rotation * self.rotation_x_axis * self.rotation_y_axis;

        let rotated = self.applied_rotation.transform_vector(vec3(0.0, 0.0, 1.0));
        let ang1 = rotated.angle(vec3(0.0, 1.0, 0.0)).sin_cos();
        let ang2 = rotated.angle(vec3(0.0, 0.0, 1.0)).sin_cos();

        self.rotation_x = (Deg::asin(ang1.1).0).round();
        self.rotation_y = (Deg::acos(ang2.1).0).round();

        self.matrix = Matrix4::from_translation(self.movement_collision.position) * self.applied_rotation;

        self.matrix.w.x = self.matrix.w.x + self.dir.x;
        self.matrix.w.y = self.matrix.w.y + self.dir.y;
        self.matrix.w.z = self.matrix.w.z + self.dir.z;

        if GRAVITY {
            self.matrix.w.y = self.matrix.w.y - self.gravity;
        }
        if tick % 60 <= 1 {
            if self.gravity <= GRAVITY_MAX {
                //self.dir = self.dir * 0.9;
                self.gravity = self.gravity + GRAVITY_ADD;
            }
        }
        self.update_position();
    }
    fn do_some_street_smarts(&mut self, ground: &Ground, chase_target: Vector3<f32>, original_matrix: Matrix4<f32>, old_pos: Vector3<f32>) {
        let test_direction = self.matrix * self.rotation_y_axis * self.rotation_x_axis * self.ahead_force;
        let okay_forward = self.see_if_road_ahead(ground, test_direction);

        if !okay_forward {
            self.matrix = original_matrix * self.rotation_y_axis * self.rotation_x_axis;
            self.update_position();
        }

        let test_distance = 1.5_f32;

        let north = self.see_if_road_ahead(ground, Matrix4::from_translation(vec3(0.0, 0.0, -test_distance) + old_pos));
        let south = self.see_if_road_ahead(ground, Matrix4::from_translation(vec3(0.0, 0.0, test_distance) + old_pos));
        let east = self.see_if_road_ahead(ground, Matrix4::from_translation(vec3(test_distance, 0.0, 0.0) + old_pos));
        let west = self.see_if_road_ahead(ground, Matrix4::from_translation(vec3(-test_distance, 0.0, 0.0) + old_pos));

        let current_nsew = vec![north, south, east, west];

        self.nsew_change_clicks = self.nsew_change_clicks + 1;

        if okay_forward && !current_nsew.eq(&self.nsew) && self.moves_since_last_change > 60 {
            self.nsew_change_clicks = 0;
            self.old_nsew = self.nsew.clone();
            //print!("Something changed  old ");
            //Passenger::print_nsew(&self.nsew);
            //print!(" now=  ");
            //Passenger::print_nsew(&current_nsew);
            //println!(" ");
        }

        if self.nsew_change_clicks == 10 {
            let save = self.target_angle;
            if self.old_nsew[2] != self.nsew[2] && self.target_angle != 270.0 && self.movement_collision.position.x < chase_target.x && east {
                //print!("E ");
                self.target_angle = 90.0;
            }
            if self.old_nsew[3] != self.nsew[2] && self.target_angle != 90.0 && self.movement_collision.position.x > chase_target.x && west {
                //print!("W ");
                self.target_angle = 270.0;
            }
            if self.old_nsew[1] != self.nsew[1] && self.target_angle != 0.0 && self.movement_collision.position.z < chase_target.z && south {
                //print!("S ");
                self.target_angle = 180.0;
            }
            if self.old_nsew[0] != self.nsew[0] && self.target_angle != 180.0 && self.movement_collision.position.z > chase_target.z && north {
                //print!("N ");
                self.target_angle = 0.0;
            }

            if self.target_angle != save {
                self.previous_target_angle = save;
            }
            //println!("{}", format!("see2 you target={} previous={} north okay={} south okay={} east okay={} west okay={}", self.target_angle, self.previous_target_angle, north, south, east, west));
        }


        if !okay_forward && self.target_angle == self.rotation_angle {
            let save = self.target_angle;
            if self.target_angle == 0.0 || self.target_angle == 180.0 {
                if self.movement_collision.position.x < chase_target.x {
                    if east {
                        self.target_angle = 90.0
                    } else if west {
                        self.target_angle = 270.0
                    } else {
                        self.stuck_random_move();
                    }
                } else {
                    if west {
                        self.target_angle = 270.0
                    } else if east {
                        self.target_angle = 90.0
                    } else {
                        self.stuck_random_move();
                    }
                }
            } else {
                if self.movement_collision.position.z < chase_target.z {
                    if south {
                        self.target_angle = 180.0
                    } else if north {
                        self.target_angle = 0.0
                    } else {
                        self.stuck_random_move();
                    }
                } else {
                    if north {
                        self.target_angle = 0.0
                    } else if south {
                        self.target_angle = 180.0
                    } else {
                        self.stuck_random_move();
                    }
                }
            }
            self.previous_target_angle = save;
            //println!("{}", format!("target={} previous={} north okay={} south okay={} east okay={} west okay={}", self.target_angle, self.previous_target_angle, north, south, east, west));
        }
        //tickprintln!(format!("debug target={} previous={} north okay={} south okay={} east okay={} west okay={}",self.target_angle,self.previous_target_angle,north,south,east,west));

        self.nsew = current_nsew;
    }

    fn stuck_random_move(&mut self) {
        let mut rng = rand::thread_rng();
        self.target_angle = (rng.gen_range(0, 3) * 90) as f32;
    }

    fn print_nsew(current_nsew: &Vec<bool>) {
        print!("N={} ", current_nsew[0]);
        print!("S={} ", current_nsew[1]);
        print!("E={} ", current_nsew[2]);
        print!("W={} ", current_nsew[3]);
    }
    fn see_if_road_ahead(&mut self, ground: &Ground, ahead_matrix: Matrix4<f32>) -> bool {
        let ahead = Passenger::position_ahead(ahead_matrix);
        let over = ground.object_at(ahead.x, ahead.z);
        return over.is_some();
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

    pub(crate) fn render(&mut self, gl: &gl::Gl, view: &Matrix4<f32>, projection: &Matrix4<f32>, our_shader: u32, offset: Vector3<f32>, tick: i128) {
        //self.model_instance.matrix = self.matrix;
        self.model_instance[self.animate as usize].matrix = Matrix4::from_translation(self.movement_collision.position + offset) * self.applied_rotation;
        if self.zombie {
            if self.zombie_exploding {
                if tick % 3 == 0 {
                    //self.model_instance[self.animate as usize].scale = ZOMBIE_SCALE * self.zombie_countdown * 2.0 ;
                    self.model_instance[self.animate as usize].scale = ZOMBIE_SCALE;
                }
            } else {
                self.model_instance[self.animate as usize].scale = ZOMBIE_SCALE;
            }
            self.model_instance[self.animate as usize].render(gl, &view, &projection, our_shader, true);
        } else {
            self.model_instance[self.animate as usize].render(gl, &view, &projection, our_shader, false);
        }
    }
}