use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Label {
    pub position: Vec<f32>,
    pub txt: String
}

#[derive(Debug, Deserialize)]
pub struct DisplayGoal {
    pub dim: u32,
    pub labels: Vec<Label>,
    pub context: String
}

#[derive(Debug, Deserialize)]
pub enum Message {
    DisplayGoal(DisplayGoal)
}
