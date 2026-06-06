use gtk::prelude::*;
use gtk::{
    Box, Button, ComboBoxText, Entry, Frame, Grid, Label, Orientation, ScrolledWindow, TextView,
};

use crate::tr;

fn run_systemctl(args: &[&str]) -> String {
    std::process::Command::new("systemctl")
        .args(args)
        .output()
        .map(|o| {
            let stdout = String::from_utf8_lossy(&o.stdout).to_string();
            if o.status.success() {
                stdout
            } else {
                let stderr = String::from_utf8_lossy(&o.stderr).to_string();
                if stderr.is_empty() {
                    stdout
                } else {
                    format!("{}\n{}", stdout, stderr)
                }
            }
        })
        .unwrap_or_else(|e| format!("Error running systemctl: {}", e))
}

fn list_units(service_type: &str) -> Vec<(String, String, String)> {
    let output = run_systemctl(&[
        "list-unit-files",
        &format!("--type={}", service_type),
        "--no-pager",
        "--no-legend",
    ]);

    let mut units = Vec::new();
    for line in output.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            let unit = parts[0].to_string();
            let state = parts[1].to_string();
            let preset = if parts.len() >= 3 {
                parts[2].to_string()
            } else {
                String::new()
            };
            units.push((unit, state, preset));
        }
    }
    units.sort_by(|a, b| a.0.cmp(&b.0));
    units
}

fn get_service_status(unit: &str) -> String {
    let output = run_systemctl(&["status", unit, "--no-pager"]);
    output
}

fn get_service_properties(unit: &str) -> String {
    let props = [
        "ActiveState",
        "SubState",
        "LoadState",
        "Description",
        "ExecStart",
        "Restart",
        "Type",
        "WantedBy",
        "After",
        "Before",
    ];

    let mut lines = Vec::new();
    lines.push(format!("Properties for: {}", unit));
    lines.push("─".repeat(50));

    for prop in &props {
        let output = run_systemctl(&[
            "show",
            unit,
            &format!("--property={}", prop),
            "--no-pager",
        ]);
        let val = output.trim();
        lines.push(val.to_string());
    }

    lines.join("\n")
}

fn get_journal_logs(unit: &str, lines_count: &str) -> String {
    std::process::Command::new("journalctl")
        .args(["-u", unit, "-n", lines_count, "--no-pager"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_else(|e| format!("Error running journalctl: {}", e))
}

fn load_service_list(buffer: &gtk::TextBuffer, filter: &str, service_type: &str) {
    let units = list_units(service_type);
    let filter_lower = filter.to_lowercase();

    let filtered: Vec<_> = units
        .iter()
        .filter(|(unit, _, _)| {
            if filter_lower.is_empty() {
                return true;
            }
            unit.to_lowercase().contains(&filter_lower)
        })
        .collect();

    if filtered.is_empty() {
        buffer.set_text("No services found.");
        return;
    }

    let header = format!(
        "{:<50} {:<16} {:<16}",
        "UNIT", "STATE", "PRESET"
    );
    let separator = "─".repeat(header.len());
    let mut lines = vec![header, separator];

    for (unit, state, preset) in &filtered {
        let display_unit = if unit.len() > 50 {
            format!("{}…", &unit[..49])
        } else {
            unit.clone()
        };
        lines.push(format!("{:<50} {:<16} {:<16}", display_unit, state, preset));
    }

    buffer.set_text(&lines.join("\n"));
}

pub fn build_service_page() -> gtk::Widget {
    let main_box = Box::new(Orientation::Vertical, 20);
    main_box.set_margin_top(20);
    main_box.set_margin_bottom(20);
    main_box.set_margin_start(20);
    main_box.set_margin_end(20);

    let title = Label::new(Some(tr!("service_manager")));
    title.set_css_classes(&["title-1"]);
    main_box.append(&title);

    let filter_frame = Frame::new(Some(tr!("filter_services")));
    filter_frame.set_css_classes(&["card"]);

    let filter_grid = Grid::new();
    filter_grid.set_column_spacing(12);
    filter_grid.set_row_spacing(10);
    filter_grid.set_margin_top(10);
    filter_grid.set_margin_bottom(10);
    filter_grid.set_margin_start(12);
    filter_grid.set_margin_end(12);

    let filter_label = Label::new(Some(tr!("search")));
    filter_label.set_halign(gtk::Align::Start);
    let filter_entry = Entry::new();
    filter_entry.set_hexpand(true);
    filter_entry.set_placeholder_text(Some(tr!("filter_hint")));

    let type_label = Label::new(Some(tr!("type")));
    type_label.set_halign(gtk::Align::Start);
    let type_combo = ComboBoxText::new();
    type_combo.append_text("All");
    type_combo.append_text("service");
    type_combo.append_text("socket");
    type_combo.append_text("timer");
    type_combo.append_text("path");
    type_combo.append_text("mount");
    type_combo.append_text("automount");
    type_combo.append_text("swap");
    type_combo.append_text("target");
    type_combo.append_text("device");
    type_combo.append_text("scope");
    type_combo.append_text("slice");
    type_combo.set_active(Some(0));

    let refresh_button = Button::with_label(tr!("refresh"));
    refresh_button.set_css_classes(&["suggested-action"]);

    filter_grid.attach(&filter_label, 0, 0, 1, 1);
    filter_grid.attach(&filter_entry, 1, 0, 2, 1);
    filter_grid.attach(&type_label, 3, 0, 1, 1);
    filter_grid.attach(&type_combo, 4, 0, 1, 1);
    filter_grid.attach(&refresh_button, 5, 0, 1, 1);

    filter_frame.set_child(Some(&filter_grid));
    main_box.append(&filter_frame);

    let list_frame = Frame::new(Some(tr!("service_list")));
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

    let control_frame = Frame::new(Some(tr!("service_control")));
    control_frame.set_css_classes(&["card"]);

    let control_grid = Grid::new();
    control_grid.set_column_spacing(12);
    control_grid.set_row_spacing(10);
    control_grid.set_margin_top(10);
    control_grid.set_margin_bottom(10);
    control_grid.set_margin_start(12);
    control_grid.set_margin_end(12);

    let service_label = Label::new(Some(tr!("service")));
    service_label.set_halign(gtk::Align::Start);
    let service_entry = Entry::new();
    service_entry.set_hexpand(true);
    service_entry.set_placeholder_text(Some("e.g. sshd.service"));

    let start_button = Button::with_label(tr!("start"));
    start_button.set_css_classes(&["suggested-action"]);

    let stop_button = Button::with_label(tr!("stop"));
    stop_button.set_css_classes(&["destructive-action"]);

    let restart_button = Button::with_label(tr!("restart"));
    restart_button.set_css_classes(&["suggested-action"]);

    let enable_button = Button::with_label(tr!("enable"));
    enable_button.set_halign(gtk::Align::Start);

    let disable_button = Button::with_label(tr!("disable"));
    disable_button.set_halign(gtk::Align::Start);

    let status_button = Button::with_label(tr!("status"));
    status_button.set_halign(gtk::Align::Start);

    let logs_button = Button::with_label(tr!("logs"));
    logs_button.set_halign(gtk::Align::Start);

    control_grid.attach(&service_label, 0, 0, 1, 1);
    control_grid.attach(&service_entry, 1, 0, 5, 1);
    control_grid.attach(&start_button, 0, 1, 1, 1);
    control_grid.attach(&stop_button, 1, 1, 1, 1);
    control_grid.attach(&restart_button, 2, 1, 1, 1);
    control_grid.attach(&enable_button, 3, 1, 1, 1);
    control_grid.attach(&disable_button, 4, 1, 1, 1);
    control_grid.attach(&status_button, 5, 1, 1, 1);
    control_grid.attach(&logs_button, 6, 1, 1, 1);

    control_frame.set_child(Some(&control_grid));
    main_box.append(&control_frame);

    let detail_frame = Frame::new(Some(tr!("service_details")));
    detail_frame.set_css_classes(&["card"]);

    let detail_scrolled = ScrolledWindow::new();
    detail_scrolled.set_min_content_height(200);
    detail_scrolled.set_max_content_height(400);

    let detail_text_view = TextView::new();
    detail_text_view.set_editable(false);
    detail_text_view.set_monospace(true);
    detail_text_view.set_wrap_mode(gtk::WrapMode::Word);
    detail_text_view.set_left_margin(8);
    detail_text_view.set_top_margin(8);

    let detail_buffer = detail_text_view.buffer();
    detail_buffer.set_text("Select a service and click 'Status' or 'Logs' to view details.");

    detail_scrolled.set_child(Some(&detail_text_view));
    detail_scrolled.set_margin_top(10);
    detail_scrolled.set_margin_bottom(10);
    detail_scrolled.set_margin_start(12);
    detail_scrolled.set_margin_end(12);

    detail_frame.set_child(Some(&detail_scrolled));
    main_box.append(&detail_frame);

    {
        let list_buffer_c = list_buffer.clone();
        let filter_entry_c = filter_entry.clone();
        let type_combo_c = type_combo.clone();
        refresh_button.connect_clicked(move |_| {
            let filter = filter_entry_c.text().to_string();
            let service_type = match type_combo_c.active_text().as_deref() {
                Some("service") => "service",
                Some("socket") => "socket",
                Some("timer") => "timer",
                Some("path") => "path",
                Some("mount") => "mount",
                Some("automount") => "automount",
                Some("swap") => "swap",
                Some("target") => "target",
                Some("device") => "device",
                Some("scope") => "scope",
                Some("slice") => "slice",
                _ => "",
            };
            load_service_list(&list_buffer_c, &filter, service_type);
        });
    }

    {
        let list_buffer_c = list_buffer.clone();
        let filter_entry_c = filter_entry.clone();
        let type_combo_c = type_combo.clone();
        filter_entry.connect_activate(move |_| {
            let filter = filter_entry_c.text().to_string();
            let service_type = match type_combo_c.active_text().as_deref() {
                Some("service") => "service",
                Some("socket") => "socket",
                Some("timer") => "timer",
                Some("path") => "path",
                Some("mount") => "mount",
                Some("automount") => "automount",
                Some("swap") => "swap",
                Some("target") => "target",
                Some("device") => "device",
                Some("scope") => "scope",
                Some("slice") => "slice",
                _ => "",
            };
            load_service_list(&list_buffer_c, &filter, service_type);
        });
    }

    {
        let type_combo_c = type_combo.clone();
        let list_buffer_c = list_buffer.clone();
        let filter_entry_c = filter_entry.clone();
        type_combo.connect_changed(move |_| {
            let filter = filter_entry_c.text().to_string();
            let service_type = match type_combo_c.active_text().as_deref() {
                Some("service") => "service",
                Some("socket") => "socket",
                Some("timer") => "timer",
                Some("path") => "path",
                Some("mount") => "mount",
                Some("automount") => "automount",
                Some("swap") => "swap",
                Some("target") => "target",
                Some("device") => "device",
                Some("scope") => "scope",
                Some("slice") => "slice",
                _ => "",
            };
            load_service_list(&list_buffer_c, &filter, service_type);
        });
    }

    {
        let detail_buffer_c = detail_buffer.clone();
        let service_entry_c = service_entry.clone();
        start_button.connect_clicked(move |_| {
            let service = service_entry_c.text().to_string();
            if service.is_empty() {
                detail_buffer_c.set_text("Error: Enter a service name.");
                return;
            }
            let output = run_systemctl(&["start", &service]);
            let result = if output.trim().is_empty() {
                format!("Service '{}' started successfully.", service)
            } else {
                format!("Start result:\n{}", output)
            };
            detail_buffer_c.set_text(&result);
        });
    }

    {
        let detail_buffer_c = detail_buffer.clone();
        let service_entry_c = service_entry.clone();
        stop_button.connect_clicked(move |_| {
            let service = service_entry_c.text().to_string();
            if service.is_empty() {
                detail_buffer_c.set_text("Error: Enter a service name.");
                return;
            }
            let output = run_systemctl(&["stop", &service]);
            let result = if output.trim().is_empty() {
                format!("Service '{}' stopped successfully.", service)
            } else {
                format!("Stop result:\n{}", output)
            };
            detail_buffer_c.set_text(&result);
        });
    }

    {
        let detail_buffer_c = detail_buffer.clone();
        let service_entry_c = service_entry.clone();
        restart_button.connect_clicked(move |_| {
            let service = service_entry_c.text().to_string();
            if service.is_empty() {
                detail_buffer_c.set_text("Error: Enter a service name.");
                return;
            }
            let output = run_systemctl(&["restart", &service]);
            let result = if output.trim().is_empty() {
                format!("Service '{}' restarted successfully.", service)
            } else {
                format!("Restart result:\n{}", output)
            };
            detail_buffer_c.set_text(&result);
        });
    }

    {
        let detail_buffer_c = detail_buffer.clone();
        let service_entry_c = service_entry.clone();
        enable_button.connect_clicked(move |_| {
            let service = service_entry_c.text().to_string();
            if service.is_empty() {
                detail_buffer_c.set_text("Error: Enter a service name.");
                return;
            }
            let output = run_systemctl(&["enable", &service]);
            detail_buffer_c.set_text(&format!("Enable result:\n{}", output));
        });
    }

    {
        let detail_buffer_c = detail_buffer.clone();
        let service_entry_c = service_entry.clone();
        disable_button.connect_clicked(move |_| {
            let service = service_entry_c.text().to_string();
            if service.is_empty() {
                detail_buffer_c.set_text("Error: Enter a service name.");
                return;
            }
            let output = run_systemctl(&["disable", &service]);
            detail_buffer_c.set_text(&format!("Disable result:\n{}", output));
        });
    }

    {
        let detail_buffer_c = detail_buffer.clone();
        let service_entry_c = service_entry.clone();
        status_button.connect_clicked(move |_| {
            let service = service_entry_c.text().to_string();
            if service.is_empty() {
                detail_buffer_c.set_text("Error: Enter a service name.");
                return;
            }
            let status = get_service_status(&service);
            let props = get_service_properties(&service);
            detail_buffer_c.set_text(&format!("{}\n\n{}", status, props));
        });
    }

    {
        let detail_buffer_c = detail_buffer.clone();
        let service_entry_c = service_entry.clone();
        logs_button.connect_clicked(move |_| {
            let service = service_entry_c.text().to_string();
            if service.is_empty() {
                detail_buffer_c.set_text("Error: Enter a service name.");
                return;
            }
            let logs = get_journal_logs(&service, "100");
            detail_buffer_c.set_text(&logs);
        });
    }

    load_service_list(&list_buffer, "", "");

    main_box.upcast()
}
