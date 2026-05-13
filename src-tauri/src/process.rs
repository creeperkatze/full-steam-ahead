use std::process::{Child, Command, Output};

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

#[cfg(windows)]
pub fn steam_process_name() -> &'static str {
    "steam.exe"
}

#[cfg(target_os = "macos")]
pub fn steam_process_name() -> &'static str {
    "steam_osx"
}

#[cfg(all(unix, not(target_os = "macos")))]
pub fn steam_process_name() -> &'static str {
    "steam"
}

#[cfg(not(any(windows, unix)))]
pub fn steam_process_name() -> &'static str {
    "steam"
}

#[cfg(windows)]
pub fn is_process_running(process_name: &str) -> bool {
    use windows_sys::Win32::{
        Foundation::{CloseHandle, INVALID_HANDLE_VALUE},
        System::Diagnostics::ToolHelp::{
            CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
            TH32CS_SNAPPROCESS,
        },
    };

    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
    if snapshot == INVALID_HANDLE_VALUE {
        return false;
    }

    let mut entry = PROCESSENTRY32W {
        dwSize: size_of::<PROCESSENTRY32W>() as u32,
        ..Default::default()
    };

    let mut found = false;
    let mut has_entry = unsafe { Process32FirstW(snapshot, &mut entry) } != 0;
    while has_entry {
        if exe_name(&entry).eq_ignore_ascii_case(process_name) {
            found = true;
            break;
        }
        has_entry = unsafe { Process32NextW(snapshot, &mut entry) } != 0;
    }

    unsafe {
        CloseHandle(snapshot);
    }

    found
}

#[cfg(not(windows))]
pub fn is_process_running(process_name: &str) -> bool {
    Command::new("pgrep")
        .args(["-x", process_name])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

pub fn stop_steam() -> std::io::Result<Output> {
    #[cfg(windows)]
    {
        return command_output_no_window(Command::new("taskkill").args([
            "/F",
            "/T",
            "/IM",
            steam_process_name(),
        ]));
    }

    #[cfg(not(windows))]
    {
        Command::new("pkill")
            .args(["-x", steam_process_name()])
            .output()
    }
}

pub fn restart_steam(install_path: &std::path::Path) -> std::io::Result<Option<Child>> {
    #[cfg(windows)]
    {
        let steam_exe = install_path.join("steam.exe");
        if steam_exe.exists() {
            return command_spawn_no_window(&mut Command::new(steam_exe)).map(Some);
        }
    }

    #[cfg(target_os = "macos")]
    {
        return Command::new("open").args(["-a", "Steam"]).spawn().map(Some);
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let steam = install_path.join("steam.sh");
        if steam.exists() {
            return Command::new(steam).spawn().map(Some);
        }
        return Command::new("steam").spawn().map(Some);
    }

    Ok(None)
}

pub fn command_output_no_window(command: &mut Command) -> std::io::Result<Output> {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(CREATE_NO_WINDOW);
    }

    command.output()
}

pub fn command_spawn_no_window(command: &mut Command) -> std::io::Result<Child> {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(CREATE_NO_WINDOW);
    }

    command.spawn()
}

#[cfg(windows)]
fn exe_name(entry: &windows_sys::Win32::System::Diagnostics::ToolHelp::PROCESSENTRY32W) -> String {
    let length = entry
        .szExeFile
        .iter()
        .position(|character| *character == 0)
        .unwrap_or(entry.szExeFile.len());

    String::from_utf16_lossy(&entry.szExeFile[..length])
}
