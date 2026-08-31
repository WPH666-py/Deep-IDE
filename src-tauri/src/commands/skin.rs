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
                        // 3) 下载立绘素材（可选）
                        let portrait = fetch_portrait(&client, &owner, &repo, &branches).await;
                        return Ok(json!({
                            "repo": format!("{}/{}", owner, repo),
                            "file": file,
                            "name": name.clone(),
                            "tagline": tagline.clone(),
                            "palette": is_palette,
                            "portrait": portrait,
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

/// 从仓库素材目录中挑选立绘（优先 web 版小图），下载并以 data URL 返回
async fn fetch_portrait(
    client: &reqwest::Client,
    owner: &str,
    repo: &str,
    branches: &[&str],
) -> Option<String> {
    const DIRS: &[&str] = &["素材", "dsh-plugin/素材", "assets", "assets/background", ""];
    const MAX_BYTES: usize = 3 * 1024 * 1024;

    for branch in branches {
        for dir in DIRS {
            let api_url = if dir.is_empty() {
                format!("https://api.github.com/repos/{}/{}/contents?ref={}", owner, repo, branch)
            } else {
                format!(
                    "https://api.github.com/repos/{}/{}/contents/{}?ref={}",
                    owner, repo, dir, branch
                )
            };
            let resp = match client.get(&api_url).send().await {
                Ok(r) if r.status().is_success() => r,
                _ => continue,
            };
            let listing: Vec<serde_json::Value> = match resp.json().await {
                Ok(v) => v,
                Err(_) => continue,
            };

            // 候选优先级：web 版 > 立绘命名 > 通用图片
            let mut candidates: Vec<(usize, String)> = Vec::new();
            for item in &listing {
                if item.get("type").and_then(|t| t.as_str()) != Some("file") {
                    continue;
                }
                let name = item.get("name").and_then(|n| n.as_str()).unwrap_or("");
                let lower = name.to_lowercase();
                if !(lower.ends_with(".png") || lower.ends_with(".jpg") || lower.ends_with(".jpeg") || lower.ends_with(".webp")) {
                    continue;
                }
                if lower.contains("壁纸") || lower.contains("wallpaper") || lower.contains("background") {
                    continue; // 立绘优先，壁纸排除
                }
                let score = if lower.contains("web") { 0 } else if lower.contains("立绘") || lower.contains("girl") { 1 } else { 2 };
                candidates.push((score, name.to_string()));
            }
            candidates.sort_by(|a, b| a.0.cmp(&b.0));
            for (_score, name) in candidates {
                // 用 reqwest::Url 做路径转义（中文文件名）
                let mut url = reqwest::Url::parse("https://raw.githubusercontent.com").ok()?;
                {
                    let mut segs = url.path_segments_mut().ok()?;
                    segs.pop_if_empty();
                    segs.push(owner).push(repo).push(branch);
                    for part in dir.split('/').filter(|s| !s.is_empty()) {
                        segs.push(part);
                    }
                    segs.push(&name);
                }
                let resp = match client.get(url).send().await {
                    Ok(r) if r.status().is_success() => r,
                    _ => continue,
                };
                match resp.bytes().await {
                    Ok(bytes) => {
                        if bytes.len() > MAX_BYTES || bytes.len() < 100 {
                            continue;
                        }
                        let mime = if name.to_lowercase().ends_with(".png") {
                            "image/png"
                        } else if name.to_lowercase().ends_with(".webp") {
                            "image/webp"
                        } else {
                            "image/jpeg"
                        };
                        use base64::Engine as _;
                        let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
                        return Some(format!("data:{};base64,{}", mime, b64));
                    }
                    Err(_) => continue,
                }
            }
        }
    }
    None
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
