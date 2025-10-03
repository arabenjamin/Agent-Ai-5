use anyhow::Result;
use clap::Parser;
use tracing::{info, error};
use tracing_subscriber::{prelude::*, EnvFilter};

mod ollama;
mod mcp;

#[derive(Parser)]
#[command(name = "mcp-client")]
#[command(about = "A CLI tool to interact with Ollama and MCP server")]
struct Cli {
    #[arg(long, default_value = "http://localhost:11434")]
    ollama_url: String,
    
    #[arg(long, default_value = "http://localhost:3001")]
    mcp_url: String,
    
    #[arg(long, default_value = "info")]
    log_level: String,
    
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand)]
enum Commands {
    /// List available tools from MCP server
    ListTools,
    
    /// Call a specific tool
    CallTool {
        /// Name of the tool to call
        #[arg(long)]
        name: String,
        
        /// Arguments for the tool (as JSON string)
        #[arg(long)]
        args: Option<String>,
    },
    
    /// List available Ollama models
    ListModels,
    
    /// Ask a question to an Ollama model
    Ask {
        /// Name of the model to use
        #[arg(long)]
        model: String,
        
        /// The prompt/question to send
        #[arg(long)]
        prompt: String,
    },

    /// Chat with a model and let it use MCP tools
    Chat {
        /// Name of the model to use
        #[arg(long)]
        model: String,
        
        /// The prompt/question to send
        #[arg(long)]
        prompt: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new(&cli.log_level))
        .init();
        
    info!("Starting MCP Client");
    
    match cli.command {
        Commands::ListTools => {
            let client = mcp::McpClient::new(&cli.mcp_url);
            match client.list_tools().await {
                Ok(tools) => {
                    println!("Available tools:");
                    for tool in tools {
                        println!("- {}: {}", tool.name, tool.description);
                    }
                }
                Err(e) => error!("Failed to list tools: {}", e),
            }
        }
        
        Commands::CallTool { name, args } => {
            let client = mcp::McpClient::new(&cli.mcp_url);
            let args = if let Some(args_str) = args {
                serde_json::from_str(&args_str)?
            } else {
                serde_json::Map::new()
            };
            
            match client.call_tool(&name, args).await {
                Ok(response) => println!("{}", serde_json::to_string_pretty(&response)?),
                Err(e) => error!("Failed to call tool: {}", e),
            }
        }
        
        Commands::ListModels => {
            let client = ollama::OllamaClient::new(&cli.ollama_url);
            match client.list_models().await {
                Ok(models) => {
                    println!("Available models:");
                    for model in models {
                        println!("- {}", model.name);
                    }
                }
                Err(e) => error!("Failed to list models: {}", e),
            }
        }
        
        Commands::Ask { model, prompt } => {
            let client = ollama::OllamaClient::new(&cli.ollama_url);
            match client.generate(&model, &prompt).await {
                Ok(response) => println!("{}", response),
                Err(e) => error!("Failed to generate response: {}", e),
            }
        }

        Commands::Chat { model, prompt } => {
            let mcp_client = mcp::McpClient::new(&cli.mcp_url);
            let ollama_client = ollama::OllamaClient::new(&cli.ollama_url);

            // First get the list of available tools
            let tools = match mcp_client.list_tools().await {
                Ok(tools) => tools,
                Err(e) => {
                    error!("Failed to list tools: {}", e);
                    return Ok(());
                }
            };
            
            // Create a system prompt that describes the available tools
            let mut system_prompt = String::from(
                "You are a helpful AI assistant with access to the following tools:\n\n"
            );
            
            for tool in &tools {
                system_prompt.push_str(&format!(
                    "Tool: {}\nDescription: {}\nInput Schema: {}\n\n",
                    tool.name,
                    tool.description,
                    serde_json::to_string_pretty(&tool.input_schema)?
                ));
            }
            
            system_prompt.push_str(
                "\nRules for our interaction:\n\n"
            );
            system_prompt.push_str(
                "1. When I ask about available tools, give me a natural language description of each tool.\n\n"
            );
            system_prompt.push_str(
                "2. When you need to USE a tool, your entire response must be ONLY the JSON tool call:\n"
            );
            system_prompt.push_str(
                r#"{"type":"tool","tool_name":"example","arguments":{"key":"value"}}"#
            );
            system_prompt.push_str(
                "\n\nCritical rules for tool usage:\n"
            );
            system_prompt.push_str(
                "- Your ENTIRE response must be the JSON object - no other text\n"
            );
            system_prompt.push_str(
                "- No explanations before or after the JSON\n"
            );
            system_prompt.push_str(
                "- No 'I will use' or other commentary\n"
            );
            system_prompt.push_str(
                "- One JSON object on a single line\n"
            );
            system_prompt.push_str(
                "- After getting tool results, you can explain them in natural language\n\n"
            );
            system_prompt.push_str(
                "What would you like me to do?"
            );

            // Combine system prompt with user's prompt
            let full_prompt = format!("{}\n\nUser: {}", system_prompt, prompt);
            
            // Get the model's response
            match ollama_client.generate(&model, &full_prompt).await {
                Ok(response) => {
                    println!("Raw response from model: {}", response);
                    
                    // Extract JSON from the response by looking for the first '{' and last '}'
                    let json_str = if let (Some(start), Some(end)) = (
                        response.find('{'),
                        response.rfind('}').map(|i| i + 1)
                    ) {
                        &response[start..end]
                    } else {
                        response.trim()
                    };

                    println!("Extracted JSON: {}", json_str);
                    match serde_json::from_str::<serde_json::Value>(json_str) {
                        Ok(tool_call) => {
                            if tool_call["type"] == "tool" {
                                if let (Some(tool_name), Some(arguments)) = (
                                    tool_call["tool_name"].as_str(),
                                    tool_call["arguments"].as_object()
                                ) {
                                    println!("Using tool: {} with arguments: {}", 
                                        tool_name, 
                                        serde_json::to_string_pretty(arguments).unwrap()
                                    );
                                    
                                    match mcp_client.call_tool(tool_name, arguments.clone()).await {
                                        Ok(result) => {
                                            let mut tool_result = String::new();
                                            for block in result {
                                                match block {
                                                    mcp::ContentBlock::Text { text } => {
                                                        tool_result.push_str(&text);
                                                        tool_result.push('\n');
                                                    }
                                                }
                                            }
                                            println!("Tool result:\n{}", tool_result);
                                            
                                            // Ask the model to interpret the results
                                        let interpret_prompt = format!(
                                            "I received this result from running a tool:\n\n{}\n\nPlease explain what this means in plain English. Do NOT return JSON - just explain the results as you would to a user.",
                                            tool_result
                                        );                                            match ollama_client.generate(&model, &interpret_prompt).await {
                                                Ok(interpretation) => println!("\nInterpretation:\n{}", interpretation),
                                                Err(e) => error!("Failed to interpret results: {}", e),
                                            }
                                        }
                                        Err(e) => error!("Failed to call tool: {}. Tool: {}, Args: {}", 
                                            e, 
                                            tool_name, 
                                            serde_json::to_string_pretty(arguments).unwrap()
                                        ),
                                    }
                                } else {
                                    println!("Invalid tool call format in response: {}", response);
                                }
                            } else {
                                println!("{}", response);
                            }
                        }
                        Err(e) => {
                            println!("Failed to parse JSON: {}", e);
                            println!("Original response: {}", response);
                        }
                    }
                }
                Err(e) => error!("Failed to generate response: {}", e),
            }
        }
    }
    
    Ok(())
}