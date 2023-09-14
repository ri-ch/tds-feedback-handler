use serde::Deserialize;

#[derive(Deserialize)]
pub struct FeedbackResponse {
    pub name: String,
    pub message: String,
}
