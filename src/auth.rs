//! OAuth token lookup for the scoped-usage fetch. The token is read at call
//! time and never logged, cached, or included in any output.

pub fn read_token() -> Option<String> {
    if let Some(token) = token_from_credentials_file() {
        return Some(token);
    }
    #[cfg(target_os = "macos")]
    if let Some(token) = token_from_keychain() {
        return Some(token);
    }
    None
}

fn token_from_credentials_file() -> Option<String> {
    let path = std::env::home_dir()?.join(".claude/.credentials.json");
    let body = std::fs::read_to_string(path).ok()?;
    token_from_json(&body)
}

fn token_from_json(body: &str) -> Option<String> {
    let doc: serde_json::Value = serde_json::from_str(body).ok()?;
    doc.pointer("/claudeAiOauth/accessToken")?
        .as_str()
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
}

#[cfg(target_os = "macos")]
fn token_from_keychain() -> Option<String> {
    let output = std::process::Command::new("security")
        .args([
            "find-generic-password",
            "-s",
            "Claude Code-credentials",
            "-w",
        ])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    token_from_json(&String::from_utf8_lossy(&output.stdout))
}

#[cfg(test)]
mod tests {
    use super::token_from_json;

    #[test]
    fn extracts_access_token() {
        let body = r#"{"claudeAiOauth":{"accessToken":"tok-123","refreshToken":"r"}}"#;
        assert_eq!(token_from_json(body).as_deref(), Some("tok-123"));
    }

    #[test]
    fn missing_or_empty_token_is_none() {
        assert_eq!(token_from_json(r#"{"claudeAiOauth":{}}"#), None);
        assert_eq!(
            token_from_json(r#"{"claudeAiOauth":{"accessToken":""}}"#),
            None
        );
        assert_eq!(token_from_json("not json"), None);
    }
}
