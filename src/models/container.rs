use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct ContainerSummary {
    pub id: String,
    pub names: Vec<String>,
    pub image: String,
    pub status: String,
    pub state: String,
}
