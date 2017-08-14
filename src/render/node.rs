use std::rc::Rc;

use gltf;

use render::math::*;
use render::mesh::Mesh;
use render::scene::Scene;
use shader::Shader;

pub struct Node {
    // TODO!!: camera?
    pub children: Vec<Node>,
    pub matrix: Matrix4,
    pub mesh: Option<Rc<Mesh>>,
    pub rotation: Quaternion,
    pub scale: Vector3,
    pub translation: Vector3,
    // TODO: weights
    // weights_id: usize,
    pub name: Option<String>,

    final_transform: Matrix4, // including parent transforms
    model_loc: Option<i32>,

    pub bounds: Bounds,
}

impl Node {
    pub fn from_gltf(g_node: gltf::Loaded<gltf::Node>, scene: &mut Scene) -> Node {
        // convert matrix in 3 steps due to type system weirdness
        let matrix = &g_node.matrix();
        let matrix: &Matrix4 = matrix.into();
        let matrix = *matrix;

        let r = &g_node.rotation();
        let rotation = Quaternion::new(r[3], r[0], r[1], r[2]); // NOTE: different element order!

        let mut mesh = None;
        let mut bounds = None;
        if let Some(g_mesh) = g_node.mesh() {
            if let Some(existing_mesh) = scene.meshes.iter().find(|mesh| (***mesh).index == g_mesh.index()) {
                mesh = Some(existing_mesh.clone());
            }

            if mesh.is_none() { // not using else due to borrow-checking madness
                mesh = Some(Rc::new(Mesh::from_gltf(g_mesh, scene)));
                scene.meshes.push(mesh.clone().unwrap());
            }

            bounds = mesh.as_ref().unwrap().bounds.clone().into();
        }
        let children: Vec<_> = g_node.children()
                .map(|g_node| Node::from_gltf(g_node, scene))
                .collect();

        let mut bounds = if let Some(bounds) = bounds { bounds } else { children[0].bounds.clone() };
        bounds = children.iter().skip(1).fold(bounds, |bounds, ref node| node.bounds.union(&bounds));

        Node {
            children,
            matrix,
            mesh,
            rotation,
            scale: g_node.scale().into(),
            translation: g_node.translation().into(),
            name: g_node.name().map(|s| s.into()),

            final_transform: Matrix4::identity(),
            model_loc: None,

            bounds,
        }
    }

    pub fn update_transform(&mut self, parent_transform: &Matrix4) {
        self.final_transform = *parent_transform;

        if !self.matrix.is_identity() {
            self.final_transform = self.final_transform * self.matrix;
        }
        else {
            // TODO?: detect if all default and set None? does NOT happen for any sample model
            self.final_transform = self.final_transform *
                Matrix4::from_translation(self.translation) *
                Matrix4::from_nonuniform_scale(self.scale.x, self.scale.y, self.scale.z) *
                Matrix4::from(self.rotation);
        }

        for node in &mut self.children {
            node.update_transform(&self.final_transform);
        }
    }

    pub fn update_bounds(&mut self) {
        // TODO: implement for/after animation
        unimplemented!()
    }

    pub fn draw(&mut self, shader: &mut Shader) {
        if let Some(ref mesh) = self.mesh {
            unsafe {
                if self.model_loc.is_none() {
                    self.model_loc = Some(shader.uniform_location("model"));
                }
                shader.set_mat4(self.model_loc.unwrap(), &self.final_transform);
            }

            (*mesh).draw(shader);
        }
        for node in &mut self.children {
            node.draw(shader);
        }
    }
}
