//! Arknights (Yostar global launcher) and Arknights: Endfield (Gryphline launcher).
//!
//! **Endfield** – `launcher.gryphline.com/api/game/get_latest` returns the
//! current version and the game as a split zip (`….zip.001`, `.002`, …) with
//! MD5 and size per part; GachaHub's package downloader fetches, verifies and
//! extracts it.
//!
//! **Arknights** – the Yostar launcher API needs a signed `Authorization`
//! header: `{"head":{…},"sign":md5(JSON(head) + body + salt)}`. It names the
//! current version, CDN and a file list (`file[].{path, hash, size}`, hash =
//! CRC-64/XZ in decimal). Files are downloaded one by one from
//! `<cdn><source>/<path>`, so updates and repairs only touch changed files.

use crate::{kuro::KuroFile, providers::PackageFile};
use serde_json::Value;

// ------------------------------------------------------------------ Endfield

const ENDFIELD_LATEST: &str =
    "https://launcher.gryphline.com/api/game/get_latest?appcode=YDUTE5gscDZ229CW&channel=6&sub_channel=6";

#[derive(Debug, Clone)]
pub struct EndfieldBuild {
    pub version: String,
    pub packs: Vec<PackageFile>,
}

pub fn parse_endfield(doc: &Value) -> Result<EndfieldBuild, String> {
    let version = text(doc.get("version")).ok_or("Endfield: version missing")?;
    let packs: Vec<PackageFile> = doc
        .pointer("/pkg/packs")
        .and_then(Value::as_array)
        .ok_or("Endfield: package list missing")?
        .iter()
        .filter_map(|p| {
            let url = text(p.get("url"))
                .filter(|u| u.starts_with("https://") || u.starts_with("http://127.0.0.1"))?;
            let destination = url.split('?').next()?.rsplit('/').next()?.to_string();
            Some(PackageFile {
                url,
                destination,
                size: num(p.get("package_size")),
                hash: text(p.get("md5")).map(|m| m.to_ascii_lowercase()),
            })
        })
        .collect();
    if packs.is_empty() {
        return Err("Endfield: no download packages".into());
    }
    Ok(EndfieldBuild { version, packs })
}

pub async fn fetch_endfield(client: &reqwest::Client) -> Result<EndfieldBuild, String> {
    let doc: Value = client
        .get(ENDFIELD_LATEST)
        .send()
        .await
        .and_then(|r| r.error_for_status())
        .map_err(|e| format!("Gryphline launcher API: {e}"))?
        .json()
        .await
        .map_err(|e| format!("Gryphline launcher API: {e}"))?;
    parse_endfield(&doc)
}

// ------------------------------------------------------------------ Arknights

const YOSTAR_API: &str = "https://api-launcher-en.yo-star.com";
const ARKNIGHTS_TAG: &str = "Arknights_EN";
const LAUNCHER_VERSION: &str = "1.8.1";
const SALT: &str = "DE7108E9B2842FD460F4777702727869";

/// `Authorization` header of the Yostar launcher API. The key order of
/// `head` is part of the signature.
pub fn yostar_auth(game_tag: &str, time: u64, body: &str) -> String {
    let head =
        format!(r#"{{"game_tag":"{game_tag}","time":{time},"version":"{LAUNCHER_VERSION}"}}"#);
    let sign = format!("{:x}", md5::compute(format!("{head}{body}{SALT}")));
    format!(r#"{{"head":{head},"sign":"{sign}"}}"#)
}

async fn yostar_get(
    client: &reqwest::Client,
    path: &str,
    query: &[(&str, &str)],
) -> Result<Value, String> {
    let time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let doc: Value = client
        .get(format!("{YOSTAR_API}{path}"))
        .query(query)
        .header("Authorization", yostar_auth(ARKNIGHTS_TAG, time, ""))
        .header("content-type", "application/json;charset=UTF-8")
        .send()
        .await
        .and_then(|r| r.error_for_status())
        .map_err(|e| format!("Yostar launcher API {path}: {e}"))?
        .json()
        .await
        .map_err(|e| format!("Yostar launcher API {path}: {e}"))?;
    let code = doc.get("code").and_then(Value::as_i64).unwrap_or(-1);
    if code != 200 {
        let msg = text(doc.get("msg")).unwrap_or_default();
        return Err(format!("Yostar launcher API {path}: error {code} {msg}"));
    }
    Ok(doc.get("data").cloned().unwrap_or(Value::Null))
}

#[derive(Debug, Clone)]
pub struct ArknightsBuild {
    pub version: String,
    pub exe: String,
    pub files: Vec<KuroFile>,
}

/// Percent-encode characters that would end the URL path (`#`, `?`, `%`, space).
fn encode_path(path: &str) -> String {
    path.replace('%', "%25")
        .replace('#', "%23")
        .replace('?', "%3F")
        .replace(' ', "%20")
}

/// Parse the Yostar file list below `base`.
pub fn parse_yostar_files(index: &Value, base: &str) -> Result<Vec<KuroFile>, String> {
    let files: Vec<KuroFile> = index
        .get("file")
        .and_then(Value::as_array)
        .ok_or("Arknights file list: 'file' missing")?
        .iter()
        .filter_map(|f| {
            let dest = text(f.get("path"))?
                .trim_start_matches(['/', '\\'])
                .replace('\\', "/");
            if dest.is_empty() || dest.split('/').any(|c| c == "..") {
                return None;
            }
            let hash = text(f.get("hash")).unwrap_or_default();
            Some(KuroFile {
                url: format!("{}/{}", base.trim_end_matches('/'), encode_path(&dest)),
                size: num(f.get("size")),
                md5: if hash.is_empty() {
                    String::new()
                } else {
                    format!("crc64:{hash}")
                },
                dest,
            })
        })
        .collect();
    if files.is_empty() {
        return Err("Arknights file list is empty".into());
    }
    Ok(files)
}

pub async fn arknights_version(client: &reqwest::Client) -> Result<String, String> {
    let cfg = yostar_get(client, "/api/launcher/game/config", &[]).await?;
    text(cfg.get("game_latest_version")).ok_or_else(|| "Arknights: version missing".into())
}

pub async fn fetch_arknights(client: &reqwest::Client) -> Result<ArknightsBuild, String> {
    let cfg = yostar_get(client, "/api/launcher/game/config", &[]).await?;
    let version = text(cfg.get("game_latest_version")).ok_or("Arknights: version missing")?;
    let file_path = text(cfg.get("game_latest_file_path")).unwrap_or_default();
    let exe = text(cfg.get("game_start_exe_name")).unwrap_or_else(|| "Arknights".into());
    let cdn = yostar_get(client, "/api/launcher/advanced/game/download/cdn", &[]).await?;
    let primary = text(cdn.get("primary_cdn")).ok_or("Arknights: CDN missing")?;
    let list = yostar_get(
        client,
        "/api/launcher/game/config/json",
        &[
            ("version", version.as_str()),
            ("file_path", file_path.as_str()),
        ],
    )
    .await?;
    let list_url = text(list.get("url")).ok_or("Arknights: file list URL missing")?;
    let index: Value = client
        .get(&list_url)
        .send()
        .await
        .and_then(|r| r.error_for_status())
        .map_err(|e| format!("Arknights file list: {e}"))?
        .json()
        .await
        .map_err(|e| format!("Arknights file list: {e}"))?;
    let source = text(index.get("source")).unwrap_or_default();
    let base = format!("{}{}", primary.trim_end_matches('/'), ensure_slash(&source));
    Ok(ArknightsBuild {
        version,
        exe: format!("{exe}.exe"),
        files: parse_yostar_files(&index, &base)?,
    })
}

fn ensure_slash(s: &str) -> String {
    if s.is_empty() || s.starts_with('/') {
        s.to_string()
    } else {
        format!("/{s}")
    }
}

// ------------------------------------------------------------------ helpers

fn text(v: Option<&Value>) -> Option<String> {
    match v? {
        Value::String(s) if !s.trim().is_empty() => Some(s.trim().to_string()),
        Value::Number(n) => Some(n.to_string()),
        _ => None,
    }
}

fn num(v: Option<&Value>) -> u64 {
    match v {
        Some(Value::Number(n)) => n.as_u64().unwrap_or(0),
        Some(Value::String(s)) => s.trim().parse().unwrap_or(0),
        _ => 0,
    }
}

/// CRC-64/XZ (ECMA-182, reflected) – the checksum in Yostar file lists.
pub struct Crc64(u64);

impl Crc64 {
    fn table() -> &'static [u64; 256] {
        static TABLE: std::sync::OnceLock<[u64; 256]> = std::sync::OnceLock::new();
        TABLE.get_or_init(|| {
            let mut t = [0u64; 256];
            for (i, slot) in t.iter_mut().enumerate() {
                let mut c = i as u64;
                for _ in 0..8 {
                    c = if c & 1 == 1 {
                        (c >> 1) ^ 0xC96C_5795_D787_0F42
                    } else {
                        c >> 1
                    };
                }
                *slot = c;
            }
            t
        })
    }
    pub fn new() -> Self {
        Crc64(!0)
    }
    pub fn update(&mut self, data: &[u8]) {
        let t = Self::table();
        let mut c = self.0;
        for &b in data {
            c = t[((c ^ b as u64) & 0xff) as usize] ^ (c >> 8);
        }
        self.0 = c;
    }
    pub fn finish(&self) -> u64 {
        !self.0
    }
}

impl Default for Crc64 {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crc64_xz_check_value() {
        let mut c = Crc64::new();
        c.update(b"123456789");
        assert_eq!(c.finish(), 0x995D_C9BB_DF19_39FA);
    }

    #[test]
    fn yostar_signature_format() {
        let h = yostar_auth("Arknights_EN", 1_700_000_000, "");
        let v: Value = serde_json::from_str(&h).unwrap();
        assert_eq!(v["head"]["game_tag"], "Arknights_EN");
        assert_eq!(v["head"]["time"], 1_700_000_000);
        let head = r#"{"game_tag":"Arknights_EN","time":1700000000,"version":"1.8.1"}"#;
        let expect = format!(
            "{:x}",
            md5::compute(format!("{head}DE7108E9B2842FD460F4777702727869"))
        );
        assert_eq!(v["sign"], expect);
        assert!(h.starts_with(r#"{"head":{"game_tag":"#));
    }

    #[test]
    fn parses_endfield_and_yostar_lists() {
        let doc: Value = serde_json::from_str(
            r#"{"version":"1.7.2","pkg":{"file_path":"https://beyond.example/files","total_size":"3",
               "packs":[{"url":"https://beyond.example/packs/Beyond.zip.001","md5":"ABC","package_size":"1073741824"},
                        {"url":"https://beyond.example/packs/Beyond.zip.002","md5":"def","package_size":5}]}}"#,
        )
        .unwrap();
        let b = parse_endfield(&doc).unwrap();
        assert_eq!(b.version, "1.7.2");
        assert_eq!(b.packs[0].destination, "Beyond.zip.001");
        assert_eq!(b.packs[0].size, 1_073_741_824);
        assert_eq!(b.packs[0].hash.as_deref(), Some("abc"));

        let idx: Value = serde_json::from_str(
            r#"{"source":"/Arknights_EN-042.0.0-game","file":[
                {"path":"/Arknights.exe","hash":"123","size":"42"},
                {"path":"/Arknights_Data/a#b.ab","hash":"9","size":"1"},
                {"path":"/../evil","hash":"1","size":"1"}]}"#,
        )
        .unwrap();
        let files =
            parse_yostar_files(&idx, "https://cdn.example/Arknights_EN-042.0.0-game").unwrap();
        assert_eq!(files.len(), 2);
        assert_eq!(files[0].dest, "Arknights.exe");
        assert_eq!(files[0].md5, "crc64:123");
        assert_eq!(
            files[1].url,
            "https://cdn.example/Arknights_EN-042.0.0-game/Arknights_Data/a%23b.ab"
        );
    }
}

/// Real-server checks: `cargo test -- --ignored live_`.
#[cfg(test)]
mod live {
    #[tokio::test]
    #[ignore]
    async fn live_endfield() {
        let client = reqwest::Client::builder()
            .user_agent(crate::USER_AGENT)
            .build()
            .unwrap();
        let b = super::fetch_endfield(&client).await.unwrap();
        let total: u64 = b.packs.iter().map(|p| p.size).sum();
        println!(
            "LIVE endfield: version {} – {} parts, {:.1} GB, first {}",
            b.version,
            b.packs.len(),
            total as f64 / 1e9,
            b.packs[0].url
        );
        let head = client.head(&b.packs[0].url).send().await.unwrap();
        println!(
            "LIVE endfield: part 1 HTTP {} {:?}",
            head.status(),
            head.headers().get("content-length")
        );
        assert!(head.status().is_success());
    }

    #[tokio::test]
    #[ignore]
    async fn live_arknights() {
        let client = reqwest::Client::builder()
            .user_agent(crate::USER_AGENT)
            .build()
            .unwrap();
        let b = super::fetch_arknights(&client).await.unwrap();
        let total: u64 = b.files.iter().map(|f| f.size).sum();
        println!(
            "LIVE arknights: version {} exe {} – {} files, {:.1} GB",
            b.version,
            b.exe,
            b.files.len(),
            total as f64 / 1e9
        );
        // smallest file: download and check its CRC-64
        let f = b
            .files
            .iter()
            .filter(|f| f.size > 0)
            .min_by_key(|f| f.size)
            .unwrap();
        let body = client
            .get(&f.url)
            .send()
            .await
            .unwrap()
            .error_for_status()
            .unwrap()
            .bytes()
            .await
            .unwrap();
        let mut c = super::Crc64::new();
        c.update(&body);
        println!(
            "LIVE arknights: sample {} ({} bytes) crc64 {} expected {}",
            f.dest,
            body.len(),
            c.finish(),
            f.md5
        );
        assert_eq!(format!("crc64:{}", c.finish()), f.md5);
    }
}
