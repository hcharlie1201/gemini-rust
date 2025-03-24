use crate::{
    models::{
        Content, FunctionCallingConfig, FunctionCallingMode, GenerateContentRequest,
        GenerationConfig, GenerationResponse, Message, Role, ToolConfig,
    },
    tools::{FunctionDeclaration, Tool},
    Error, Result,
};
use reqwest::Client;
use std::sync::Arc;
use url::Url;
use futures_util::StreamExt;
use futures::stream::Stream;
use std::pin::Pin;

const BASE_URL: &str = "https://generativelanguage.googleapis.com/v1beta/";
const DEFAULT_MODEL: &str = "models/gemini-2.0-flash";

/// Builder for content generation requests
pub struct ContentBuilder {
    client: Arc<GeminiClient>,
    pub contents: Vec<Content>,
    generation_config: Option<GenerationConfig>,
    tools: Option<Vec<Tool>>,
    tool_config: Option<ToolConfig>,
}

impl ContentBuilder {
    /// Create a new content builder
    fn new(client: Arc<GeminiClient>) -> Self {
        Self {
            client,
            contents: Vec::new(),
            generation_config: None,
            tools: None,
            tool_config: None,
        }
    }

    /// Add a system prompt to the request
    pub fn with_system_prompt(mut self, text: impl Into<String>) -> Self {
        let content = Content::text(text).with_role(Role::System);
        self.contents.push(content);
        self
    }

    /// Add a user message to the request
    pub fn with_user_message(mut self, text: impl Into<String>) -> Self {
        let content = Content::text(text).with_role(Role::User);
        self.contents.push(content);
        self
    }

    /// Add a model message to the request
    pub fn with_model_message(mut self, text: impl Into<String>) -> Self {
        let content = Content::text(text).with_role(Role::Model);
        self.contents.push(content);
        self
    }

    /// Add a function response to the request using a JSON value
    pub fn with_function_response(
        mut self,
        name: impl Into<String>,
        response: serde_json::Value,
    ) -> Self {
        let content = Content::function_response_json(name, response)
            .with_role(Role::Function);
        self.contents.push(content);
        self
    }
    
    /// Add a function response to the request using a JSON string
    pub fn with_function_response_str(
        mut self,
        name: impl Into<String>,
        response: impl Into<String>,
    ) -> std::result::Result<Self, serde_json::Error> {
        let response_str = response.into();
        let json = serde_json::from_str(&response_str)?;
        let content = Content::function_response_json(name, json)
            .with_role(Role::Function);
        self.contents.push(content);
        Ok(self)
    }

    /// Add a message to the request
    pub fn with_message(mut self, message: Message) -> Self {
        let content = message.content.clone();
        match &content.role {
            Some(role) => {
                let role_clone = role.clone();
                self.contents.push(content.with_role(role_clone));
            }
            None => {
                self.contents.push(content.with_role(message.role));
            }
        }
        self
    }

    /// Add multiple messages to the request
    pub fn with_messages(mut self, messages: impl IntoIterator<Item = Message>) -> Self {
        for message in messages {
            self = self.with_message(message);
        }
        self
    }

    /// Set the generation config for the request
    pub fn with_generation_config(mut self, config: GenerationConfig) -> Self {
        self.generation_config = Some(config);
        self
    }
    
    /// Set the temperature for the request
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        if self.generation_config.is_none() {
            self.generation_config = Some(GenerationConfig::default());
        }
        if let Some(config) = &mut self.generation_config {
            config.temperature = Some(temperature);
        }
        self
    }
    
    /// Set the top-p value for the request
    pub fn with_top_p(mut self, top_p: f32) -> Self {
        if self.generation_config.is_none() {
            self.generation_config = Some(GenerationConfig::default());
        }
        if let Some(config) = &mut self.generation_config {
            config.top_p = Some(top_p);
        }
        self
    }
    
    /// Set the top-k value for the request
    pub fn with_top_k(mut self, top_k: i32) -> Self {
        if self.generation_config.is_none() {
            self.generation_config = Some(GenerationConfig::default());
        }
        if let Some(config) = &mut self.generation_config {
            config.top_k = Some(top_k);
        }
        self
    }
    
    /// Set the maximum output tokens for the request
    pub fn with_max_output_tokens(mut self, max_output_tokens: i32) -> Self {
        if self.generation_config.is_none() {
            self.generation_config = Some(GenerationConfig::default());
        }
        if let Some(config) = &mut self.generation_config {
            config.max_output_tokens = Some(max_output_tokens);
        }
        self
    }
    
    /// Set the candidate count for the request
    pub fn with_candidate_count(mut self, candidate_count: i32) -> Self {
        if self.generation_config.is_none() {
            self.generation_config = Some(GenerationConfig::default());
        }
        if let Some(config) = &mut self.generation_config {
            config.candidate_count = Some(candidate_count);
        }
        self
    }
    
    /// Set the stop sequences for the request
    pub fn with_stop_sequences(mut self, stop_sequences: Vec<String>) -> Self {
        if self.generation_config.is_none() {
            self.generation_config = Some(GenerationConfig::default());
        }
        if let Some(config) = &mut self.generation_config {
            config.stop_sequences = Some(stop_sequences);
        }
        self
    }
    
    /// Set the response mime type for the request
    pub fn with_response_mime_type(mut self, mime_type: impl Into<String>) -> Self {
        if self.generation_config.is_none() {
            self.generation_config = Some(GenerationConfig::default());
        }
        if let Some(config) = &mut self.generation_config {
            config.response_mime_type = Some(mime_type.into());
        }
        self
    }
    
    /// Set the response schema for structured output
    pub fn with_response_schema(mut self, schema: serde_json::Value) -> Self {
        if self.generation_config.is_none() {
            self.generation_config = Some(GenerationConfig::default());
        }
        if let Some(config) = &mut self.generation_config {
            config.response_schema = Some(schema);
        }
        self
    }

    /// Add a tool to the request
    pub fn with_tool(mut self, tool: Tool) -> Self {
        if self.tools.is_none() {
            self.tools = Some(Vec::new());
        }
        if let Some(tools) = &mut self.tools {
            tools.push(tool);
        }
        self
    }

    /// Add a function declaration as a tool
    pub fn with_function(mut self, function: FunctionDeclaration) -> Self {
        let tool = Tool::new(function);
        self = self.with_tool(tool);
        self
    }

    /// Set the function calling mode for the request
    pub fn with_function_calling_mode(mut self, mode: FunctionCallingMode) -> Self {
        if self.tool_config.is_none() {
            self.tool_config = Some(ToolConfig {
                function_calling_config: Some(FunctionCallingConfig { mode }),
            });
        } else if let Some(tool_config) = &mut self.tool_config {
            tool_config.function_calling_config = Some(FunctionCallingConfig { mode });
        }
        self
    }

    /// Execute the request
    pub async fn execute(self) -> Result<GenerationResponse> {
        let request = GenerateContentRequest {
            contents: self.contents,
            generation_config: self.generation_config,
            safety_settings: None,
            tools: self.tools,
            tool_config: self.tool_config,
        };

        self.client.generate_content_raw(request).await
    }

    /// Execute the request with streaming
    pub async fn execute_stream(
        self,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<GenerationResponse>> + Send>>> {
        let request = GenerateContentRequest {
            contents: self.contents,
            generation_config: self.generation_config,
            safety_settings: None,
            tools: self.tools,
            tool_config: self.tool_config,
        };

        self.client.generate_content_stream(request).await
    }
}

/// Internal client for making requests to the Gemini API
struct GeminiClient {
    http_client: Client,
    api_key: String,
    model: String,
}

impl GeminiClient {
    /// Create a new client
    fn new(api_key: impl Into<String>, model: String) -> Self {
        Self {
            http_client: Client::new(),
            api_key: api_key.into(),
            model,
        }
    }

    /// Generate content
    async fn generate_content_raw(
        &self,
        request: GenerateContentRequest,
    ) -> Result<GenerationResponse> {
        let url = self.build_url("generateContent")?;

        let response = self
            .http_client
            .post(url)
            .json(&request)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await?;
            return Err(Error::ApiError {
                status_code: status.as_u16(),
                message: error_text,
            });
        }

        let response = response.json().await?;
        Ok(response)
    }

    /// Generate content with streaming
    async fn generate_content_stream(
        &self,
        request: GenerateContentRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<GenerationResponse>> + Send>>> {
        let url = self.build_url("streamGenerateContent")?;

        let response = self
            .http_client
            .post(url)
            .json(&request)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await?;
            return Err(Error::ApiError {
                status_code: status.as_u16(),
                message: error_text,
            });
        }

        let stream = response
            .bytes_stream()
            .map(|result| {
                match result {
                    Ok(bytes) => {
                        let text = String::from_utf8_lossy(&bytes);
                        // The stream returns each chunk as a separate JSON object
                        // Each line that starts with "data: " contains a JSON object
                        let mut responses = Vec::new();
                        for line in text.lines() {
                            if let Some(json_str) = line.strip_prefix("data: ") {
                                if json_str == "[DONE]" {
                                    continue;
                                }
                                match serde_json::from_str::<GenerationResponse>(json_str) {
                                    Ok(response) => responses.push(Ok(response)),
                                    Err(e) => responses.push(Err(Error::JsonError(e))),
                                }
                            }
                        }
                        futures::stream::iter(responses)
                    }
                    Err(e) => futures::stream::iter(vec![Err(Error::HttpError(e))]),
                }
            })
            .flatten();

        Ok(Box::pin(stream))
    }

    /// Build a URL for the API
    fn build_url(&self, endpoint: &str) -> Result<Url> {
        // All Gemini API endpoints now use the format with colon:
        // "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.0-flash:generateContent?key=$API_KEY"
        let url_str = format!(
            "{}{}:{}?key={}",
            BASE_URL, self.model, endpoint, self.api_key
        );
        Url::parse(&url_str).map_err(|e| Error::RequestError(e.to_string()))
    }
}

/// Client for the Gemini API
#[derive(Clone)]
pub struct Gemini {
    client: Arc<GeminiClient>,
}

impl Gemini {
    /// Create a new client with the specified API key
    pub fn new(api_key: impl Into<String>) -> Self {
        Self::with_model(api_key, DEFAULT_MODEL.to_string())
    }
    
    /// Create a new client for the Gemini Pro model
    pub fn pro(api_key: impl Into<String>) -> Self {
        Self::with_model(api_key, "models/gemini-2.0-pro".to_string())
    }

    /// Create a new client with the specified API key and model
    pub fn with_model(api_key: impl Into<String>, model: String) -> Self {
        let client = GeminiClient::new(api_key, model);
        Self {
            client: Arc::new(client),
        }
    }

    /// Start building a content generation request
    pub fn generate_content(&self) -> ContentBuilder {
        ContentBuilder::new(self.client.clone())
    }
}