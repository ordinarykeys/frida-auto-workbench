use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AiProvider {
    pub id: String,
    pub name: String,
    pub base_url: String,
    pub api_key: String,
    pub model: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatRequest {
    pub provider: AiProvider,
    pub messages: Vec<ChatMessage>,
    pub temperature: Option<f32>,
}

#[tauri::command]
pub async fn ai_list_models(base_url: String, api_key: String) -> Result<Vec<String>, String> {
    let url = format!("{}/models", base_url.trim_end_matches('/'));
    let client = reqwest::Client::new();
    let resp = client
        .get(&url)
        .bearer_auth(&api_key)
        .send()
        .await
        .map_err(|e| format!("请求模型列表失败: {}", e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("模型接口返回 {}: {}", status, body));
    }

    let v: Value = resp.json().await.map_err(|e| format!("解析 JSON 失败: {}", e))?;
    let mut ids: Vec<String> = Vec::new();

    if let Some(arr) = v.get("data").and_then(|d| d.as_array()) {
        for item in arr {
            if let Some(id) = item.get("id").and_then(|i| i.as_str()) {
                ids.push(id.to_string());
            }
        }
    } else if let Some(arr) = v.get("models").and_then(|d| d.as_array()) {
        for item in arr {
            if let Some(id) = item.get("name").and_then(|i| i.as_str()) {
                ids.push(id.to_string());
            } else if let Some(id) = item.get("id").and_then(|i| i.as_str()) {
                ids.push(id.to_string());
            }
        }
    }

    if ids.is_empty() {
        return Err("接口未返回任何模型 (data/models 字段为空)".into());
    }
    ids.sort();
    Ok(ids)
}

#[tauri::command]
pub async fn ai_chat(req: ChatRequest) -> Result<String, String> {
    let url = format!("{}/chat/completions", req.provider.base_url.trim_end_matches('/'));
    let client = reqwest::Client::new();

    let messages_json: Vec<Value> = req.messages.iter().map(|m| json!({
        "role": m.role,
        "content": m.content,
    })).collect();

    let payload = json!({
        "model": req.provider.model,
        "messages": messages_json,
        "temperature": req.temperature.unwrap_or(0.3),
        "stream": false,
    });

    let resp = client
        .post(&url)
        .bearer_auth(&req.provider.api_key)
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("AI 接口返回 {}: {}", status, body));
    }

    let v: Value = resp.json().await.map_err(|e| format!("解析失败: {}", e))?;
    let content = v
        .get("choices")
        .and_then(|c| c.as_array())
        .and_then(|arr| arr.first())
        .and_then(|first| first.get("message"))
        .and_then(|m| m.get("content"))
        .and_then(|c| c.as_str())
        .unwrap_or("")
        .to_string();

    if content.is_empty() {
        return Err(format!("响应中没有 content 字段: {}", v));
    }
    Ok(content)
}

/// Generate a frida hook script tailored to the target. Wraps `ai_chat` with a
/// system prompt instructing the model to emit a single JS code block.
#[tauri::command]
pub async fn ai_generate_script(
    provider: AiProvider,
    package_name: String,
    so_file: String,
    hardening: String,
    intent: String,
) -> Result<String, String> {
    let system = "你是 Android 逆向专家，精通 Frida 脚本编写。请根据用户提供的目标信息生成可直接运行的 Frida JavaScript 脚本。\n规则:\n1. 仅输出一个 ```javascript 代码块，不要任何解释\n2. 使用 Java.perform / Interceptor.attach 等标准 API\n3. 在脚本顶部用 console.log 标记 hook 加载成功\n4. 对常见加固/反调试要有规避\n5. 输出可读、带注释的代码";
    let user = format!(
        "目标包名: {}\n关注 SO: {}\n加固: {}\n用户意图: {}\n请生成对应 Frida 脚本。",
        package_name, so_file, hardening, intent
    );

    let req = ChatRequest {
        provider,
        messages: vec![
            ChatMessage { role: "system".into(), content: system.into() },
            ChatMessage { role: "user".into(), content: user },
        ],
        temperature: Some(0.2),
    };
    let raw = ai_chat(req).await?;

    // Extract first ```javascript ... ``` block
    if let Some(start) = raw.find("```") {
        let after = &raw[start+3..];
        let lang_end = after.find('\n').unwrap_or(0);
        let code_after = &after[lang_end+1..];
        if let Some(end) = code_after.find("```") {
            return Ok(code_after[..end].trim().to_string());
        }
    }
    Ok(raw)
}
