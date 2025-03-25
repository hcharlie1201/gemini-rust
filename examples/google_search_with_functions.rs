use gemini_rust::{
    FunctionDeclaration, Gemini, PropertyDetails, FunctionParameters,
    FunctionCallingMode, Tool
};
use serde_json::json;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get API key from environment variable
    let api_key = env::var("GOOGLE_API_KEY")
        .expect("GOOGLE_API_KEY environment variable not set");

    // Create client
    let client = Gemini::new(api_key);

    println!("--- Google Search with Function Calling example ---");
    
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
    
    // Create function tool
    let function_tool = Tool::new(calculate);
    
    // Create Google Search tool
    let google_search_tool = Tool::google_search();
    
    // Create a request with both tools
    let response = client
        .generate_content()
        .with_user_message("What is the current Google stock price multiplied by 2?")
        .with_tool(google_search_tool)
        .with_tool(function_tool)
        .with_function_calling_mode(FunctionCallingMode::Any)
        .execute()
        .await?;
    
    // Check if there are function calls
    if let Some(function_call) = response.function_calls().first() {
        println!(
            "Function call: {} with args: {}",
            function_call.name,
            function_call.args
        );
        
        // Handle the calculate function
        if function_call.name == "calculate" {
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
                .with_user_message("What is the current Google stock price multiplied by 2?")
                .with_function_response_str("calculate", function_response)?
                .execute()
                .await?;
            
            println!("Final response: {}", final_response.text());
        } else {
            println!("Unknown function call: {}", function_call.name);
        }
    } else {
        println!("No function calls in the response.");
        println!("Direct response: {}", response.text());
    }

    Ok(())
}