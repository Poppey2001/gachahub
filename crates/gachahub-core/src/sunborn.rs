//! GIRLS' FRONTLINE 2: EXILIUM direct download (Darkwinter global and Haoplay builds).
//!
//! Both PC launchers read a public `flexible_config` document. Its
//! `nexon_plug` section names
//! - the game client (`game_download_addr` / `game_client_version`): one zip
//!   with `GF2_Exilium.exe` and the Unity player, and
//! - the resources (`ab_resource_addr` / `ab_resource_version`): every asset
//!   bundle listed in `AssetBundles_Windows/<bucket>/Version.txt` as
//!   `{0}/<name>#<MD5>#<size>#<bucket>`.
//!
//! GachaHub installs the client zip, downloads the bundles file by file with
//! MD5 checks (resumable, only changed files on update) and writes the
//! `Version.txt` the game reads, keeping lines the game added itself.

use crate::kuro::KuroFile;
use serde_json::Value;
use std::path::Path;

/// Darkwinter (global) launcher config.
pub const DARKWINTER_CONFIG: &str =
    "https://gf2-launcher-us.sunborngame.com/flexible_config?version=1.0.2";
/// Haoplay (Asia) launcher config.
pub const HAOPLAY_CONFIG: &str = "https://gf2-launcher-intl.haoplay.com/flexible_config";

pub const CLIENT_PACKAGE: &str = "GF2_Exilium_Origin.zip";
pub const EXE: &str = "GF2_Exilium.exe";
const AB_REMOTE_DIR: &str = "AssetBundles_Windows";
pub const AB_DIR: &str = "GF2_Exilium_Data/LocalCache/Data/AssetBundles_Windows";
const LOCAL_MANIFEST: &str = "Version.txt";
/// Installed client version (GachaHub marker).
pub const CLIENT_MARKER: &str = ".gachahub-gf2-client";

pub fn supports(game_id: &str) -> bool {
    game_id == "gfl2"
}

pub fn config_url(edition: Option<&str>) -> &'static str {
    match edition {
        Some("haoplay") => HAOPLAY_CONFIG,
        _ => DARKWINTER_CONFIG,
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Plug {
    pub client_version: String,
    pub download_addr: String,
    pub ab_version: String,
    pub ab_addr: String,
}

impl Plug {
    /// Version string GachaHub records: client, then resources ("1.0.3-1.5.27767").
    pub fn version(&self) -> String {
        format!("{}-{}", self.client_version, self.ab_version)
    }
    pub fn client_url(&self) -> String {
        join(
            &self.download_addr,
            &format!("{}/{CLIENT_PACKAGE}", self.client_version),
        )
    }
    /// The bucket is the last component of the resource version.
    pub fn bucket(&self) -> &str {
        self.ab_version
            .rsplit('.')
            .next()
            .unwrap_or(&self.ab_version)
    }
    pub fn manifest_url(&self) -> String {
        join(
            &self.ab_addr,
            &format!("{AB_REMOTE_DIR}/{}/{LOCAL_MANIFEST}", self.bucket()),
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Bundle {
    pub name: String,
    pub md5: String,
    pub size: u64,
    pub bucket: String,
}

impl Bundle {
    pub fn line(&self) -> String {
        format!(
            "{{0}}/{}#{}#{}#{}",
            self.name,
            self.md5.to_uppercase(),
            self.size,
            self.bucket
        )
    }
    /// Voice packs: two upper-case letters + 32 hex digits (optional, the game
    /// downloads the selected language itself).
    pub fn is_voice_pack(&self) -> bool {
        let stem = self
            .name
            .rsplit_once('.')
            .map(|(s, _)| s)
            .unwrap_or(&self.name);
        stem.len() == 34
            && stem[..2].chars().all(|c| c.is_ascii_uppercase())
            && stem[2..].chars().all(|c| c.is_ascii_hexdigit())
    }
    pub fn to_file(&self, ab_addr: &str) -> KuroFile {
        KuroFile {
            dest: format!("{AB_DIR}/{}", self.name),
            url: join(
                ab_addr,
                &format!("{AB_REMOTE_DIR}/{}/{}", self.bucket, self.name),
            ),
            size: self.size,
            md5: self.md5.to_lowercase(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Build {
    pub plug: Plug,
    pub bundles: Vec<Bundle>,
}

impl Build {
    pub fn version(&self) -> String {
        self.plug.version()
    }
    pub fn files(&self) -> Vec<KuroFile> {
        self.bundles
            .iter()
            .filter(|b| !b.is_voice_pack())
            .map(|b| b.to_file(&self.plug.ab_addr))
            .collect()
    }
    pub fn bundle_bytes(&self) -> u64 {
        self.bundles
            .iter()
            .filter(|b| !b.is_voice_pack())
            .map(|b| b.size)
            .sum()
    }
}

fn join(base: &str, path: &str) -> String {
    format!(
        "{}/{}",
        base.trim_end_matches('/'),
        path.trim_start_matches('/')
    )
}

fn plug_str(plug: &Value, key: &str) -> Result<String, String> {
    match plug.get(key) {
        Some(Value::String(s)) if !s.trim().is_empty() => Ok(s.trim().to_string()),
        Some(Value::Number(n)) => Ok(n.to_string()),
        _ => Err(format!("GFL2 launcher config has no {key}")),
    }
}

fn https(url: String, what: &str) -> Result<String, String> {
    if url.starts_with("https://") || url.starts_with("http://127.0.0.1") {
        Ok(url)
    } else {
        Err(format!("GFL2 launcher config: {what} is not an https URL"))
    }
}

/// Parse the `flexible_config` response.
pub fn parse_plug(doc: &Value) -> Result<Plug, String> {
    if let Some(code) = doc.get("code") {
        let ok = code.as_i64() == Some(0) || code.as_str() == Some("0");
        if !ok {
            let msg = doc.get("msg").and_then(Value::as_str).unwrap_or("");
            return Err(format!("GFL2 launcher config error {code}: {msg}"));
        }
    }
    let raw = doc
        .pointer("/data/flexible_config/nexon_plug")
        .ok_or("GFL2 launcher config has no nexon_plug section")?;
    let plug: Value = match raw {
        Value::String(s) => {
            serde_json::from_str(s).map_err(|e| format!("invalid nexon_plug payload: {e}"))?
        }
        Value::Object(_) => raw.clone(),
        _ => return Err("unexpected nexon_plug payload".into()),
    };
    Ok(Plug {
        client_version: plug_str(&plug, "game_client_version")?,
        download_addr: https(plug_str(&plug, "game_download_addr")?, "game_download_addr")?,
        ab_version: plug_str(&plug, "ab_resource_version")?,
        ab_addr: https(plug_str(&plug, "ab_resource_addr")?, "ab_resource_addr")?,
    })
}

/// Parse a `Version.txt` (remote resource list or the install's own copy).
pub fn parse_manifest(text: &str) -> Vec<Bundle> {
    text.lines()
        .filter_map(|line| {
            let mut f = line.trim().split('#');
            let path = f.next()?;
            let md5 = f.next()?.trim();
            let size = f.next()?.trim().parse().ok()?;
            let bucket = f.next()?.trim();
            if md5.len() != 32 || !md5.chars().all(|c| c.is_ascii_hexdigit()) || bucket.is_empty() {
                return None;
            }
            let name = path.rsplit('/').next()?.trim();
            if name.is_empty() || name.contains('\\') || name == ".." || name == "." {
                return None;
            }
            Some(Bundle {
                name: name.to_string(),
                md5: md5.to_lowercase(),
                size,
                bucket: bucket.to_string(),
            })
        })
        .collect()
}

async fn get(client: &reqwest::Client, url: &str) -> Result<String, String> {
    client
        .get(url)
        .send()
        .await
        .and_then(|r| r.error_for_status())
        .map_err(|e| format!("request failed ({url}): {e}"))?
        .text()
        .await
        .map_err(|e| format!("cannot read {url}: {e}"))
}

pub async fn fetch_plug(client: &reqwest::Client, config_url: &str) -> Result<Plug, String> {
    let text = get(client, config_url).await?;
    let doc: Value =
        serde_json::from_str(&text).map_err(|e| format!("invalid JSON from {config_url}: {e}"))?;
    parse_plug(&doc)
}

pub async fn fetch_build(client: &reqwest::Client, config_url: &str) -> Result<Build, String> {
    let plug = fetch_plug(client, config_url).await?;
    let manifest_url = plug.manifest_url();
    let bundles = parse_manifest(&get(client, &manifest_url).await?);
    if bundles.is_empty() {
        return Err(format!(
            "GFL2 resource list at {manifest_url} listed no files"
        ));
    }
    Ok(Build { plug, bundles })
}

/// Client version installed by GachaHub (None → client must be (re)installed).
pub fn installed_client(root: &Path) -> Option<String> {
    if !root.join(EXE).is_file() {
        return None;
    }
    std::fs::read_to_string(root.join(CLIENT_MARKER))
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

/// Write the install's `Version.txt`: every downloaded bundle plus lines the
/// game recorded itself (e.g. voice packs) whose files are still complete.
pub fn write_local_manifest(root: &Path, build: &Build) -> Result<(), String> {
    let dir = root.join(AB_DIR);
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = dir.join(LOCAL_MANIFEST);
    let previous = std::fs::read_to_string(&path)
        .map(|t| parse_manifest(&t))
        .unwrap_or_default();
    let mut body = String::new();
    let mut written = std::collections::HashSet::new();
    for b in build.bundles.iter().filter(|b| !b.is_voice_pack()) {
        body.push_str(&b.line());
        body.push('\n');
        written.insert(b.name.clone());
    }
    let present =
        |b: &Bundle| std::fs::metadata(dir.join(&b.name)).is_ok_and(|m| m.len() == b.size);
    // voice packs the game already fetched, then other lines the game added
    for b in build
        .bundles
        .iter()
        .filter(|b| b.is_voice_pack())
        .chain(previous.iter())
    {
        if !written.contains(&b.name) && present(b) {
            body.push_str(&b.line());
            body.push('\n');
            written.insert(b.name.clone());
        }
    }
    let tmp = dir.join(format!("{LOCAL_MANIFEST}.tmp"));
    std::fs::write(&tmp, body).map_err(|e| e.to_string())?;
    std::fs::rename(&tmp, &path).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    pub fn sample_config(addr: &str) -> String {
        let plug = serde_json::json!({
            "game_client_version": "1.0.3",
            "game_download_addr": format!("{addr}/client"),
            "ab_resource_version": "1.5.27767",
            "ab_resource_addr": format!("{addr}/res"),
        });
        serde_json::json!({"code": 0, "msg": "ok", "data": {"flexible_config": {"nexon_plug": plug.to_string()}}})
            .to_string()
    }

    #[test]
    fn parses_flexible_config() {
        let doc: Value = serde_json::from_str(&sample_config("https://cdn.example")).unwrap();
        let plug = parse_plug(&doc).unwrap();
        assert_eq!(plug.client_version, "1.0.3");
        assert_eq!(plug.bucket(), "27767");
        assert_eq!(plug.version(), "1.0.3-1.5.27767");
        assert_eq!(
            plug.client_url(),
            "https://cdn.example/client/1.0.3/GF2_Exilium_Origin.zip"
        );
        assert_eq!(
            plug.manifest_url(),
            "https://cdn.example/res/AssetBundles_Windows/27767/Version.txt"
        );
        let err: Value = serde_json::from_str(r#"{"code":1001,"msg":"maintenance"}"#).unwrap();
        assert!(parse_plug(&err).unwrap_err().contains("maintenance"));
        let insecure: Value = serde_json::from_str(&sample_config("http://evil.example")).unwrap();
        assert!(parse_plug(&insecure).is_err());
    }

    #[test]
    fn parses_resource_list() {
        let text = "{0}/ui_main.bundle#0123456789ABCDEF0123456789ABCDEF#42#27767\n\
                    {0}/JA0123456789abcdef0123456789abcdef.pck#0123456789ABCDEF0123456789ABCDEF#7#27700\n\
                    garbage line\n{0}/..#0123456789ABCDEF0123456789ABCDEF#1#1\n";
        let list = parse_manifest(text);
        assert_eq!(list.len(), 2);
        assert!(!list[0].is_voice_pack());
        assert!(list[1].is_voice_pack());
        assert_eq!(
            list[0].line(),
            "{0}/ui_main.bundle#0123456789ABCDEF0123456789ABCDEF#42#27767"
        );
        let f = list[0].to_file("https://cdn.example/res/");
        assert_eq!(
            f.url,
            "https://cdn.example/res/AssetBundles_Windows/27767/ui_main.bundle"
        );
        assert_eq!(f.dest, format!("{AB_DIR}/ui_main.bundle"));
        assert_eq!(f.md5, "0123456789abcdef0123456789abcdef");
    }
}

/// Checks against the real servers. Run with `cargo test -- --ignored live_`
/// (used by the "Live check" workflow; blocked in sandboxes without internet).
#[cfg(test)]
mod live {
    use super::*;

    async fn check(edition: &str) {
        let client = reqwest::Client::builder()
            .user_agent(crate::USER_AGENT)
            .build()
            .unwrap();
        let url = config_url(Some(edition));
        let build = fetch_build(&client, url)
            .await
            .unwrap_or_else(|e| panic!("{edition}: {e}"));
        let files = build.files();
        let voice = build.bundles.len() - files.len();
        println!(
            "LIVE gfl2/{edition}: client {} ({}), resources {} -> {} bundles ({} voice packs), {:.1} GB",
            build.plug.client_version,
            build.plug.client_url(),
            build.plug.ab_version,
            files.len(),
            voice,
            build.bundle_bytes() as f64 / 1e9
        );
        let head = client.head(build.plug.client_url()).send().await.unwrap();
        println!(
            "LIVE gfl2/{edition}: client package HTTP {} size {:?}",
            head.status(),
            head.content_length()
        );
        assert!(head.status().is_success(), "client package not reachable");
        let first = &files[0];
        let resp = client.get(&first.url).send().await.unwrap();
        assert!(
            resp.status().is_success(),
            "{} → {}",
            first.url,
            resp.status()
        );
        let body = resp.bytes().await.unwrap();
        assert_eq!(body.len() as u64, first.size, "size of {}", first.url);
        assert_eq!(
            format!("{:x}", md5::compute(&body)),
            first.md5,
            "md5 of {}",
            first.url
        );
        println!(
            "LIVE gfl2/{edition}: sample bundle {} OK (md5 matches)",
            first.dest
        );
    }

    #[tokio::test]
    #[ignore]
    async fn live_gfl2_darkwinter() {
        check("darkwinter").await;
    }

    #[tokio::test]
    #[ignore]
    async fn live_gfl2_haoplay() {
        check("haoplay").await;
    }
}
