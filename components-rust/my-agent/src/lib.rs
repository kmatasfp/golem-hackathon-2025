mod bindings;

use crate::bindings::exports::my::agent_exports::my_agent_api::*;
use crate::bindings::golem::llm::llm::{self, ChatEvent, CompleteResponse, ContentPart};
use reqwest::{Client, Response};
use scraper::{Html, Selector};
use std::collections::HashMap;
use std::sync::{LazyLock, RwLock};

/// The internal status of our component. This will be automatically persisted by golem.
struct State {
    context: Vec<String>,
    history: Vec<Exchange>,
}

static STATE: LazyLock<RwLock<State>> = LazyLock::new(|| {
    RwLock::new(State {
        context: Vec::new(),
        history: Vec::new(),
    })
});

static LLM_CONFIG: LazyLock<llm::Config> = LazyLock::new(|| llm::Config {
    model: "gpt-4.1-nano".to_string(),
    temperature: Some(0.2),
    tools: vec![],
    provider_options: vec![],
    max_tokens: None,
    tool_choice: None,
    stop_sequences: None,
});

struct Component;

impl Guest for Component {
    fn add_context(context: String) {
        let mut state = STATE.write().unwrap();

        if let Some(url) = extract_url(&context) {
            let content = download_html_body_from_url(&url).unwrap();
            state.context.push(content);
        } else {
            state.context.push(context);
        }
    }

    fn get_contexts() -> Vec<String> {
        let state = STATE.read().unwrap();
        state.context.clone()
    }

    fn get_history() -> Vec<Exchange> {
        let state = STATE.read().unwrap();

        state.history.clone()
    }

    fn clear_contexts() {
        let mut state = STATE.write().unwrap();
        state.context.clear();
    }

    fn prompt(input: String) -> String {
        if let Some(url) = extract_url(&input) {
            let content = download_html_body_from_url(&url).unwrap();
            Self::add_context(content);
        }

        let context = {
            let state = STATE.read().unwrap();
            state.context.join(" ")
        };

        // let parsed_llm_response = ask_model(&input, &context).unwrap();

        let prompt = format!("Query: {}\nContext: {}\nResponse:", &input, &context);

        let llm_response = llm::send(
            &[llm::Message {
                role: llm::Role::System,
                name: None,
                content: vec![llm::ContentPart::Text(prompt)],
            }],
            &LLM_CONFIG,
        );

        let parsed_llm_response = parse_llm_response(llm_response);

        let exchange = Exchange {
            prompt: input,
            response: parsed_llm_response.clone(),
        };

        Self::add_context(parsed_llm_response.clone());

        let mut state = STATE.write().unwrap();

        state.history.push(exchange);

        parsed_llm_response
    }
}

fn ask_model(prompt: &str, context: &str) -> std::result::Result<String, String> {
    let open_api_key = std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not present");

    let bearer_token = format!("Bearer {}", open_api_key);

    let prompt = format!("Query: {}\nContext: {}\nResponse:", prompt, context);

    let client = Client::new();

    let body = serde_json::json!({
        "model": "gpt-4o",
        "messages": [{"role": "system", "content": prompt}],
    });

    let response: Response = client
        .post("https://api.openai.com/v1/chat/completions".to_string())
        .json(&body)
        .header("Authorization", bearer_token)
        .send()
        .expect("Request failed");

    let result: HashMap<String, serde_json::Value> =
        response.json().map_err(|err| err.to_string())?;

    let llm_response = result["choices"][0]["message"]["content"]
        .as_str()
        .unwrap()
        .to_string();

    Ok(llm_response)
}

fn extract_url(text: &str) -> Option<String> {
    // Look for the start of the URL in the string.
    let start_index = text.find("http://").or_else(|| text.find("https://"))?;

    // Slice the string starting at the URL.
    let url_fragment = &text[start_index..];

    // Find the first whitespace after the URL start.
    let end_index = url_fragment
        .find(char::is_whitespace)
        .unwrap_or(url_fragment.len());

    // Return the extracted URL.
    Some(url_fragment[..end_index].to_string())
}

fn download_html_body_from_url(url: &str) -> std::result::Result<String, String> {
    let client = Client::new();

    let response = client.get(url).send().expect("Request failed");

    let text = response.text().map_err(|e| format!("error was: {}", e))?;

    let document = Html::parse_document(&text);

    let body_selector = Selector::parse("body").unwrap();

    if let Some(body) = document.select(&body_selector).next() {
        // Extract and join all text nodes from the body.
        let cleaned_text = body
            .text()
            .filter(|s| !s.trim().is_empty())
            .collect::<Vec<_>>()
            .join(" ");
        Ok(cleaned_text)
    } else {
        Err("failed to parse body".to_string())
    }
}

fn parse_llm_response(raw_response: ChatEvent) -> String {
    match raw_response {
        ChatEvent::Message(CompleteResponse { content, .. }) => match content.as_slice() {
            [ContentPart::Text(text)] => text.to_string(),
            _ => panic!("received unexpected content from llm"),
        },
        _ => panic!("received unexpected response from llm"),
    }
}

bindings::export!(Component with_types_in bindings);
