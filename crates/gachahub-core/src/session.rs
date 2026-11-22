//! Tracks a running game for play time and "stop game".
//!
//! Many games are started through a wrapper (umu-run, a publisher launcher,
//! XXMI) that exits while the game keeps running. A session therefore follows
//! the spawned process *and* any process belonging to the game executable.

use std::{
    ffi::OsStr,
    process::Child,
    sync::{
        atomic::{AtomicBool, Ordering},
        Mutex,
    },
    time::{Duration, Instant},
};
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};

/// How long to wait for the real game process after the wrapper exited.
const GAME_APPEAR_GRACE: Duration = Duration::from_secs(25);

pub struct GameSession {
    child: Mutex<Option<Child>>,
    /// File name of the game executable, e.g. `GenshinImpact.exe`.
    exe_name: String,
    stopped: AtomicBool,
}

impl std::fmt::Debug for GameSession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GameSession")
            .field("exe_name", &self.exe_name)
            .finish()
    }
}

impl GameSession {
    pub fn new(child: Child, game_exe: &str) -> Self {
        let exe_name = std::path::Path::new(game_exe.trim())
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        Self {
            child: Mutex::new(Some(child)),
            exe_name,
            stopped: AtomicBool::new(false),
        }
    }

    /// Block until the game has ended. Returns the play time in seconds.
    pub fn wait(&self) -> u64 {
        let started = Instant::now();
        // 1) the spawned process
        loop {
            let done = match self.child.lock() {
                Ok(mut guard) => match guard.as_mut() {
                    Some(child) => !matches!(child.try_wait(), Ok(None)),
                    None => true,
                },
                Err(_) => true,
            };
            if done {
                break;
            }
            std::thread::sleep(Duration::from_millis(500));
        }
        // 2) the real game, if the wrapper handed over to it
        if !self.exe_name.is_empty() && !self.stopped.load(Ordering::Relaxed) {
            let mut sys = System::new();
            let handed_over = Instant::now();
            let mut seen = false;
            loop {
                let running = !game_pids(&mut sys, &self.exe_name).is_empty();
                seen |= running;
                if self.stopped.load(Ordering::Relaxed)
                    || (!running && (seen || handed_over.elapsed() > GAME_APPEAR_GRACE))
                {
                    break;
                }
                std::thread::sleep(Duration::from_secs(if seen { 2 } else { 1 }));
            }
        }
        started.elapsed().as_secs()
    }

    /// Terminate the wrapper and every process of the game executable.
    pub fn kill(&self) {
        self.stopped.store(true, Ordering::Relaxed);
        if let Ok(mut guard) = self.child.lock() {
            if let Some(child) = guard.as_mut() {
                let _ = child.kill();
            }
        }
        if !self.exe_name.is_empty() {
            let mut sys = System::new();
            for pid in game_pids(&mut sys, &self.exe_name) {
                if let Some(p) = sys.process(pid) {
                    p.kill();
                }
            }
        }
    }
}

/// Processes that belong to the game executable. Under Wine/Proton the Linux
/// process name is truncated (15 chars), so the command line is checked too.
fn game_pids(sys: &mut System, exe_name: &str) -> Vec<sysinfo::Pid> {
    sys.refresh_processes_specifics(
        ProcessesToUpdate::All,
        true,
        ProcessRefreshKind::nothing()
            .with_cmd(UpdateKind::OnlyIfNotSet)
            .with_exe(UpdateKind::OnlyIfNotSet),
    );
    sys.processes()
        .iter()
        .filter(|(_, p)| matches_exe(p.name(), p.cmd(), exe_name))
        .map(|(pid, _)| *pid)
        .collect()
}

fn file_name_of(arg: &str) -> &str {
    arg.rsplit(['/', '\\']).next().unwrap_or(arg)
}

pub(crate) fn matches_exe(name: &OsStr, cmd: &[std::ffi::OsString], exe_name: &str) -> bool {
    let exe = exe_name.to_ascii_lowercase();
    let name = name.to_string_lossy().to_ascii_lowercase();
    if name == exe || (name.len() >= 15 && exe.starts_with(&name)) {
        return true;
    }
    // Only the program itself (first argument), so e.g. "umu-run Game.exe" is not
    // mistaken for the game once the wrapper is gone.
    cmd.first()
        .map(|a| file_name_of(&a.to_string_lossy()).eq_ignore_ascii_case(exe_name))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;

    #[test]
    fn matches_wine_and_native_processes() {
        let cmd = |v: &[&str]| v.iter().map(OsString::from).collect::<Vec<_>>();
        // native Windows
        assert!(matches_exe(
            OsStr::new("GenshinImpact.exe"),
            &[],
            "GenshinImpact.exe"
        ));
        // Linux comm truncated to 15 chars
        assert!(matches_exe(
            OsStr::new("GenshinImpact.e"),
            &[],
            "GenshinImpact.exe"
        ));
        // Wine: windows path in argv[0]
        assert!(matches_exe(
            OsStr::new("wine64-preloader"),
            &cmd(&["C:\\Games\\Genshin Impact\\GenshinImpact.exe"]),
            "GenshinImpact.exe"
        ));
        // wrapper that merely mentions the exe is not the game
        assert!(!matches_exe(
            OsStr::new("umu-run"),
            &cmd(&["umu-run", "/g/GenshinImpact.exe"]),
            "GenshinImpact.exe"
        ));
        assert!(!matches_exe(
            OsStr::new("StarRail.exe"),
            &[],
            "GenshinImpact.exe"
        ));
    }
}
