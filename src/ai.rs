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
                "gemini-2.5-flash",
                "gemini-2.0-flash",
                "gemini-3.6-flash",
                "gemini-3.5-flash",
                "gemini-3.5-flash-lite",
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

    #[serde(default)]
    pub api_key: String,

    #[serde(default = "default_ai_temperature")]
    pub temperature: f32,

    #[serde(default = "default_ai_system_prompt")]
    pub system_prompt: String,
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

    let chat_options = genai::chat::ChatOptions::default()
        .with_temperature(0.2)
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
        Ok(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ai_settings_default() {
        let settings = AiSettings::default();
        assert!(settings.enabled);
        assert_eq!(settings.provider, AiProvider::Gemini);
        assert_eq!(settings.model, "gemini-3.6-flash");
        assert_eq!(settings.temperature, 0.7);
        assert!(settings.system_prompt.contains("developer note assistant"));
    }

    #[test]
    fn test_ai_providers_all_have_defaults_and_labels() {
        for provider in AiProvider::ALL {
            assert!(!provider.label().is_empty());
            assert!(!provider.default_model().is_empty());
        }
    }

    #[test]
    fn test_ai_action_prompt_instructions() {
        let fix = AiAction::FixAndPolish;
        assert!(fix.prompt_instruction().contains("Fix all spelling"));
        assert_eq!(fix.title(), "Fix & Polish");

        let complete = AiAction::Complete;
        assert!(
            complete
                .prompt_instruction()
                .contains("Seamlessly complete")
        );
        assert_eq!(complete.title(), "Complete");

        let explain = AiAction::Explain;
        assert!(explain.prompt_instruction().contains("crystal-clear"));
        assert_eq!(explain.title(), "Explain");

        let custom = AiAction::Custom("Convert to json".to_string());
        assert_eq!(custom.prompt_instruction(), "Convert to json");
        assert_eq!(custom.title(), "Custom Prompt");
    }
}
