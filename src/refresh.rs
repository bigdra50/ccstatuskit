//! `ccstatuskit --refresh-usage`: refreshes the scoped-usage SWR cache in a
//! detached process so renders never wait on the network.
//!
//! Talks to the unofficial `api.anthropic.com/api/oauth/usage` endpoint —
//! the same one Claude Code itself uses — which may break without notice.
//! A mkdir-based lock (stale after 60s) prevents refresh stampedes across
//! concurrent sessions.

pub fn run() {
    #[cfg(feature = "scoped-usage")]
    run_impl();
}

#[cfg(feature = "scoped-usage")]
fn run_impl() {
    use crate::config::{Config, default_cache_dir};

    let config = Config::load();
    let Some(dir) = config.usage.cache_dir.clone().or_else(default_cache_dir) else {
        return;
    };
    let _ = std::fs::create_dir_all(&dir);

    let lock = dir.join("refresh.lock.d");
    if std::fs::create_dir(&lock).is_err() {
        let stale = std::fs::metadata(&lock)
            .ok()
            .and_then(|m| m.modified().ok())
            .and_then(|t| std::time::SystemTime::now().duration_since(t).ok())
            .is_some_and(|age| age.as_secs() > 60);
        if !stale {
            return;
        }
        let _ = std::fs::remove_dir(&lock);
        if std::fs::create_dir(&lock).is_err() {
            return;
        }
    }

    if let Some(body) = fetch() {
        let tmp = dir.join("usage.json.tmp");
        if std::fs::write(&tmp, &body).is_ok() {
            let _ = std::fs::rename(&tmp, dir.join("usage.json"));
        }
    }
    let _ = std::fs::remove_dir(&lock);
}

#[cfg(feature = "scoped-usage")]
fn fetch() -> Option<String> {
    let token = crate::auth::read_token()?;
    let version = std::env::var("CCSTATUSKIT_CC_VERSION").unwrap_or_else(|_| "2.0.0".to_string());
    let body = ureq::get("https://api.anthropic.com/api/oauth/usage")
        .set("Authorization", &format!("Bearer {token}"))
        .set("anthropic-beta", "oauth-2025-04-20")
        .set("User-Agent", &format!("claude-code/{version}"))
        .set("Content-Type", "application/json")
        .timeout(std::time::Duration::from_secs(6))
        .call()
        .ok()?
        .into_string()
        .ok()?;
    // Only persist plausible payloads; never persist error bodies.
    let doc: serde_json::Value = serde_json::from_str(&body).ok()?;
    doc.get("limits")?;
    Some(body)
}
