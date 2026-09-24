# GachaHub Download Integration v0.9

v0.9 replaces the old fake install progress with the first real backend download path.

## HoYoverse provider

For Genshin Impact, Honkai: Star Rail, Zenless Zone Zero and Honkai Impact 3rd, GachaHub queries HoYoPlay package metadata through the global HoYoPlay package endpoint.

The resolver reads:

- game/provider ID
- current major version
- direct game package URLs when exposed
- package size
- package MD5 when exposed
- `res_list_url` for Sophon-based distributions

## Direct package downloader

When the provider exposes direct package URLs, GachaHub now performs real HTTP downloads in Rust.

Implemented in v0.9:

- real backend HTTP requests
- `.part` files
- HTTP Range resume
- pause/resume in the current session
- cancel
- download speed
- byte progress
- Tauri progress events
- package-size validation
- MD5 verification when HoYoPlay supplies a package MD5
- downloaded package cache under `<game install path>/.gachahub-downloads/`

## Sophon

Current HoYoPlay versions may use the Sophon chunk distribution method. GachaHub v0.9 detects this mode and returns the real version/resource-list metadata instead of showing fake progress.

The next downloader milestone is the native GachaHub Sophon engine:

1. resolve Sophon resource/category metadata
2. fetch manifest protobufs
3. enumerate deduplicated chunks
4. parallel chunk download
5. Zstandard decompression
6. chunk/hash verification
7. write chunks to target file offsets
8. full-file verification
9. update/repair reuse

No external GPL downloader is embedded in v0.9; this keeps the GachaHub implementation independent.
