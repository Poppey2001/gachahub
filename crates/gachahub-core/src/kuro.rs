//! Kuro Games launcher distribution (Wuthering Waves, Punishing: Gray Raven).
//!
//! The official launcher reads an `index.json` that names the current game
//! version, CDN hosts and a resource index. The resource index lists every game
//! file with its size and MD5, so GachaHub can download, update (changed files
//! only) and verify an installation file by file.
//!
//! The endpoint layout is based on the public launcher behaviour and is parsed
//! defensively; unexpected responses produce a clear error instead of a guess.

use serde_json::Value;

/// Launcher index of the global Wuthering Waves client.
const WUWA_INDEX_URLS: &[&str] = &[
    "https://prod-alicdn-gamestarter.kurogame.com/launcher/game/G153/50004_obOHXFrFanqsaIEOmuKroCcbZkQRBC7c/index.json",
    "https://prod-awscdn-gamestarter.kurogame.net/launcher/game/G153/50004_obOHXFrFanqsaIEOmuKroCcbZkQRBC7c/index.json",
];

#[derive(Debug, Clone, PartialEq)]
pub struct KuroFile {
    /// Path relative to the install folder, `/`-separated.
    pub dest: String,
    pub url: String,
    pub size: u64,
    pub md5: String,
}

#[derive(Debug, Clone)]
pub struct KuroBuild {
    pub version: String,
    pub files: Vec<KuroFile>,
}

pub fn supports(game_id: &str) -> bool {
    game_id == "wuwa"
}

fn index_urls(game_id: &str) -> &'static [&'static str] {
    match game_id {
        "wuwa" => WUWA_INDEX_URLS,
        _ => &[],
    }
}

fn s<'a>(v: &'a Value, path: &str) -> Option<&'a str> {
    v.pointer(path)
        .and_then(Value::as_str)
        .filter(|x| !x.is_empty())
}

fn num(v: Option<&Value>) -> u64 {
    match v {
        Some(Value::Number(n)) => n.as_u64().unwrap_or(0),
        Some(Value::String(x)) => x.parse().unwrap_or(0),
        _ => 0,
    }
}

fn join_url(base: &str, path: &str) -> String {
    format!(
        "{}/{}",
        base.trim_end_matches('/'),
        path.trim_start_matches('/')
    )
}

/// Information extracted from `index.json`.
#[derive(Debug, Clone, PartialEq)]
pub struct KuroIndex {
    pub version: String,
    pub cdn: String,
    pub resource_index_path: String,
    pub resources_base_path: String,
}

pub fn parse_index(root: &Value) -> Result<KuroIndex, String> {
    let d = root
        .get("default")
        .ok_or("launcher index: 'default' section missing")?;
    let version = s(d, "/version")
        .or_else(|| s(d, "/config/version"))
        .ok_or("launcher index: version missing")?;
    // CDN with the highest priority value ("P")
    let cdn = d
        .get("cdnList")
        .and_then(Value::as_array)
        .and_then(|list| {
            list.iter()
                .filter(|c| c.get("url").and_then(Value::as_str).is_some())
                .max_by_key(|c| num(c.get("P")))
                .and_then(|c| c.get("url").and_then(Value::as_str))
        })
        .ok_or("launcher index: cdnList missing")?;
    let resource_index_path = s(d, "/config/indexFile")
        .or_else(|| s(d, "/resources"))
        .ok_or("launcher index: resource index path missing")?;
    let resources_base_path = s(d, "/resourcesBasePath")
        .or_else(|| s(d, "/config/baseUrl"))
        .ok_or("launcher index: resourcesBasePath missing")?;
    Ok(KuroIndex {
        version: version.into(),
        cdn: cdn.into(),
        resource_index_path: resource_index_path.into(),
        resources_base_path: resources_base_path.into(),
    })
}

pub fn parse_resources(root: &Value, index: &KuroIndex) -> Result<Vec<KuroFile>, String> {
    let list = root
        .get("resource")
        .and_then(Value::as_array)
        .ok_or("resource index: 'resource' list missing")?;
    let base = join_url(&index.cdn, &index.resources_base_path);
    let files: Vec<KuroFile> = list
        .iter()
        .filter_map(|r| {
            let dest = r
                .get("dest")
                .and_then(Value::as_str)?
                .trim_start_matches(['/', '\\'])
                .replace('\\', "/");
            if dest.is_empty() {
                return None;
            }
            Some(KuroFile {
                url: join_url(&base, &dest),
                size: num(r.get("size")),
                md5: r
                    .get("md5")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_ascii_lowercase(),
                dest,
            })
        })
        .collect();
    if files.is_empty() {
        return Err("resource index lists no files".into());
    }
    Ok(files)
}

async fn get_json(client: &reqwest::Client, url: &str) -> Result<Value, String> {
    client
        .get(url)
        .send()
        .await
        .and_then(|r| r.error_for_status())
        .map_err(|e| format!("request failed ({url}): {e}"))?
        .json()
        .await
        .map_err(|e| format!("invalid JSON from {url}: {e}"))
}

pub async fn fetch_index(client: &reqwest::Client, game_id: &str) -> Result<KuroIndex, String> {
    let mut last_err = format!("no Kuro launcher index known for {game_id}");
    for url in index_urls(game_id) {
        match get_json(client, url).await.and_then(|v| parse_index(&v)) {
            Ok(index) => return Ok(index),
            Err(e) => last_err = e,
        }
    }
    Err(last_err)
}

pub async fn fetch_build(client: &reqwest::Client, game_id: &str) -> Result<KuroBuild, String> {
    let index = fetch_index(client, game_id).await?;
    let resources = get_json(client, &join_url(&index.cdn, &index.resource_index_path)).await?;
    let files = parse_resources(&resources, &index)?;
    Ok(KuroBuild {
        version: index.version,
        files,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_index_and_resources() {
        let index: Value = serde_json::from_str(
            r#"{"default":{"version":"2.7.0","cdnList":[{"url":"https://slow.example/","P":10},{"url":"https://fast.example","P":100}],
               "config":{"indexFile":"launcher/x/2.7.0/indexFile.json"},"resourcesBasePath":"launcher/x/2.7.0/resource/"}}"#,
        )
        .unwrap();
        let idx = parse_index(&index).unwrap();
        assert_eq!(idx.version, "2.7.0");
        assert_eq!(idx.cdn, "https://fast.example");
        let res: Value = serde_json::from_str(
            r#"{"resource":[{"dest":"/Client/Binaries/Win64/Client-Win64-Shipping.exe","md5":"ABC","size":"42"},{"dest":""}]}"#,
        )
        .unwrap();
        let files = parse_resources(&res, &idx).unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(
            files[0].dest,
            "Client/Binaries/Win64/Client-Win64-Shipping.exe"
        );
        assert_eq!(files[0].url, "https://fast.example/launcher/x/2.7.0/resource/Client/Binaries/Win64/Client-Win64-Shipping.exe");
        assert_eq!(files[0].size, 42);
        assert_eq!(files[0].md5, "abc");
        assert!(parse_index(&serde_json::json!({})).is_err());
    }
}

/// Real-server check: `cargo test -- --ignored live_`.
#[cfg(test)]
mod live {
    #[tokio::test]
    #[ignore]
    async fn live_wuwa_index() {
        let client = reqwest::Client::builder()
            .user_agent(crate::USER_AGENT)
            .build()
            .unwrap();
        for url in super::WUWA_INDEX_URLS {
            match client.get(*url).send().await {
                Ok(r) => {
                    let status = r.status();
                    let text = r.text().await.unwrap_or_default();
                    println!("LIVE wuwa {url}: HTTP {status}, {} bytes", text.len());
                    println!(
                        "LIVE wuwa body: {}",
                        &text.chars().take(3000).collect::<String>()
                    );
                }
                Err(e) => println!("LIVE wuwa {url}: {e}"),
            }
        }
        let build = super::fetch_build(&client, "wuwa").await.unwrap();
        println!(
            "LIVE wuwa: version {} with {} files",
            build.version,
            build.files.len()
        );
        assert!(!build.files.is_empty());
    }
}
