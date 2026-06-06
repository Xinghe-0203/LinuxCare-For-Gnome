use gtk::prelude::*;
use gtk::{
    Box, Button, ComboBoxText, Entry, Frame, Grid, Label, Orientation, ScrolledWindow, SpinButton,
    Adjustment, TextView,
};

use crate::tr;

fn run_journalctl(args: &[&str]) -> String {
    std::process::Command::new("journalctl")
        .args(args)
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_else(|e| format!("Error running journalctl: {}", e))
}

fn read_log_file(path: &str, max_lines: usize) -> String {
    match std::fs::read_to_string(path) {
        Ok(content) => {
            let lines: Vec<&str> = content.lines().collect();
            let start = if lines.len() > max_lines {
                lines.len() - max_lines
            } else {
                0
            };
            lines[start..].join("\n")
        }
        Err(e) => format!("Error reading {}: {}", path, e),
    }
}

fn get_journal_logs(
    service: &str,
    priority: &str,
    since: &str,
    until: &str,
    lines_count: &str,
) -> String {
    let mut args = vec![
        "--no-pager",
        "-n",
        lines_count,
        "--output=short-iso",
    ];

    if !service.is_empty() {
        args.push("-u");
        args.push(service);
    }

    if !priority.is_empty() {
        args.push("-p");
        args.push(priority);
    }

    if !since.is_empty() {
        args.push("--since");
        args.push(since);
    }

    if !until.is_empty() {
        args.push("--until");
        args.push(until);
    }

    run_journalctl(&args)
}

fn search_logs(logs: &str, pattern: &str) -> String {
    if pattern.is_empty() {
        return logs.to_string();
    }

    let pattern_lower = pattern.to_lowercase();
    let matching_lines: Vec<&str> = logs
        .lines()
        .filter(|line| line.to_lowercase().contains(&pattern_lower))
        .collect();

    if matching_lines.is_empty() {
        format!("No matches found for pattern: {}", pattern)
    } else {
        format!(
            "Found {} matches:\n\n{}",
            matching_lines.len(),
            matching_lines.join("\n")
        )
    }
}

fn get_syslog() -> String {
    let mut output = String::new();
    let paths = [
        "/var/log/syslog",
        "/var/log/messages",
        "/var/log/kern.log",
    ];

    for path in &paths {
        if std::path::Path::new(path).exists() {
            output.push_str(&format!("=== {} ===\n", path));
            output.push_str(&read_log_file(path, 200));
            output.push('\n');
        }
    }

    if output.is_empty() {
        "No syslog files found. They may require root access.".to_string()
    } else {
        output
    }
}

fn get_auth_log() -> String {
    let path = "/var/log/auth.log";
    if std::path::Path::new(path).exists() {
        let content = read_log_file(path, 200);
        format!("=== {} ===\n{}", path, content)
    } else {
        "No auth.log found. It may require root access.".to_string()
    }
}

pub fn build_logview_page() -> gtk::Widget {
    let main_box = Box::new(Orientation::Vertical, 20);
    main_box.set_margin_top(20);
    main_box.set_margin_bottom(20);
    main_box.set_margin_start(20);
    main_box.set_margin_end(20);

    let title = Label::new(Some(tr!("log_viewer")));
    title.set_css_classes(&["title-1"]);
    main_box.append(&title);

    let filter_frame = Frame::new(Some("Journal Filter"));
    filter_frame.set_css_classes(&["card"]);

    let filter_grid = Grid::new();
    filter_grid.set_column_spacing(12);
    filter_grid.set_row_spacing(10);
    filter_grid.set_margin_top(10);
    filter_grid.set_margin_bottom(10);
    filter_grid.set_margin_start(12);
    filter_grid.set_margin_end(12);

    let service_label = Label::new(Some(tr!("service")));
    service_label.set_halign(gtk::Align::Start);
    let service_entry = Entry::new();
    service_entry.set_hexpand(true);
    service_entry.set_placeholder_text(Some("e.g. sshd, NetworkManager (empty for all)"));

    let priority_label = Label::new(Some("Priority:"));
    priority_label.set_halign(gtk::Align::Start);
    let priority_combo = ComboBoxText::new();
    priority_combo.append_text(tr!("all"));
    priority_combo.append_text("0 - emerg");
    priority_combo.append_text("1 - alert");
    priority_combo.append_text("2 - crit");
    priority_combo.append_text("3 - err");
    priority_combo.append_text("4 - warning");
    priority_combo.append_text("5 - notice");
    priority_combo.append_text("6 - info");
    priority_combo.append_text("7 - debug");
    priority_combo.set_active(Some(0));

    let since_label = Label::new(Some("Since:"));
    since_label.set_halign(gtk::Align::Start);
    let since_entry = Entry::new();
    since_entry.set_hexpand(true);
    since_entry.set_placeholder_text(Some("e.g. '1 hour ago', '2024-01-01'"));

    let until_label = Label::new(Some("Until:"));
    until_label.set_halign(gtk::Align::Start);
    let until_entry = Entry::new();
    until_entry.set_hexpand(true);
    until_entry.set_placeholder_text(Some("e.g. 'now', '2024-12-31'"));

    let lines_label = Label::new(Some(tr!("lines")));
    lines_label.set_halign(gtk::Align::Start);
    let lines_adj = Adjustment::new(500.0, 10.0, 10000.0, 10.0, 100.0, 0.0);
    let lines_spin = SpinButton::new(Some(&lines_adj), 1.0, 0);
    lines_spin.set_halign(gtk::Align::Start);

    let load_button = Button::with_label(tr!("load_log"));
    load_button.set_css_classes(&["suggested-action"]);

    filter_grid.attach(&service_label, 0, 0, 1, 1);
    filter_grid.attach(&service_entry, 1, 0, 2, 1);
    filter_grid.attach(&priority_label, 3, 0, 1, 1);
    filter_grid.attach(&priority_combo, 4, 0, 1, 1);
    filter_grid.attach(&since_label, 0, 1, 1, 1);
    filter_grid.attach(&since_entry, 1, 1, 2, 1);
    filter_grid.attach(&until_label, 3, 1, 1, 1);
    filter_grid.attach(&until_entry, 4, 1, 1, 1);
    filter_grid.attach(&lines_label, 0, 2, 1, 1);
    filter_grid.attach(&lines_spin, 1, 2, 1, 1);
    filter_grid.attach(&load_button, 2, 2, 1, 1);

    filter_frame.set_child(Some(&filter_grid));
    main_box.append(&filter_frame);

    let search_frame = Frame::new(Some("Search"));
    search_frame.set_css_classes(&["card"]);

    let search_grid = Grid::new();
    search_grid.set_column_spacing(12);
    search_grid.set_row_spacing(10);
    search_grid.set_margin_top(10);
    search_grid.set_margin_bottom(10);
    search_grid.set_margin_start(12);
    search_grid.set_margin_end(12);

    let search_label = Label::new(Some("Pattern:"));
    search_label.set_halign(gtk::Align::Start);
    let search_entry = Entry::new();
    search_entry.set_hexpand(true);
    search_entry.set_placeholder_text(Some("Search pattern..."));

    let search_button = Button::with_label(tr!("search"));
    search_button.set_css_classes(&["suggested-action"]);

    let clear_search_button = Button::with_label(tr!("clear"));
    clear_search_button.set_halign(gtk::Align::Start);

    search_grid.attach(&search_label, 0, 0, 1, 1);
    search_grid.attach(&search_entry, 1, 0, 2, 1);
    search_grid.attach(&search_button, 3, 0, 1, 1);
    search_grid.attach(&clear_search_button, 4, 0, 1, 1);

    search_frame.set_child(Some(&search_grid));
    main_box.append(&search_frame);

    let quick_frame = Frame::new(Some("Quick Access"));
    quick_frame.set_css_classes(&["card"]);

    let quick_grid = Grid::new();
    quick_grid.set_column_spacing(12);
    quick_grid.set_row_spacing(10);
    quick_grid.set_margin_top(10);
    quick_grid.set_margin_bottom(10);
    quick_grid.set_margin_start(12);
    quick_grid.set_margin_end(12);

    let journal_button = Button::with_label(tr!("journalctl"));
    journal_button.set_css_classes(&["suggested-action"]);

    let syslog_button = Button::with_label(tr!("system_log"));

    let authlog_button = Button::with_label(tr!("auth_log"));

    let kernlog_button = Button::with_label(tr!("kernel_log"));

    let dmesg_button = Button::with_label(tr!("dmesg"));

    let boot_button = Button::with_label(tr!("boot_logs"));

    quick_grid.attach(&journal_button, 0, 0, 1, 1);
    quick_grid.attach(&syslog_button, 1, 0, 1, 1);
    quick_grid.attach(&authlog_button, 2, 0, 1, 1);
    quick_grid.attach(&kernlog_button, 3, 0, 1, 1);
    quick_grid.attach(&dmesg_button, 4, 0, 1, 1);
    quick_grid.attach(&boot_button, 5, 0, 1, 1);

    quick_frame.set_child(Some(&quick_grid));
    main_box.append(&quick_frame);

    let log_frame = Frame::new(Some("Log Output"));
    log_frame.set_css_classes(&["card"]);

    let log_scrolled = ScrolledWindow::new();
    log_scrolled.set_min_content_height(400);
    log_scrolled.set_vexpand(true);

    let log_text_view = TextView::new();
    log_text_view.set_editable(false);
    log_text_view.set_monospace(true);
    log_text_view.set_wrap_mode(gtk::WrapMode::None);
    log_text_view.set_left_margin(8);
    log_text_view.set_top_margin(8);

    let log_buffer = log_text_view.buffer();
    log_buffer.set_text("Click a button above to load logs...");

    log_scrolled.set_child(Some(&log_text_view));
    log_scrolled.set_margin_top(10);
    log_scrolled.set_margin_bottom(10);
    log_scrolled.set_margin_start(12);
    log_scrolled.set_margin_end(12);

    log_frame.set_child(Some(&log_scrolled));
    main_box.append(&log_frame);

    {
        let log_buffer_c = log_buffer.clone();
        let service_entry_c = service_entry.clone();
        let priority_combo_c = priority_combo.clone();
        let since_entry_c = since_entry.clone();
        let until_entry_c = until_entry.clone();
        let lines_spin_c = lines_spin.clone();
        load_button.connect_clicked(move |_| {
            let service = service_entry_c.text().to_string();
            let priority = match priority_combo_c.active_text().as_deref() {
                Some("0 - emerg") => "0",
                Some("1 - alert") => "1",
                Some("2 - crit") => "2",
                Some("3 - err") => "3",
                Some("4 - warning") => "4",
                Some("5 - notice") => "5",
                Some("6 - info") => "6",
                Some("7 - debug") => "7",
                _ => "",
            };
            let since = since_entry_c.text().to_string();
            let until = until_entry_c.text().to_string();
            let lines_count = lines_spin_c.value() as u64;

            let logs = get_journal_logs(&service, priority, &since, &until, &lines_count.to_string());
            log_buffer_c.set_text(&logs);
        });
    }

    {
        let log_buffer_c = log_buffer.clone();
        let search_entry_c = search_entry.clone();
        search_button.connect_clicked(move |_| {
            let pattern = search_entry_c.text().to_string();
            if pattern.is_empty() {
                log_buffer_c.set_text(tr!("enter_search_pattern"));
                return;
            }
            let start_iter = log_buffer_c.start_iter();
            let end_iter = log_buffer_c.end_iter();
            let current_text = log_buffer_c.text(&start_iter, &end_iter, false).to_string();
            let results = search_logs(&current_text, &pattern);
            log_buffer_c.set_text(&results);
        });
    }

    {
        let search_entry_c = search_entry.clone();
        let log_buffer_c = log_buffer.clone();
        clear_search_button.connect_clicked(move |_| {
            search_entry_c.set_text("");
            let start_iter = log_buffer_c.start_iter();
            let end_iter = log_buffer_c.end_iter();
            let current_text = log_buffer_c.text(&start_iter, &end_iter, false).to_string();
            if !current_text.starts_with("Found") && !current_text.starts_with("No matches") {
                return;
            }
        });
    }

    {
        let log_buffer_c = log_buffer.clone();
        let service_entry_c = service_entry.clone();
        let priority_combo_c = priority_combo.clone();
        let lines_spin_c = lines_spin.clone();
        journal_button.connect_clicked(move |_| {
            let service = service_entry_c.text().to_string();
            let priority = match priority_combo_c.active_text().as_deref() {
                Some("0 - emerg") => "0",
                Some("1 - alert") => "1",
                Some("2 - crit") => "2",
                Some("3 - err") => "3",
                Some("4 - warning") => "4",
                Some("5 - notice") => "5",
                Some("6 - info") => "6",
                Some("7 - debug") => "7",
                _ => "",
            };
            let lines_count = lines_spin_c.value() as u64;
            let logs = get_journal_logs(&service, priority, "", "", &lines_count.to_string());
            log_buffer_c.set_text(&logs);
        });
    }

    {
        let log_buffer_c = log_buffer.clone();
        syslog_button.connect_clicked(move |_| {
            log_buffer_c.set_text(tr!("loading_syslog"));
            let logs = get_syslog();
            log_buffer_c.set_text(&logs);
        });
    }

    {
        let log_buffer_c = log_buffer.clone();
        authlog_button.connect_clicked(move |_| {
            log_buffer_c.set_text(tr!("loading_auth_log"));
            let logs = get_auth_log();
            log_buffer_c.set_text(&logs);
        });
    }

    {
        let log_buffer_c = log_buffer.clone();
        kernlog_button.connect_clicked(move |_| {
            log_buffer_c.set_text(tr!("loading_kernel_log"));
            let logs = run_journalctl(&[
                "--no-pager",
                "-n",
                "500",
                "-k",
                "--output=short-iso",
            ]);
            log_buffer_c.set_text(&logs);
        });
    }

    {
        let log_buffer_c = log_buffer.clone();
        dmesg_button.connect_clicked(move |_| {
            log_buffer_c.set_text(tr!("loading_dmesg"));
            let output = std::process::Command::new("dmesg")
                .args(["--time-format=iso", "-T"])
                .output()
                .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
                .unwrap_or_else(|e| format!("Error running dmesg: {}", e));
            log_buffer_c.set_text(&output);
        });
    }

    {
        let log_buffer_c = log_buffer.clone();
        boot_button.connect_clicked(move |_| {
            log_buffer_c.set_text(tr!("loading_boot_logs"));
            let logs = run_journalctl(&[
                "--no-pager",
                "-b",
                "-n",
                "500",
                "--output=short-iso",
            ]);
            log_buffer_c.set_text(&logs);
        });
    }

    main_box.upcast()
}
