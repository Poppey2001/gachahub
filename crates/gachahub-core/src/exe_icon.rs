//! Extract the embedded application icon from a Windows game executable.
//!
//! Works on every platform (the PE format is parsed directly), so the real
//! game icon also shows up for games run through Wine/Proton on Linux.

use pelite::{FileMap, PeFile};
use std::{io::Cursor, path::Path};

/// Read the largest icon of `exe` and store it as a PNG at `out`.
pub fn extract_png(exe: &Path, out: &Path) -> Result<(), String> {
    let map = FileMap::open(exe).map_err(|e| format!("cannot open {}: {e}", exe.display()))?;
    let file = PeFile::from_bytes(&map).map_err(|e| format!("not a Windows executable: {e}"))?;
    let resources = file.resources().map_err(|e| format!("no resources: {e}"))?;

    // The first icon group is the one Explorer shows for the file.
    let (_, group) = resources
        .icons()
        .find_map(Result::ok)
        .ok_or_else(|| "executable contains no icon".to_string())?;

    let mut ico = Vec::new();
    group.write(&mut ico).map_err(|e| e.to_string())?;
    let image = image::load(Cursor::new(ico), image::ImageFormat::Ico)
        .map_err(|e| format!("cannot decode icon: {e}"))?;

    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    image
        .save_with_format(out, image::ImageFormat::Png)
        .map_err(|e| format!("cannot save icon: {e}"))
}
