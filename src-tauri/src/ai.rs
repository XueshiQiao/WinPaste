use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::error::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
    pub provider: String, // "openai", "deepseek", "custom"
    pub api_key: String,
    pub model: String,
    pub base_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AiAction {
    Summarize,
    Translate,
    ExplainCode,
    FixGrammar,
}

#[derive(Deserialize)]
struct OpenAiResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: Message,
}

#[derive(Deserialize)]
struct Message {
    content: String,
}

pub async fn process_text(
    text: &str,
    action: AiAction,
    config: &AiConfig,
) -> Result<String, Box<dyn Error>> {
    let client = Client::new();

    let system_prompt = match action {
        AiAction::Summarize => "You are a helpful assistant. Summarize the following text concisely.",
        AiAction::Translate => "You are a helpful assistant. Translate the following text to English (or the user's likely preferred language based on context).",
        AiAction::ExplainCode => "You are a helpful assistant. Explain what the following code does.",
        AiAction::FixGrammar => "You are a helpful assistant. Fix the grammar and improve the style of the following text.",
    };

    let base_url = config.base_url.as_deref().unwrap_or("https://api.openai.com/v1");
    let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));

    let body = json!({
        "model": config.model,
        "messages": [
            {"role": "system", "content": system_prompt},
            {"role": "user", "content": text}
        ],
        "temperature": 0.7
    });

    let res = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", config.api_key))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await?;

    if !res.status().is_success() {
        let error_text = res.text().await?;
        return Err(format!("API Error: {}", error_text).into());
    }

    let response_data: OpenAiResponse = res.json().await?;
    
    if let Some(choice) = response_data.choices.first() {
        Ok(choice.message.content.clone())
    } else {
        Err("No response from AI".into())
    }
}
