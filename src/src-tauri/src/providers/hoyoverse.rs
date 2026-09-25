use super::{GameProvider, PackageFile, ProviderDownloadPlan, ProviderError};
use serde_json::Value;

pub struct HoyoverseProvider;

const GLOBAL_PACKAGES_API: &str = "https://sg-hyp-api.hoyoverse.com/hyp/hyp-connect/api/getGamePackages?launcher_id=VYTpXlbWo8&language=en-us";

fn game_identity(game_id: &str) -> Option<(&'static str, &'static str)> {
    match game_id {
        "genshin" => Some(("gopR6Cufr3", "hk4e_global")),
        "hsr" => Some(("4ziysqXOQ8", "hkrpg_global")),
        "zzz" => Some(("U5hbdsT9W7", "nap_global")),
        "hi3" => Some(("5TIVvvcwtM", "bh3_global")),
        _ => None,
    }
}

fn number(value: Option<&Value>) -> u64 {
    let Some(value) = value else { return 0 };
    if let Some(n) = value.as_u64() {
        return n;
    }
    if let Some(n) = value.as_i64() {
        return n.max(0) as u64;
    }
    value.as_str().and_then(|s| s.parse().ok()).unwrap_or(0)
}

fn package_name(url: &str, index: usize) -> String {
    let without_query = url.split('?').next().unwrap_or(url);
    let file = without_query.rsplit('/').next().unwrap_or("").trim();
    if file.is_empty() {
        format!("package-{index}.bin")
    } else {
        file.to_string()
    }
}

pub async fn fetch_download_plan(game_id: &str) -> Result<ProviderDownloadPlan, ProviderError> {
    let (provider_game_id, biz) = game_identity(game_id)
        .ok_or_else(|| ProviderError::GameNotFound(game_id.into()))?;

    let client = reqwest::Client::builder()
        .user_agent("GachaHub/0.14.1")
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| ProviderError::Network(e.to_string()))?;

    let response = client
        .get(GLOBAL_PACKAGES_API)
        .send()
        .await
        .map_err(|e| ProviderError::Network(e.to_string()))?
        .error_for_status()
        .map_err(|e| ProviderError::Network(e.to_string()))?;

    let root: Value = response
        .json()
        .await
        .map_err(|e| ProviderError::InvalidResponse(e.to_string()))?;

    let entries = root
        .pointer("/data/game_packages")
        .and_then(Value::as_array)
        .ok_or_else(|| ProviderError::InvalidResponse("data.game_packages missing".into()))?;

    let entry = entries
        .iter()
        .find(|entry| {
            entry.pointer("/game/id").and_then(Value::as_str) == Some(provider_game_id)
                || entry.pointer("/game/biz").and_then(Value::as_str) == Some(biz)
        })
        .ok_or_else(|| ProviderError::GameNotFound(game_id.into()))?;

    let major = entry
        .pointer("/main/major")
        .ok_or_else(|| ProviderError::InvalidResponse("main.major missing".into()))?;

    let version = major
        .get("version")
        .and_then(Value::as_str)
        .unwrap_or("unknown")
        .to_string();

    let resource_list_url = major
        .get("res_list_url")
        .or_else(|| major.get("resListUrl"))
        .and_then(Value::as_str)
        .map(ToString::to_string)
        .filter(|s| !s.is_empty());

    let mut packages = Vec::new();
    if let Some(values) = major.get("game_pkgs").and_then(Value::as_array) {
        for (index, package) in values.iter().enumerate() {
            let Some(url) = package.get("url").and_then(Value::as_str) else { continue };
            if url.trim().is_empty() {
                continue;
            }
            let size = number(package.get("size"))
                .max(number(package.get("package_size")))
                .max(number(package.get("decompressed_size")));
            let hash = package
                .get("md5")
                .and_then(Value::as_str)
                .map(ToString::to_string)
                .filter(|s| !s.is_empty());
            packages.push(PackageFile {
                url: url.to_string(),
                destination: package_name(url, index),
                size,
                hash,
            });
        }
    }

    let total_bytes = packages.iter().map(|p| p.size).sum();
    let mode = if !packages.is_empty() {
        "direct"
    } else if resource_list_url.is_some() {
        "sophon"
    } else {
        "metadata-only"
    };

    let mut notes = vec![format!("HoYoPlay metadata resolved for {biz}")];
    if mode == "sophon" {
        notes.push("This build uses HoYoPlay Sophon chunk distribution. The v0.9 queue resolves the manifest; chunk assembly is the next downloader milestone.".into());
    }
    if !packages.is_empty() {
        notes.push(format!("{} direct package(s) available", packages.len()));
    }

    Ok(ProviderDownloadPlan {
        game_id: game_id.into(),
        provider: "hoyoverse".into(),
        version,
        mode: mode.into(),
        packages,
        total_bytes,
        resource_list_url,
        notes,
    })
}

impl GameProvider for HoyoverseProvider {
    fn id(&self) -> &'static str { "hoyoverse" }
    fn supports(&self, game_id: &str) -> bool { matches!(game_id, "genshin" | "hsr" | "zzz" | "hi3") }
}
