use nalgebra_glm as glm;
use std::fmt;

/// The SceneNode data structure from the handout code, rewritten in safe Rust
pub struct SceneNode {
    pub position: glm::Vec3,        // Where I should be in relation to my parent
    pub rotation: glm::Vec3,        // How I should be rotated, around the X, the Y and the Z axes
    pub scale: glm::Vec3,           // How I should be scaled
    pub reference_point: glm::Vec3, // The point I shall rotate and scale about

    pub vao_id: u32,      // What I should draw
    pub index_count: i32, // How much of it there is to draw

    children: Vec<SceneNode>, // Those I command
}

impl Default for SceneNode {
    fn default() -> Self {
        Self {
            position: glm::zero(),
            rotation: glm::zero(),
            scale: glm::vec3(1.0, 1.0, 1.0),
            reference_point: glm::zero(),
            vao_id: 0,
            index_count: -1,
            children: Vec::new(),
        }
    }
}

impl SceneNode {
    pub fn new(vao_id: u32, index_count: i32) -> Self {
        Self {
            position: glm::zero(),
            rotation: glm::zero(),
            scale: glm::vec3(1.0, 1.0, 1.0),
            reference_point: glm::zero(),
            vao_id,
            index_count,
            children: Vec::new(),
        }
    }

    pub fn add_child(&mut self, child: SceneNode) {
        self.children.push(child)
    }

    #[allow(dead_code)]
    pub fn get_child_mut(&mut self, index: usize) -> Option<&mut SceneNode> {
        self.children.get_mut(index)
    }

    /// Returns the number of children of this node.
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.children.len()
    }
    
    pub fn iter(&self) -> impl Iterator<Item = &SceneNode> {
        self.children.iter()
    }
    
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut SceneNode> {
        self.children.iter_mut()
    }
}

impl fmt::Debug for SceneNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "SceneNode {{
    VAO:       {}
    Indices:   {}
    Children:  {}
    Position:  [{:.2}, {:.2}, {:.2}]
    Rotation:  [{:.2}, {:.2}, {:.2}]
    Reference: [{:.2}, {:.2}, {:.2}]
}}",
            self.vao_id,
            self.index_count,
            self.children.len(),
            self.position.x,
            self.position.y,
            self.position.z,
            self.rotation.x,
            self.rotation.y,
            self.rotation.z,
            self.reference_point.x,
            self.reference_point.y,
            self.reference_point.z,
        )
    }
}
