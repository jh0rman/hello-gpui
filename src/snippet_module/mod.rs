// Code snippet generators — pure functions, no dependencies beyond std.

#[derive(Clone, Copy)]
pub enum SnippetLang {
    Curl,
    Fetch,
    Reqwest,
}

pub fn generate(
    lang: SnippetLang,
    method: &str,
    url: &str,
    headers: &[(String, String)],
    body: Option<&str>,
) -> (String, String) {
    match lang {
        SnippetLang::Curl => ("cURL".to_string(), to_curl(method, url, headers, body)),
        SnippetLang::Fetch => ("JavaScript (fetch)".to_string(), to_fetch(method, url, headers, body)),
        SnippetLang::Reqwest => ("Rust (reqwest)".to_string(), to_reqwest(method, url, headers, body)),
    }
}

fn to_curl(method: &str, url: &str, headers: &[(String, String)], body: Option<&str>) -> String {
    let mut parts = vec![format!("curl -X {} \\", method)];
    parts.push(format!("  '{}' \\", url));
    for (k, v) in headers {
        parts.push(format!("  -H '{}: {}' \\", k, v));
    }
    if let Some(b) = body {
        let escaped = b.replace('\'', "'\\''");
        parts.push(format!("  -d '{}'", escaped));
    } else {
        // Remove trailing backslash from last line
        if let Some(last) = parts.last_mut() {
            *last = last.trim_end_matches(" \\").to_string();
        }
    }
    parts.join("\n")
}

fn to_fetch(method: &str, url: &str, headers: &[(String, String)], body: Option<&str>) -> String {
    let mut lines = vec![];
    lines.push(format!("const response = await fetch('{}', {{", url));
    lines.push(format!("  method: '{}',", method));

    if !headers.is_empty() {
        lines.push("  headers: {".to_string());
        for (k, v) in headers {
            lines.push(format!("    '{}': '{}',", k, v));
        }
        lines.push("  },".to_string());
    }

    if let Some(b) = body {
        let escaped = b.replace('`', "\\`");
        lines.push(format!("  body: `{}`,", escaped));
    }

    lines.push("});".to_string());
    lines.push(String::new());
    lines.push("const data = await response.json();".to_string());
    lines.push("console.log(data);".to_string());
    lines.join("\n")
}

fn to_reqwest(method: &str, url: &str, headers: &[(String, String)], body: Option<&str>) -> String {
    let method_lower = method.to_lowercase();
    let mut lines = vec![];
    lines.push("let client = reqwest::Client::new();".to_string());
    lines.push(format!("let response = client.{}(\"{}\")", method_lower, url));

    for (k, v) in headers {
        lines.push(format!("    .header(\"{}\", \"{}\")", k, v));
    }

    if let Some(b) = body {
        let escaped = b.replace('"', "\\\"");
        lines.push(format!("    .body(\"{}\")", escaped));
    }

    lines.push("    .send()".to_string());
    lines.push("    .await?;".to_string());
    lines.push(String::new());
    lines.push("let text = response.text().await?;".to_string());
    lines.push("println!(\"{}\", text);".to_string());
    lines.join("\n")
}
