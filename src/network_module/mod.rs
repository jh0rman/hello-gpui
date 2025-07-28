// Network layer — executes HTTP requests synchronously in a background thread.
// Called via std::thread::spawn + futures::channel::oneshot from the UI layer.

use std::time::Instant;

pub struct HttpRequest {
    pub method: String,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: Option<String>,
}

pub struct HttpResponse {
    pub status: u16,
    pub duration_ms: u128,
    pub body: String,
}

pub fn execute(req: HttpRequest) -> Result<HttpResponse, String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| e.to_string())?;

    let method = reqwest::Method::from_bytes(req.method.as_bytes())
        .map_err(|e| e.to_string())?;

    let mut builder = client.request(method, &req.url);

    for (k, v) in req.headers {
        builder = builder.header(k, v);
    }

    if let Some(body) = req.body {
        if !body.trim().is_empty() {
            builder = builder
                .header("content-type", "application/json")
                .body(body);
        }
    }

    let start = Instant::now();
    let response = builder.send().map_err(|e| e.to_string())?;
    let duration_ms = start.elapsed().as_millis();

    let status = response.status().as_u16();
    let body = response.text().map_err(|e| e.to_string())?;

    Ok(HttpResponse { status, duration_ms, body })
}
