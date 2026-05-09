use anyhow::{anyhow, Context, Result};
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde_json::{json, Value};

use crate::models::{AiProviderConfig, AiScriptResult, ModelListResponse, ProviderModel};

pub async fn fetch_models(config: &AiProviderConfig) -> Result<Vec<ProviderModel>> {
    let endpoint = format!(
        "{}{}",
        config.base_url.trim_end_matches('/'),
        config
            .models_endpoint
            .as_deref()
            .unwrap_or("/models")
    );

    let client = reqwest::Client::new();
    let mut request = client.get(endpoint);
    if let Some(api_key) = &config.api_key {
        if !api_key.is_empty() {
            request = request.header(AUTHORIZATION, format!("Bearer {api_key}"));
        }
    }

    let response = request
        .send()
        .await
        .context("failed to fetch models from provider")?
        .error_for_status()
        .context("provider returned error while listing models")?;

    let payload: ModelListResponse = response.json().await.context("invalid model list payload")?;
    Ok(payload
        .data
        .into_iter()
        .map(|item| ProviderModel {
            label: item.id.clone(),
            id: item.id,
            context_window: None,
        })
        .collect())
}

pub async fn generate_script(
    config: &AiProviderConfig,
    package_name: &str,
    so_name: Option<&str>,
    hardening_info: Option<&str>,
    analysis_notes: &[String],
) -> Result<AiScriptResult> {
    let model_id = config
        .model_id
        .clone()
        .filter(|value| !value.is_empty())
        .ok_or_else(|| anyhow!("missing model id"))?;

    let prompt = format!(
        "You are an Android Frida automation engine. Generate one production-ready Frida JavaScript script for a rooted Android device.\n\
Target package: {package_name}\n\
Selected shared object: {}\n\
Hardening: {}\n\
Manifest signals:\n- {}\n\
Requirements:\n\
1. No markdown fences.\n\
2. Produce runnable Frida JavaScript.\n\
3. Focus on process discovery, Java/native hook bootstrap, and detailed send() logs.\n\
4. If a shared object is provided, include dlopen/load hooks and module load observation for it.\n\
5. Add cautious try/catch blocks.\n\
6. Return concise summary plus self-check notes.\n\
7. Do not ask the user questions.\n",
        so_name.unwrap_or("none"),
        hardening_info.unwrap_or("unknown"),
        if analysis_notes.is_empty() {
            "No additional manifest signals.".to_string()
        } else {
            analysis_notes.join("\n- ")
        }
    );

    let body = json!({
        "model": model_id,
        "temperature": config.temperature,
        "messages": [
            {
                "role": "system",
                "content": "Return JSON with keys script, summary, self_check_notes. self_check_notes must be an array of short strings."
            },
            {
                "role": "user",
                "content": prompt
            }
        ],
        "response_format": {
            "type": "json_schema",
            "json_schema": {
                "name": "frida_script_result",
                "schema": {
                    "type": "object",
                    "properties": {
                        "script": { "type": "string" },
                        "summary": { "type": "string" },
                        "self_check_notes": {
                            "type": "array",
                            "items": { "type": "string" }
                        }
                    },
                    "required": ["script", "summary", "self_check_notes"],
                    "additionalProperties": false
                }
            }
        }
    });

    let endpoint = format!("{}/chat/completions", config.base_url.trim_end_matches('/'));
    let client = reqwest::Client::new();
    let mut request = client.post(endpoint).header(CONTENT_TYPE, "application/json");
    if let Some(api_key) = &config.api_key {
        if !api_key.is_empty() {
            request = request.header(AUTHORIZATION, format!("Bearer {api_key}"));
        }
    }

    let response = request
        .json(&body)
        .send()
        .await
        .context("failed to generate script from provider")?
        .error_for_status()
        .context("provider returned error during script generation")?;

    let payload: Value = response.json().await.context("invalid provider response")?;
    let content = payload
        .pointer("/choices/0/message/content")
        .and_then(|value| value.as_str())
        .ok_or_else(|| anyhow!("provider response missing message content"))?;

    let mut parsed: AiScriptResult = serde_json::from_str(content).context("provider content was not valid JSON schema output")?;

    if config.enable_self_check {
        parsed.self_check_notes.push(format!(
            "Self-check enabled with up to {} reasoning passes.",
            config.max_iterations.max(1)
        ));
    }

    Ok(parsed)
}
