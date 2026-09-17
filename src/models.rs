use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemOneRequest {
    pub state: serde_json::Value,
    pub model: String,
    pub questions: HashMap<String, Question>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Question {
    Choice(ChoiceQuestion),
    Noul(NoulQuestion),
    Score(ScoreQuestion),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChoiceQuestion {
    pub instructions: serde_json::Value,
    pub criteria: HashMap<String, Option<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NoulQuestion {
    pub instructions: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub criteria: Option<NoulCriteria>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NoulCriteria {
    #[serde(rename = "true", skip_serializing_if = "Option::is_none")]
    pub true_desc: Option<String>,
    #[serde(rename = "false", skip_serializing_if = "Option::is_none")]
    pub false_desc: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScoreQuestion {
    pub instructions: serde_json::Value,
    pub criteria: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemOneResponse {
    pub model: String,
    pub answers: HashMap<String, Answer>,
    pub usage: Usage,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Answer {
    Choice(ChoiceAnswer),
    Noul(NoulAnswer),
    Score(ScoreAnswer),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChoiceAnswer {
    pub choice: String,
    pub confidence: f64,
    pub probabilities: HashMap<String, f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NoulAnswer {
    pub noul: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScoreAnswer {
    pub score: f64,
    #[serde(default)]
    pub confidence: f64,
    #[serde(default)]
    pub legend: HashMap<String, String>,
    #[serde(default)]
    pub probabilities: HashMap<String, f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Usage {
    pub input_tokens: u64,
    pub output_tokens: u64,
}
