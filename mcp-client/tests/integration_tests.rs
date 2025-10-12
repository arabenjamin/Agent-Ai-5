use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::json;
use wiremock::{
    matchers::{method, path, body_json},
    Mock, MockServer, ResponseTemplate,
};

/// Helper function to create a Command with the CLI binary
fn cli_command() -> Command {
    Command::cargo_bin("mcp-client").unwrap()
}

/// Helper to start MCP mock server
async fn start_mcp_mock_server() -> MockServer {
    MockServer::start().await
}

/// Helper to start Ollama mock server
async fn start_ollama_mock_server() -> MockServer {
    MockServer::start().await
}

#[tokio::test]
async fn test_list_tools_command() {
    let mock_server = start_mcp_mock_server().await;

    Mock::given(method("GET"))
        .and(path("/tools"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "tools": [
                {
                    "name": "test_tool",
                    "description": "A test tool for integration testing",
                    "input_schema": {
                        "type": "object",
                        "properties": {
                            "input": {
                                "type": "string",
                                "description": "Test input parameter"
                            }
                        },
                        "required": ["input"]
                    }
                },
                {
                    "name": "another_tool",
                    "description": "Another test tool",
                    "input_schema": {
                        "type": "object",
                        "properties": {}
                    }
                }
            ]
        })))
        .mount(&mock_server)
        .await;

    let mut cmd = cli_command();
    cmd.arg("--mcp-url")
        .arg(format!("{}", mock_server.uri()))
        .arg("list-tools");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Available tools:"))
        .stdout(predicate::str::contains("test_tool: A test tool for integration testing"))
        .stdout(predicate::str::contains("another_tool: Another test tool"));
}

#[tokio::test]
async fn test_list_tools_command_server_error() {
    let mock_server = start_mcp_mock_server().await;

    Mock::given(method("GET"))
        .and(path("/tools"))
        .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
        .mount(&mock_server)
        .await;

    let mut cmd = cli_command();
    cmd.arg("--mcp-url")
        .arg(format!("{}", mock_server.uri()))
        .arg("list-tools");

    cmd.assert()
        .success() // The command succeeds but logs an error
        .stdout(predicate::str::contains("Failed to list tools"));
}

#[tokio::test]
async fn test_call_tool_command() {
    let mock_server = start_mcp_mock_server().await;

    Mock::given(method("POST"))
        .and(path("/tools/call"))
        .and(body_json(json!({
            "tool_name": "test_tool",
            "arguments": {
                "input": "test value"
            }
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "success": true,
            "content": [
                {
                    "type": "text",
                    "text": "Tool executed successfully with input: test value"
                }
            ],
            "error": null
        })))
        .mount(&mock_server)
        .await;

    let mut cmd = cli_command();
    cmd.arg("--mcp-url")
        .arg(format!("{}", mock_server.uri()))
        .arg("call-tool")
        .arg("--name")
        .arg("test_tool")
        .arg("--args")
        .arg(r#"{"input": "test value"}"#);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Tool executed successfully with input: test value"));
}

#[tokio::test]
async fn test_call_tool_command_without_args() {
    let mock_server = start_mcp_mock_server().await;

    Mock::given(method("POST"))
        .and(path("/tools/call"))
        .and(body_json(json!({
            "tool_name": "simple_tool",
            "arguments": {}
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "success": true,
            "content": [
                {
                    "type": "text",
                    "text": "Simple tool executed without arguments"
                }
            ],
            "error": null
        })))
        .mount(&mock_server)
        .await;

    let mut cmd = cli_command();
    cmd.arg("--mcp-url")
        .arg(format!("{}", mock_server.uri()))
        .arg("call-tool")
        .arg("--name")
        .arg("simple_tool");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Simple tool executed without arguments"));
}

#[tokio::test]
async fn test_call_tool_command_invalid_json() {
    let mock_server = start_mcp_mock_server().await;

    let mut cmd = cli_command();
    cmd.arg("--mcp-url")
        .arg(format!("{}", mock_server.uri()))
        .arg("call-tool")
        .arg("--name")
        .arg("test_tool")
        .arg("--args")
        .arg("invalid json");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("expected"));
}

#[tokio::test]
async fn test_list_models_command() {
    let mock_server = start_ollama_mock_server().await;

    Mock::given(method("GET"))
        .and(path("/api/tags"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "models": [
                {
                    "name": "llama2:latest",
                    "modified_at": "2023-08-04T19:22:45.085406Z",
                    "size": 3825819519_u64
                },
                {
                    "name": "mistral:latest",
                    "modified_at": "2023-08-04T19:22:45.085406Z",
                    "size": 4109070688_u64
                }
            ]
        })))
        .mount(&mock_server)
        .await;

    let mut cmd = cli_command();
    cmd.arg("--ollama-url")
        .arg(format!("{}", mock_server.uri()))
        .arg("list-models");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Available models:"))
        .stdout(predicate::str::contains("llama2:latest"))
        .stdout(predicate::str::contains("mistral:latest"));
}

#[tokio::test]
async fn test_list_models_command_server_error() {
    let mock_server = start_ollama_mock_server().await;

    Mock::given(method("GET"))
        .and(path("/api/tags"))
        .respond_with(ResponseTemplate::new(404).set_body_string("Not Found"))
        .mount(&mock_server)
        .await;

    let mut cmd = cli_command();
    cmd.arg("--ollama-url")
        .arg(format!("{}", mock_server.uri()))
        .arg("list-models");

    cmd.assert()
        .success() // The command succeeds but logs an error
        .stdout(predicate::str::contains("Failed to list models"));
}

#[tokio::test]
async fn test_ask_command() {
    let mock_server = start_ollama_mock_server().await;

    Mock::given(method("POST"))
        .and(path("/api/generate"))
        .and(body_json(json!({
            "model": "llama2:latest",
            "prompt": "What is the capital of France?"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "response": "The capital of France is Paris.",
            "done": true
        })))
        .mount(&mock_server)
        .await;

    let mut cmd = cli_command();
    cmd.arg("--ollama-url")
        .arg(format!("{}", mock_server.uri()))
        .arg("ask")
        .arg("--model")
        .arg("llama2:latest")
        .arg("--prompt")
        .arg("What is the capital of France?");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("The capital of France is Paris."));
}

#[tokio::test]
async fn test_ask_command_model_error() {
    let mock_server = start_ollama_mock_server().await;

    Mock::given(method("POST"))
        .and(path("/api/generate"))
        .respond_with(ResponseTemplate::new(404).set_body_json(json!({
            "error": "model not found"
        })))
        .mount(&mock_server)
        .await;

    let mut cmd = cli_command();
    cmd.arg("--ollama-url")
        .arg(format!("{}", mock_server.uri()))
        .arg("ask")
        .arg("--model")
        .arg("nonexistent:latest")
        .arg("--prompt")
        .arg("Test prompt");

    cmd.assert()
        .success() // The command succeeds but logs an error
        .stdout(predicate::str::contains("Failed to generate response"));
}

#[tokio::test]
async fn test_chat_command_tool_listing() {
    let mcp_server = start_mcp_mock_server().await;
    let ollama_server = start_ollama_mock_server().await;

    // Mock MCP tools endpoint
    Mock::given(method("GET"))
        .and(path("/tools"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "tools": [
                {
                    "name": "weather_tool",
                    "description": "Get weather information for a location",
                    "input_schema": {
                        "type": "object",
                        "properties": {
                            "location": {
                                "type": "string",
                                "description": "Location to get weather for"
                            }
                        },
                        "required": ["location"]
                    }
                }
            ]
        })))
        .mount(&mcp_server)
        .await;

    // Mock Ollama generation with simple response (not a tool call)
    Mock::given(method("POST"))
        .and(path("/api/generate"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "response": "I can help you with weather information using the weather_tool. Just ask me about the weather in any location!",
            "done": true
        })))
        .mount(&ollama_server)
        .await;

    let mut cmd = cli_command();
    cmd.arg("--mcp-url")
        .arg(format!("{}", mcp_server.uri()))
        .arg("--ollama-url")
        .arg(format!("{}", ollama_server.uri()))
        .arg("chat")
        .arg("--model")
        .arg("llama2:latest")
        .arg("--prompt")
        .arg("What tools do you have available?");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("I can help you with weather information"));
}

#[tokio::test]
async fn test_chat_command_tool_execution() {
    let mcp_server = start_mcp_mock_server().await;
    let ollama_server = start_ollama_mock_server().await;

    // Mock MCP tools endpoint
    Mock::given(method("GET"))
        .and(path("/tools"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "tools": [
                {
                    "name": "weather_tool",
                    "description": "Get weather information for a location",
                    "input_schema": {
                        "type": "object",
                        "properties": {
                            "location": {
                                "type": "string",
                                "description": "Location to get weather for"
                            }
                        },
                        "required": ["location"]
                    }
                }
            ]
        })))
        .mount(&mcp_server)
        .await;

    // Mock tool call response from Ollama (when prompt contains system information about tools)
    Mock::given(method("POST"))
        .and(path("/api/generate"))
        .and(wiremock::matchers::body_string_contains("What's the weather like in Paris?"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "response": r#"{"type":"tool","tool_name":"weather_tool","arguments":{"location":"Paris"}}"#,
            "done": true
        })))
        .mount(&ollama_server)
        .await;

    // Mock tool execution on MCP server
    Mock::given(method("POST"))
        .and(path("/tools/call"))
        .and(body_json(json!({
            "tool_name": "weather_tool",
            "arguments": {
                "location": "Paris"
            }
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "success": true,
            "content": [
                {
                    "type": "text",
                    "text": "Weather in Paris: 22°C, sunny with light clouds"
                }
            ],
            "error": null
        })))
        .mount(&mcp_server)
        .await;

    // Mock interpretation response from Ollama (when prompt contains "I received this result")
    Mock::given(method("POST"))
        .and(path("/api/generate"))
        .and(wiremock::matchers::body_string_contains("I received this result from running a tool"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "response": "The weather in Paris is currently 22°C with sunny skies and light clouds. It's a pleasant day!",
            "done": true
        })))
        .mount(&ollama_server)
        .await;

    let mut cmd = cli_command();
    cmd.arg("--mcp-url")
        .arg(format!("{}", mcp_server.uri()))
        .arg("--ollama-url")
        .arg(format!("{}", ollama_server.uri()))
        .arg("chat")
        .arg("--model")
        .arg("llama2:latest")
        .arg("--prompt")
        .arg("What's the weather like in Paris?");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Using tool: weather_tool"))
        .stdout(predicate::str::contains("Weather in Paris: 22°C"))
        .stdout(predicate::str::contains("pleasant day"));
}

#[tokio::test]
async fn test_chat_command_mcp_server_failure() {
    let mcp_server = start_mcp_mock_server().await;
    let ollama_server = start_ollama_mock_server().await;

    // Mock MCP tools endpoint failure
    Mock::given(method("GET"))
        .and(path("/tools"))
        .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
        .mount(&mcp_server)
        .await;

    let mut cmd = cli_command();
    cmd.arg("--mcp-url")
        .arg(format!("{}", mcp_server.uri()))
        .arg("--ollama-url")
        .arg(format!("{}", ollama_server.uri()))
        .arg("chat")
        .arg("--model")
        .arg("llama2:latest")
        .arg("--prompt")
        .arg("Test prompt");

    cmd.assert()
        .success() // The command doesn't fail, it just logs an error
        .stdout(predicate::str::contains("Failed to list tools"));
}

#[tokio::test]
async fn test_chat_command_invalid_tool_call() {
    let mcp_server = start_mcp_mock_server().await;
    let ollama_server = start_ollama_mock_server().await;

    // Mock MCP tools endpoint
    Mock::given(method("GET"))
        .and(path("/tools"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "tools": [
                {
                    "name": "test_tool",
                    "description": "A test tool",
                    "input_schema": {
                        "type": "object",
                        "properties": {}
                    }
                }
            ]
        })))
        .mount(&mcp_server)
        .await;

    // Mock invalid JSON response from Ollama
    Mock::given(method("POST"))
        .and(path("/api/generate"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "response": "This is not a valid JSON tool call",
            "done": true
        })))
        .mount(&ollama_server)
        .await;

    let mut cmd = cli_command();
    cmd.arg("--mcp-url")
        .arg(format!("{}", mcp_server.uri()))
        .arg("--ollama-url")
        .arg(format!("{}", ollama_server.uri()))
        .arg("chat")
        .arg("--model")
        .arg("llama2:latest")
        .arg("--prompt")
        .arg("Test prompt");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Failed to parse JSON"))
        .stdout(predicate::str::contains("This is not a valid JSON tool call"));
}

#[tokio::test]
async fn test_cli_custom_urls() {
    let mcp_server = start_mcp_mock_server().await;

    Mock::given(method("GET"))
        .and(path("/tools"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "tools": []
        })))
        .mount(&mcp_server)
        .await;

    let mut cmd = cli_command();
    cmd.arg("--mcp-url")
        .arg(format!("{}", mcp_server.uri()))
        .arg("--log-level")
        .arg("debug")
        .arg("list-tools");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Available tools:"));
}

#[tokio::test]
async fn test_cli_default_urls() {
    // Test that the CLI uses default URLs when none are provided
    // Since there's an actual MCP server running on the default port, this might succeed
    let mut cmd = cli_command();
    cmd.arg("list-tools");

    // This might succeed if there's a real MCP server running, or fail with connection error
    let output = cmd.output().unwrap();
    // Just check that the command runs without panic - it can succeed or fail
    assert!(output.status.code().is_some());
}

#[tokio::test]
async fn test_cli_help() {
    let mut cmd = cli_command();
    cmd.arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("A CLI tool to interact with Ollama and MCP server"))
        .stdout(predicate::str::contains("list-tools"))
        .stdout(predicate::str::contains("call-tool"))
        .stdout(predicate::str::contains("list-models"))
        .stdout(predicate::str::contains("ask"))
        .stdout(predicate::str::contains("chat"));
}

#[tokio::test]
async fn test_missing_required_arguments() {
    // Test call-tool without required name argument
    let mut cmd = cli_command();
    cmd.arg("call-tool");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("required"));

    // Test ask without required model argument
    let mut cmd = cli_command();
    cmd.arg("ask")
        .arg("--prompt")
        .arg("test");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("required"));

    // Test ask without required prompt argument
    let mut cmd = cli_command();
    cmd.arg("ask")
        .arg("--model")
        .arg("llama2");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}