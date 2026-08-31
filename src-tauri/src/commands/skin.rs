use serde_json::json;

/// 解析 https://github.com/owner/repo(.git) 地址
fn parse_github_repo(url: &str) -> Result<(String, String), String> {
    let clean = url.trim().trim_end_matches('/').trim_end_matches(".git");
    let rest = clean
        .strip_prefix("https://github.com/")
        .or_else(|| clean.strip_prefix("http://github.com/"))
        .or_else(|| clean.strip_prefix("github.com/"))
        .ok_or_else(|| "仅支持 GitHub 仓库地址，例如 https://github.com/owner/repo".to_string())?;
    let parts: Vec<&str> = rest.split('/').filter(|s| !s.is_empty()).collect();
    if parts.len() < 2 {
        return Err("地址格式错误，应为 https://github.com/owner/repo".to_string());
    }
    let owner = parts[0];
    let repo = parts[1];
    if owner.is_empty() || repo.is_empty() {
        return Err("地址格式错误，应为 https://github.com/owner/repo".to_string());
    }
    Ok((owner.to_string(), repo.to_string()))
}

/// 从 GitHub 仓库下载皮肤（"转换并添加"）
/// 先读 skin.json（DeepKing 皮肤规范：名称/标语），
/// 再按候选路径找 DeepKing 调色板 CSS（:root 变量 + .dark 变量）
/// 失败时回退尝试 theme.css / skin.css / style.css / main.css 等普通 CSS
#[tauri::command]
pub async fn fetch_github_skin(repo_url: String) -> Result<serde_json::Value, String> {
    let (owner, repo) = parse_github_repo(&repo_url)?;

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(25))
        .user_agent("Deep-IDE-skin-fetcher")
        .build()
        .map_err(|e| e.to_string())?;

    let branches = ["main", "master"];

    // 1) skin.json 描述（name/tagline）
    let mut name: Option<String> = None;
    let mut tagline: Option<String> = None;
    for branch in &branches {
        let url = format!(
            "https://raw.githubusercontent.com/{}/{}/{}/skin.json",
            owner, repo, branch
        );
        if let Ok(resp) = client.get(&url).send().await {
            if let Ok(body) = resp.text().await {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&body) {
                    if let Some(s) = v.get("name").and_then(|x| x.as_str()) {
                        name = Some(s.to_string());
                    }
                    if let Some(s) = v.get("tagline").and_then(|x| x.as_str()) {
                        tagline = Some(s.to_string());
                    }
                    break;
                }
            }
        }
    }

    // 2) DeepKing 调色板 CSS（含 --bg-base 等变量）
    let palette_candidates = [
        "src/client/deepking-skin.module.css",
        "deepking-skin.module.css",
        "skin.css",
        "theme-dark.css",
        "theme-light.css",
        "theme.css",
        "style.css",
        "main.css",
    ];
    for branch in &branches {
        for file in &palette_candidates {
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
                    if body.len() > 30 && body.contains('{') {
                        let is_palette = body.contains("--bg-base") && body.contains(":root");
                        return Ok(json!({
                            "repo": format!("{}/{}", owner, repo),
                            "file": file,
                            "name": name.clone(),
                            "tagline": tagline.clone(),
                            "palette": is_palette,
                            "css": body,
                        }));
                    }
                }
                Err(_) => continue,
            }
        }
    }

    Err(format!(
        "仓库 {}/{} 中未找到可用的皮肤 CSS（DeepKing 调色板或 theme.css / skin.css / style.css / main.css）",
        owner, repo
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_various_repo_urls() {
        assert_eq!(
            parse_github_repo("https://github.com/WPH666-py/Genshen-Furina-Skin").unwrap(),
            ("WPH666-py".to_string(), "Genshen-Furina-Skin".to_string())
        );
        assert_eq!(
            parse_github_repo("https://github.com/owner/repo.git").unwrap(),
            ("owner".to_string(), "repo".to_string())
        );
        assert_eq!(
            parse_github_repo(" https://github.com/a/b/ ").unwrap(),
            ("a".to_string(), "b".to_string())
        );
        assert_eq!(
            parse_github_repo("github.com/a/b").unwrap(),
            ("a".to_string(), "b".to_string())
        );
        assert!(parse_github_repo("https://example.com/a/b").is_err());
        assert!(parse_github_repo("https://github.com/solo").is_err());
        assert!(parse_github_repo("not a url").is_err());
    }
}
