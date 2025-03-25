use gemini_rust::{
    FunctionDeclaration, Gemini, PropertyDetails, FunctionParameters,
    FunctionCallingMode, Tool
};
use serde_json::json;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get API key from environment variable
    let api_key = env::var("GEMINI_API_KEY")
        .expect("GEMINI_API_KEY environment variable not set");

    // Create client
    let client = Gemini::new(api_key);

    println!("--- Tools example with multiple functions ---");
    
    // Define a weather function
    let get_weather = FunctionDeclaration::new(
        "get_weather",
        "Get the current weather for a location",
        FunctionParameters::object()
            .with_property(
                "location",
                PropertyDetails::string("The city and state, e.g., San Francisco, CA"),
                true,
            )
            .with_property(
                "unit",
                PropertyDetails::enum_type(
                    "The unit of temperature", 
                    ["celsius", "fahrenheit"]
                ),
                false,
            ),
    );
    
    // Define a calculator function
    let calculate = FunctionDeclaration::new(
        "calculate",
        "Perform a calculation",
        FunctionParameters::object()
            .with_property(
                "operation",
                PropertyDetails::enum_type(
                    "The mathematical operation to perform",
                    ["add", "subtract", "multiply", "divide"]
                ),
                true,
            )
            .with_property(
                "a",
                PropertyDetails::number("The first number"),
                true,
            )
            .with_property(
                "b",
                PropertyDetails::number("The second number"),
                true,
            ),
    );
    
    // Create a tool with multiple functions
    let tool = Tool::with_functions(vec![get_weather, calculate]);

    // Create a request with tool functions
    let response = client
        .generate_content()
        .with_system_prompt("You are a helpful assistant that can check weather and perform calculations.")
        .with_user_message("What's 42 times 12?")
        .with_tool(tool)
        .with_function_calling_mode(FunctionCallingMode::Any)
        .execute()
        .await?;

    // Process function calls
    if let Some(function_call) = response.function_calls().first() {
        println!(
            "Function call: {} with args: {}",
            function_call.name,
            function_call.args
        );
        
        // Handle different function calls
        match function_call.name.as_str() {
            "calculate" => {
                let operation: String = function_call.get("operation")?;
                let a: f64 = function_call.get("a")?;
                let b: f64 = function_call.get("b")?;
                
                println!("Calculation: {} {} {}", a, operation, b);
                
                let result = match operation.as_str() {
                    "add" => a + b,
                    "subtract" => a - b,
                    "multiply" => a * b,
                    "divide" => a / b,
                    _ => panic!("Unknown operation"),
                };
                
                let function_response = json!({
                    "result": result,
                }).to_string();
                
                // Continue the conversation with the function result
                let final_response = client
                    .generate_content()
                    .with_system_prompt("You are a helpful assistant that can check weather and perform calculations.")
                    .with_user_message("What's 42 times 12?")
                    .with_function_response_str("calculate", function_response)?
                    .execute()
                    .await?;
                
                println!("Final response: {}", final_response.text());
            },
            "get_weather" => {
                let location: String = function_call.get("location")?;
                let unit = function_call.get::<String>("unit").unwrap_or_else(|_| String::from("celsius"));
                
                println!("Weather request for: {}, Unit: {}", location, unit);
                
                let weather_response = json!({
                    "temperature": 22,
                    "unit": unit,
                    "condition": "sunny"
                }).to_string();
                
                // Continue the conversation with the function result
                let final_response = client
                    .generate_content()
                    .with_system_prompt("You are a helpful assistant that can check weather and perform calculations.")
                    .with_user_message("What's 42 times 12?")
                    .with_function_response_str("get_weather", weather_response)?
                    .execute()
                    .await?;
                
                println!("Final response: {}", final_response.text());
            },
            _ => println!("Unknown function"),
        }
    } else {
        println!("No function calls in the response.");
        println!("Response: {}", response.text());
    }

    Ok(())
}