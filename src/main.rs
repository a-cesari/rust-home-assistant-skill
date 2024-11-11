use lambda_runtime::{service_fn, LambdaEvent, Error};
use log::debug;
use reqwest::blocking::Client;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde_json::{json, Value};
use std::env;

async fn lambda_handler(event: LambdaEvent<Value>) -> Result<Value, Error> {
    let _ = env_logger::try_init();

    let (alexa_event, _context) = event.into_parts();
    let base_url = env::var("BASE_URL").expect("BASE_URL environment variable not set");
    let long_lived_access_token = env::var("LONG_LIVED_ACCESS_TOKEN").ok();
    let not_verify_ssl = env::var("NOT_VERIFY_SSL").is_ok();

    let base_url = base_url.trim_end_matches('/');

    // Ensure payload version is "3"
    let payload_version = alexa_event["directive"]["header"]["payloadVersion"]
        .as_str()
        .ok_or("Only payloadVersion 3 is supported")?;
    if payload_version != "3" {
        return Ok(json!({
            "event": {
                "payload": {
                    "type": "INVALID_DIRECTIVE",
                    "message": "Only payloadVersion 3 is supported"
                }
            }
        }));
    }

    // Get token
    let scope = alexa_event["directive"]["endpoint"]["scope"]
        .as_object()
        .or_else(|| alexa_event["directive"]["payload"]["scope"].as_object())
        .or_else(|| alexa_event["directive"]["payload"]["grantee"].as_object())
        .ok_or("Malformatted request - missing endpoint.scope")?;

    let scope_type = scope.get("type").and_then(|s| s.as_str()).ok_or("Invalid scope type")?;
    if scope_type != "BearerToken" {
        return Ok(json!({
            "event": {
                "payload": {
                    "type": "INVALID_SCOPE",
                    "message": "Only BearerToken is supported"
                }
            }
        }));
    }

    let token = scope
        .get("token")
        .and_then(|s| s.as_str())
        .map(|s| s.to_string())
        .or(long_lived_access_token)
        .ok_or("Missing token")?;

    // Create HTTP client
    let client = Client::builder()
        .danger_accept_invalid_certs(not_verify_ssl)
        .timeout(std::time::Duration::new(10, 0))
        .build()?;

    let response = client
        .post(&format!("{}/api/alexa/smart_home", base_url))
        .header(AUTHORIZATION, format!("Bearer {}", token))
        .header(CONTENT_TYPE, "application/json")
        .json(&alexa_event) 
        .send()?;

    if response.status().is_success() {
        let response_body: Value = response.json()?;
        debug!("Response: {:?}", response_body);
        Ok(response_body)
    } else {
        let error_type = match response.status().as_u16() {
            401 | 403 => "INVALID_AUTHORIZATION_CREDENTIAL",
            _ => "INTERNAL_ERROR",
        };
        let error_message = response.text().unwrap_or_else(|_| "Unknown error".to_string());
        Ok(json!({
            "event": {
                "payload": {
                    "type": error_type,
                    "message": error_message
                }
            }
        }))
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let func = service_fn(lambda_handler);
    lambda_runtime::run(func).await
}
