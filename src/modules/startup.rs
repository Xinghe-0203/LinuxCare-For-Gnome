use gtk::prelude::*;
use gtk::{Box, Button, Entry, Frame, Grid, Label, Orientation, ScrolledWindow, TextView};

use crate::tr;

fn _run_command(cmd: &str, args: &[&str]) -> String {
    std::process::Command::new(cmd)
        .args(args)
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_else(|e| format!("Error: {}", e))
}

fn get_autostart_dir() -> std::path::PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/root".to_string());
    std::path::PathBuf::from(home).join(".config/autostart")
}

fn list_autostart_files() -> Vec<std::path::PathBuf> {
    let dir = get_autostart_dir();
    if !dir.is_dir() {
        return Vec::new();
    }
    let mut entries: Vec<std::path::PathBuf> = std::fs::read_dir(&dir)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .map(|ext| ext == "desktop")
                .unwrap_or(false)
        })
        .map(|e| e.path())
        .collect();
    entries.sort_by(|a, b| a.file_name().cmp(&b.file_name()));
    entries
}

fn parse_desktop_entry(path: &std::path::Path) -> (String, String, bool) {
    let content = std::fs::read_to_string(path).unwrap_or_default();
    let mut name = path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    let mut exec = String::new();
    let mut hidden = false;

    for line in content.lines() {
        if let Some(val) = line.strip_prefix("Name=") {
            name = val.trim().to_string();
        } else if let Some(val) = line.strip_prefix("Exec=") {
            exec = val.trim().to_string();
        } else if let Some(val) = line.strip_prefix("Hidden=") {
            hidden = val.trim().eq_ignore_ascii_case("true");
        }
    }

    if name.is_empty() {
        name = path
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "Unknown".to_string());
    }

    (name, exec, hidden)
}

fn load_startup_list(buffer: &gtk::TextBuffer) {
    let files = list_autostart_files();
    if files.is_empty() {
        buffer.set_text("No startup applications found in ~/.config/autostart/");
        return;
    }

    let header = format!(
        "{:<8} {:<32} {:<40} {:<10}",
        "STATUS", "NAME", "EXEC", "FILE"
    );
    let separator = "─".repeat(header.len());
    let mut lines = vec![header, separator];

    for path in &files {
        let (name, exec, hidden) = parse_desktop_entry(path);
        let status = if hidden { "DISABLED" } else { "ENABLED" };
        let filename = path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        let display_name = if name.chars().count() > 32 {
            let truncated: String = name.chars().take(31).collect();
            format!("{}…", truncated)
        } else {
            name
        };
        let display_exec = if exec.chars().count() > 40 {
            let truncated: String = exec.chars().take(39).collect();
            format!("{}…", truncated)
        } else {
            exec
        };
        lines.push(format!(
            "{:<8} {:<32} {:<40} {:<10}",
            status, display_name, display_exec, filename
        ));
    }

    buffer.set_text(&lines.join("\n"));
}

fn toggle_autostart(path: &std::path::Path, enable: bool) {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return,
    };

    let mut new_lines = Vec::new();
    let mut found_hidden = false;

    for line in content.lines() {
        if line.starts_with("Hidden=") {
            new_lines.push(format!("Hidden={}", !enable));
            found_hidden = true;
        } else {
            new_lines.push(line.to_string());
        }
    }

    if !found_hidden {
        new_lines.push(format!("Hidden={}", !enable));
    }

    let _ = std::fs::write(path, new_lines.join("\n"));
}

fn remove_autostart_entry(path: &std::path::Path) -> Result<(), String> {
    std::fs::remove_file(path).map_err(|e| format!("Failed to remove: {}", e))
}

fn create_desktop_entry(name: &str, exec: &str) -> Result<(), String> {
    let dir = get_autostart_dir();
    std::fs::create_dir_all(&dir).map_err(|e| format!("Cannot create autostart dir: {}", e))?;

    let safe_name = name
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect::<String>();

    let path = dir.join(format!("{}.desktop", safe_name));
    if path.exists() {
        return Err("Entry already exists".to_string());
    }

    let content = format!(
        "[Desktop Entry]\nType=Application\nName={}\nExec={}\nHidden=false\n",
        name, exec
    );
    std::fs::write(&path, content).map_err(|e| format!("Failed to write: {}", e))
}

pub fn build_startup_page() -> gtk::Widget {
    let main_box = Box::new(Orientation::Vertical, 20);
    main_box.set_margin_top(20);
    main_box.set_margin_bottom(20);
    main_box.set_margin_start(20);
    main_box.set_margin_end(20);

    let title = Label::new(Some(tr!("startup_apps")));
    title.set_css_classes(&["title-1"]);
    main_box.append(&title);

    let info_frame = Frame::new(Some(tr!("information")));
    info_frame.set_css_classes(&["card"]);

    let info_label = Label::new(Some(
        "Manage applications that start automatically at login.\n\
         Startup entries are stored as .desktop files in ~/.config/autostart/",
    ));
    info_label.set_halign(gtk::Align::Start);
    info_label.set_wrap(true);
    info_label.set_margin_top(10);
    info_label.set_margin_bottom(10);
    info_label.set_margin_start(12);
    info_label.set_margin_end(12);

    info_frame.set_child(Some(&info_label));
    main_box.append(&info_frame);

    let list_frame = Frame::new(Some(tr!("startup_apps")));
    list_frame.set_css_classes(&["card"]);

    let list_scrolled = ScrolledWindow::new();
    list_scrolled.set_min_content_height(300);
    list_scrolled.set_vexpand(true);

    let list_text_view = TextView::new();
    list_text_view.set_editable(false);
    list_text_view.set_monospace(true);
    list_text_view.set_wrap_mode(gtk::WrapMode::None);
    list_text_view.set_left_margin(8);
    list_text_view.set_top_margin(8);

    let list_buffer = list_text_view.buffer();
    list_buffer.set_text(tr!("loading"));

    list_scrolled.set_child(Some(&list_text_view));
    list_scrolled.set_margin_top(10);
    list_scrolled.set_margin_bottom(10);
    list_scrolled.set_margin_start(12);
    list_scrolled.set_margin_end(12);

    list_frame.set_child(Some(&list_scrolled));
    main_box.append(&list_frame);

    let actions_frame = Frame::new(Some(tr!("actions")));
    actions_frame.set_css_classes(&["card"]);

    let actions_grid = Grid::new();
    actions_grid.set_column_spacing(12);
    actions_grid.set_row_spacing(10);
    actions_grid.set_margin_top(10);
    actions_grid.set_margin_bottom(10);
    actions_grid.set_margin_start(12);
    actions_grid.set_margin_end(12);

    let add_name_label = Label::new(Some(tr!("app_name")));
    add_name_label.set_halign(gtk::Align::Start);
    let add_name_entry = Entry::new();
    add_name_entry.set_hexpand(true);
    add_name_entry.set_placeholder_text(Some("My Application"));

    let add_exec_label = Label::new(Some(tr!("command")));
    add_exec_label.set_halign(gtk::Align::Start);
    let add_exec_entry = Entry::new();
    add_exec_entry.set_hexpand(true);
    add_exec_entry.set_placeholder_text(Some("/usr/bin/myapp --option"));

    let add_button = Button::with_label(tr!("add_entry"));
    add_button.set_css_classes(&["suggested-action"]);

    let toggle_label = Label::new(Some(tr!("file_name")));
    toggle_label.set_halign(gtk::Align::Start);
    let toggle_entry = Entry::new();
    toggle_entry.set_hexpand(true);
    toggle_entry.set_placeholder_text(Some("myapp.desktop"));

    let enable_button = Button::with_label(tr!("enable"));
    enable_button.set_halign(gtk::Align::Start);
    enable_button.set_css_classes(&["suggested-action"]);

    let disable_button = Button::with_label(tr!("disable"));
    disable_button.set_halign(gtk::Align::Start);

    let remove_button = Button::with_label(tr!("remove"));
    remove_button.set_css_classes(&["destructive-action"]);

    let refresh_button = Button::with_label(tr!("refresh"));
    refresh_button.set_css_classes(&["flat"]);

    actions_grid.attach(&add_name_label, 0, 0, 1, 1);
    actions_grid.attach(&add_name_entry, 1, 0, 3, 1);
    actions_grid.attach(&add_exec_label, 0, 1, 1, 1);
    actions_grid.attach(&add_exec_entry, 1, 1, 2, 1);
    actions_grid.attach(&add_button, 3, 1, 1, 1);
    actions_grid.attach(&toggle_label, 0, 2, 1, 1);
    actions_grid.attach(&toggle_entry, 1, 2, 1, 1);
    actions_grid.attach(&enable_button, 2, 2, 1, 1);
    actions_grid.attach(&disable_button, 3, 2, 1, 1);
    actions_grid.attach(&remove_button, 1, 3, 1, 1);
    actions_grid.attach(&refresh_button, 2, 3, 1, 1);

    actions_frame.set_child(Some(&actions_grid));
    main_box.append(&actions_frame);

    let output_frame = Frame::new(Some(tr!("output")));
    output_frame.set_css_classes(&["card"]);

    let output_scrolled = ScrolledWindow::new();
    output_scrolled.set_min_content_height(80);
    output_scrolled.set_max_content_height(150);

    let output_text_view = TextView::new();
    output_text_view.set_editable(false);
    output_text_view.set_monospace(true);
    output_text_view.set_vexpand(true);

    let output_buffer = output_text_view.buffer();
    output_buffer.set_text("Ready.");

    output_scrolled.set_child(Some(&output_text_view));
    output_scrolled.set_margin_top(10);
    output_scrolled.set_margin_bottom(10);
    output_scrolled.set_margin_start(12);
    output_scrolled.set_margin_end(12);

    output_frame.set_child(Some(&output_scrolled));
    main_box.append(&output_frame);

    {
        let list_buffer_c = list_buffer.clone();
        let output_buffer_c = output_buffer.clone();
        let add_name_entry_c = add_name_entry.clone();
        let add_exec_entry_c = add_exec_entry.clone();
        add_button.connect_clicked(move |_| {
            let name = add_name_entry_c.text().to_string();
            let exec = add_exec_entry_c.text().to_string();
            if name.is_empty() || exec.is_empty() {
                output_buffer_c.set_text("Error: Name and command are required.");
                return;
            }
            match create_desktop_entry(&name, &exec) {
                Ok(()) => {
                    output_buffer_c.set_text(&format!("Added startup entry: {}", name));
                    add_name_entry_c.set_text("");
                    add_exec_entry_c.set_text("");
                    load_startup_list(&list_buffer_c);
                }
                Err(e) => {
                    output_buffer_c.set_text(&format!("Error: {}", e));
                }
            }
        });
    }

    {
        let list_buffer_c = list_buffer.clone();
        let output_buffer_c = output_buffer.clone();
        let toggle_entry_c = toggle_entry.clone();
        enable_button.connect_clicked(move |_| {
            let filename = toggle_entry_c.text().to_string();
            if filename.is_empty() {
                output_buffer_c.set_text("Error: Enter a file name to enable.");
                return;
            }
            let dir = get_autostart_dir();
            let path = dir.join(&filename);
            if !path.exists() {
                output_buffer_c.set_text(&format!("Error: File '{}' not found.", filename));
                return;
            }
            toggle_autostart(&path, true);
            output_buffer_c.set_text(&format!("Enabled: {}", filename));
            load_startup_list(&list_buffer_c);
        });
    }

    {
        let list_buffer_c = list_buffer.clone();
        let output_buffer_c = output_buffer.clone();
        let toggle_entry_c = toggle_entry.clone();
        disable_button.connect_clicked(move |_| {
            let filename = toggle_entry_c.text().to_string();
            if filename.is_empty() {
                output_buffer_c.set_text("Error: Enter a file name to disable.");
                return;
            }
            let dir = get_autostart_dir();
            let path = dir.join(&filename);
            if !path.exists() {
                output_buffer_c.set_text(&format!("Error: File '{}' not found.", filename));
                return;
            }
            toggle_autostart(&path, false);
            output_buffer_c.set_text(&format!("Disabled: {}", filename));
            load_startup_list(&list_buffer_c);
        });
    }

    {
        let list_buffer_c = list_buffer.clone();
        let output_buffer_c = output_buffer.clone();
        let toggle_entry_c = toggle_entry.clone();
        remove_button.connect_clicked(move |_| {
            let filename = toggle_entry_c.text().to_string();
            if filename.is_empty() {
                output_buffer_c.set_text("Error: Enter a file name to remove.");
                return;
            }
            let dir = get_autostart_dir();
            let path = dir.join(&filename);
            if !path.exists() {
                output_buffer_c.set_text(&format!("Error: File '{}' not found.", filename));
                return;
            }
            match remove_autostart_entry(&path) {
                Ok(()) => {
                    output_buffer_c.set_text(&format!("Removed: {}", filename));
                    toggle_entry_c.set_text("");
                    load_startup_list(&list_buffer_c);
                }
                Err(e) => {
                    output_buffer_c.set_text(&format!("Error: {}", e));
                }
            }
        });
    }

    {
        let list_buffer_c = list_buffer.clone();
        refresh_button.connect_clicked(move |_| {
            load_startup_list(&list_buffer_c);
        });
    }

    load_startup_list(&list_buffer);

    main_box.upcast()
}
