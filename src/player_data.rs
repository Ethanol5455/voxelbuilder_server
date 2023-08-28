use serde::{Deserialize, Serialize};

use crate::vector_types::*;

#[derive(Serialize, Deserialize)]
pub struct Player {
    pub username: String,
    pub position: Vec3<f32>,
    pub rotation: Vec2<f32>,
}
