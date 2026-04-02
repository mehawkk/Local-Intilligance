use std::process::Command;

#[tauri::command]
pub fn open_file(path: String) -> Result<(), String> {
    // Use Windows 'start' command to open file with default application
    Command::new("cmd")
        .args(["/C", "start", "", &path])
        .spawn()
        .map_err(|e| format!("Failed to open file: {}", e))?;
    Ok(())
}

#[tauri::command]
pub fn open_containing_folder(path: String) -> Result<(), String> {
    // Use explorer.exe /select to open folder with file selected
    Command::new("explorer.exe")
        .args(["/select,", &path])
        .spawn()
        .map_err(|e| format!("Failed to open folder: {}", e))?;
    Ok(())
}
