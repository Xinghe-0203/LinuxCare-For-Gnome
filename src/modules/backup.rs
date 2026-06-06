use gtk::glib;
use gtk::prelude::*;
use gtk::{
    Box, Button, ComboBoxText, Entry, Frame, Grid, Label, Orientation, ScrolledWindow, TextView,
};

use crate::tr;

fn run_command(args: &[&str]) -> String {
    std::process::Command::new(args[0])
        .args(&args[1..])
        .output()
        .map(|o| {
            let stdout = String::from_utf8_lossy(&o.stdout).to_string();
            let stderr = String::from_utf8_lossy(&o.stderr).to_string();
            if !o.status.success() {
                if !stderr.is_empty() {
                    format!("Error (exit {}):\n{}", o.status.code().unwrap_or(-1), stderr)
                } else if !stdout.is_empty() {
                    stdout
                } else {
                    format!("Error: command exited with status {}", o.status.code().unwrap_or(-1))
                }
            } else if !stderr.is_empty() {
                format!("{}\n{}", stdout, stderr)
            } else {
                stdout
            }
        })
        .unwrap_or_else(|e| format!("Error: {}", e))
}

/// Validate a path for use in cron entries (no shell metacharacters).
fn validate_cron_path(path: &str) -> bool {
    !path.is_empty()
        && path.starts_with('/')
        && !path.contains(';')
        && !path.contains('|')
        && !path.contains('&')
        && !path.contains('$')
        && !path.contains('`')
        && !path.contains('"')
        && !path.contains('\'')
}

fn get_backup_history() -> String {
    let paths = [
        "/var/backups",
        &format!("{}/Backups", std::env::var("HOME").unwrap_or_default()),
    ];
    let mut output = String::new();
    for path in &paths {
        if std::path::Path::new(path).exists() {
            let listing = run_command(&["ls", "-lht", path]);
            if !listing.trim().is_empty() && !listing.starts_with("Error") {
                output.push_str(&format!("=== {} ===\n{}\n", path, listing));
            }
        }
    }
    if output.is_empty() {
        "No backup history found.\n\nTip: Create your first backup using one of the backup methods below.".to_string()
    } else {
        output
    }
}

fn check_tool_available(tool: &str) -> bool {
    std::process::Command::new("which")
        .arg(tool)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn get_backup_methods_info() -> String {
    let mut methods = Vec::new();

    methods.push("1. rsync - Incremental file synchronization");
    if check_tool_available("rsync") {
        methods.push("   Status: Available");
    } else {
        methods.push("   Status: Not installed (sudo apt install rsync)");
    }

    methods.push("\n2. Timeshift - System snapshot utility (Btrfs/rsync)");
    if check_tool_available("timeshift") {
        methods.push("   Status: Available");
    } else {
        methods.push("   Status: Not installed (sudo apt install timeshift)");
    }

    methods.push("\n3. Borg - Deduplicated backups with compression");
    if check_tool_available("borg") {
        methods.push("   Status: Available");
    } else {
        methods.push("   Status: Not installed (sudo apt install borgbackup)");
    }

    methods.push("\n4. Simple Copy - Basic file/directory copy to backup location");

    methods.join("\n")
}

pub fn build_backup_page() -> gtk::Widget {
    let main_box = Box::new(Orientation::Vertical, 20);
    main_box.set_margin_top(20);
    main_box.set_margin_bottom(20);
    main_box.set_margin_start(20);
    main_box.set_margin_end(20);

    let title = Label::new(Some(tr!("backup_management")));
    title.set_css_classes(&["title-1"]);
    main_box.append(&title);

    // ── Backup Methods ──

    let methods_frame = Frame::new(Some(tr!("backup_tool")));
    methods_frame.set_css_classes(&["card"]);

    let methods_scrolled = ScrolledWindow::new();
    methods_scrolled.set_min_content_height(120);
    methods_scrolled.set_max_content_height(200);

    let methods_text = TextView::new();
    methods_text.set_editable(false);
    methods_text.set_monospace(true);
    methods_text.set_vexpand(true);
    methods_text.buffer().set_text(&get_backup_methods_info());

    methods_scrolled.set_child(Some(&methods_text));
    methods_scrolled.set_margin_top(10);
    methods_scrolled.set_margin_bottom(10);
    methods_scrolled.set_margin_start(12);
    methods_scrolled.set_margin_end(12);

    methods_frame.set_child(Some(&methods_scrolled));
    main_box.append(&methods_frame);

    // ── Create Backup ──

    let create_frame = Frame::new(Some("Create Backup"));
    create_frame.set_css_classes(&["card"]);

    let create_grid = Grid::new();
    create_grid.set_column_spacing(12);
    create_grid.set_row_spacing(10);
    create_grid.set_margin_top(10);
    create_grid.set_margin_bottom(10);
    create_grid.set_margin_start(12);
    create_grid.set_margin_end(12);

    let method_label = Label::new(Some("Backup Method:"));
    method_label.set_halign(gtk::Align::Start);
    let method_combo = ComboBoxText::new();
    method_combo.append_text("rsync (Incremental)");
    method_combo.append_text("Timeshift (Snapshot)");
    method_combo.append_text("borg (Deduplicated)");
    method_combo.append_text("Simple Copy");
    method_combo.set_active(Some(0));
    method_combo.set_halign(gtk::Align::Start);

    let source_label = Label::new(Some(tr!("backup_source")));
    source_label.set_halign(gtk::Align::Start);
    let source_entry = Entry::new();
    source_entry.set_hexpand(true);
    source_entry.set_placeholder_text(Some("/home or /etc"));
    source_entry.set_text("/home");

    let dest_label = Label::new(Some(tr!("backup_dest")));
    dest_label.set_halign(gtk::Align::Start);
    let dest_entry = Entry::new();
    dest_entry.set_hexpand(true);
    dest_entry.set_placeholder_text(Some("/mnt/backup or ~/Backups"));
    dest_entry.set_text(&format!(
        "{}/Backups",
        std::env::var("HOME").unwrap_or_default()
    ));

    let backup_button = Button::with_label(&tr!("start_backup"));
    backup_button.set_css_classes(&["suggested-action"]);
    backup_button.set_halign(gtk::Align::Start);

    create_grid.attach(&method_label, 0, 0, 1, 1);
    create_grid.attach(&method_combo, 1, 0, 2, 1);
    create_grid.attach(&source_label, 0, 1, 1, 1);
    create_grid.attach(&source_entry, 1, 1, 2, 1);
    create_grid.attach(&dest_label, 0, 2, 1, 1);
    create_grid.attach(&dest_entry, 1, 2, 2, 1);
    create_grid.attach(&backup_button, 0, 3, 3, 1);

    create_frame.set_child(Some(&create_grid));
    main_box.append(&create_frame);

    // ── Backup Status ──

    let status_frame = Frame::new(Some("Backup Status"));
    status_frame.set_css_classes(&["card"]);

    let status_scrolled = ScrolledWindow::new();
    status_scrolled.set_min_content_height(150);
    status_scrolled.set_max_content_height(250);

    let status_text = TextView::new();
    status_text.set_editable(false);
    status_text.set_monospace(true);
    status_text.set_vexpand(true);
    status_text.buffer().set_text("Ready. Select a backup method and click 'Create Backup'.");

    status_scrolled.set_child(Some(&status_text));
    status_scrolled.set_margin_top(10);
    status_scrolled.set_margin_bottom(10);
    status_scrolled.set_margin_start(12);
    status_scrolled.set_margin_end(12);

    status_frame.set_child(Some(&status_scrolled));
    main_box.append(&status_frame);

    // ── Backup History ──

    let history_frame = Frame::new(Some(tr!("backup_history")));
    history_frame.set_css_classes(&["card"]);

    let history_scrolled = ScrolledWindow::new();
    history_scrolled.set_min_content_height(150);
    history_scrolled.set_max_content_height(250);

    let history_text = TextView::new();
    history_text.set_editable(false);
    history_text.set_monospace(true);
    history_text.set_vexpand(true);
    history_text.buffer().set_text(&get_backup_history());

    history_scrolled.set_child(Some(&history_text));
    history_scrolled.set_margin_top(10);
    history_scrolled.set_margin_bottom(10);
    history_scrolled.set_margin_start(12);
    history_scrolled.set_margin_end(12);

    history_frame.set_child(Some(&history_scrolled));
    main_box.append(&history_frame);

    // ── Restore ──

    let restore_frame = Frame::new(Some("Restore from Backup"));
    restore_frame.set_css_classes(&["card"]);

    let restore_grid = Grid::new();
    restore_grid.set_column_spacing(12);
    restore_grid.set_row_spacing(10);
    restore_grid.set_margin_top(10);
    restore_grid.set_margin_bottom(10);
    restore_grid.set_margin_start(12);
    restore_grid.set_margin_end(12);

    let restore_path_label = Label::new(Some("Backup to Restore:"));
    restore_path_label.set_halign(gtk::Align::Start);
    let restore_path_entry = Entry::new();
    restore_path_entry.set_hexpand(true);
    restore_path_entry.set_placeholder_text(Some("Path to backup archive or directory"));

    let restore_target_label = Label::new(Some("Restore to:"));
    restore_target_label.set_halign(gtk::Align::Start);
    let restore_target_entry = Entry::new();
    restore_target_entry.set_hexpand(true);
    restore_target_entry.set_placeholder_text(Some("/ (original location) or custom path"));
    restore_target_entry.set_text("/");

    let restore_button = Button::with_label(&tr!("start_restore"));
    restore_button.set_css_classes(&["destructive-action"]);
    restore_button.set_halign(gtk::Align::Start);

    restore_grid.attach(&restore_path_label, 0, 0, 1, 1);
    restore_grid.attach(&restore_path_entry, 1, 0, 2, 1);
    restore_grid.attach(&restore_target_label, 0, 1, 1, 1);
    restore_grid.attach(&restore_target_entry, 1, 1, 2, 1);
    restore_grid.attach(&restore_button, 0, 2, 3, 1);

    restore_frame.set_child(Some(&restore_grid));
    main_box.append(&restore_frame);

    // ── Scheduled Backups ──

    let schedule_frame = Frame::new(Some("Scheduled Automatic Backups"));
    schedule_frame.set_css_classes(&["card"]);

    let schedule_grid = Grid::new();
    schedule_grid.set_column_spacing(12);
    schedule_grid.set_row_spacing(10);
    schedule_grid.set_margin_top(10);
    schedule_grid.set_margin_bottom(10);
    schedule_grid.set_margin_start(12);
    schedule_grid.set_margin_end(12);

    let schedule_source_label = Label::new(Some("Source Directory:"));
    schedule_source_label.set_halign(gtk::Align::Start);
    let schedule_source_entry = Entry::new();
    schedule_source_entry.set_hexpand(true);
    schedule_source_entry.set_placeholder_text(Some("/home"));
    schedule_source_entry.set_text("/home");

    let schedule_dest_label = Label::new(Some(tr!("backup_dest")));
    schedule_dest_label.set_halign(gtk::Align::Start);
    let schedule_dest_entry = Entry::new();
    schedule_dest_entry.set_hexpand(true);
    schedule_dest_entry.set_placeholder_text(Some("/mnt/backup"));
    schedule_dest_entry.set_text(&format!(
        "{}/Backups",
        std::env::var("HOME").unwrap_or_default()
    ));

    let schedule_freq_label = Label::new(Some("Frequency:"));
    schedule_freq_label.set_halign(gtk::Align::Start);
    let schedule_freq_combo = ComboBoxText::new();
    schedule_freq_combo.append_text("Daily");
    schedule_freq_combo.append_text("Weekly");
    schedule_freq_combo.append_text("Monthly");
    schedule_freq_combo.set_active(Some(0));
    schedule_freq_combo.set_halign(gtk::Align::Start);

    let schedule_enable_button = Button::with_label("Enable Scheduled Backup");
    schedule_enable_button.set_css_classes(&["suggested-action"]);
    schedule_enable_button.set_halign(gtk::Align::Start);

    let schedule_disable_button = Button::with_label("Disable Scheduled Backup");
    schedule_disable_button.set_css_classes(&["destructive-action"]);
    schedule_disable_button.set_halign(gtk::Align::Start);

    schedule_grid.attach(&schedule_source_label, 0, 0, 1, 1);
    schedule_grid.attach(&schedule_source_entry, 1, 0, 2, 1);
    schedule_grid.attach(&schedule_dest_label, 0, 1, 1, 1);
    schedule_grid.attach(&schedule_dest_entry, 1, 1, 2, 1);
    schedule_grid.attach(&schedule_freq_label, 0, 2, 1, 1);
    schedule_grid.attach(&schedule_freq_combo, 1, 2, 2, 1);
    schedule_grid.attach(&schedule_enable_button, 0, 3, 1, 1);
    schedule_grid.attach(&schedule_disable_button, 1, 3, 1, 1);

    schedule_frame.set_child(Some(&schedule_grid));
    main_box.append(&schedule_frame);

    // ── Actions ──

    let actions_frame = Frame::new(Some("Actions"));
    actions_frame.set_css_classes(&["card"]);

    let actions_grid = Grid::new();
    actions_grid.set_column_spacing(12);
    actions_grid.set_row_spacing(10);
    actions_grid.set_margin_top(10);
    actions_grid.set_margin_bottom(10);
    actions_grid.set_margin_start(12);
    actions_grid.set_margin_end(12);

    let refresh_button = Button::with_label(&tr!("refresh"));
    refresh_button.set_halign(gtk::Align::Start);
    refresh_button.set_css_classes(&["suggested-action"]);

    let install_tools_button = Button::with_label("Install Backup Tools");
    install_tools_button.set_halign(gtk::Align::Start);

    actions_grid.attach(&refresh_button, 0, 0, 1, 1);
    actions_grid.attach(&install_tools_button, 1, 0, 1, 1);

    actions_frame.set_child(Some(&actions_grid));
    main_box.append(&actions_frame);

    // ── Signal handlers ──

    {
        let status_text = status_text.clone();
        let method_combo = method_combo.clone();
        let source_entry = source_entry.clone();
        let dest_entry = dest_entry.clone();

        backup_button.connect_clicked(move |_| {
            let method = method_combo.active_text().unwrap_or_default();
            let source = source_entry.text().to_string();
            let dest = dest_entry.text().to_string();

            if source.is_empty() || dest.is_empty() {
                status_text.buffer().set_text("Error: Source and destination cannot be empty.");
                return;
            }

            let status_buffer = status_text.buffer();
            let _ = std::fs::create_dir_all(&dest);

            match method.as_str() {
                "rsync (Incremental)" => {
                    status_buffer.set_text(&format!(
                        "Starting rsync backup...\nSource: {}\nDestination: {}\n\nRunning...",
                        source, dest
                    ));
                    let status_text_c = status_text.clone();
                    let source_c = source.clone();
                    let dest_c = dest.clone();
                    glib::spawn_future_local(async move {
                        let output = run_command(&[
                            "rsync", "-avh", "--progress", &source_c, &dest_c,
                        ]);
                        status_text_c.buffer().set_text(&format!(
                            "rsync backup complete.\n\nSource: {}\nDestination: {}\n\nOutput:\n{}",
                            source_c, dest_c, output
                        ));
                    });
                }
                "Timeshift (Snapshot)" => {
                    status_buffer.set_text("Creating Timeshift snapshot...\n");
                    let status_text_c = status_text.clone();
                    glib::spawn_future_local(async move {
                        let output = run_command(&[
                            "pkexec", "timeshift", "--create", "--comments", "LinuxCare backup",
                        ]);
                        status_text_c.buffer().set_text(&format!(
                            "Timeshift snapshot result:\n\n{}", output
                        ));
                    });
                }
                "borg (Deduplicated)" => {
                    let repo = format!("{}/borg-repo", dest);
                    status_buffer.set_text(&format!(
                        "Creating borg backup...\nRepository: {}\n\nRunning...",
                        repo
                    ));
                    let status_text_c = status_text.clone();
                    let source_c = source.clone();
                    let repo_c = repo.clone();
                    glib::spawn_future_local(async move {
                        let _ = std::process::Command::new("borg")
                            .args(["init", "--encryption=none", &repo_c])
                            .output();
                        let archive = format!("{}::{}", repo_c, chrono_now());
                        let output = run_command(&[
                            "borg", "create", &archive, &source_c,
                        ]);
                        status_text_c.buffer().set_text(&format!(
                            "borg backup complete.\nRepository: {}\n\nOutput:\n{}",
                            repo_c, output
                        ));
                    });
                }
                "Simple Copy" => {
                    let timestamp = chrono_now();
                    let backup_dest = format!("{}/backup-{}", dest, timestamp);
                    status_buffer.set_text(&format!(
                        "Copying files...\nSource: {}\nDestination: {}\n\nRunning...",
                        source, backup_dest
                    ));
                    let status_text_c = status_text.clone();
                    let source_c = source.clone();
                    let backup_dest_c = backup_dest.clone();
                    glib::spawn_future_local(async move {
                        let _ = std::fs::create_dir_all(&backup_dest_c);
                        let output = run_command(&[
                            "cp", "-av", &source_c, &backup_dest_c,
                        ]);
                        status_text_c.buffer().set_text(&format!(
                            "Simple copy backup complete.\nSource: {}\nDestination: {}\n\nOutput:\n{}",
                            source_c, backup_dest_c, output
                        ));
                    });
                }
                _ => {
                    status_buffer.set_text("Error: Unknown backup method selected.");
                }
            }
        });
    }

    {
        let status_text = status_text.clone();
        let restore_path_entry = restore_path_entry.clone();
        let restore_target_entry = restore_target_entry.clone();

        restore_button.connect_clicked(move |_| {
            let backup_path = restore_path_entry.text().to_string();
            let target = restore_target_entry.text().to_string();

            if backup_path.is_empty() || target.is_empty() {
                status_text.buffer().set_text("Error: Backup path and restore target cannot be empty.");
                return;
            }

            if target == "/" {
                status_text.buffer().set_text("Error: Restoring to '/' is not allowed. Please specify a subdirectory.");
                return;
            }

            if !std::path::Path::new(&backup_path).exists() {
                status_text.buffer().set_text(&format!(
                    "Error: Backup path does not exist: {}", backup_path
                ));
                return;
            }

            let status_buffer = status_text.buffer();
            status_buffer.set_text(&format!(
                "Restoring from:\n{}\nto:\n{}\n\nRunning...",
                backup_path, target
            ));
            let status_text_c = status_text.clone();
            let backup_path_c = backup_path.clone();
            let target_c = target.clone();
            glib::spawn_future_local(async move {
                let output = run_command(&["cp", "-av", &backup_path_c, &target_c]);
                status_text_c.buffer().set_text(&format!(
                    "Restore complete.\nSource: {}\nTarget: {}\n\nOutput:\n{}",
                    backup_path_c, target_c, output
                ));
            });
        });
    }

    {
        let history_text = history_text.clone();
        refresh_button.connect_clicked(move |_| {
            history_text.buffer().set_text(&get_backup_history());
        });
    }

    {
        let schedule_source_entry = schedule_source_entry.clone();
        let schedule_dest_entry = schedule_dest_entry.clone();
        let schedule_freq_combo = schedule_freq_combo.clone();
        let status_text = status_text.clone();

        schedule_enable_button.connect_clicked(move |_| {
            let source = schedule_source_entry.text().to_string();
            let dest = schedule_dest_entry.text().to_string();
            let freq = schedule_freq_combo.active_text().unwrap_or_default();

            if source.is_empty() || dest.is_empty() {
                status_text.buffer().set_text("Error: Source and destination cannot be empty.");
                return;
            }

            if !validate_cron_path(&source) || !validate_cron_path(&dest) {
                status_text.buffer().set_text("Error: Invalid path. Paths must be absolute and contain no shell metacharacters (;|&$`\"').");
                return;
            }

            let cron_time = match freq.as_str() {
                "Daily" => "0 2 * * *",
                "Weekly" => "0 2 * * 0",
                "Monthly" => "0 2 1 * *",
                _ => "0 2 * * *",
            };

            let cron_entry = format!(
                "{} rsync -av {} {} >> /var/log/linuxcare-backup.log 2>&1",
                cron_time, source, dest
            );

            let status_buffer = status_text.buffer();
            status_buffer.set_text(&format!(
                "Scheduled backup configured:\n\n{}\nFrequency: {}\nSource: {}\nDestination: {}\n\nNote: Run 'crontab -e' to verify the entry was added.",
                cron_entry, freq, source, dest
            ));

            let status_text_c = status_text.clone();
            let cron_entry_c = cron_entry.clone();
            glib::spawn_future_local(async move {
                let current = run_command(&["crontab", "-l"]);
                let new_cron = if current.starts_with("Error") || current.trim().is_empty() {
                    cron_entry_c.clone()
                } else {
                    format!("{}\n{}", current.trim(), cron_entry_c)
                };

                let tmp_path = std::env::temp_dir().join("linuxcare-crontab");
                let _ = std::fs::write(&tmp_path, &new_cron);
                let output = run_command(&["crontab", tmp_path.to_str().unwrap_or("/tmp/linuxcare-crontab")]);
                let _ = std::fs::remove_file(&tmp_path);

                if output.trim().is_empty() {
                    status_text_c.buffer().set_text(&format!(
                        "Scheduled backup enabled successfully.\n\nCron entry:\n{}\n\nTo manage: run 'crontab -e'",
                        cron_entry_c
                    ));
                } else {
                    status_text_c.buffer().set_text(&format!(
                        "Failed to set up scheduled backup.\n\n{}", output
                    ));
                }
            });
        });
    }

    {
        let schedule_source_entry = schedule_source_entry.clone();
        let schedule_dest_entry = schedule_dest_entry.clone();
        let _schedule_freq_combo = schedule_freq_combo.clone();
        let status_text = status_text.clone();

        schedule_disable_button.connect_clicked(move |_| {
            let _source = schedule_source_entry.text().to_string();
            let _dest = schedule_dest_entry.text().to_string();

            let status_buffer = status_text.buffer();
            status_buffer.set_text("Disabling scheduled backups...\n");

            let status_text_c = status_text.clone();
            glib::spawn_future_local(async move {
                let current = run_command(&["crontab", "-l"]);
                if current.starts_with("Error") || current.trim().is_empty() {
                    status_text_c.buffer().set_text("No crontab entries found. No scheduled backups to disable.");
                    return;
                }

                let filtered: Vec<&str> = current
                    .lines()
                    .filter(|line| !line.contains("linuxcare-backup.log"))
                    .collect();
                let new_cron = filtered.join("\n");

                let tmp_path = std::env::temp_dir().join("linuxcare-crontab");
                let _ = std::fs::write(&tmp_path, &new_cron);
                let output = run_command(&["crontab", tmp_path.to_str().unwrap_or("/tmp/linuxcare-crontab")]);
                let _ = std::fs::remove_file(&tmp_path);

                status_text_c.buffer().set_text(&format!(
                    "Scheduled backups disabled.\n\n{}", output
                ));
            });
        });
    }

    {
        let status_text = status_text.clone();
        install_tools_button.connect_clicked(move |_| {
            let status_buffer = status_text.buffer();
            status_buffer.set_text("Installing backup tools...\n");
            let status_text_c = status_text.clone();
            glib::spawn_future_local(async move {
                let output = run_command(&["pkexec", "apt", "install", "-y", "rsync", "borgbackup"]);
                status_text_c.buffer().set_text(&format!(
                    "Installation result:\n\n{}", output
                ));
            });
        });
    }

    main_box.upcast()
}

fn chrono_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("{}", secs)
}
