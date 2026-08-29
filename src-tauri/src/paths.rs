use std::path::PathBuf;

pub fn app_data_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Full Steam Ahead")
}

pub fn logs_dir() -> PathBuf {
    app_data_dir().join("logs")
}

pub fn backups_dir() -> PathBuf {
    app_data_dir().join("backups")
}

pub fn settings_path() -> PathBuf {
    app_data_dir().join("settings.json")
}
