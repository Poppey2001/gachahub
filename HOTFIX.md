# GachaHub v0.13.1 warning cleanup

This hotfix removes the Rust compiler warnings reported after v0.13.0.

Changes:
- removed unused `PackageFile` import from `download_manager.rs`
- removed unused `Write` import from `runners.rs`
- moved download speed baseline initialization to the point where the final HTTP resume offset is known
- platform-gated the embedded Windows/Linux Genshin post-install scripts
- removed the unused legacy synchronous `InstallPlan` API and dead provider error variants

The active HoYo download flow continues to use the asynchronous `ProviderDownloadPlan` API.
