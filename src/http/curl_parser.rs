use crate::http::models::{KeyValue, Method, RequestModel};

/// Parse a cURL command string into a RequestModel
pub fn parse_curl(input: &str) -> Option<RequestModel> {
    let normalized = input
        .replace("\\\n", " ")
        .replace("\\\r\n", " ");

    let tokens = shell_split(&normalized);
    if tokens.is_empty() || tokens[0] != "curl" {
        return None;
    }

    let mut method = Method::Get;
    let mut url = String::new();
    let mut headers: Vec<KeyValue> = Vec::new();
    let mut body = String::new();

    let mut i = 1;
    while i < tokens.len() {
        match tokens[i].as_str() {
            "-X" | "--request" => {
                i += 1;
                if i < tokens.len() {
                    method = match tokens[i].to_uppercase().as_str() {
                        "POST" => Method::Post,
                        "PUT" => Method::Put,
                        "DELETE" => Method::Delete,
                        "PATCH" => Method::Patch,
                        _ => Method::Get,
                    };
                }
            }
            "-H" | "--header" => {
                i += 1;
                if i < tokens.len() {
                    let header = unquote(&tokens[i]);
                    if let Some(colon_pos) = header.find(':') {
                        headers.push(KeyValue {
                            key: header[..colon_pos].trim().to_string(),
                            value: header[colon_pos + 1..].trim().to_string(),
                        });
                    }
                }
            }
            "-d" | "--data" | "--data-raw" => {
                i += 1;
                if i < tokens.len() {
                    body = unquote(&tokens[i]);
                    if method == Method::Get {
                        method = Method::Post;
                    }
                }
            }
            arg if !arg.starts_with('-') => {
                url = unquote(arg);
            }
            _ => {}
        }
        i += 1;
    }

    if headers.is_empty() {
        headers.push(KeyValue { key: String::new(), value: String::new() });
    }

    // Extract query params from URL
    let (clean_url, params) = extract_params(&url);

    Some(RequestModel {
        method,
        url: clean_url,
        headers,
        body,
        params,
    })
}

/// Generate cURL text from a RequestModel
pub fn to_curl(req: &RequestModel) -> String {
    let mut parts: Vec<String> = Vec::new();

    parts.push("curl".to_string());

    if req.method != Method::Get {
        parts.push(format!("-X {}", req.method.as_str()));
    }

    // URL with params
    let url = build_url_with_params(req);
    parts.push(format!("'{}'", url));

    for kv in &req.headers {
        if !kv.key.is_empty() {
            parts.push(format!("-H '{}: {}'", kv.key, kv.value));
        }
    }

    if !req.body.is_empty() {
        let escaped = req.body.replace('\'', "'\\''");
        parts.push(format!("-d '{}'", escaped));
    }

    parts.join(" \\\n  ")
}

fn build_url_with_params(req: &RequestModel) -> String {
    let params: Vec<_> = req.params.iter()
        .filter(|kv| !kv.key.is_empty())
        .map(|kv| format!("{}={}", kv.key, kv.value))
        .collect();

    if params.is_empty() {
        req.url.clone()
    } else {
        let sep = if req.url.contains('?') { "&" } else { "?" };
        format!("{}{}{}", req.url, sep, params.join("&"))
    }
}

fn extract_params(url: &str) -> (String, Vec<KeyValue>) {
    if let Some(q_pos) = url.find('?') {
        let base = url[..q_pos].to_string();
        let query = &url[q_pos + 1..];
        let params: Vec<KeyValue> = query.split('&')
            .map(|pair| {
                if let Some(eq_pos) = pair.find('=') {
                    KeyValue {
                        key: pair[..eq_pos].to_string(),
                        value: pair[eq_pos + 1..].to_string(),
                    }
                } else {
                    KeyValue { key: pair.to_string(), value: String::new() }
                }
            })
            .collect();
        (base, params)
    } else {
        let mut params = Vec::new();
        params.push(KeyValue { key: String::new(), value: String::new() });
        (url.to_string(), params)
    }
}

fn unquote(s: &str) -> String {
    let s = s.trim();
    if (s.starts_with('\'') && s.ends_with('\'')) || (s.starts_with('"') && s.ends_with('"')) {
        s[1..s.len() - 1].to_string()
    } else {
        s.to_string()
    }
}

/// Simple shell-like splitting (handles single and double quotes)
fn shell_split(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if in_single_quote {
            if c == '\'' {
                in_single_quote = false;
            } else {
                current.push(c);
            }
        } else if in_double_quote {
            if c == '"' {
                in_double_quote = false;
            } else if c == '\\' {
                if let Some(&next) = chars.peek() {
                    if next == '"' || next == '\\' {
                        current.push(chars.next().unwrap());
                    } else {
                        current.push(c);
                    }
                }
            } else {
                current.push(c);
            }
        } else {
            match c {
                '\'' => in_single_quote = true,
                '"' => in_double_quote = true,
                ' ' | '\t' => {
                    if !current.is_empty() {
                        tokens.push(current.clone());
                        current.clear();
                    }
                }
                '\\' => {
                    if let Some(&next) = chars.peek() {
                        if next == '\n' {
                            chars.next();
                        } else {
                            current.push(chars.next().unwrap());
                        }
                    }
                }
                _ => current.push(c),
            }
        }
    }

    if !current.is_empty() {
        tokens.push(current);
    }

    tokens
}
