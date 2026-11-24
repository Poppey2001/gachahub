//! Kuro Games launcher distribution (Wuthering Waves).
//!
//! The official launcher reads an `index.json` ("game config"). It comes in two
//! shapes:
//! - **channel**: `default.{version, cdnList, config.{indexFile, baseUrl}}`;
//! - **bundles** (current Wuthering Waves): top-level `cdnList`, `bundles.<Q>.resourcePacks`
//!   naming packs per resource quality (SD/HD/UHD), and `resourcePacks.<pack>.{version, indexFile, baseUrl}`.
//!
//! Every pack's `indexFile` lists the game files (`resource[].{dest, md5, size}`),
//! downloaded from `<cdn>/<baseUrl>/<dest>`. GachaHub downloads, updates
//! (changed files only) and verifies installations file by file.

use serde_json::Value;
use std::collections::HashSet;

/// Game config of the global Wuthering Waves client (current URL first).
const WUWA_INDEX_URLS: &[&str] = &[
    "https://prod-alicdn-gamestarter.kurogame.com/launcher/game/50004_P7xcUZnEr1AXIGON25E6KjpOgTlVrg6e/G153/official/index.json",
    "https://prod-alicdn-gamestarter.kurogame.com/launcher/game/G153/50004_obOHXFrFanqsaIEOmuKroCcbZkQRBC7c/index.json",
];
/// Extra download mirror used by the official launcher.
const WUWA_EXTRA_CDNS: &[&str] = &["https://hw-pcdownload-aliyun.aki-game.net"];
/// Resource quality GachaHub installs (the official launcher's default).
pub const DEFAULT_QUALITY: &str = "HD";

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

/// Extra launch arguments the official launcher passes (resource quality).
pub fn launch_args(game_id: &str) -> Vec<String> {
    match game_id {
        "wuwa" => vec![format!("-krqlv={}", DEFAULT_QUALITY.to_ascii_lowercase())],
        _ => Vec::new(),
    }
}

fn index_urls(game_id: &str) -> &'static [&'static str] {
    match game_id {
        "wuwa" => WUWA_INDEX_URLS,
        _ => &[],
    }
}

fn extra_cdns(game_id: &str) -> &'static [&'static str] {
    match game_id {
        "wuwa" => WUWA_EXTRA_CDNS,
        _ => &[],
    }
}

fn num(v: Option<&Value>) -> u64 {
    match v {
        Some(Value::Number(n)) => n.as_u64().unwrap_or(0),
        Some(Value::String(x)) => x.trim().parse().unwrap_or(0),
        _ => 0,
    }
}

fn text(v: Option<&Value>) -> Option<String> {
    match v? {
        Value::String(s) if !s.trim().is_empty() => Some(s.trim().to_string()),
        Value::Number(n) => Some(n.to_string()),
        _ => None,
    }
}

fn join_url(base: &str, path: &str) -> String {
    format!(
        "{}/{}",
        base.trim_end_matches('/'),
        path.trim_start_matches('/')
    )
}

/// One file list to download: `<cdn>/<index_file>` describes files below `<cdn>/<base>`.
#[derive(Debug, Clone, PartialEq)]
pub struct Part {
    pub index_file: String,
    pub base: String,
}

/// What a game config resolves to.
#[derive(Debug, Clone, PartialEq)]
pub struct KuroIndex {
    pub version: String,
    /// CDN hosts, best first.
    pub cdns: Vec<String>,
    pub parts: Vec<Part>,
}

/// CDN urls of a `cdnList`, highest priority ("P") first, without duplicates.
fn cdn_list(v: &Value) -> Vec<String> {
    let mut list: Vec<(u64, String)> = v
        .get("cdnList")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|c| Some((num(c.get("P")), text(c.get("url"))?)))
        .collect();
    list.sort_by_key(|(p, _)| std::cmp::Reverse(*p));
    let mut out: Vec<String> = Vec::new();
    for (_, url) in list {
        if !out.contains(&url) {
            out.push(url);
        }
    }
    out
}

pub fn is_bundle_config(root: &Value) -> bool {
    root.get("bundles").is_some_and(Value::is_object)
        && root.get("resourcePacks").is_some_and(Value::is_object)
}

/// Parse a game config. `quality` picks the bundle (SD/HD/UHD) in bundle configs.
pub fn parse_index(root: &Value, quality: &str) -> Result<KuroIndex, String> {
    if is_bundle_config(root) {
        let quality = quality.to_ascii_uppercase();
        let bundles = &root["bundles"];
        let bundle = bundles
            .get(&quality)
            .or_else(|| bundles.as_object().and_then(|m| m.values().next()))
            .ok_or("launcher config: no resource bundle")?;
        let packs: Vec<&str> = bundle["resourcePacks"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .collect();
        if packs.is_empty() {
            return Err(format!("launcher config: {quality} bundle lists no packs"));
        }
        let mut version = String::new();
        let mut parts = Vec::new();
        for name in packs {
            let pack = &root["resourcePacks"][name];
            let index_file = text(pack.get("indexFile"))
                .ok_or_else(|| format!("launcher config: pack {name} has no file list"))?;
            if version.is_empty() {
                version = text(pack.get("version")).unwrap_or_default();
            }
            parts.push(Part {
                index_file,
                base: text(pack.get("baseUrl")).unwrap_or_default(),
            });
        }
        let mut cdns = cdn_list(root);
        if cdns.is_empty() {
            cdns = cdn_list(&root["default"]);
        }
        if cdns.is_empty() {
            return Err("launcher config: cdnList missing".into());
        }
        if version.is_empty() {
            return Err("launcher config: version missing".into());
        }
        return Ok(KuroIndex {
            version,
            cdns,
            parts,
        });
    }
    let d = root
        .get("default")
        .ok_or("launcher config: 'default' section missing")?;
    let version = text(d.get("version"))
        .or_else(|| text(d.pointer("/config/version")))
        .ok_or("launcher config: version missing")?;
    let cdns = cdn_list(d);
    if cdns.is_empty() {
        return Err("launcher config: cdnList missing".into());
    }
    let index_file = text(d.pointer("/config/indexFile"))
        .or_else(|| text(d.get("resources")))
        .ok_or("launcher config: file list missing")?;
    let base = text(d.pointer("/config/baseUrl"))
        .or_else(|| text(d.get("resourcesBasePath")))
        .unwrap_or_default();
    Ok(KuroIndex {
        version,
        cdns,
        parts: vec![Part { index_file, base }],
    })
}

/// Files of one pack index, below `base_url`.
pub fn parse_resources(root: &Value, base_url: &str) -> Result<Vec<KuroFile>, String> {
    let list = root
        .get("resource")
        .or_else(|| root.get("resources"))
        .and_then(Value::as_array)
        .ok_or("file list: 'resource' missing")?;
    let files: Vec<KuroFile> = list
        .iter()
        .filter_map(|r| {
            let dest = r
                .get("dest")
                .and_then(Value::as_str)?
                .trim()
                .trim_start_matches(['/', '\\'])
                .replace('\\', "/");
            if dest.is_empty() || dest.split('/').any(|c| c == "..") {
                return None;
            }
            Some(KuroFile {
                url: join_url(base_url, &dest),
                size: num(r.get("size")),
                md5: r
                    .get("md5")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .trim()
                    .to_ascii_lowercase(),
                dest,
            })
        })
        .collect();
    if files.is_empty() {
        return Err("file list is empty".into());
    }
    Ok(files)
}

/// Kuro's CDN serves some JSON gzip-compressed without a Content-Encoding header.
pub fn decode_json(bytes: &[u8], url: &str) -> Result<Value, String> {
    let plain;
    let bytes = if bytes.starts_with(&[0x1f, 0x8b]) {
        let mut out = Vec::new();
        std::io::Read::read_to_end(&mut flate2::read::GzDecoder::new(bytes), &mut out)
            .map_err(|e| format!("cannot unpack {url}: {e}"))?;
        plain = out;
        &plain[..]
    } else {
        bytes
    };
    serde_json::from_slice(bytes).map_err(|e| format!("invalid JSON from {url}: {e}"))
}

async fn get_json(client: &reqwest::Client, url: &str) -> Result<Value, String> {
    let bytes = client
        .get(url)
        .send()
        .await
        .and_then(|r| r.error_for_status())
        .map_err(|e| format!("request failed ({url}): {e}"))?
        .bytes()
        .await
        .map_err(|e| format!("cannot read {url}: {e}"))?;
    decode_json(&bytes, url)
}

pub async fn fetch_index(client: &reqwest::Client, game_id: &str) -> Result<KuroIndex, String> {
    let mut last_err = format!("no Kuro launcher config known for {game_id}");
    for url in index_urls(game_id) {
        match get_json(client, url)
            .await
            .and_then(|v| parse_index(&v, DEFAULT_QUALITY))
        {
            Ok(mut index) => {
                for extra in extra_cdns(game_id) {
                    if !index.cdns.iter().any(|c| c == extra) {
                        index.cdns.push(extra.to_string());
                    }
                }
                return Ok(index);
            }
            Err(e) => last_err = e,
        }
    }
    Err(last_err)
}

pub async fn fetch_build(client: &reqwest::Client, game_id: &str) -> Result<KuroBuild, String> {
    let index = fetch_index(client, game_id).await?;
    let mut files = Vec::new();
    let mut seen = HashSet::new();
    for part in &index.parts {
        // try the CDNs in order; the first that answers serves the files
        let mut result = Err(String::from("no CDN"));
        for cdn in &index.cdns {
            result = get_json(client, &join_url(cdn, &part.index_file))
                .await
                .and_then(|list| parse_resources(&list, &join_url(cdn, &part.base)));
            if result.is_ok() {
                break;
            }
        }
        for f in result? {
            if seen.insert(f.dest.to_ascii_lowercase()) {
                files.push(f);
            }
        }
    }
    Ok(KuroBuild {
        version: index.version,
        files,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_channel_config_and_resources() {
        let index: Value = serde_json::from_str(
            r#"{"default":{"version":"2.7.0","cdnList":[{"url":"https://slow.example/","P":10},{"url":"https://fast.example","P":100}],
               "config":{"indexFile":"launcher/x/2.7.0/indexFile.json","baseUrl":"launcher/x/2.7.0/resource/"}}}"#,
        )
        .unwrap();
        let idx = parse_index(&index, "HD").unwrap();
        assert_eq!(idx.version, "2.7.0");
        assert_eq!(
            idx.cdns,
            vec!["https://fast.example", "https://slow.example/"]
        );
        let base = join_url(&idx.cdns[0], &idx.parts[0].base);
        let res: Value = serde_json::from_str(
            r#"{"resource":[{"dest":"/Client/Binaries/Win64/Client-Win64-Shipping.exe","md5":"ABC","size":"42"},{"dest":""},{"dest":"../evil"}]}"#,
        )
        .unwrap();
        let files = parse_resources(&res, &base).unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(
            files[0].url,
            "https://fast.example/launcher/x/2.7.0/resource/Client/Binaries/Win64/Client-Win64-Shipping.exe"
        );
        assert_eq!(files[0].size, 42);
        assert_eq!(files[0].md5, "abc");
        assert!(parse_index(&serde_json::json!({}), "HD").is_err());
    }

    #[test]
    fn parses_bundle_config() {
        let cfg: Value = serde_json::from_str(
            r#"{"cdnList":[{"url":"https://cdn.example/","P":5}],
                "bundles":{"SD":{"resourcePacks":["common","sd"]},"HD":{"resourcePacks":["common","hd"]}},
                "resourcePacks":{
                  "common":{"version":"3.7.0","indexFile":"launcher/G153/3.7.0/common/indexFile.json","baseUrl":"launcher/G153/3.7.0/zip/"},
                  "sd":{"version":"3.7.0","indexFile":"launcher/G153/3.7.0/sd/indexFile.json","baseUrl":"launcher/G153/3.7.0/zip/"},
                  "hd":{"version":"3.7.0","indexFile":"launcher/G153/3.7.0/hd/indexFile.json","baseUrl":"launcher/G153/3.7.0/zip/"}}}"#,
        )
        .unwrap();
        assert!(is_bundle_config(&cfg));
        let idx = parse_index(&cfg, "hd").unwrap();
        assert_eq!(idx.version, "3.7.0");
        assert_eq!(idx.parts.len(), 2);
        assert_eq!(
            idx.parts[1].index_file,
            "launcher/G153/3.7.0/hd/indexFile.json"
        );
        assert_eq!(idx.cdns, vec!["https://cdn.example/"]);
        assert_eq!(launch_args("wuwa"), vec!["-krqlv=hd"]);
        // gzip without Content-Encoding
        let mut gz = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
        std::io::Write::write_all(&mut gz, br#"{"a":1}"#).unwrap();
        let v = decode_json(&gz.finish().unwrap(), "x").unwrap();
        assert_eq!(v["a"], 1);
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
                    let bytes = r.bytes().await.unwrap_or_default();
                    println!("LIVE wuwa {url}: HTTP {status}, {} bytes", bytes.len());
                    let text = super::decode_json(&bytes, url)
                        .map(|v| v.to_string())
                        .unwrap_or_else(|e| e);
                    println!(
                        "LIVE wuwa body: {}",
                        &text.chars().take(2500).collect::<String>()
                    );
                }
                Err(e) => println!("LIVE wuwa {url}: {e}"),
            }
        }
        let index = super::fetch_index(&client, "wuwa").await.unwrap();
        println!("LIVE wuwa index: {index:?}");
        let build = super::fetch_build(&client, "wuwa").await.unwrap();
        let total: u64 = build.files.iter().map(|f| f.size).sum();
        println!(
            "LIVE wuwa: version {} with {} files, {:.1} GB; first {:?}",
            build.version,
            build.files.len(),
            total as f64 / 1e9,
            build.files.first()
        );
        assert!(!build.files.is_empty());
        let exe = build
            .files
            .iter()
            .find(|f| f.dest.ends_with(".exe"))
            .expect("an exe in the file list");
        let head = client.head(&exe.url).send().await.unwrap();
        println!(
            "LIVE wuwa: {} → HTTP {} {:?}",
            exe.url,
            head.status(),
            head.content_length()
        );
        assert!(head.status().is_success());
    }
}
