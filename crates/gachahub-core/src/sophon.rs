//! HoYoPlay "Sophon" chunk distribution.
//!
//! Current HoYoverse builds are no longer shipped as big zip archives. Instead
//! HoYoPlay publishes a protobuf manifest per category (game data, each voice
//! language) that lists every file and the zstd-compressed chunks it is made of.
//! Chunks are downloaded individually, verified by MD5 and written at their
//! offset into the target file. Because every file carries an MD5, the same
//! code path also performs updates (only changed files are fetched) and repairs.
//!
//! Flow:
//! 1. `getGameBranches`  → branch, package_id, password and tag (= game version)
//! 2. `getBuild`         → manifest + chunk download URL prefixes per category
//! 3. manifest download  → zstd → protobuf → list of assets and chunks

use serde_json::Value;

const BRANCHES_API: &str =
    "https://sg-hyp-api.hoyoverse.com/hyp/hyp-connect/api/getGameBranches?launcher_id=VYTpXlbWo8";
const BUILD_API: &str = "https://sg-public-api.hoyoverse.com/downloader/sophon_chunk/api/getBuild";

#[derive(Debug, Clone)]
pub struct Branch {
    pub package_id: String,
    pub branch: String,
    pub password: String,
    pub tag: String,
}

#[derive(Debug, Clone)]
pub struct ManifestRef {
    /// "game" for the main data, a language code (e.g. "en-us") for voice packs.
    pub matching_field: String,
    pub manifest_url: String,
    pub manifest_compressed: bool,
    pub chunk_url_prefix: String,
    pub chunk_url_suffix: String,
    pub chunks_compressed: bool,
    pub uncompressed_size: u64,
    pub compressed_size: u64,
}

#[derive(Debug, Clone)]
pub struct Build {
    pub tag: String,
    pub manifests: Vec<ManifestRef>,
}

#[derive(Debug, Clone, Default)]
pub struct Chunk {
    pub name: String,
    pub decompressed_md5: String,
    pub offset: u64,
    pub size: u64,
    pub decompressed_size: u64,
}

#[derive(Debug, Clone, Default)]
pub struct Asset {
    pub name: String,
    pub chunks: Vec<Chunk>,
    pub asset_type: u64,
    pub size: u64,
    pub md5: String,
}

impl Asset {
    pub fn is_directory(&self) -> bool {
        self.chunks.is_empty() && (self.asset_type != 0 || self.name.ends_with('/'))
    }
}

fn str_at<'a>(v: &'a Value, path: &str) -> Option<&'a str> {
    v.pointer(path).and_then(Value::as_str)
}

fn num(v: Option<&Value>) -> u64 {
    match v {
        Some(Value::Number(n)) => n.as_u64().unwrap_or(0),
        Some(Value::String(s)) => s.parse().unwrap_or(0),
        _ => 0,
    }
}

fn check_retcode(root: &Value) -> Result<(), String> {
    match root.get("retcode").and_then(Value::as_i64) {
        Some(0) | None => Ok(()),
        Some(code) => Err(format!(
            "HoYoPlay API error {code}: {}",
            root.get("message").and_then(Value::as_str).unwrap_or("")
        )),
    }
}

async fn get_json(client: &reqwest::Client, url: &str) -> Result<Value, String> {
    let value: Value = client
        .get(url)
        .send()
        .await
        .and_then(|r| r.error_for_status())
        .map_err(|e| format!("request failed ({url}): {e}"))?
        .json()
        .await
        .map_err(|e| format!("invalid JSON from {url}: {e}"))?;
    check_retcode(&value)?;
    Ok(value)
}

/// Resolve the current `main` branch of a HoYoverse game.
pub async fn branch(client: &reqwest::Client, game_id: &str) -> Result<Branch, String> {
    let (hyp_id, biz) = crate::providers::hoyoverse::identity(game_id)
        .ok_or_else(|| format!("{game_id} is not a HoYoverse game"))?;
    let root = get_json(client, BRANCHES_API).await?;
    let entries = root
        .pointer("/data/game_branches")
        .and_then(Value::as_array)
        .ok_or("getGameBranches: data.game_branches missing")?;
    let entry = entries
        .iter()
        .find(|e| str_at(e, "/game/id") == Some(hyp_id) || str_at(e, "/game/biz") == Some(biz))
        .ok_or_else(|| format!("{game_id} not listed in getGameBranches"))?;
    let main = entry
        .get("main")
        .ok_or("getGameBranches: main branch missing")?;
    let field = |k: &str| {
        main.get(k)
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string()
    };
    let branch = Branch {
        package_id: field("package_id"),
        branch: field("branch"),
        password: field("password"),
        tag: field("tag"),
    };
    if branch.package_id.is_empty() || branch.branch.is_empty() {
        return Err("getGameBranches: incomplete branch information".into());
    }
    Ok(branch)
}

/// Resolve manifest locations for a branch.
pub async fn build(client: &reqwest::Client, branch: &Branch) -> Result<Build, String> {
    let mut url = reqwest::Url::parse(BUILD_API).map_err(|e| e.to_string())?;
    url.query_pairs_mut()
        .append_pair("branch", &branch.branch)
        .append_pair("package_id", &branch.package_id)
        .append_pair("password", &branch.password);
    let root = get_json(client, url.as_str()).await?;
    let data = root.get("data").ok_or("getBuild: data missing")?;
    let tag = data
        .get("tag")
        .and_then(Value::as_str)
        .unwrap_or(&branch.tag)
        .to_string();
    let list = data
        .get("manifests")
        .and_then(Value::as_array)
        .ok_or("getBuild: manifests missing")?;

    let join = |prefix: &str, name: &str| format!("{}/{}", prefix.trim_end_matches('/'), name);
    let compressed = |v: Option<&Value>| match v {
        Some(Value::String(s)) => s != "0" && !s.is_empty(),
        Some(Value::Number(n)) => n.as_u64().unwrap_or(0) != 0,
        Some(Value::Bool(b)) => *b,
        _ => false,
    };

    let mut manifests = Vec::new();
    for m in list {
        let id = str_at(m, "/manifest/id").unwrap_or_default();
        let manifest_prefix = str_at(m, "/manifest_download/url_prefix").unwrap_or_default();
        let chunk_prefix = str_at(m, "/chunk_download/url_prefix").unwrap_or_default();
        if id.is_empty() || manifest_prefix.is_empty() || chunk_prefix.is_empty() {
            continue;
        }
        let manifest_suffix = str_at(m, "/manifest_download/url_suffix").unwrap_or_default();
        manifests.push(ManifestRef {
            matching_field: m
                .get("matching_field")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
            manifest_url: format!("{}{}", join(manifest_prefix, id), manifest_suffix),
            manifest_compressed: compressed(m.pointer("/manifest_download/compression")),
            chunk_url_prefix: chunk_prefix.trim_end_matches('/').to_string(),
            chunk_url_suffix: str_at(m, "/chunk_download/url_suffix")
                .unwrap_or_default()
                .to_string(),
            chunks_compressed: compressed(m.pointer("/chunk_download/compression")),
            uncompressed_size: num(m.pointer("/stats/uncompressed_size")),
            compressed_size: num(m.pointer("/stats/compressed_size")),
        });
    }
    if manifests.is_empty() {
        return Err("getBuild returned no usable manifests".into());
    }
    Ok(Build { tag, manifests })
}

impl ManifestRef {
    pub fn chunk_url(&self, chunk: &Chunk) -> String {
        format!(
            "{}/{}{}",
            self.chunk_url_prefix, chunk.name, self.chunk_url_suffix
        )
    }
}

/// Download, decompress and parse a manifest.
pub async fn fetch_assets(
    client: &reqwest::Client,
    manifest: &ManifestRef,
) -> Result<Vec<Asset>, String> {
    let bytes = client
        .get(&manifest.manifest_url)
        .send()
        .await
        .and_then(|r| r.error_for_status())
        .map_err(|e| format!("manifest download failed: {e}"))?
        .bytes()
        .await
        .map_err(|e| format!("manifest download failed: {e}"))?;
    let raw = if manifest.manifest_compressed {
        zstd::stream::decode_all(&bytes[..])
            .map_err(|e| format!("manifest decompression failed: {e}"))?
    } else {
        bytes.to_vec()
    };
    parse_manifest(&raw)
}

// --------------------------------------------------------------------------
// Minimal protobuf decoding (no codegen needed for three small messages).
//
// message Manifest { repeated Asset assets = 1; }
// message Asset    { string name = 1; repeated Chunk chunks = 2; int32 type = 3;
//                    int64 size = 4; string md5 = 5; }
// message Chunk    { string name = 1; string decompressed_md5 = 2; int64 offset = 3;
//                    int64 size = 4; int64 decompressed_size = 5;
//                    uint64 xxh64 = 6; string compressed_md5 = 7; }

enum Field<'a> {
    Varint(u64),
    Bytes(&'a [u8]),
    Other,
}

struct Reader<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    fn new(buf: &'a [u8]) -> Self {
        Self { buf, pos: 0 }
    }

    fn varint(&mut self) -> Result<u64, String> {
        let mut out = 0u64;
        for shift in (0..64).step_by(7) {
            let b = *self.buf.get(self.pos).ok_or("truncated varint")?;
            self.pos += 1;
            out |= u64::from(b & 0x7f) << shift;
            if b & 0x80 == 0 {
                return Ok(out);
            }
        }
        Err("varint too long".into())
    }

    fn next(&mut self) -> Result<Option<(u64, Field<'a>)>, String> {
        if self.pos >= self.buf.len() {
            return Ok(None);
        }
        let key = self.varint()?;
        let (number, wire) = (key >> 3, key & 7);
        let field = match wire {
            0 => Field::Varint(self.varint()?),
            1 => {
                self.pos += 8;
                Field::Other
            }
            2 => {
                let len = self.varint()? as usize;
                let end = self
                    .pos
                    .checked_add(len)
                    .filter(|e| *e <= self.buf.len())
                    .ok_or("truncated field")?;
                let bytes = &self.buf[self.pos..end];
                self.pos = end;
                Field::Bytes(bytes)
            }
            5 => {
                self.pos += 4;
                Field::Other
            }
            w => return Err(format!("unsupported protobuf wire type {w}")),
        };
        if self.pos > self.buf.len() {
            return Err("truncated field".into());
        }
        Ok(Some((number, field)))
    }
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

fn parse_chunk(buf: &[u8]) -> Result<Chunk, String> {
    let mut chunk = Chunk::default();
    let mut r = Reader::new(buf);
    while let Some((n, f)) = r.next()? {
        match (n, f) {
            (1, Field::Bytes(b)) => chunk.name = text(b),
            (2, Field::Bytes(b)) => chunk.decompressed_md5 = text(b),
            (3, Field::Varint(v)) => chunk.offset = v,
            (4, Field::Varint(v)) => chunk.size = v,
            (5, Field::Varint(v)) => chunk.decompressed_size = v,
            _ => {}
        }
    }
    Ok(chunk)
}

fn parse_asset(buf: &[u8]) -> Result<Asset, String> {
    let mut asset = Asset::default();
    let mut r = Reader::new(buf);
    while let Some((n, f)) = r.next()? {
        match (n, f) {
            (1, Field::Bytes(b)) => asset.name = text(b),
            (2, Field::Bytes(b)) => asset.chunks.push(parse_chunk(b)?),
            (3, Field::Varint(v)) => asset.asset_type = v,
            (4, Field::Varint(v)) => asset.size = v,
            (5, Field::Bytes(b)) => asset.md5 = text(b),
            _ => {}
        }
    }
    Ok(asset)
}

pub fn parse_manifest(buf: &[u8]) -> Result<Vec<Asset>, String> {
    let mut assets = Vec::new();
    let mut r = Reader::new(buf);
    while let Some((n, f)) = r.next()? {
        if let (1, Field::Bytes(b)) = (n, f) {
            let asset = parse_asset(b)?;
            if !asset.name.is_empty() {
                assets.push(asset);
            }
        }
    }
    if assets.is_empty() {
        return Err("manifest contains no files".into());
    }
    Ok(assets)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn varint(mut v: u64, out: &mut Vec<u8>) {
        loop {
            let b = (v & 0x7f) as u8;
            v >>= 7;
            if v == 0 {
                out.push(b);
                break;
            }
            out.push(b | 0x80);
        }
    }
    fn bytes_field(n: u64, data: &[u8], out: &mut Vec<u8>) {
        varint(n << 3 | 2, out);
        varint(data.len() as u64, out);
        out.extend_from_slice(data);
    }
    fn int_field(n: u64, v: u64, out: &mut Vec<u8>) {
        varint(n << 3, out);
        varint(v, out);
    }

    #[test]
    fn parses_manifest() {
        let mut chunk = Vec::new();
        bytes_field(1, b"abc_123", &mut chunk);
        bytes_field(2, b"d41d8cd98f00b204e9800998ecf8427e", &mut chunk);
        int_field(3, 4096, &mut chunk);
        int_field(4, 100, &mut chunk);
        int_field(5, 300, &mut chunk);
        int_field(6, 0xdead_beef, &mut chunk);
        let mut asset = Vec::new();
        bytes_field(1, b"GenshinImpact_Data/data.unity3d", &mut asset);
        bytes_field(2, &chunk, &mut asset);
        int_field(4, 4396, &mut asset);
        bytes_field(5, b"0123456789abcdef0123456789abcdef", &mut asset);
        let mut dir = Vec::new();
        bytes_field(1, b"GenshinImpact_Data", &mut dir);
        int_field(3, 64, &mut dir);
        let mut manifest = Vec::new();
        bytes_field(1, &asset, &mut manifest);
        bytes_field(1, &dir, &mut manifest);

        let assets = parse_manifest(&manifest).unwrap();
        assert_eq!(assets.len(), 2);
        let a = &assets[0];
        assert_eq!(a.name, "GenshinImpact_Data/data.unity3d");
        assert_eq!(a.size, 4396);
        assert_eq!(a.chunks[0].offset, 4096);
        assert_eq!(a.chunks[0].decompressed_size, 300);
        assert!(!a.is_directory());
        assert!(assets[1].is_directory());
    }

    #[test]
    fn rejects_truncated_data() {
        assert!(parse_manifest(&[0x0a, 0x05, 0x01]).is_err());
    }
}
