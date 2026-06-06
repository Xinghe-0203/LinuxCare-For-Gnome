use std::sync::mpsc;

use gtk::glib;
use gtk::prelude::*;
use gtk::{Box, Button, ComboBoxText, Frame, Grid, Label, Orientation, ScrolledWindow, SearchEntry, Switch, TextView};

use crate::tr;

/// Run a command and capture stdout/stderr.
fn run_command(cmd: &str, args: &[&str]) -> String {
    std::process::Command::new(cmd)
        .args(args)
        .output()
        .map(|o| {
            let stdout = String::from_utf8_lossy(&o.stdout).to_string();
            let stderr = String::from_utf8_lossy(&o.stderr).to_string();
            if !stderr.is_empty() {
                format!("{}\n{}", stdout, stderr)
            } else {
                stdout
            }
        })
        .unwrap_or_else(|e| format!("Error: {}", e))
}

/// Run a command via pkexec (for privileged operations).
fn run_pkexec(cmd: &str, args: &[&str]) -> String {
    std::process::Command::new("pkexec")
        .arg(cmd)
        .args(args)
        .output()
        .map(|o| {
            let stdout = String::from_utf8_lossy(&o.stdout).to_string();
            let stderr = String::from_utf8_lossy(&o.stderr).to_string();
            if stdout.is_empty() && !stderr.is_empty() {
                stderr
            } else {
                stdout
            }
        })
        .unwrap_or_else(|e| format!("Error: {}", e))
}

/// Get a gsettings value as a string.
fn gsettings_get(schema: &str, key: &str) -> Option<String> {
    std::process::Command::new("gsettings")
        .args(["get", schema, key])
        .output()
        .ok()
        .and_then(|o| {
            let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if s.is_empty() { None } else { Some(s) }
        })
}

/// Set a gsettings value.
fn gsettings_set(schema: &str, key: &str, value: &str) {
    let _ = std::process::Command::new("gsettings")
        .args(["set", schema, key, value])
        .output();
}

/// Search packages using the selected package manager.
fn search_packages(manager_index: u32, query: &str) -> String {
    match manager_index {
        1 => run_command("flatpak", &["search", query]),
        2 => run_command("snap", &["find", query]),
        _ => run_command("apt", &["search", query]),
    }
}

/// Install a package using the selected package manager.
fn install_package(manager_index: u32, package: &str) -> String {
    match manager_index {
        1 => run_pkexec("flatpak", &["install", "-y", package]),
        2 => run_pkexec("snap", &["install", package]),
        _ => run_pkexec("apt", &["install", "-y", package]),
    }
}

/// Remove a package using the selected package manager.
fn remove_package(manager_index: u32, package: &str) -> String {
    match manager_index {
        1 => run_pkexec("flatpak", &["uninstall", "-y", package]),
        2 => run_pkexec("snap", &["remove", package]),
        _ => run_pkexec("apt", &["remove", "-y", package]),
    }
}

/// List installed packages for the selected package manager.
fn list_installed(manager_index: u32) -> String {
    match manager_index {
        1 => run_command("flatpak", &["list", "--columns=name,version"]),
        2 => run_command("snap", &["list"]),
        _ => run_command("apt", &["list", "--installed"]),
    }
}

pub fn build_software_page() -> gtk::Widget {
    let main_box = Box::new(Orientation::Vertical, 20);
    main_box.set_margin_top(20);
    main_box.set_margin_bottom(20);
    main_box.set_margin_start(20);
    main_box.set_margin_end(20);

    let title = Label::new(Some(tr!("software")));
    title.set_css_classes(&["title-1"]);
    main_box.append(&title);

    let manager_frame = Frame::new(Some(tr!("package_manager")));
    manager_frame.set_css_classes(&["card"]);

    let manager_grid = Grid::new();
    manager_grid.set_column_spacing(20);
    manager_grid.set_row_spacing(10);
    manager_grid.set_margin_top(10);
    manager_grid.set_margin_bottom(10);
    manager_grid.set_margin_start(10);
    manager_grid.set_margin_end(10);

    let manager_label = Label::new(Some(tr!("package_manager")));
    manager_label.set_halign(gtk::Align::Start);
    let manager_combo = ComboBoxText::new();
    manager_combo.append_text("APT (Debian/Ubuntu)");
    manager_combo.append_text("Flatpak");
    manager_combo.append_text("Snap");
    manager_combo.set_active(Some(0));
    manager_combo.set_halign(gtk::Align::Start);
    manager_grid.attach(&manager_label, 0, 0, 1, 1);
    manager_grid.attach(&manager_combo, 1, 0, 1, 1);

    manager_frame.set_child(Some(&manager_grid));
    main_box.append(&manager_frame);

    let search_frame = Frame::new(Some(tr!("search_and_install")));
    search_frame.set_css_classes(&["card"]);

    let search_grid = Grid::new();
    search_grid.set_column_spacing(10);
    search_grid.set_row_spacing(10);
    search_grid.set_margin_top(10);
    search_grid.set_margin_bottom(10);
    search_grid.set_margin_start(10);
    search_grid.set_margin_end(10);

    let search_entry = SearchEntry::new();
    search_entry.set_hexpand(true);
    search_entry.set_placeholder_text(Some(&tr!("search_packages")));

    let search_button = Button::with_label(&tr!("search"));
    search_button.set_css_classes(&["suggested-action"]);

    let install_button = Button::with_label(&tr!("install"));
    install_button.set_css_classes(&["suggested-action"]);
    install_button.set_sensitive(false);

    let remove_button = Button::with_label(&tr!("remove"));
    remove_button.set_css_classes(&["destructive-action"]);
    remove_button.set_sensitive(false);

    search_grid.attach(&search_entry, 0, 0, 3, 1);
    search_grid.attach(&search_button, 3, 0, 1, 1);
    search_grid.attach(&install_button, 4, 0, 1, 1);
    search_grid.attach(&remove_button, 5, 0, 1, 1);

    search_frame.set_child(Some(&search_grid));
    main_box.append(&search_frame);

    let list_frame = Frame::new(Some(tr!("installed_packages")));
    list_frame.set_css_classes(&["card"]);
    list_frame.set_vexpand(true);

    let scrolled = ScrolledWindow::new();
    scrolled.set_min_content_height(300);

    let text_view = TextView::new();
    text_view.set_editable(false);
    text_view.set_monospace(true);
    text_view.set_vexpand(true);
    let buffer = text_view.buffer();
    buffer.set_text(&tr!("loading"));

    scrolled.set_child(Some(&text_view));
    list_frame.set_child(Some(&scrolled));
    main_box.append(&list_frame);

    let update_frame = Frame::new(Some(tr!("update")));
    update_frame.set_css_classes(&["card"]);

    let update_grid = Grid::new();
    update_grid.set_column_spacing(20);
    update_grid.set_row_spacing(10);
    update_grid.set_margin_top(10);
    update_grid.set_margin_bottom(10);
    update_grid.set_margin_start(10);
    update_grid.set_margin_end(10);

    let update_label = Label::new(Some(tr!("auto_check_updates")));
    update_label.set_halign(gtk::Align::Start);
    let update_switch = Switch::new();
    // Read initial state from gsettings
    let initial_active = gsettings_get("org.gnome.software", "allow-updates")
        .map(|v| v == "true")
        .unwrap_or(true);
    update_switch.set_active(initial_active);
    update_switch.set_halign(gtk::Align::Start);
    // Bind switch to gsettings
    update_switch.connect_active_notify(|switch| {
        let active = switch.is_active();
        gsettings_set("org.gnome.software", "allow-updates", if active { "true" } else { "false" });
    });
    update_grid.attach(&update_label, 0, 0, 1, 1);
    update_grid.attach(&update_switch, 1, 0, 1, 1);

    let check_update_button = Button::with_label(&tr!("check_updates"));
    check_update_button.set_halign(gtk::Align::Start);
    update_grid.attach(&check_update_button, 0, 1, 2, 1);

    let upgrade_button = Button::with_label(&tr!("upgrade_all"));
    upgrade_button.set_css_classes(&["suggested-action"]);
    upgrade_button.set_halign(gtk::Align::Start);
    update_grid.attach(&upgrade_button, 0, 2, 2, 1);

    update_frame.set_child(Some(&update_grid));
    main_box.append(&update_frame);

    // -- Enable/disable install and remove buttons based on search entry text --

    {
        let install_button = install_button.clone();
        let remove_button = remove_button.clone();
        search_entry.connect_changed(move |entry| {
            let has_text = !entry.text().is_empty();
            install_button.set_sensitive(has_text);
            remove_button.set_sensitive(has_text);
        });
    }

    // -- Load installed packages on page creation --

    {
        let buffer = buffer.clone();
        let idx = manager_combo.active().unwrap_or(0);
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let output = list_installed(idx);
            let lines: Vec<&str> = output
                .lines()
                .filter(|l| !l.is_empty() && !l.starts_with("Listing"))
                .collect();
            let text = if lines.is_empty() {
                "No installed packages found.".to_string()
            } else {
                let header = format!("{:<40} {}", "PACKAGE", "VERSION");
                let separator = "-".repeat(60);
                let mut display = vec![header, separator];
                for line in lines {
                    if idx == 1 {
                        let parts: Vec<&str> = line.split('\t').collect();
                        if parts.len() >= 2 {
                            display.push(format!("{:<40} {}", parts[0], parts[1]));
                        } else {
                            display.push(line.to_string());
                        }
                    } else if idx == 2 {
                        // snap: space-separated (Name Version Rev Tracking ...)
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 2 {
                            display.push(format!("{:<40} {}", parts[0], parts[1]));
                        } else {
                            display.push(line.to_string());
                        }
                    } else if let Some((pkg, rest)) = line.split_once('/') {
                        let version = rest.split_whitespace().next().unwrap_or("");
                        display.push(format!("{:<40} {}", pkg, version));
                    } else {
                        display.push(line.to_string());
                    }
                }
                display.join("\n")
            };
            let _ = tx.send(text);
        });
        glib::timeout_add_local(std::time::Duration::from_millis(50), move || {
            match rx.try_recv() {
                Ok(text) => {
                    buffer.set_text(&text);
                    glib::ControlFlow::Break
                }
                Err(mpsc::TryRecvError::Empty) => glib::ControlFlow::Continue,
                Err(mpsc::TryRecvError::Disconnected) => glib::ControlFlow::Break,
            }
        });
    }

    // -- Search button --

    {
        let search_entry = search_entry.clone();
        let buffer = buffer.clone();
        let manager_combo = manager_combo.clone();
        search_button.connect_clicked(move |_| {
            let query = search_entry.text().to_string();
            if query.is_empty() {
                return;
            }
            let idx = manager_combo.active().unwrap_or(0);
            buffer.set_text(&format!("Searching for '{}'...", query));
            let (tx, rx) = mpsc::channel();
            let query_clone = query.clone();
            std::thread::spawn(move || {
                let output = search_packages(idx, &query_clone);
                let lines: Vec<&str> = output.lines().take(50).collect();
                let text = if lines.is_empty() {
                    format!("No packages found for '{}'.", query_clone)
                } else {
                    lines.join("\n")
                };
                let _ = tx.send(text);
            });
            let buffer = buffer.clone();
            glib::timeout_add_local(std::time::Duration::from_millis(50), move || {
                match rx.try_recv() {
                    Ok(text) => {
                        buffer.set_text(&text);
                        glib::ControlFlow::Break
                    }
                    Err(mpsc::TryRecvError::Empty) => glib::ControlFlow::Continue,
                    Err(mpsc::TryRecvError::Disconnected) => glib::ControlFlow::Break,
                }
            });
        });
    }

    // -- Install button --

    {
        let search_entry = search_entry.clone();
        let buffer = buffer.clone();
        let manager_combo = manager_combo.clone();
        install_button.connect_clicked(move |_| {
            let package = search_entry.text().to_string();
            if package.is_empty() {
                return;
            }
            let idx = manager_combo.active().unwrap_or(0);
            buffer.set_text(&format!("Installing '{}'...", package));
            let (tx, rx) = mpsc::channel();
            let package_clone = package.clone();
            std::thread::spawn(move || {
                let output = install_package(idx, &package_clone);
                // Refresh installed list after install
                let installed = list_installed(idx);
                let lines: Vec<&str> = installed
                    .lines()
                    .filter(|l| !l.is_empty() && !l.starts_with("Listing"))
                    .collect();
                let header = format!("{:<40} {}", "PACKAGE", "VERSION");
                let separator = "-".repeat(60);
                let mut display = vec![
                    format!("Install result for '{}':", package_clone),
                    output,
                    String::new(),
                    header,
                    separator,
                ];
                for line in lines {
                    if idx == 1 {
                        let parts: Vec<&str> = line.split('\t').collect();
                        if parts.len() >= 2 {
                            display.push(format!("{:<40} {}", parts[0], parts[1]));
                        } else {
                            display.push(line.to_string());
                        }
                    } else if idx == 2 {
                        // snap: space-separated (Name Version Rev Tracking ...)
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 2 {
                            display.push(format!("{:<40} {}", parts[0], parts[1]));
                        } else {
                            display.push(line.to_string());
                        }
                    } else if let Some((pkg, rest)) = line.split_once('/') {
                        let version = rest.split_whitespace().next().unwrap_or("");
                        display.push(format!("{:<40} {}", pkg, version));
                    } else {
                        display.push(line.to_string());
                    }
                }
                let _ = tx.send(display.join("\n"));
            });
            let buffer = buffer.clone();
            glib::timeout_add_local(std::time::Duration::from_millis(50), move || {
                match rx.try_recv() {
                    Ok(text) => {
                        buffer.set_text(&text);
                        glib::ControlFlow::Break
                    }
                    Err(mpsc::TryRecvError::Empty) => glib::ControlFlow::Continue,
                    Err(mpsc::TryRecvError::Disconnected) => glib::ControlFlow::Break,
                }
            });
        });
    }

    // -- Remove button --

    {
        let search_entry = search_entry.clone();
        let buffer = buffer.clone();
        let manager_combo = manager_combo.clone();
        remove_button.connect_clicked(move |_| {
            let package = search_entry.text().to_string();
            if package.is_empty() {
                return;
            }
            let idx = manager_combo.active().unwrap_or(0);
            buffer.set_text(&format!("Removing '{}'...", package));
            let (tx, rx) = mpsc::channel();
            let package_clone = package.clone();
            std::thread::spawn(move || {
                let output = remove_package(idx, &package_clone);
                // Refresh installed list after remove
                let installed = list_installed(idx);
                let lines: Vec<&str> = installed
                    .lines()
                    .filter(|l| !l.is_empty() && !l.starts_with("Listing"))
                    .collect();
                let header = format!("{:<40} {}", "PACKAGE", "VERSION");
                let separator = "-".repeat(60);
                let mut display = vec![
                    format!("Remove result for '{}':", package_clone),
                    output,
                    String::new(),
                    header,
                    separator,
                ];
                for line in lines {
                    if idx == 1 {
                        let parts: Vec<&str> = line.split('\t').collect();
                        if parts.len() >= 2 {
                            display.push(format!("{:<40} {}", parts[0], parts[1]));
                        } else {
                            display.push(line.to_string());
                        }
                    } else if idx == 2 {
                        // snap: space-separated (Name Version Rev Tracking ...)
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 2 {
                            display.push(format!("{:<40} {}", parts[0], parts[1]));
                        } else {
                            display.push(line.to_string());
                        }
                    } else if let Some((pkg, rest)) = line.split_once('/') {
                        let version = rest.split_whitespace().next().unwrap_or("");
                        display.push(format!("{:<40} {}", pkg, version));
                    } else {
                        display.push(line.to_string());
                    }
                }
                let _ = tx.send(display.join("\n"));
            });
            let buffer = buffer.clone();
            glib::timeout_add_local(std::time::Duration::from_millis(50), move || {
                match rx.try_recv() {
                    Ok(text) => {
                        buffer.set_text(&text);
                        glib::ControlFlow::Break
                    }
                    Err(mpsc::TryRecvError::Empty) => glib::ControlFlow::Continue,
                    Err(mpsc::TryRecvError::Disconnected) => glib::ControlFlow::Break,
                }
            });
        });
    }

    // -- Check for Updates button --

    {
        let buffer = buffer.clone();
        let manager_combo = manager_combo.clone();
        check_update_button.connect_clicked(move |_| {
            let idx = manager_combo.active().unwrap_or(0);
            buffer.set_text("Updating package lists...");
            let (tx, rx) = mpsc::channel();
            std::thread::spawn(move || {
                let update_output = match idx {
                    1 => run_command("flatpak", &["update", "--appstream"]),
                    2 => run_command("snap", &["refresh", "--list"]),
                    _ => run_command("apt", &["update"]),
                };
                let upgradable = match idx {
                    1 => run_command("flatpak", &["remote-ls", "--updates"]),
                    2 => run_command("snap", &["refresh", "--list"]),
                    _ => run_command("apt", &["list", "--upgradable"]),
                };
                let lines: Vec<&str> = upgradable
                    .lines()
                    .filter(|l| !l.is_empty() && !l.starts_with("Listing"))
                    .collect();
                let text = if lines.is_empty() {
                    format!("{}\n\nAll packages are up to date.", update_output)
                } else {
                    let header = format!("Upgradable packages ({}):\n{:<40} {}", lines.len(), "PACKAGE", "VERSION");
                    let separator = "-".repeat(60);
                    let mut display = vec![
                        format!("Update output:\n{}", update_output),
                        String::new(),
                        header,
                        separator,
                    ];
                    for line in lines {
                        if idx == 1 {
                            let parts: Vec<&str> = line.split('\t').collect();
                            if parts.len() >= 2 {
                                display.push(format!("{:<40} {}", parts[0], parts[1]));
                            } else {
                                display.push(line.to_string());
                            }
                        } else if idx == 2 {
                            // snap: space-separated (Name Version Rev Tracking ...)
                            let parts: Vec<&str> = line.split_whitespace().collect();
                            if parts.len() >= 2 {
                                display.push(format!("{:<40} {}", parts[0], parts[1]));
                            } else {
                                display.push(line.to_string());
                            }
                        } else if let Some((pkg, rest)) = line.split_once('/') {
                            let version = rest.split_whitespace().next().unwrap_or("");
                            display.push(format!("{:<40} {}", pkg, version));
                        } else {
                            display.push(line.to_string());
                        }
                    }
                    display.join("\n")
                };
                let _ = tx.send(text);
            });
            let buffer = buffer.clone();
            glib::timeout_add_local(std::time::Duration::from_millis(50), move || {
                match rx.try_recv() {
                    Ok(text) => {
                        buffer.set_text(&text);
                        glib::ControlFlow::Break
                    }
                    Err(mpsc::TryRecvError::Empty) => glib::ControlFlow::Continue,
                    Err(mpsc::TryRecvError::Disconnected) => glib::ControlFlow::Break,
                }
            });
        });
    }

    // -- Upgrade All button --

    {
        let buffer = buffer.clone();
        let manager_combo = manager_combo.clone();
        upgrade_button.connect_clicked(move |_| {
            let idx = manager_combo.active().unwrap_or(0);
            buffer.set_text("Upgrading all packages...");
            let (tx, rx) = mpsc::channel();
            std::thread::spawn(move || {
                let output = match idx {
                    1 => run_pkexec("flatpak", &["update", "-y"]),
                    2 => run_pkexec("snap", &["refresh"]),
                    _ => run_pkexec("apt", &["upgrade", "-y"]),
                };
                let text = format!("Upgrade result:\n\n{}", output);
                let _ = tx.send(text);
            });
            let buffer = buffer.clone();
            glib::timeout_add_local(std::time::Duration::from_millis(50), move || {
                match rx.try_recv() {
                    Ok(text) => {
                        buffer.set_text(&text);
                        glib::ControlFlow::Break
                    }
                    Err(mpsc::TryRecvError::Empty) => glib::ControlFlow::Continue,
                    Err(mpsc::TryRecvError::Disconnected) => glib::ControlFlow::Break,
                }
            });
        });
    }

    main_box.upcast()
}
