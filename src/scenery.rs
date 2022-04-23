use crate::gl_helper::model::Model;
use cgmath::{Matrix4, Vector3, vec3};
use crate::gl_helper::instance_model::ModelInstance;
use crate::game::{MovementAndCollision, };
use crate::{gl, };

#[derive(Debug)]
pub enum SceneryType {
    TREE,
    HOUSE,
    OFFICE1,
}

pub struct Scenery {
    pub(crate) model_instance: ModelInstance,
    pub(crate) movement_collision: MovementAndCollision,
    matrix:Matrix4<f32>,
    pub scenery_type:SceneryType,
    pub position: Vector3<f32>,
    pub collision_radius:f32,
}
impl Scenery {

    pub fn setup_tree(gl: &gl::Gl) -> Model {
        let model = Model::new(gl, "resources/models/tree.obj", "resources/models/tree.png");
        model
    }
    pub fn setup_house(gl: &gl::Gl) -> Model {
        let model = Model::new(gl, "resources/models/house.obj", "resources/models/house.png");
        model
    }
    pub fn setup_office1(gl: &gl::Gl) -> Model {
        let model = Model::new(gl, "resources/models/office1.obj", "resources/models/office1.png");
        model
    }


    pub fn new_tree(gl: &gl::Gl,position:Vector3<f32>,model:&Model) -> Scenery {
        let model = model.clone();
        let model_instance = ModelInstance::new(gl,model, 0.01,None);
        Scenery {
            model_instance,
            movement_collision:MovementAndCollision::new(0.4, position),
            matrix:Matrix4::from_translation(position),
            scenery_type:SceneryType::TREE,
            position,
            collision_radius:0.25,
        }
    }
    pub fn new_house(gl: &gl::Gl,position:Vector3<f32>,model:&Model) -> Scenery {
        let model = model.clone();
        let model_instance = ModelInstance::new(gl,model, 0.01,None);
        Scenery {
            model_instance,
            movement_collision:MovementAndCollision::new(0.6, position),
            matrix:Matrix4::from_translation(position + vec3(0.0,0.0,0.0)),
            scenery_type:SceneryType::HOUSE,
            position,
            collision_radius:1.0,
        }
    }
    pub fn new_office1(gl: &gl::Gl,position:Vector3<f32>,model:&Model) -> Scenery {
        let model = model.clone();
        let model_instance = ModelInstance::new(gl,model, 0.09,None);
        Scenery {
            model_instance,
            movement_collision:MovementAndCollision::new(0.9, position),
            matrix:Matrix4::from_translation(position + vec3(0.125,0.0,0.125)),
            scenery_type:SceneryType::OFFICE1,
            position,
            collision_radius:1.0,
        }
    }

    pub(crate) fn render(&mut self, gl: &gl::Gl, view: &Matrix4<f32>, projection: &Matrix4<f32>,our_shader:u32,wrapped_position:Vector3<f32>) {

        self.model_instance.matrix = self.matrix *Matrix4::from_translation(wrapped_position);
        self.model_instance.render(gl, &view, &projection,our_shader,false);
    }
}