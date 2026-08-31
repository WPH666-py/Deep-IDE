use serde_json::json;

/// 解析 https://github.com/owner/repo(.git) 地址
fn parse_github_repo(url: &str) -> Result<(String, String), String> {
    let clean = url.trim().trim_end_matches('/').trim_end_matches(".git");
    if !(clean.starts_with("https://github.com/") || clean.starts_with("http://github.com/")) {
        return Err("仅支持 GitHub 仓库地址，例如 https://github.com/owner/repo".to_string());
    }
    let parts: Vec<&str> = clean.split('/').filter(|s| !s.is_empty()).collect();
    if parts.len() < 5 {
        return Err("地址格式错误，应为 https://github.com/owner/repo".to_string());
    }
    let owner = parts[3];
    let repo = parts[4];
    if owner.is_empty() || repo.is_empty() {
        return Err("地址格式错误，应为 https://github.com/owner/repo".to_string());
    }
    Ok((owner.to_string(), repo.to_string()))
}

/// 从 GitHub 仓库下载皮肤 CSS（"转换并添加"）
/// 依次尝试 theme-dark.css / theme-light.css / theme.css / skin.css / style.css / main.css
#[tauri::command]
pub async fn fetch_github_skin(repo_url: String) -> Result<serde_json::Value, String> {
    let (owner, repo) = parse_github_repo(&repo_url)?;

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(20))
        .user_agent("Deep-IDE-skin-fetcher")
        .build()
        .map_err(|e| e.to_string())?;

    let candidates = [
        "theme-dark.css", "theme-light.css",
        "theme.css", "skin.css", "style.css", "main.css",
    ];
    let branches = ["main", "master"];

    for branch in &branches {
        for file in &candidates {
            let url = format!(
                "https://raw.githubusercontent.com/{}/{}/{}/{}",
                owner, repo, branch, file
            );
            let resp = match client.get(&url).send().await {
                Ok(r) => r,
                Err(_) => continue,
            };
            if !resp.status().is_success() {
                continue;
            }
            match resp.text().await {
                Ok(body) => {
                    let body = body.trim().to_string();
                    // 简单校验：足够长且包含花括号，基本可判定为 CSS
                    if body.len() > 50 && body.contains('{') {
                        return Ok(json!({
                            "repo": format!("{}/{}", owner, repo),
                            "file": file,
                            "css": body,
                        }));
                    }
                }
                Err(_) => continue,
            }
        }
    }

    Err(format!(
        "仓库 {}/{} 中未找到可用的皮肤 CSS（尝试了 theme.css / skin.css / style.css / main.css，分支 main/master）",
        owner, repo
    ))
}
