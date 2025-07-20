use serde::{Deserialize, Serialize};
use bincode::{Decode, Encode};

#[derive(Debug, Encode, Decode, Serialize, Deserialize, Default, Clone)]
pub struct SkillMoveOptionData {
    pub down_time: Option<f32>,
    pub stand_up_time: Option<f32>,
    pub move_time: Option<f32>,
}