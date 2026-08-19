//! AI Copilot, Fixer, and Tab Completion engine powered by the `genai` multi-provider crate.

use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use std::sync::mpsc;

/// Supported AI backend providers for genai.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum AiProvider {
    #[default]
    Gemini,
    OpenAi,
    Anthropic,
    DeepSeek,
    Groq,
    Ollama,
    OpenRouter,
    Custom,
}

impl AiProvider {
    pub const ALL: [Self; 8] = [
        Self::Gemini,
        Self::OpenAi,
        Self::Anthropic,
        Self::DeepSeek,
        Self::Groq,
        Self::Ollama,
        Self::OpenRouter,
        Self::Custom,
    ];

    /// Human-friendly display label and icon.
    pub fn label(self) -> &'static str {
        match self {
            Self::Gemini => "Google Gemini",
            Self::OpenAi => "OpenAI",
            Self::Anthropic => "Anthropic Claude",
            Self::DeepSeek => "DeepSeek",
            Self::Groq => "Groq (Ultra-Fast)",
            Self::Ollama => "Ollama (Local/Offline)",
            Self::OpenRouter => "OpenRouter",
            Self::Custom => "Custom Provider",
        }
    }

    /// Default recommended model name for this provider.
    pub fn default_model(self) -> &'static str {
        match self {
            Self::Gemini => "gemini-3.6-flash",
            Self::OpenAi => "gpt-5.5",
            Self::Anthropic => "claude-opus-4.8",
            Self::DeepSeek => "deepseek-v4-pro",
            Self::Groq => "openai/gpt-oss-120b",
            Self::Ollama => "qwen3.6:27b",
            Self::OpenRouter => "openrouter/auto",
            Self::Custom => "custom-model",
        }
    }

    /// List of real, modern suggested model identifiers for quick 1-click selection in UI.
    pub fn suggested_models(self) -> &'static [&'static str] {
        match self {
            Self::Gemini => &[
                "gemini-3.5-flash-lite",
                "gemini-3.6-flash",
                "gemini-3.5-flash",
                "gemini-3.1-pro-preview",
                "gemini-3.1-flash-lite",
            ],
            Self::OpenAi => &[
                "gpt-5.5",
                "gpt-5.4",
                "gpt-5.4-mini",
                "gpt-5.4-nano",
                "gpt-oss-120b",
            ],
            Self::Anthropic => &["claude-opus-4.8", "claude-sonnet-4.8", "claude-haiku-4.5"],
            Self::DeepSeek => &["deepseek-v4-pro", "deepseek-v4-flash"],
            Self::Groq => &[
                "openai/gpt-oss-120b",
                "openai/gpt-oss-20b",
                "llama-3.3-70b-versatile",
                "llama-3.1-8b-instant",
                "groq/compound",
            ],
            Self::Ollama => &[
                "qwen3.6:27b",
                "qwen3.5:27b",
                "qwen3-coder",
                "gemma4:31b",
                "gpt-oss:120b",
                "deepseek-r1",
            ],
            Self::OpenRouter => &[
                "openrouter/auto",
                "openai/gpt-5.5",
                "anthropic/claude-opus-4.8",
                "google/gemini-3.6-flash",
                "deepseek/deepseek-v4-pro",
            ],
            Self::Custom => &[],
        }
    }

    /// Standard environment variable name for this provider's API key.
    pub fn env_var_name(self) -> Option<&'static str> {
        match self {
            Self::Gemini => Some("GEMINI_API_KEY"),
            Self::OpenAi => Some("OPENAI_API_KEY"),
            Self::Anthropic => Some("ANTHROPIC_API_KEY"),
            Self::DeepSeek => Some("DEEPSEEK_API_KEY"),
            Self::Groq => Some("GROQ_API_KEY"),
            Self::Ollama => None,
            Self::OpenRouter => Some("OPENROUTER_API_KEY"),
            Self::Custom => None,
        }
    }
}

/// User configurable settings for AI Copilot.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AiSettings {
    #[serde(default = "default_ai_enabled")]
    pub enabled: bool,

    #[serde(default)]
    pub provider: AiProvider,

    #[serde(default = "default_ai_model")]
    pub model: String,

    #[serde(
        default,
        serialize_with = "serialize_api_key",
        deserialize_with = "deserialize_api_key"
    )]
    pub api_key: String,

    #[serde(default = "default_ai_temperature")]
    pub temperature: f32,

    #[serde(default = "default_ai_system_prompt")]
    pub system_prompt: String,
}

fn serialize_api_key<S>(key: &str, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let obfuscated = crate::storage::crypto::obfuscate_key(key);
    serializer.serialize_str(&obfuscated)
}

fn deserialize_api_key<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(crate::storage::crypto::deobfuscate_key(&s))
}

fn default_ai_enabled() -> bool {
    true
}

fn default_ai_model() -> String {
    AiProvider::Gemini.default_model().to_string()
}

fn default_ai_temperature() -> f32 {
    0.7
}

fn default_ai_system_prompt() -> String {
    "You are an elite, concise developer note assistant. When fixing, completing, or transforming notes or code, provide ONLY the direct replacement/completion text. Do NOT include conversational filler, preamble (e.g. 'Here is...'), or markdown wrapping unless explicitly requested.".to_string()
}

impl Default for AiSettings {
    fn default() -> Self {
        Self {
            enabled: default_ai_enabled(),
            provider: AiProvider::default(),
            model: default_ai_model(),
            api_key: String::new(),
            temperature: default_ai_temperature(),
            system_prompt: default_ai_system_prompt(),
        }
    }
}

/// Action preset for AI Copilot processing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AiAction {
    FixAndPolish,
    Complete,
    Summarize,
    Explain,
    Custom(String),
}

impl AiAction {
    #[allow(dead_code)]
    pub fn title(&self) -> &str {
        match self {
            Self::FixAndPolish => "Fix & Polish",
            Self::Complete => "Complete",
            Self::Summarize => "Summarize",
            Self::Explain => "Explain",
            Self::Custom(_) => "Custom Prompt",
        }
    }

    pub fn prompt_instruction(&self) -> String {
        match self {
            Self::FixAndPolish => {
                "Fix all spelling, grammar, syntax, formatting, and coding bugs in the provided text. Return ONLY the polished replacement text.".to_string()
            }
            Self::Complete => {
                "Seamlessly complete and continue the following text or code in the exact same style, tone, and language. Return ONLY the continuous continuation text.".to_string()
            }
            Self::Summarize => {
                "Summarize the following text clearly and concisely using structured bullet points.".to_string()
            }
            Self::Explain => {
                "Provide a crystal-clear, concise explanation of what this code or text does, highlighting key concepts, logic flow, and edge cases.".to_string()
            }
            Self::Custom(instruction) => instruction.clone(),
        }
    }
}

/// A request to execute an AI task asynchronously in the background.
#[derive(Debug, Clone)]
pub struct AiRequest {
    pub target_text: String,
    pub context_before: String,
    pub action: AiAction,
    pub provider: AiProvider,
    pub model: String,
    pub api_key: String,
    pub temperature: Option<f32>,
    pub system_prompt: String,
}

/// Result payload emitted from the async AI worker.
#[derive(Debug, Clone)]
pub enum AiResult {
    Success {
        #[allow(dead_code)]
        original_text: String,
        #[allow(dead_code)]
        action: AiAction,
        result_text: String,
    },
    Error(String),
}

static TOKIO_RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();

fn get_or_init_runtime() -> &'static tokio::runtime::Runtime {
    TOKIO_RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .expect("Failed to create tokio runtime for AI service")
    })
}

/// Asynchronously dispatches an AI prompt via genai on background threads.
pub fn spawn_ai_request(req: AiRequest) -> mpsc::Receiver<AiResult> {
    let (tx, rx) = mpsc::channel();
    let rt = get_or_init_runtime();

    rt.spawn(async move {
        let result = execute_ai_chat(req.clone()).await;
        let payload = match result {
            Ok(output) => AiResult::Success {
                original_text: req.target_text,
                action: req.action,
                result_text: output,
            },
            Err(err) => AiResult::Error(err),
        };
        let _ = tx.send(payload);
    });

    rx
}

async fn execute_ai_chat(req: AiRequest) -> Result<String, String> {
    let mut client_builder = genai::Client::builder();

    // If a custom API key was configured in AppSettings, provide it to the client resolver
    if !req.api_key.trim().is_empty() {
        let key_str = req.api_key.trim().to_string();
        let auth_resolver = genai::resolver::AuthResolver::from_resolver_fn(move |_model_iden| {
            Ok(Some(genai::resolver::AuthData::from_single(
                key_str.clone(),
            )))
        });
        client_builder = client_builder.with_auth_resolver(auth_resolver);
    }

    let client = client_builder.build();

    let instruction = req.action.prompt_instruction();
    let prompt_body = if req.target_text.trim().is_empty() {
        format!("{}\n\n--- Context ---\n{}", instruction, req.context_before)
    } else {
        format!(
            "{}\n\n--- Target Text ---\n{}\n\n--- Preceding Context ---\n{}",
            instruction, req.target_text, req.context_before
        )
    };

    let chat_req = genai::chat::ChatRequest::new(vec![
        genai::chat::ChatMessage::system(req.system_prompt),
        genai::chat::ChatMessage::user(prompt_body),
    ]);

    let model_name = if req.model.trim().is_empty() {
        req.provider.default_model()
    } else {
        req.model.trim()
    };

    let temp = req.temperature.unwrap_or(0.7).clamp(0.0, 2.0) as f64;
    let chat_options = genai::chat::ChatOptions::default()
        .with_temperature(temp)
        .with_max_tokens(2048);

    let chat_future = client.exec_chat(model_name, chat_req, Some(&chat_options));
    let chat_res = match tokio::time::timeout(std::time::Duration::from_secs(25), chat_future).await
    {
        Ok(Ok(res)) => res,
        Ok(Err(e)) => return Err(format!("AI provider error: {}", e)),
        Err(_) => {
            return Err(
                "AI request timed out after 25s. Check network / API endpoint.".to_string(),
            );
        }
    };

    let text = chat_res
        .into_first_text()
        .unwrap_or_default()
        .trim()
        .to_string();

    if text.is_empty() {
        Err("AI returned an empty response".to_string())
    } else {
        let cleaned = clean_ai_output(&text, &req.action);
        Ok(cleaned)
    }
}

/// Cleans raw AI response text, stripping accidental outer markdown code fences
/// (e.g. ```` ```rust ... ``` ```` or trailing ```` ``` ````) that LLMs frequently wrap around direct code/text replacements.
pub fn clean_ai_output(raw: &str, action: &AiAction) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    let is_summary_or_explain = matches!(action, AiAction::Summarize | AiAction::Explain);

    // 1. Check if the response is completely wrapped in an outermost code fence
    if trimmed.starts_with("```")
        && let Some(first_newline_idx) = trimmed.find('\n')
    {
        let first_line = &trimmed[..first_newline_idx];
        let after_first_line = &trimmed[first_newline_idx + 1..];

        if let Some(last_fence_idx) = after_first_line.rfind("```") {
            let after_fence = after_first_line[last_fence_idx + 3..].trim();
            // If there's no content after the closing fence
            if after_fence.is_empty() {
                let inner = after_first_line[..last_fence_idx]
                    .trim_end_matches('\r')
                    .trim_end_matches('\n');
                let fence_header = first_line.trim();

                // Ensure the first line is a pure fence header (e.g. ```, ```rust, ```python)
                if fence_header.starts_with("```")
                    && !fence_header[3..].contains(' ')
                    && (!is_summary_or_explain
                        || fence_header == "```"
                        || fence_header.eq_ignore_ascii_case("```markdown")
                        || fence_header.eq_ignore_ascii_case("```md")
                        || fence_header.eq_ignore_ascii_case("```text"))
                {
                    return inner.to_string();
                }
            }
        }
    }

    // 2. Guard against accidental lone trailing backtick fence at the end of the response
    if !is_summary_or_explain
        && trimmed.ends_with("```")
        && !trimmed.starts_with("```")
        && trimmed.matches("```").count() % 2 == 1
        && let Some(idx) = trimmed.rfind("```")
    {
        return trimmed[..idx].trim_end().to_string();
    }

    trimmed.to_string()
}

/// Generated color theme and glass styling payload from AI.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeneratedTheme {
    pub name: String,
    pub bg: String,
    pub card: String,
    pub border: String,
    pub accent: String,
    pub secondary_accent: String,
    pub text: String,
    pub muted_text: String,
    pub danger: String,
    #[serde(default)]
    pub opacity: Option<f32>,
    #[serde(default)]
    pub blur_strength: Option<f32>,
}

/// Parses standard 3-digit or 6-digit hex color strings into RGB u8 array.
pub fn parse_hex_color(hex: &str) -> Option<[u8; 3]> {
    let clean = hex.trim().trim_start_matches('#');
    if clean.len() == 6 {
        let r = u8::from_str_radix(&clean[0..2], 16).ok()?;
        let g = u8::from_str_radix(&clean[2..4], 16).ok()?;
        let b = u8::from_str_radix(&clean[4..6], 16).ok()?;
        Some([r, g, b])
    } else if clean.len() == 3 {
        let r = u8::from_str_radix(&clean[0..1], 16).ok()? * 17;
        let g = u8::from_str_radix(&clean[1..2], 16).ok()? * 17;
        let b = u8::from_str_radix(&clean[2..3], 16).ok()? * 17;
        Some([r, g, b])
    } else {
        None
    }
}

/// Extracts outermost JSON payload from a raw LLM text response.
pub fn extract_json_block(raw: &str) -> &str {
    let trimmed = raw.trim();
    if let Some(start) = trimmed.find('{')
        && let Some(end) = trimmed.rfind('}')
        && start < end
    {
        &trimmed[start..=end]
    } else {
        trimmed
    }
}

/// Asynchronously generates a custom glassmorphism theme via AI.
pub fn spawn_generate_theme_request(
    prompt: String,
    provider: AiProvider,
    model: String,
    api_key: String,
    temperature: Option<f32>,
) -> mpsc::Receiver<Result<GeneratedTheme, String>> {
    let (tx, rx) = mpsc::channel();
    let rt = get_or_init_runtime();

    rt.spawn(async move {
        let result = execute_theme_generation(prompt, provider, model, api_key, temperature).await;
        let _ = tx.send(result);
    });

    rx
}

async fn execute_theme_generation(
    prompt: String,
    provider: AiProvider,
    model: String,
    api_key: String,
    temperature: Option<f32>,
) -> Result<GeneratedTheme, String> {
    let mut client_builder = genai::Client::builder();

    if !api_key.trim().is_empty() {
        let key_str = api_key.trim().to_string();
        let auth_resolver = genai::resolver::AuthResolver::from_resolver_fn(move |_model_iden| {
            Ok(Some(genai::resolver::AuthData::from_single(
                key_str.clone(),
            )))
        });
        client_builder = client_builder.with_auth_resolver(auth_resolver);
    }

    let client = client_builder.build();

    let system_msg = "You are a world-class UI/UX theme designer and color theorist specializing in dark glassmorphism desktop themes. When given a theme prompt or visual mood, generate a harmonious, beautiful 8-color palette formatted strictly as JSON. Ensure high contrast readability for text against backgrounds.\nOutput format must match:\n{\n  \"name\": \"Theme Name\",\n  \"bg\": \"#120c1c\",\n  \"card\": \"#1c142a\",\n  \"border\": \"#5a3282\",\n  \"accent\": \"#a855f7\",\n  \"secondary_accent\": \"#38bdf8\",\n  \"text\": \"#ebf0fa\",\n  \"muted_text\": \"#9ba5b4\",\n  \"danger\": \"#ef4444\",\n  \"opacity\": 0.85,\n  \"blur_strength\": 0.80\n}\nOutput ONLY the JSON object, with no explanation or introductory text.";

    let user_msg = format!(
        "Generate a glassmorphism color palette for the aesthetic / theme: '{}'",
        prompt
    );

    let chat_req = genai::chat::ChatRequest::new(vec![
        genai::chat::ChatMessage::system(system_msg),
        genai::chat::ChatMessage::user(user_msg),
    ]);

    let model_name = if model.trim().is_empty() {
        provider.default_model()
    } else {
        model.trim()
    };

    let temp = temperature.unwrap_or(0.4).clamp(0.0, 2.0) as f64;
    let chat_options = genai::chat::ChatOptions::default()
        .with_temperature(temp)
        .with_max_tokens(1024);

    let chat_future = client.exec_chat(model_name, chat_req, Some(&chat_options));
    let chat_res = match tokio::time::timeout(std::time::Duration::from_secs(25), chat_future).await
    {
        Ok(Ok(res)) => res,
        Ok(Err(e)) => return Err(format!("AI provider error: {}", e)),
        Err(_) => return Err("AI theme generation timed out after 25s".to_string()),
    };

    let text = chat_res
        .into_first_text()
        .unwrap_or_default()
        .trim()
        .to_string();
    let json_slice = extract_json_block(&text);

    match serde_json::from_str::<GeneratedTheme>(json_slice) {
        Ok(theme) => Ok(theme),
        Err(e) => Err(format!(
            "Failed to parse generated theme JSON: {} (Raw: {})",
            e, text
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hex_color() {
        assert_eq!(parse_hex_color("#ff5733"), Some([255, 87, 51]));
        assert_eq!(parse_hex_color("120c1c"), Some([18, 12, 28]));
        assert_eq!(parse_hex_color("#fff"), Some([255, 255, 255]));
        assert_eq!(parse_hex_color("invalid"), None);
    }

    #[test]
    fn test_extract_json_block() {
        let raw = "Here is your theme:\n```json\n{\n  \"name\": \"Test\"\n}\n```\nEnjoy!";
        let extracted = extract_json_block(raw);
        assert!(extracted.starts_with('{'));
        assert!(extracted.ends_with('}'));
        assert!(extracted.contains("\"name\": \"Test\""));
    }

    #[test]
    fn test_generated_theme_deserialization() {
        let json = r##"{
            "name": "Cyberpunk Neon",
            "bg": "#0f051d",
            "card": "#18092e",
            "border": "#ff007f",
            "accent": "#00f0ff",
            "secondary_accent": "#ffe600",
            "text": "#ffffff",
            "muted_text": "#a0a0c0",
            "danger": "#ff0033",
            "opacity": 0.90,
            "blur_strength": 0.85
        }"##;

        let theme: Result<GeneratedTheme, _> = serde_json::from_str(json);
        assert!(theme.is_ok());
        let t = theme.unwrap();
        assert_eq!(t.name, "Cyberpunk Neon");
        assert_eq!(parse_hex_color(&t.accent), Some([0, 240, 255]));
        assert_eq!(t.opacity, Some(0.90));
    }

    #[test]
    fn test_ai_request_temperature_field() {
        let req = AiRequest {
            target_text: "test".to_string(),
            context_before: "".to_string(),
            action: AiAction::FixAndPolish,
            provider: AiProvider::Gemini,
            model: "gemini-2.5-flash".to_string(),
            api_key: "".to_string(),
            temperature: Some(0.85),
            system_prompt: "test prompt".to_string(),
        };
        assert_eq!(req.temperature, Some(0.85));
    }
}
