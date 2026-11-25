//! Official game icons, fetched at runtime from the publishers' own sources
//! (never bundled with GachaHub):
//! - HoYoverse games: the HoYoPlay launcher API (`getGames` → `display.icon`);
//! - otherwise the game's official mobile app icon from Apple's public
//!   iTunes Search API (512 px, matched by exact app name),
//! - or the app icon of the official website
//!   (`apple-touch-icon` / largest `<link rel="icon">`, then `/favicon.ico`).
//!
//! The result is normalised to a square PNG (max. 256 px) for the UI cache.

use crate::{games::Game, USER_AGENT};
use serde_json::Value;
use std::io::Cursor;

const HOYOPLAY_GAMES_API: &str =
    "https://sg-hyp-api.hoyoverse.com/hyp/hyp-connect/api/getGames?launcher_id=VYTpXlbWo8&language=en-us";

/// Largest edge of the cached icon.
const ICON_SIZE: u32 = 256;
/// Icons smaller than this are favicons that would look blurry → rejected.
const MIN_SIZE: u32 = 48;

pub fn client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .timeout(std::time::Duration::from_secs(20))
        .build()
        .map_err(|e| e.to_string())
}

/// Fetch the official icon of `game` as PNG bytes.
pub async fn fetch(client: &reqwest::Client, game: &Game) -> Result<Vec<u8>, String> {
    let mut errors = Vec::new();
    if let Some((id, biz)) = crate::providers::hoyoverse::identity(&game.id) {
        match hoyoplay_icon_url(client, id, biz).await {
            Ok(url) => match download_image(client, &url).await {
                Ok(png) => return Ok(png),
                Err(e) => errors.push(e),
            },
            Err(e) => errors.push(e),
        }
    }
    match app_store_icon_url(client, game.app_store_name.as_deref().unwrap_or(&game.name)).await {
        Ok(url) => match download_image(client, &url).await {
            Ok(png) => return Ok(png),
            Err(e) => errors.push(e),
        },
        Err(e) => errors.push(e),
    }
    if let Some(site) = game.website.as_deref().filter(|s| !s.is_empty()) {
        match website_icon_urls(client, site).await {
            Ok(urls) => {
                for url in urls {
                    match download_image(client, &url).await {
                        Ok(png) => return Ok(png),
                        Err(e) => errors.push(e),
                    }
                }
            }
            Err(e) => errors.push(e),
        }
    }
    if errors.is_empty() {
        errors.push(format!("no official icon source for {}", game.id));
    }
    Err(errors.join("; "))
}

fn norm(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

async fn app_store_icon_url(client: &reqwest::Client, name: &str) -> Result<String, String> {
    // some games are only listed in some regions' stores
    for country in ["us", "gb", "jp", "kr", "tw", "sg"] {
        let doc: Value = client
            .get("https://itunes.apple.com/search")
            .query(&[
                ("term", name),
                ("entity", "software"),
                ("limit", "15"),
                ("country", country),
            ])
            .send()
            .await
            .and_then(|r| r.error_for_status())
            .map_err(|e| format!("App Store: {e}"))?
            .json()
            .await
            .map_err(|e| format!("App Store: {e}"))?;
        if let Some(url) = parse_app_store_icon(&doc, name) {
            return Ok(url);
        }
    }
    Err(format!("App Store: no app named {name}"))
}

/// Icon of the app whose name matches `name` (exactly, ignoring case and
/// punctuation; otherwise an app name that starts with it).
pub fn parse_app_store_icon(doc: &Value, name: &str) -> Option<String> {
    let want = norm(name);
    let results = doc.get("results")?.as_array()?;
    let icon = |r: &Value| {
        r.get("artworkUrl512")
            .or_else(|| r.get("artworkUrl100"))
            .and_then(Value::as_str)
            .filter(|u| u.starts_with("https://"))
            .map(str::to_string)
    };
    let track = |r: &Value| norm(r.get("trackName").and_then(Value::as_str).unwrap_or(""));
    results
        .iter()
        .find(|r| track(r) == want)
        .or_else(|| {
            results.iter().find(|r| {
                let t = track(r);
                !t.is_empty() && t.starts_with(&want) && t.len() <= want.len() + 16
            })
        })
        .and_then(icon)
}

async fn hoyoplay_icon_url(
    client: &reqwest::Client,
    id: &str,
    biz: &str,
) -> Result<String, String> {
    let doc: Value = client
        .get(HOYOPLAY_GAMES_API)
        .send()
        .await
        .and_then(|r| r.error_for_status())
        .map_err(|e| format!("HoYoPlay: {e}"))?
        .json()
        .await
        .map_err(|e| format!("HoYoPlay: {e}"))?;
    parse_hoyoplay_icon(&doc, id, biz).ok_or_else(|| format!("HoYoPlay lists no icon for {biz}"))
}

pub fn parse_hoyoplay_icon(doc: &Value, id: &str, biz: &str) -> Option<String> {
    doc.pointer("/data/games")?
        .as_array()?
        .iter()
        .find(|g| g["id"].as_str() == Some(id) || g["biz"].as_str() == Some(biz))
        .and_then(|g| g.pointer("/display/icon/url"))
        .and_then(Value::as_str)
        .filter(|u| u.starts_with("https://"))
        .map(str::to_string)
}

async fn website_icon_urls(client: &reqwest::Client, site: &str) -> Result<Vec<String>, String> {
    let resp = client
        .get(site)
        .send()
        .await
        .and_then(|r| r.error_for_status())
        .map_err(|e| format!("{site}: {e}"))?;
    let base = resp.url().to_string();
    let html = resp.text().await.map_err(|e| format!("{site}: {e}"))?;
    let mut urls = icon_links(&html, &base);
    urls.push(resolve(&base, "/apple-touch-icon.png"));
    urls.push(resolve(&base, "/favicon.ico"));
    urls.dedup();
    Ok(urls)
}

/// Icon links of an HTML page, best first: apple-touch-icon, then by size.
pub fn icon_links(html: &str, base: &str) -> Vec<String> {
    let mut found: Vec<(u32, String)> = Vec::new();
    let lower = html.to_ascii_lowercase();
    let mut pos = 0;
    while let Some(start) = lower[pos..].find("<link") {
        let start = pos + start;
        let end = lower[start..]
            .find('>')
            .map(|e| start + e)
            .unwrap_or(lower.len());
        let tag = &html[start..end];
        let tag_l = &lower[start..end];
        pos = end;
        let Some(rel) = attr(tag, tag_l, "rel") else {
            continue;
        };
        let rel = rel.to_ascii_lowercase();
        if !rel
            .split_whitespace()
            .any(|r| r == "icon" || r.starts_with("apple-touch-icon"))
        {
            continue;
        }
        let Some(href) = attr(tag, tag_l, "href") else {
            continue;
        };
        if href.starts_with("data:") {
            continue;
        }
        let size = attr(tag, tag_l, "sizes")
            .and_then(|s| {
                s.split(['x', 'X'])
                    .next()
                    .and_then(|n| n.trim().parse().ok())
            })
            .unwrap_or(0u32);
        let score = if rel.contains("apple-touch-icon") {
            10_000 + size.max(180)
        } else {
            size
        };
        found.push((score, resolve(base, &href)));
    }
    found.sort_by_key(|(score, _)| std::cmp::Reverse(*score));
    found.into_iter().map(|(_, url)| url).collect()
}

fn attr(tag: &str, tag_lower: &str, name: &str) -> Option<String> {
    let mut search = 0;
    while let Some(i) = tag_lower[search..].find(name) {
        let i = search + i;
        search = i + name.len();
        let before_ok = i == 0 || tag_lower.as_bytes()[i - 1].is_ascii_whitespace();
        let rest = tag_lower[search..].trim_start();
        if !before_ok || !rest.starts_with('=') {
            continue;
        }
        let offset = tag_lower.len() - rest.len() + 1;
        let value = tag[offset..].trim_start();
        let (quote, body) = match value.chars().next() {
            Some(q @ ('"' | '\'')) => (Some(q), &value[1..]),
            _ => (None, value),
        };
        let end = match quote {
            Some(q) => body.find(q).unwrap_or(body.len()),
            None => body
                .find(|c: char| c.is_whitespace() || c == '/')
                .unwrap_or(body.len()),
        };
        return Some(body[..end].trim().replace("&amp;", "&"));
    }
    None
}

/// Resolve `href` against the page URL.
fn resolve(base: &str, href: &str) -> String {
    if href.starts_with("https://") || href.starts_with("http://") {
        return href.to_string();
    }
    let scheme_end = base.find("://").map(|i| i + 3).unwrap_or(0);
    let host_end = base[scheme_end..]
        .find('/')
        .map(|i| scheme_end + i)
        .unwrap_or(base.len());
    if let Some(rest) = href.strip_prefix("//") {
        return format!("{}{rest}", &base[..scheme_end]);
    }
    if href.starts_with('/') {
        return format!("{}{href}", &base[..host_end]);
    }
    let dir_end = base
        .rfind('/')
        .filter(|i| *i >= host_end)
        .map(|i| i + 1)
        .unwrap_or(base.len());
    let dir = &base[..dir_end];
    if dir.ends_with('/') {
        format!("{dir}{href}")
    } else {
        format!("{dir}/{href}")
    }
}

async fn download_image(client: &reqwest::Client, url: &str) -> Result<Vec<u8>, String> {
    let resp = client
        .get(url)
        .send()
        .await
        .and_then(|r| r.error_for_status())
        .map_err(|e| format!("{url}: {e}"))?;
    let bytes = resp.bytes().await.map_err(|e| format!("{url}: {e}"))?;
    if bytes.len() > 8 * 1024 * 1024 {
        return Err(format!("{url}: image too large"));
    }
    to_png(&bytes).map_err(|e| format!("{url}: {e}"))
}

/// Decode any supported image (PNG, JPEG, WebP, ICO, BMP), crop to a square
/// and store it as a PNG of at most 256 px.
pub fn to_png(bytes: &[u8]) -> Result<Vec<u8>, String> {
    let img = image::load_from_memory(bytes).map_err(|e| format!("not an image: {e}"))?;
    let (w, h) = (img.width(), img.height());
    if w.min(h) < MIN_SIZE {
        return Err(format!("icon too small ({w}×{h})"));
    }
    let side = w.min(h);
    let img = img.crop_imm((w - side) / 2, (h - side) / 2, side, side);
    let img = if side > ICON_SIZE {
        img.resize_exact(ICON_SIZE, ICON_SIZE, image::imageops::FilterType::Lanczos3)
    } else {
        img
    };
    let mut out = Vec::new();
    img.write_to(&mut Cursor::new(&mut out), image::ImageFormat::Png)
        .map_err(|e| e.to_string())?;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_best_icon_link() {
        let html = r#"<head><link rel="stylesheet" href="/a.css">
            <link rel="icon" type="image/png" sizes="32x32" href="/fav32.png">
            <LINK REL="icon" sizes="192x192" href='img/icon192.png'>
            <link href="https://cdn.example/touch.png" rel="apple-touch-icon">
            <link rel="shortcut icon" href="//cdn.example/f.ico"></head>"#;
        let links = icon_links(html, "https://game.example/en/home");
        assert_eq!(links[0], "https://cdn.example/touch.png");
        assert_eq!(links[1], "https://game.example/en/img/icon192.png");
        assert_eq!(links[2], "https://game.example/fav32.png");
        assert_eq!(links[3], "https://cdn.example/f.ico");
    }

    #[test]
    fn matches_app_store_names() {
        let doc: Value = serde_json::from_str(
            r#"{"results":[
                {"trackName":"Arknights: Endfield","artworkUrl512":"https://is1.example/endfield.png"},
                {"trackName":"Arknights","artworkUrl512":"https://is1.example/arknights.png"},
                {"trackName":"Wuthering Waves","artworkUrl100":"https://is1.example/ww.png"}]}"#,
        )
        .unwrap();
        assert_eq!(
            parse_app_store_icon(&doc, "Arknights").as_deref(),
            Some("https://is1.example/arknights.png")
        );
        assert_eq!(
            parse_app_store_icon(&doc, "Arknights: Endfield").as_deref(),
            Some("https://is1.example/endfield.png")
        );
        assert_eq!(
            parse_app_store_icon(&doc, "Wuthering Waves").as_deref(),
            Some("https://is1.example/ww.png")
        );
        assert!(parse_app_store_icon(&doc, "Limbus Company").is_none());
    }

    #[test]
    fn parses_hoyoplay_games() {
        let doc: Value = serde_json::from_str(
            r#"{"retcode":0,"data":{"games":[{"id":"gopR6Cufr3","biz":"hk4e_global",
                "display":{"icon":{"url":"https://cdn.example/genshin.png"}}}]}}"#,
        )
        .unwrap();
        assert_eq!(
            parse_hoyoplay_icon(&doc, "x", "hk4e_global").as_deref(),
            Some("https://cdn.example/genshin.png")
        );
        assert!(parse_hoyoplay_icon(&doc, "y", "nap_global").is_none());
    }

    #[test]
    fn normalises_to_square_png() {
        let img = image::RgbaImage::from_pixel(400, 300, image::Rgba([200, 10, 10, 255]));
        let mut bytes = Vec::new();
        image::DynamicImage::ImageRgba8(img)
            .write_to(&mut Cursor::new(&mut bytes), image::ImageFormat::Png)
            .unwrap();
        let png = to_png(&bytes).unwrap();
        let back = image::load_from_memory(&png).unwrap();
        assert_eq!((back.width(), back.height()), (256, 256));
        let tiny = image::RgbaImage::new(16, 16);
        let mut small = Vec::new();
        image::DynamicImage::ImageRgba8(tiny)
            .write_to(&mut Cursor::new(&mut small), image::ImageFormat::Png)
            .unwrap();
        assert!(to_png(&small).is_err());
    }
}

/// Real-server check: `cargo test -- --ignored live_`.
#[cfg(test)]
mod live {
    #[tokio::test]
    #[ignore]
    async fn live_official_icons() {
        let client = super::client().unwrap();
        let registry = crate::games::GameRegistry::builtin();
        let mut ok = 0;
        for game in &registry.games {
            match super::fetch(&client, game).await {
                Ok(png) => {
                    ok += 1;
                    println!("LIVE icon {}: OK ({} bytes)", game.id, png.len());
                    // small thumbnail for a visual check of the CI log
                    let img = image::load_from_memory(&png).unwrap().thumbnail(48, 48);
                    let mut thumb = Vec::new();
                    img.write_to(
                        &mut std::io::Cursor::new(&mut thumb),
                        image::ImageFormat::Png,
                    )
                    .unwrap();
                    println!("LIVE thumb {} {}", game.id, hex(&thumb));
                }
                Err(e) => println!("LIVE icon {}: FAILED {e}", game.id),
            }
        }
        println!("LIVE icons: {ok}/{} games", registry.games.len());
        fn hex(b: &[u8]) -> String {
            b.iter().map(|x| format!("{x:02x}")).collect()
        }
        assert!(ok >= 4, "at least the HoYoverse icons must load");
    }
}
