use crate::tr;
use gtk::prelude::*;
use gtk::{
    Box, Button, Entry, Frame, Grid, Label, Orientation, ScrolledWindow, TextView,
};

fn get_nautilus_scripts_dir() -> std::path::PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/root".to_string());
    std::path::PathBuf::from(home)
        .join(".local")
        .join("share")
        .join("nautilus")
        .join("scripts")
}

fn list_nautilus_scripts() -> Vec<String> {
    let dir = get_nautilus_scripts_dir();
    if !dir.is_dir() {
        return Vec::new();
    }
    let mut scripts: Vec<String> = std::fs::read_dir(&dir)
        .ok()
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .filter(|e| {
                    let meta = e.metadata().ok();
                    meta.is_some() && !meta.unwrap().is_dir()
                })
                .map(|e| e.file_name().to_string_lossy().to_string())
                .collect()
        })
        .unwrap_or_default();
    scripts.sort();
    scripts
}

fn read_script_content(name: &str) -> String {
    let path = get_nautilus_scripts_dir().join(name);
    std::fs::read_to_string(&path).unwrap_or_else(|e| format!("Error reading {}: {}", name, e))
}

fn write_script_content(name: &str, content: &str) -> Result<(), String> {
    let dir = get_nautilus_scripts_dir();
    std::fs::create_dir_all(&dir).map_err(|e| format!("Failed to create scripts dir: {}", e))?;
    let path = dir.join(name);
    std::fs::write(&path, content).map_err(|e| format!("Failed to write script: {}", e))?;
    // Make the script executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o755);
        std::fs::set_permissions(&path, perms)
            .map_err(|e| format!("Failed to set permissions: {}", e))?;
    }
    Ok(())
}

fn remove_script(name: &str) -> Result<(), String> {
    let path = get_nautilus_scripts_dir().join(name);
    if !path.exists() {
        return Err(format!("Script '{}' does not exist", name));
    }
    std::fs::remove_file(&path).map_err(|e| format!("Failed to remove script: {}", e))
}

fn get_nautilus_bg_setting() -> String {
    let output = std::process::Command::new("gsettings")
        .args(["get", "org.gnome.nautilus.preferences", "default-folder-viewer"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|e| format!("Error: {}", e));
    output
}

fn set_nautilus_bg_setting(value: &str) -> Result<String, String> {
    let output = std::process::Command::new("gsettings")
        .args([
            "set",
            "org.gnome.nautilus.preferences",
            "default-folder-viewer",
            value,
        ])
        .output()
        .map_err(|e| format!("gsettings error: {}", e))?;
    if output.status.success() {
        Ok(format!("Set default-folder-viewer to '{}'", value))
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        Err(format!("Failed: {}", stderr))
    }
}

pub fn build_menu_page() -> gtk::Widget {
    let main_box = Box::new(Orientation::Vertical, 20);
    main_box.set_margin_top(20);
    main_box.set_margin_bottom(20);
    main_box.set_margin_start(20);
    main_box.set_margin_end(20);

    let title = Label::new(Some(tr!("menu")));
    title.set_css_classes(&["title-1"]);
    main_box.append(&title);

    // ── Context Menu Settings ──

    let context_frame = Frame::new(Some(tr!("context_menu")));
    context_frame.set_css_classes(&["card"]);

    let context_grid = Grid::new();
    context_grid.set_column_spacing(20);
    context_grid.set_row_spacing(10);
    context_grid.set_margin_top(10);
    context_grid.set_margin_bottom(10);
    context_grid.set_margin_start(10);
    context_grid.set_margin_end(10);

    let nautilus_label = Label::new(Some(tr!("nautilus_context")));
    nautilus_label.set_halign(gtk::Align::Start);
    let current_viewer = get_nautilus_bg_setting();
    let nautilus_info = Label::new(Some(&format!(
        "Default viewer: {}",
        current_viewer
    )));
    nautilus_info.set_halign(gtk::Align::Start);
    nautilus_info.set_selectable(true);
    nautilus_info.set_wrap(true);

    let viewer_icon_label = Label::new(Some("Icon view:"));
    viewer_icon_label.set_halign(gtk::Align::Start);
    let viewer_icon_button = Button::with_label(tr!("apply"));
    viewer_icon_button.set_halign(gtk::Align::Start);
    viewer_icon_button.set_css_classes(&["suggested-action"]);

    let viewer_list_label = Label::new(Some("List view:"));
    viewer_list_label.set_halign(gtk::Align::Start);
    let viewer_list_button = Button::with_label(tr!("apply"));
    viewer_list_button.set_halign(gtk::Align::Start);
    viewer_list_button.set_css_classes(&["suggested-action"]);

    let desktop_label = Label::new(Some(tr!("desktop_context")));
    desktop_label.set_halign(gtk::Align::Start);
    let desktop_info = Label::new(Some("Nautilus scripts directory:"));
    desktop_info.set_halign(gtk::Align::Start);
    let scripts_dir = get_nautilus_scripts_dir();
    let dir_display = if scripts_dir.exists() {
        scripts_dir.to_string_lossy().to_string()
    } else {
        format!("{} (will be created on first use)", scripts_dir.to_string_lossy())
    };
    let dir_label = Label::new(Some(&dir_display));
    dir_label.set_halign(gtk::Align::Start);
    dir_label.set_selectable(true);
    dir_label.set_wrap(true);

    context_grid.attach(&nautilus_label, 0, 0, 1, 1);
    context_grid.attach(&nautilus_info, 1, 0, 2, 1);
    context_grid.attach(&viewer_icon_label, 0, 1, 1, 1);
    context_grid.attach(&viewer_icon_button, 1, 1, 1, 1);
    context_grid.attach(&viewer_list_label, 0, 2, 1, 1);
    context_grid.attach(&viewer_list_button, 1, 2, 1, 1);
    context_grid.attach(&desktop_label, 0, 3, 1, 1);
    context_grid.attach(&desktop_info, 1, 3, 1, 1);
    context_grid.attach(&dir_label, 2, 3, 1, 1);

    context_frame.set_child(Some(&context_grid));
    main_box.append(&context_frame);

    // ── Custom Menu Items ──

    let custom_frame = Frame::new(Some(tr!("custom_menu_items")));
    custom_frame.set_css_classes(&["card"]);

    let custom_grid = Grid::new();
    custom_grid.set_column_spacing(10);
    custom_grid.set_row_spacing(10);
    custom_grid.set_margin_top(10);
    custom_grid.set_margin_bottom(10);
    custom_grid.set_margin_start(10);
    custom_grid.set_margin_end(10);

    // Add script controls
    let add_label = Label::new(Some("Name:"));
    add_label.set_halign(gtk::Align::Start);
    let name_entry = Entry::new();
    name_entry.set_hexpand(true);
    name_entry.set_placeholder_text(Some("my-script"));

    let cmd_label = Label::new(Some("Command:"));
    cmd_label.set_halign(gtk::Align::Start);
    let cmd_entry = Entry::new();
    cmd_entry.set_hexpand(true);
    cmd_entry.set_placeholder_text(Some("echo \"Hello from $NAUTILUS_SCRIPT_SELECTED_FILE_PATHS\""));

    let add_item_button = Button::with_label(tr!("add_custom_item"));
    add_item_button.set_halign(gtk::Align::Start);
    add_item_button.set_css_classes(&["suggested-action"]);

    let remove_item_button = Button::with_label(tr!("remove_selected"));
    remove_item_button.set_halign(gtk::Align::Start);
    remove_item_button.set_css_classes(&["destructive-action"]);

    let refresh_button = Button::with_label(tr!("refresh"));
    refresh_button.set_halign(gtk::Align::Start);

    custom_grid.attach(&add_label, 0, 0, 1, 1);
    custom_grid.attach(&name_entry, 1, 0, 2, 1);
    custom_grid.attach(&cmd_label, 0, 1, 1, 1);
    custom_grid.attach(&cmd_entry, 1, 1, 2, 1);
    custom_grid.attach(&add_item_button, 0, 2, 1, 1);
    custom_grid.attach(&remove_item_button, 1, 2, 1, 1);
    custom_grid.attach(&refresh_button, 2, 2, 1, 1);

    // Script list
    let scrolled = ScrolledWindow::new();
    scrolled.set_min_content_height(200);

    let text_view = TextView::new();
    text_view.set_editable(false);
    text_view.set_vexpand(true);
    text_view.set_monospace(true);
    let buffer = text_view.buffer();

    scrolled.set_child(Some(&text_view));
    custom_grid.attach(&scrolled, 0, 3, 3, 1);

    custom_frame.set_child(Some(&custom_grid));
    main_box.append(&custom_frame);

    // ── Operations Output ──

    let output_frame = Frame::new(Some(tr!("operations_output")));
    output_frame.set_css_classes(&["card"]);

    let output_scrolled = ScrolledWindow::new();
    output_scrolled.set_min_content_height(100);
    output_scrolled.set_max_content_height(200);

    let output_text = TextView::new();
    output_text.set_editable(false);
    output_text.set_vexpand(true);
    output_text.set_monospace(true);
    let output_buffer = output_text.buffer();

    output_scrolled.set_child(Some(&output_text));
    output_scrolled.set_margin_top(10);
    output_scrolled.set_margin_bottom(10);
    output_scrolled.set_margin_start(10);
    output_scrolled.set_margin_end(10);

    output_frame.set_child(Some(&output_scrolled));
    main_box.append(&output_frame);

    // ── Helper to refresh the script list ──

    {
        let buffer = buffer.clone();
        let _output_buffer = output_buffer.clone();
        refresh_button.connect_clicked(move |_| {
            let scripts = list_nautilus_scripts();
            let dir = get_nautilus_scripts_dir();
            if scripts.is_empty() {
                buffer.set_text(&format!(
                    "No scripts found in:\n{}\n\nAdd a script above to get started.\n\nTip: Nautilus scripts receive environment variables:\n  $NAUTILUS_SCRIPT_SELECTED_FILE_PATHS\n  $NAUTILUS_SCRIPT_SELECTED_URIS\n  $NAUTILUS_SCRIPT_CURRENT_URI\n  $NAUTILUS_SCRIPT_WINDOW_GEOMETRY",
                    dir.display()
                ));
            } else {
                let mut text = format!(
                    "Scripts directory: {}\nFound {} script(s):\n",
                    dir.display(),
                    scripts.len()
                );
                text.push_str(&"─".repeat(50));
                text.push('\n');
                for (i, script) in scripts.iter().enumerate() {
                    let content = read_script_content(script);
                    let first_line = content.lines().next().unwrap_or("(empty)");
                    text.push_str(&format!(
                        "\n  [{}] {}\n      Shebang: {}\n",
                        i + 1,
                        script,
                        if first_line.starts_with("#!") {
                            first_line
                        } else {
                            "(no shebang line)"
                        }
                    ));
                }
                text.push_str(&format!(
                    "\nTotal: {} script(s)",
                    scripts.len()
                ));
                buffer.set_text(&text);
            }
        });
    }

    // Initial load of scripts
    {
        let buffer = buffer.clone();
        let scripts = list_nautilus_scripts();
        let dir = get_nautilus_scripts_dir();
        if scripts.is_empty() {
            buffer.set_text(&format!(
                "No scripts found in:\n{}\n\nAdd a script above to get started.\n\nTip: Nautilus scripts receive environment variables:\n  $NAUTILUS_SCRIPT_SELECTED_FILE_PATHS\n  $NAUTILUS_SCRIPT_SELECTED_URIS\n  $NAUTILUS_SCRIPT_CURRENT_URI\n  $NAUTILUS_SCRIPT_WINDOW_GEOMETRY",
                dir.display()
            ));
        } else {
            let mut text = format!(
                "Scripts directory: {}\nFound {} script(s):\n",
                dir.display(),
                scripts.len()
            );
            text.push_str(&"─".repeat(50));
            text.push('\n');
            for (i, script) in scripts.iter().enumerate() {
                let content = read_script_content(script);
                let first_line = content.lines().next().unwrap_or("(empty)");
                text.push_str(&format!(
                    "\n  [{}] {}\n      Shebang: {}\n",
                    i + 1,
                    script,
                    if first_line.starts_with("#!") {
                        first_line
                    } else {
                        "(no shebang line)"
                    }
                ));
            }
            text.push_str(&format!("\nTotal: {} script(s)", scripts.len()));
            buffer.set_text(&text);
        }
    }

    // ── Add Custom Item ──

    {
        let name_entry = name_entry.clone();
        let cmd_entry = cmd_entry.clone();
        let output_buffer = output_buffer.clone();
        let buffer = buffer.clone();
        add_item_button.connect_clicked(move |_| {
            let name = name_entry.text().to_string();
            let cmd = cmd_entry.text().to_string();

            if name.is_empty() {
                output_buffer.set_text("Error: Script name cannot be empty.");
                return;
            }
            if cmd.is_empty() {
                output_buffer.set_text("Error: Script command cannot be empty.");
                return;
            }

            let script_content = format!(
                "#!/bin/bash\n# Created by linuxcare menu manager\n{}\n",
                cmd
            );

            match write_script_content(&name, &script_content) {
                Ok(()) => {
                    output_buffer.set_text(&format!(
                        "Successfully created script: {}\nPath: {}\nMake sure Nautilus is restarted or press Refresh to see changes.",
                        name,
                        get_nautilus_scripts_dir().join(&name).display()
                    ));
                    name_entry.set_text("");
                    cmd_entry.set_text("");

                    // Refresh the list
                    let scripts = list_nautilus_scripts();
                    let dir = get_nautilus_scripts_dir();
                    let mut text = format!(
                        "Scripts directory: {}\nFound {} script(s):\n",
                        dir.display(),
                        scripts.len()
                    );
                    text.push_str(&"─".repeat(50));
                    text.push('\n');
                    for (i, script) in scripts.iter().enumerate() {
                        let content = read_script_content(script);
                        let first_line = content.lines().next().unwrap_or("(empty)");
                        text.push_str(&format!(
                            "\n  [{}] {}\n      Shebang: {}\n",
                            i + 1,
                            script,
                            if first_line.starts_with("#!") {
                                first_line
                            } else {
                                "(no shebang line)"
                            }
                        ));
                    }
                    text.push_str(&format!("\nTotal: {} script(s)", scripts.len()));
                    buffer.set_text(&text);
                }
                Err(e) => {
                    output_buffer.set_text(&format!("Error creating script: {}", e));
                }
            }
        });
    }

    // ── Remove Selected ──

    {
        let name_entry = name_entry.clone();
        let output_buffer = output_buffer.clone();
        let buffer = buffer.clone();
        remove_item_button.connect_clicked(move |_| {
            let name = name_entry.text().to_string();
            if name.is_empty() {
                output_buffer.set_text("Error: Enter the script name to remove in the Name field.");
                return;
            }

            match remove_script(&name) {
                Ok(()) => {
                    output_buffer.set_text(&format!("Successfully removed script: {}", name));
                    name_entry.set_text("");

                    // Refresh the list
                    let scripts = list_nautilus_scripts();
                    let dir = get_nautilus_scripts_dir();
                    let mut text = format!(
                        "Scripts directory: {}\nFound {} script(s):\n",
                        dir.display(),
                        scripts.len()
                    );
                    text.push_str(&"─".repeat(50));
                    text.push('\n');
                    for (i, script) in scripts.iter().enumerate() {
                        let content = read_script_content(script);
                        let first_line = content.lines().next().unwrap_or("(empty)");
                        text.push_str(&format!(
                            "\n  [{}] {}\n      Shebang: {}\n",
                            i + 1,
                            script,
                            if first_line.starts_with("#!") {
                                first_line
                            } else {
                                "(no shebang line)"
                            }
                        ));
                    }
                    text.push_str(&format!("\nTotal: {} script(s)", scripts.len()));
                    buffer.set_text(&text);
                }
                Err(e) => {
                    output_buffer.set_text(&format!("Error removing script: {}", e));
                }
            }
        });
    }

    // ── Nautilus View Mode Buttons ──

    {
        let output_buffer = output_buffer.clone();
        viewer_icon_button.connect_clicked(move |_| {
            match set_nautilus_bg_setting("icon-view") {
                Ok(msg) => output_buffer.set_text(&msg),
                Err(e) => output_buffer.set_text(&e),
            }
        });
    }

    {
        let output_buffer = output_buffer.clone();
        viewer_list_button.connect_clicked(move |_| {
            match set_nautilus_bg_setting("list-view") {
                Ok(msg) => output_buffer.set_text(&msg),
                Err(e) => output_buffer.set_text(&e),
            }
        });
    }

    main_box.upcast()
}
