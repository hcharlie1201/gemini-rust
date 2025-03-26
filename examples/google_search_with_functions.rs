use gemini_rust::{
    Content, FunctionCall, FunctionCallingMode, FunctionDeclaration, FunctionParameters, Gemini,
    Message, PropertyDetails, Role, Tool,
};
use serde_json::json;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get API key from environment variable
    let api_key = env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY environment variable not set");

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
                    ["add", "subtract", "multiply", "divide"],
                ),
                true,
            )
            .with_property("a", PropertyDetails::number("The first number"), true)
            .with_property("b", PropertyDetails::number("The second number"), true),
    );

    // Create function tool
    let function_tool = Tool::new(calculate);

    // Create a request with both tools
    let response = client
        .generate_content()
        .with_user_message("What is the current Google stock price multiplied by 2?")
        .with_tool(function_tool.clone())
        .with_function_calling_mode(FunctionCallingMode::Any)
        .execute()
        .await?;

    // Check if there are function calls
    if let Some(function_call) = response.function_calls().first() {
        println!(
            "Function call: {} with args: {}",
            function_call.name, function_call.args
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
            })
            .to_string();

            // Based on the curl example, we need to structure the conversation properly:
            // 1. A user message with the original query
            // 2. A model message containing the function call
            // 3. A user message containing the function response

            // Construct conversation following the exact curl pattern
            let mut conversation = client.generate_content();

            // 1. Add user message with original query
            conversation = conversation
                .with_user_message("What is the current Google stock price multiplied by 2?");

            // 2. Create model message with function call
            let model_function_call = FunctionCall::new(
                "calculate",
                json!({
                    "operation": operation,
                    "a": a,
                    "b": b
                }),
            );

            // Create model content with function call
            let model_content = Content::function_call(model_function_call).with_role(Role::Model);

            // Add as model message
            let model_message = Message {
                content: model_content,
                role: Role::Model,
            };
            conversation = conversation.with_message(model_message);

            // 3. Add user message with function response
            conversation =
                conversation.with_function_response_str("calculate", function_response)?;

            // Execute the request
            let final_response = conversation.execute().await?;

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
