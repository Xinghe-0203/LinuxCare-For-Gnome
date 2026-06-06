use gtk::prelude::*;
use gtk::{Box, Button, Entry, Frame, Grid, Label, Orientation, ScrolledWindow, TextView};

use crate::tr;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn run_cmd(cmd: &str, args: &[&str]) -> String {
    std::process::Command::new(cmd)
        .args(args)
        .output()
        .map(|o| {
            let stdout = String::from_utf8_lossy(&o.stdout).to_string();
            if o.status.success() {
                stdout
            } else {
                let stderr = String::from_utf8_lossy(&o.stderr).to_string();
                format!("{}\n{}", stdout, stderr)
            }
        })
        .unwrap_or_else(|e| format!("Error: {}", e))
}

fn home_dir() -> std::path::PathBuf {
    std::env::var("HOME")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::path::PathBuf::from("/root"))
}

fn parse_desktop_field(content: &str, field: &str) -> String {
    for line in content.lines() {
        if let Some(val) = line.strip_prefix(&format!("{}=", field)) {
            return val.trim().to_string();
        }
    }
    String::new()
}

// ---------------------------------------------------------------------------
// Section 1 – Desktop shortcuts (.desktop files on the Desktop)
// ---------------------------------------------------------------------------

fn collect_desktop_shortcut_files() -> Vec<std::path::PathBuf> {
    let home = home_dir();
    let mut files: Vec<std::path::PathBuf> = Vec::new();

    // ~/Desktop/*.desktop
    let desktop_dir = home.join("Desktop");
    if desktop_dir.is_dir() {
        if let Ok(rd) = std::fs::read_dir(&desktop_dir) {
            for entry in rd.flatten() {
                let p = entry.path();
                if p.extension().map_or(false, |e| e == "desktop") {
                    files.push(p);
                }
            }
        }
    }

    // ~/.desktop  (user home direct)
    if let Ok(rd) = std::fs::read_dir(&home) {
        for entry in rd.flatten() {
            let p = entry.path();
            if p.is_file() && p.extension().map_or(false, |e| e == "desktop") {
                files.push(p);
            }
        }
    }

    files.sort_by(|a, b| a.file_name().cmp(&b.file_name()));
    files
}

fn load_desktop_shortcuts(buffer: &gtk::TextBuffer) {
    let files = collect_desktop_shortcut_files();
    if files.is_empty() {
        buffer.set_text("No .desktop shortcuts found on Desktop.\nUse 'Add to Desktop' to create one.");
        return;
    }

    let header = format!("{:<8} {:<28} {:<40} {}", "FILE", "NAME", "EXEC", "PATH");
    let separator = "─".repeat(header.len());
    let mut lines = vec![header, separator];

    for path in &files {
        let content = std::fs::read_to_string(path).unwrap_or_default();
        let name = parse_desktop_field(&content, "Name");
        let exec = parse_desktop_field(&content, "Exec");
        let filename = path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        let display_name = if name.chars().count() > 28 {
            let truncated: String = name.chars().take(28).collect();
            format!("{}…", truncated)
        } else {
            name
        };
        let display_exec = if exec.chars().count() > 40 {
            let truncated: String = exec.chars().take(40).collect();
            format!("{}…", truncated)
        } else {
            exec
        };
        lines.push(format!(
            "{:<8} {:<28} {:<40} {}",
            filename, display_name, display_exec,
            path.display()
        ));
    }

    buffer.set_text(&lines.join("\n"));
}

fn create_desktop_shortcut(name: &str, exec: &str) -> Result<String, String> {
    let home = home_dir();
    let desktop_dir = home.join("Desktop");
    std::fs::create_dir_all(&desktop_dir)
        .map_err(|e| format!("Cannot create ~/Desktop: {}", e))?;

    let safe_name: String = name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();

    let path = desktop_dir.join(format!("{}.desktop", safe_name));
    if path.exists() {
        return Err(format!("'{}.desktop' already exists on Desktop.", safe_name));
    }

    let content = format!(
        "[Desktop Entry]\nType=Application\nName={}\nExec={}\nTerminal=false\n",
        name, exec
    );

    // Write the file
    std::fs::write(&path, &content)
        .map_err(|e| format!("Failed to write: {}", e))?;

    // Make it executable
    let _ = std::process::Command::new("chmod")
        .args(["+x", path.to_str().unwrap_or("")])
        .output();

    Ok(path.display().to_string())
}

fn remove_desktop_shortcut(filename: &str) -> Result<(), String> {
    let home = home_dir();
    let desktop_dir = home.join("Desktop");
    let path = desktop_dir.join(filename);
    if !path.exists() {
        // Try home dir directly
        let home_path = home.join(filename);
        if home_path.exists() {
            return std::fs::remove_file(&home_path)
                .map_err(|e| format!("Failed to remove: {}", e));
        }
        return Err(format!("'{}' not found on Desktop or in home.", filename));
    }
    std::fs::remove_file(&path).map_err(|e| format!("Failed to remove: {}", e))
}

// ---------------------------------------------------------------------------
// Section 2 – Application Menu shortcuts (system & user .desktop files)
// ---------------------------------------------------------------------------

fn collect_app_desktop_files() -> Vec<(std::path::PathBuf, String)> {
    let mut files: Vec<(std::path::PathBuf, String)> = Vec::new();
    let home = home_dir();

    let dirs = vec![
        std::path::PathBuf::from("/usr/share/applications"),
        home.join(".local/share/applications"),
    ];

    for dir in &dirs {
        if dir.is_dir() {
            if let Ok(rd) = std::fs::read_dir(dir) {
                for entry in rd.flatten() {
                    let p = entry.path();
                    if p.extension().map_or(false, |e| e == "desktop") {
                        let source = dir
                            .file_name()
                            .map(|s| s.to_string_lossy().to_string())
                            .unwrap_or_else(|| "unknown".to_string());
                        files.push((p, source));
                    }
                }
            }
        }
    }

    files.sort_by(|a, b| a.0.file_name().cmp(&b.0.file_name()));
    files
}

fn load_app_menu_list(buffer: &gtk::TextBuffer) {
    let files = collect_app_desktop_files();
    if files.is_empty() {
        buffer.set_text("No .desktop files found in application directories.");
        return;
    }

    let header = format!(
        "{:<40} {:<30} {:<10}",
        "FILE", "NAME", "SOURCE"
    );
    let separator = "─".repeat(header.len());
    let mut lines = vec![header, separator];

    for (path, source) in &files {
        let content = std::fs::read_to_string(path).unwrap_or_default();
        let name = parse_desktop_field(&content, "Name");
        let filename = path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        let display_name = if name.chars().count() > 30 {
            let truncated: String = name.chars().take(30).collect();
            format!("{}…", truncated)
        } else {
            name
        };
        lines.push(format!(
            "{:<40} {:<30} {:<10}",
            filename, display_name, source
        ));
    }

    buffer.set_text(&lines.join("\n"));
}

fn copy_app_to_desktop(filename: &str) -> Result<String, String> {
    let home = home_dir();
    let desktop_dir = home.join("Desktop");
    std::fs::create_dir_all(&desktop_dir)
        .map_err(|e| format!("Cannot create ~/Desktop: {}", e))?;

    // Search for the source file
    let search_dirs = vec![
        std::path::PathBuf::from("/usr/share/applications"),
        home.join(".local/share/applications"),
    ];

    let mut source_path = None;
    for dir in &search_dirs {
        let candidate = dir.join(filename);
        if candidate.exists() {
            source_path = Some(candidate);
            break;
        }
    }

    let src = source_path
        .ok_or_else(|| format!("'{}' not found in application directories.", filename))?;

    let dest = desktop_dir.join(filename);
    std::fs::copy(&src, &dest).map_err(|e| format!("Failed to copy: {}", e))?;

    // Make executable
    let _ = std::process::Command::new("chmod")
        .args(["+x", dest.to_str().unwrap_or("")])
        .output();

    Ok(dest.display().to_string())
}

fn remove_app_from_desktop(filename: &str) -> Result<(), String> {
    let home = home_dir();
    let path = home.join("Desktop").join(filename);
    if !path.exists() {
        return Err(format!("'{}' not found on Desktop.", filename));
    }
    std::fs::remove_file(&path).map_err(|e| format!("Failed to remove: {}", e))
}

// ---------------------------------------------------------------------------
// Section 3 – Custom keyboard shortcuts (gsettings)
// ---------------------------------------------------------------------------

fn get_custom_keybindings() -> Vec<(String, String, String)> {
    // Read the list of custom keybinding paths
    let output = run_cmd(
        "gsettings",
        &[
            "get",
            "org.gnome.settings-daemon.plugins.media-keys",
            "custom-keybindings",
        ],
    );

    let trimmed = output.trim();
    if trimmed.is_empty() || trimmed == "@as []" || trimmed.starts_with("Error") {
        return Vec::new();
    }

    // Parse the gsettings output: ['/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/custom0/', ...]
    let inner = trimmed
        .trim_start_matches('[')
        .trim_end_matches(']');
    if inner.is_empty() {
        return Vec::new();
    }

    let mut results = Vec::new();
    for part in inner.split(',') {
        let path = part.trim().trim_matches('\'').trim().trim_matches('"');
        if path.is_empty() {
            continue;
        }

        let schema_path = format!(
            "org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:{}",
            path.trim_end_matches('/')
        );
        let name_out = run_cmd("gsettings", &["get", &schema_path, "name"]);
        let binding_out = run_cmd("gsettings", &["get", &schema_path, "binding"]);
        let command_out = run_cmd("gsettings", &["get", &schema_path, "command"]);

        let name = name_out.trim().trim_matches('\'').to_string();
        let binding = binding_out.trim().trim_matches('\'').to_string();
        let command = command_out.trim().trim_matches('\'').to_string();

        // Skip entries with empty names (deleted shortcuts)
        if name.is_empty() {
            continue;
        }

        results.push((name, binding, command));
    }

    results
}

fn load_custom_shortcuts(buffer: &gtk::TextBuffer) {
    let shortcuts = get_custom_keybindings();
    if shortcuts.is_empty() {
        buffer.set_text("No custom keyboard shortcuts configured.\nUse 'Add Shortcut' to create one.");
        return;
    }

    let header = format!(
        "{:<28} {:<20} {}",
        "NAME", "BINDING", "COMMAND"
    );
    let separator = "─".repeat(header.len());
    let mut lines = vec![header, separator];

    for (name, binding, command) in &shortcuts {
        let display_name = if name.chars().count() > 28 {
            let truncated: String = name.chars().take(28).collect();
            format!("{}…", truncated)
        } else {
            name.clone()
        };
        let display_binding = if binding.chars().count() > 20 {
            let truncated: String = binding.chars().take(20).collect();
            format!("{}…", truncated)
        } else {
            binding.clone()
        };
        lines.push(format!(
            "{:<28} {:<20} {}",
            display_name, display_binding, command
        ));
    }

    buffer.set_text(&lines.join("\n"));
}

fn find_next_custom_path() -> String {
    let base = "/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings";

    let output = run_cmd(
        "gsettings",
        &[
            "get",
            "org.gnome.settings-daemon.plugins.media-keys",
            "custom-keybindings",
        ],
    );

    let trimmed = output.trim();
    let mut used_indices: Vec<u32> = Vec::new();

    if !trimmed.is_empty() && trimmed != "@as []" && !trimmed.starts_with("Error") {
        let inner = trimmed
            .trim_start_matches('[')
            .trim_end_matches(']');
        for part in inner.split(',') {
            let path = part.trim().trim_matches('\'').trim().trim_matches('"');
            if let Some(suffix) = path.strip_prefix(&format!("{}/custom", base)) {
                if let Ok(idx) = suffix.trim_end_matches('/').parse::<u32>() {
                    used_indices.push(idx);
                }
            }
        }
    }

    used_indices.sort();
    let mut next = 0u32;
    for &used in &used_indices {
        if used == next {
            next += 1;
        } else {
            break;
        }
    }

    format!("{}/custom{}/", base, next)
}

fn add_custom_shortcut(name: &str, command: &str, binding: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("Name cannot be empty.".to_string());
    }
    if command.is_empty() {
        return Err("Command cannot be empty.".to_string());
    }

    let new_path = find_next_custom_path();

    // Get existing list
    let output = run_cmd(
        "gsettings",
        &[
            "get",
            "org.gnome.settings-daemon.plugins.media-keys",
            "custom-keybindings",
        ],
    );
    let trimmed = output.trim();
    let mut entries: Vec<String> = Vec::new();

    if !trimmed.is_empty() && trimmed != "@as []" && !trimmed.starts_with("Error") {
        let inner = trimmed
            .trim_start_matches('[')
            .trim_end_matches(']');
        for part in inner.split(',') {
            let path = part.trim().trim_matches('\'').trim().trim_matches('"');
            if !path.is_empty() {
                entries.push(format!("'{}'", path));
            }
        }
    }

    // Add new entry
    entries.push(format!("'{}'", new_path));

    let list_str = format!("[{}]", entries.join(", "));

    // Set the list
    let _ = run_cmd(
        "gsettings",
        &[
            "set",
            "org.gnome.settings-daemon.plugins.media-keys",
            "custom-keybindings",
            &list_str,
        ],
    );

    // Set name
    let schema_path = format!(
        "org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:{}",
        new_path.trim_end_matches('/')
    );
    let _ = run_cmd("gsettings", &["set", &schema_path, "name", name]);

    // Set command
    let _ = run_cmd("gsettings", &["set", &schema_path, "command", command]);

    // Set binding (skip if empty)
    if !binding.is_empty() {
        let _ = run_cmd("gsettings", &["set", &schema_path, "binding", binding]);
    }

    Ok(())
}

fn remove_custom_shortcut(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("Enter the shortcut name to remove.".to_string());
    }

    // Find the path for the given name
    let output = run_cmd(
        "gsettings",
        &[
            "get",
            "org.gnome.settings-daemon.plugins.media-keys",
            "custom-keybindings",
        ],
    );
    let trimmed = output.trim();

    if trimmed.is_empty() || trimmed == "@as []" || trimmed.starts_with("Error") {
        return Err("No custom shortcuts configured.".to_string());
    }

    let _base = "/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings";
    let inner = trimmed
        .trim_start_matches('[')
        .trim_end_matches(']');
    let mut target_path: Option<String> = None;
    let mut remaining: Vec<String> = Vec::new();

    for part in inner.split(',') {
        let path = part.trim().trim_matches('\'').trim().trim_matches('"');
        if path.is_empty() {
            continue;
        }

        let schema_path = format!(
            "org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:{}",
            path.trim_end_matches('/')
        );
        let name_out = run_cmd("gsettings", &["get", &schema_path, "name"]);
        let existing_name = name_out.trim().trim_matches('\'').to_string();

        if existing_name == name && target_path.is_none() {
            target_path = Some(path.to_string());
            // Delete the shortcut entry
            let _ = run_cmd("gsettings", &["set", &schema_path, "name", ""]);
            let _ = run_cmd("gsettings", &["set", &schema_path, "command", ""]);
            let _ = run_cmd("gsettings", &["set", &schema_path, "binding", ""]);
        } else {
            remaining.push(format!("'{}'", path));
        }
    }

    if target_path.is_none() {
        return Err(format!("Shortcut '{}' not found.", name));
    }

    let list_str = if remaining.is_empty() {
        "[]".to_string()
    } else {
        format!("[{}]", remaining.join(", "))
    };

    let _ = run_cmd(
        "gsettings",
        &[
            "set",
            "org.gnome.settings-daemon.plugins.media-keys",
            "custom-keybindings",
            &list_str,
        ],
    );

    Ok(())
}

// ---------------------------------------------------------------------------
// Build the page
// ---------------------------------------------------------------------------

pub fn build_shortcut_page() -> gtk::Widget {
    let main_box = Box::new(Orientation::Vertical, 20);
    main_box.set_margin_top(20);
    main_box.set_margin_bottom(20);
    main_box.set_margin_start(20);
    main_box.set_margin_end(20);

    let title = Label::new(Some(tr!("shortcuts")));
    title.set_css_classes(&["title-1"]);
    main_box.append(&title);

    // ═════════════════════════════════════════════════════════════════════
    // Section 1 – Desktop Shortcuts (.desktop files on Desktop)
    // ═════════════════════════════════════════════════════════════════════

    let desktop_frame = Frame::new(Some(tr!("desktop_shortcuts")));
    desktop_frame.set_css_classes(&["card"]);

    let desktop_grid = Grid::new();
    desktop_grid.set_column_spacing(10);
    desktop_grid.set_row_spacing(10);
    desktop_grid.set_margin_top(10);
    desktop_grid.set_margin_bottom(10);
    desktop_grid.set_margin_start(10);
    desktop_grid.set_margin_end(10);

    // Add shortcut fields
    let add_name_label = Label::new(Some(tr!("app_name")));
    add_name_label.set_halign(gtk::Align::Start);
    let add_name_entry = Entry::new();
    add_name_entry.set_hexpand(true);
    add_name_entry.set_placeholder_text(Some("My App"));

    let add_exec_label = Label::new(Some(tr!("command")));
    add_exec_label.set_halign(gtk::Align::Start);
    let add_exec_entry = Entry::new();
    add_exec_entry.set_hexpand(true);
    add_exec_entry.set_placeholder_text(Some("/usr/bin/myapp --flag"));

    let add_desktop_button = Button::with_label(&tr!("add_to_desktop"));
    add_desktop_button.set_halign(gtk::Align::Start);
    add_desktop_button.set_css_classes(&["suggested-action"]);

    desktop_grid.attach(&add_name_label, 0, 0, 1, 1);
    desktop_grid.attach(&add_name_entry, 1, 0, 3, 1);
    desktop_grid.attach(&add_exec_label, 0, 1, 1, 1);
    desktop_grid.attach(&add_exec_entry, 1, 1, 2, 1);
    desktop_grid.attach(&add_desktop_button, 3, 1, 1, 1);

    // Remove shortcut field
    let rm_desktop_label = Label::new(Some(tr!("file_name")));
    rm_desktop_label.set_halign(gtk::Align::Start);
    let rm_desktop_entry = Entry::new();
    rm_desktop_entry.set_hexpand(true);
    rm_desktop_entry.set_placeholder_text(Some("myapp.desktop"));

    let remove_desktop_button = Button::with_label(&tr!("remove_from_desktop"));
    remove_desktop_button.set_halign(gtk::Align::Start);
    remove_desktop_button.set_css_classes(&["destructive-action"]);

    let refresh_desktop_button = Button::with_label(&tr!("refresh"));
    refresh_desktop_button.set_halign(gtk::Align::Start);

    desktop_grid.attach(&rm_desktop_label, 0, 2, 1, 1);
    desktop_grid.attach(&rm_desktop_entry, 1, 2, 2, 1);
    desktop_grid.attach(&remove_desktop_button, 3, 2, 1, 1);
    desktop_grid.attach(&refresh_desktop_button, 4, 2, 1, 1);

    // Desktop shortcuts list
    let desktop_scrolled = ScrolledWindow::new();
    desktop_scrolled.set_min_content_height(200);
    desktop_scrolled.set_vexpand(true);

    let desktop_text_view = TextView::new();
    desktop_text_view.set_editable(false);
    desktop_text_view.set_monospace(true);
    desktop_text_view.set_wrap_mode(gtk::WrapMode::None);
    desktop_text_view.set_left_margin(8);
    desktop_text_view.set_top_margin(8);

    let desktop_buffer = desktop_text_view.buffer();
    desktop_buffer.set_text(tr!("loading"));

    desktop_scrolled.set_child(Some(&desktop_text_view));
    desktop_grid.attach(&desktop_scrolled, 0, 3, 5, 1);

    desktop_frame.set_child(Some(&desktop_grid));
    main_box.append(&desktop_frame);

    // ═════════════════════════════════════════════════════════════════════
    // Section 2 – Application Menu Shortcuts
    // ═════════════════════════════════════════════════════════════════════

    let menu_frame = Frame::new(Some(tr!("app_menu")));
    menu_frame.set_css_classes(&["card"]);

    let menu_grid = Grid::new();
    menu_grid.set_column_spacing(10);
    menu_grid.set_row_spacing(10);
    menu_grid.set_margin_top(10);
    menu_grid.set_margin_bottom(10);
    menu_grid.set_margin_start(10);
    menu_grid.set_margin_end(10);

    let menu_file_label = Label::new(Some(tr!("file_name")));
    menu_file_label.set_halign(gtk::Align::Start);
    let menu_file_entry = Entry::new();
    menu_file_entry.set_hexpand(true);
    menu_file_entry.set_placeholder_text(Some("firefox.desktop"));

    let copy_to_desktop_button = Button::with_label(&tr!("add_to_desktop"));
    copy_to_desktop_button.set_halign(gtk::Align::Start);
    copy_to_desktop_button.set_css_classes(&["suggested-action"]);

    let remove_from_desktop_button = Button::with_label(&tr!("remove_from_desktop"));
    remove_from_desktop_button.set_halign(gtk::Align::Start);
    remove_from_desktop_button.set_css_classes(&["destructive-action"]);

    let refresh_menu_button = Button::with_label(&tr!("refresh"));
    refresh_menu_button.set_halign(gtk::Align::Start);

    menu_grid.attach(&menu_file_label, 0, 0, 1, 1);
    menu_grid.attach(&menu_file_entry, 1, 0, 3, 1);
    menu_grid.attach(&copy_to_desktop_button, 0, 1, 1, 1);
    menu_grid.attach(&remove_from_desktop_button, 1, 1, 1, 1);
    menu_grid.attach(&refresh_menu_button, 2, 1, 1, 1);

    // App menu list
    let menu_scrolled = ScrolledWindow::new();
    menu_scrolled.set_min_content_height(200);
    menu_scrolled.set_vexpand(true);

    let menu_text_view = TextView::new();
    menu_text_view.set_editable(false);
    menu_text_view.set_monospace(true);
    menu_text_view.set_wrap_mode(gtk::WrapMode::None);
    menu_text_view.set_left_margin(8);
    menu_text_view.set_top_margin(8);

    let menu_buffer = menu_text_view.buffer();
    menu_buffer.set_text(tr!("loading"));

    menu_scrolled.set_child(Some(&menu_text_view));
    menu_grid.attach(&menu_scrolled, 0, 2, 4, 1);

    menu_frame.set_child(Some(&menu_grid));
    main_box.append(&menu_frame);

    // ═════════════════════════════════════════════════════════════════════
    // Section 3 – Custom Keyboard Shortcuts (gsettings)
    // ═════════════════════════════════════════════════════════════════════

    let kb_frame = Frame::new(Some("Custom Keyboard Shortcuts"));
    kb_frame.set_css_classes(&["card"]);

    let kb_grid = Grid::new();
    kb_grid.set_column_spacing(10);
    kb_grid.set_row_spacing(10);
    kb_grid.set_margin_top(10);
    kb_grid.set_margin_bottom(10);
    kb_grid.set_margin_start(10);
    kb_grid.set_margin_end(10);

    // Add shortcut fields
    let kb_name_label = Label::new(Some("Name:"));
    kb_name_label.set_halign(gtk::Align::Start);
    let kb_name_entry = Entry::new();
    kb_name_entry.set_hexpand(true);
    kb_name_entry.set_placeholder_text(Some("Launch Terminal"));

    let kb_command_label = Label::new(Some(tr!("command")));
    kb_command_label.set_halign(gtk::Align::Start);
    let kb_command_entry = Entry::new();
    kb_command_entry.set_hexpand(true);
    kb_command_entry.set_placeholder_text(Some("gnome-terminal"));

    let kb_binding_label = Label::new(Some("Binding:"));
    kb_binding_label.set_halign(gtk::Align::Start);
    let kb_binding_entry = Entry::new();
    kb_binding_entry.set_hexpand(true);
    kb_binding_entry.set_placeholder_text(Some("<Control><Alt>t"));

    let add_kb_button = Button::with_label("Add Shortcut");
    add_kb_button.set_halign(gtk::Align::Start);
    add_kb_button.set_css_classes(&["suggested-action"]);

    kb_grid.attach(&kb_name_label, 0, 0, 1, 1);
    kb_grid.attach(&kb_name_entry, 1, 0, 1, 1);
    kb_grid.attach(&kb_command_label, 2, 0, 1, 1);
    kb_grid.attach(&kb_command_entry, 3, 0, 1, 1);
    kb_grid.attach(&kb_binding_label, 0, 1, 1, 1);
    kb_grid.attach(&kb_binding_entry, 1, 1, 1, 1);
    kb_grid.attach(&add_kb_button, 3, 1, 1, 1);

    // Remove shortcut field
    let rm_kb_label = Label::new(Some("Name:"));
    rm_kb_label.set_halign(gtk::Align::Start);
    let rm_kb_entry = Entry::new();
    rm_kb_entry.set_hexpand(true);
    rm_kb_entry.set_placeholder_text(Some("Launch Terminal"));

    let rm_kb_button = Button::with_label("Remove Shortcut");
    rm_kb_button.set_halign(gtk::Align::Start);
    rm_kb_button.set_css_classes(&["destructive-action"]);

    let refresh_kb_button = Button::with_label(&tr!("refresh"));
    refresh_kb_button.set_halign(gtk::Align::Start);

    kb_grid.attach(&rm_kb_label, 0, 2, 1, 1);
    kb_grid.attach(&rm_kb_entry, 1, 2, 1, 1);
    kb_grid.attach(&rm_kb_button, 3, 2, 1, 1);
    kb_grid.attach(&refresh_kb_button, 2, 2, 1, 1);

    // Keyboard shortcuts list
    let kb_scrolled = ScrolledWindow::new();
    kb_scrolled.set_min_content_height(200);
    kb_scrolled.set_vexpand(true);

    let kb_text_view = TextView::new();
    kb_text_view.set_editable(false);
    kb_text_view.set_monospace(true);
    kb_text_view.set_wrap_mode(gtk::WrapMode::None);
    kb_text_view.set_left_margin(8);
    kb_text_view.set_top_margin(8);

    let kb_buffer = kb_text_view.buffer();
    kb_buffer.set_text(tr!("loading"));

    kb_scrolled.set_child(Some(&kb_text_view));
    kb_grid.attach(&kb_scrolled, 0, 3, 4, 1);

    kb_frame.set_child(Some(&kb_grid));
    main_box.append(&kb_frame);

    // ═════════════════════════════════════════════════════════════════════
    // Signal handlers – Section 1: Desktop Shortcuts
    // ═════════════════════════════════════════════════════════════════════

    // Add to Desktop
    {
        let desktop_buffer_c = desktop_buffer.clone();
        let add_name_entry_c = add_name_entry.clone();
        let add_exec_entry_c = add_exec_entry.clone();
        add_desktop_button.connect_clicked(move |_| {
            let name = add_name_entry_c.text().to_string();
            let exec = add_exec_entry_c.text().to_string();
            if name.is_empty() || exec.is_empty() {
                desktop_buffer_c.set_text("Error: Name and command are required.");
                return;
            }
            match create_desktop_shortcut(&name, &exec) {
                Ok(path) => {
                    desktop_buffer_c.set_text(&format!("Created shortcut: {}", path));
                    add_name_entry_c.set_text("");
                    add_exec_entry_c.set_text("");
                    load_desktop_shortcuts(&desktop_buffer_c);
                }
                Err(e) => {
                    desktop_buffer_c.set_text(&format!("Error: {}", e));
                }
            }
        });
    }

    // Remove from Desktop
    {
        let desktop_buffer_c = desktop_buffer.clone();
        let rm_desktop_entry_c = rm_desktop_entry.clone();
        remove_desktop_button.connect_clicked(move |_| {
            let filename = rm_desktop_entry_c.text().to_string();
            if filename.is_empty() {
                desktop_buffer_c.set_text("Error: Enter a file name to remove.");
                return;
            }
            match remove_desktop_shortcut(&filename) {
                Ok(()) => {
                    desktop_buffer_c.set_text(&format!("Removed: {}", filename));
                    rm_desktop_entry_c.set_text("");
                    load_desktop_shortcuts(&desktop_buffer_c);
                }
                Err(e) => {
                    desktop_buffer_c.set_text(&format!("Error: {}", e));
                }
            }
        });
    }

    // Refresh Desktop Shortcuts
    {
        let desktop_buffer_c = desktop_buffer.clone();
        refresh_desktop_button.connect_clicked(move |_| {
            load_desktop_shortcuts(&desktop_buffer_c);
        });
    }

    // ═════════════════════════════════════════════════════════════════════
    // Signal handlers – Section 2: Application Menu
    // ═════════════════════════════════════════════════════════════════════

    // Copy to Desktop
    {
        let desktop_buffer_c = desktop_buffer.clone();
        let menu_buffer_c = menu_buffer.clone();
        let menu_file_entry_c = menu_file_entry.clone();
        copy_to_desktop_button.connect_clicked(move |_| {
            let filename = menu_file_entry_c.text().to_string();
            if filename.is_empty() {
                menu_buffer_c.set_text("Error: Enter a .desktop file name.");
                return;
            }
            match copy_app_to_desktop(&filename) {
                Ok(path) => {
                    menu_buffer_c.set_text(&format!("Copied to desktop: {}", path));
                    load_desktop_shortcuts(&desktop_buffer_c);
                }
                Err(e) => {
                    menu_buffer_c.set_text(&format!("Error: {}", e));
                }
            }
        });
    }

    // Remove from Desktop (via app menu section)
    {
        let desktop_buffer_c = desktop_buffer.clone();
        let menu_buffer_c = menu_buffer.clone();
        let menu_file_entry_c = menu_file_entry.clone();
        remove_from_desktop_button.connect_clicked(move |_| {
            let filename = menu_file_entry_c.text().to_string();
            if filename.is_empty() {
                menu_buffer_c.set_text("Error: Enter a .desktop file name to remove from desktop.");
                return;
            }
            match remove_app_from_desktop(&filename) {
                Ok(()) => {
                    menu_buffer_c.set_text(&format!("Removed {} from Desktop.", filename));
                    load_desktop_shortcuts(&desktop_buffer_c);
                }
                Err(e) => {
                    menu_buffer_c.set_text(&format!("Error: {}", e));
                }
            }
        });
    }

    // Refresh App Menu
    {
        let menu_buffer_c = menu_buffer.clone();
        refresh_menu_button.connect_clicked(move |_| {
            load_app_menu_list(&menu_buffer_c);
        });
    }

    // ═════════════════════════════════════════════════════════════════════
    // Signal handlers – Section 3: Custom Keyboard Shortcuts
    // ═════════════════════════════════════════════════════════════════════

    // Add keyboard shortcut
    {
        let kb_buffer_c = kb_buffer.clone();
        let kb_name_entry_c = kb_name_entry.clone();
        let kb_command_entry_c = kb_command_entry.clone();
        let kb_binding_entry_c = kb_binding_entry.clone();
        add_kb_button.connect_clicked(move |_| {
            let name = kb_name_entry_c.text().to_string();
            let command = kb_command_entry_c.text().to_string();
            let binding = kb_binding_entry_c.text().to_string();

            if name.is_empty() || command.is_empty() {
                kb_buffer_c.set_text("Error: Name and command are required.");
                return;
            }

            match add_custom_shortcut(&name, &command, &binding) {
                Ok(()) => {
                    kb_buffer_c.set_text(&format!("Added shortcut: {}", name));
                    kb_name_entry_c.set_text("");
                    kb_command_entry_c.set_text("");
                    kb_binding_entry_c.set_text("");
                    load_custom_shortcuts(&kb_buffer_c);
                }
                Err(e) => {
                    kb_buffer_c.set_text(&format!("Error: {}", e));
                }
            }
        });
    }

    // Remove keyboard shortcut
    {
        let kb_buffer_c = kb_buffer.clone();
        let rm_kb_entry_c = rm_kb_entry.clone();
        rm_kb_button.connect_clicked(move |_| {
            let name = rm_kb_entry_c.text().to_string();
            if name.is_empty() {
                kb_buffer_c.set_text("Error: Enter the shortcut name to remove.");
                return;
            }
            match remove_custom_shortcut(&name) {
                Ok(()) => {
                    kb_buffer_c.set_text(&format!("Removed shortcut: {}", name));
                    rm_kb_entry_c.set_text("");
                    load_custom_shortcuts(&kb_buffer_c);
                }
                Err(e) => {
                    kb_buffer_c.set_text(&format!("Error: {}", e));
                }
            }
        });
    }

    // Refresh keyboard shortcuts
    {
        let kb_buffer_c = kb_buffer.clone();
        refresh_kb_button.connect_clicked(move |_| {
            load_custom_shortcuts(&kb_buffer_c);
        });
    }

    // ═════════════════════════════════════════════════════════════════════
    // Initial load
    // ═════════════════════════════════════════════════════════════════════

    load_desktop_shortcuts(&desktop_buffer);
    load_app_menu_list(&menu_buffer);
    load_custom_shortcuts(&kb_buffer);

    main_box.upcast()
}
