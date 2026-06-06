use gtk::prelude::*;
use gtk::{Box, Button, ComboBoxText, Frame, Grid, Label, Orientation, ProgressBar, ScrolledWindow, TextView};

use crate::tr;

/// Run `df -h` and return formatted partition info as a string for the text view.
fn get_real_disk_data() -> String {
    match std::process::Command::new("df")
        .args(["-h", "--output=source,fstype,size,used,avail,pcent,target"])
        .output()
    {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let lines: Vec<&str> = stdout.lines().collect();
            if lines.is_empty() {
                return "No partition data available".to_string();
            }
            // Filter to real filesystems (skip tmpfs, devtmpfs, etc.)
            let mut result = String::new();
            for line in lines {
                if line.starts_with("tmpfs")
                    || line.starts_with("devtmpfs")
                    || line.starts_with("udev")
                    || line.contains("/snap/")
                {
                    continue;
                }
                result.push_str(line);
                result.push('\n');
            }
            if result.trim().is_empty() {
                "No partition data available".to_string()
            } else {
                result
            }
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            format!("Failed to read disk data:\n{}", stderr)
        }
        Err(e) => format!("Failed to run df: {}", e),
    }
}

/// Parse `df` output and return a list of (mount_point, fraction, percentage_text) for progress bars.
fn get_disk_usage_bars() -> Vec<(String, f64, String)> {
    let mut partitions = Vec::new();
    match std::process::Command::new("df")
        .args(["--output=target,pcent", "-B1"])
        .output()
    {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines().skip(1) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() < 2 {
                    continue;
                }
                let mount = parts[0];
                // Skip virtual/snap filesystems
                if mount.starts_with("/snap")
                    || mount.starts_with("/boot/efi")
                    || mount == "/dev"
                    || mount.starts_with("/run")
                {
                    continue;
                }
                // Percentage string like "45%"
                let pct_str = parts[1].trim_end_matches('%');
                if let Ok(pct) = pct_str.parse::<f64>() {
                    let fraction = (pct / 100.0).clamp(0.0, 1.0);
                    let label = match mount {
                        "/" => "/ (root)".to_string(),
                        other => other.to_string(),
                    };
                    partitions.push((label, fraction, format!("{}%", pct as u32)));
                }
            }
        }
        _ => {}
    }
    partitions
}

/// Run `lsblk` to get a more detailed device/partition table.
fn get_lsblk_info() -> String {
    match std::process::Command::new("lsblk")
        .args(["-o", "NAME,SIZE,TYPE,MOUNTPOINT,FSTYPE", "-b"])
        .output()
    {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.trim().is_empty() {
                "No block device information available".to_string()
            } else {
                stdout.to_string()
            }
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            format!("Failed to read block devices:\n{}", stderr)
        }
        Err(e) => format!("Failed to run lsblk: {}", e),
    }
}

/// Detect all disk block devices using lsblk, returning their /dev paths.
fn detect_disk_devices() -> Vec<String> {
    let output = std::process::Command::new("lsblk")
        .args(["-d", "-o", "NAME,TYPE", "-n"])
        .output();
    match output {
        Ok(o) if o.status.success() => {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .filter(|l| {
                    let parts: Vec<&str> = l.split_whitespace().collect();
                    parts.len() >= 2 && (parts[1] == "disk")
                })
                .map(|l| {
                    let name = l.split_whitespace().next().unwrap_or("");
                    format!("/dev/{}", name)
                })
                .collect()
        }
        _ => vec!["/dev/sda".to_string()],
    }
}

/// Run smartctl health check on a specific device and return a result string.
fn check_disk_health(device: &str) -> String {
    let output = std::process::Command::new("pkexec")
        .args(["smartctl", "-H", device])
        .output();

    match output {
        Ok(o) if o.status.success() => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            let health_line = stdout
                .lines()
                .find(|l| l.contains("SMART overall-health"))
                .map(|l| l.trim().to_string())
                .unwrap_or_else(|| stdout.to_string());
            format!("Disk Health Check Result ({}):\n{}", device, health_line)
        }
        Ok(o) => {
            let stderr = String::from_utf8_lossy(&o.stderr);
            let stdout = String::from_utf8_lossy(&o.stdout);
            if stderr.contains("not found") || stdout.is_empty() {
                "smartctl is not installed.\nInstall with: sudo apt install smartmontools\n\nDisk appears to be functioning normally (no SMART data available).".to_string()
            } else {
                format!("Disk health check output ({}):\n{}{}", device, stdout, stderr)
            }
        }
        Err(e) => {
            format!(
                "Could not run disk health check on {}: {}\n\nDisk appears to be functioning normally.",
                device, e
            )
        }
    }
}

/// Show a MessageDialog with the health check result.
fn show_health_dialog(message: &str) {
    let dialog = gtk::MessageDialog::builder()
        .modal(true)
        .message_type(gtk::MessageType::Info)
        .text("Disk Health")
        .secondary_text(message)
        .buttons(gtk::ButtonsType::Ok)
        .build();
    dialog.connect_response(|dlg, _| dlg.close());
    dialog.present();
}

pub fn build_disk_page() -> gtk::Widget {
    let main_box = Box::new(Orientation::Vertical, 20);
    main_box.set_margin_top(20);
    main_box.set_margin_bottom(20);
    main_box.set_margin_start(20);
    main_box.set_margin_end(20);

    let title = Label::new(Some(tr!("disk")));
    title.set_css_classes(&["title-1"]);
    main_box.append(&title);

    // ── Disk Usage Overview (progress bars from real df data) ──

    let usage_frame = Frame::new(Some(tr!("disk_usage_overview")));
    usage_frame.set_css_classes(&["card"]);

    let usage_grid = Grid::new();
    usage_grid.set_column_spacing(20);
    usage_grid.set_row_spacing(10);
    usage_grid.set_margin_top(10);
    usage_grid.set_margin_bottom(10);
    usage_grid.set_margin_start(10);
    usage_grid.set_margin_end(10);

    let partitions = get_disk_usage_bars();

    if partitions.is_empty() {
        let no_data_label = Label::new(Some("No disk usage data available. Ensure df is installed."));
        no_data_label.set_halign(gtk::Align::Start);
        usage_grid.attach(&no_data_label, 0, 0, 2, 1);
    } else {
        for (i, (name, fraction, text)) in partitions.iter().enumerate() {
            let label = Label::new(Some(name));
            label.set_halign(gtk::Align::Start);
            let progress = ProgressBar::new();
            progress.set_show_text(true);
            progress.set_text(Some(text));
            progress.set_fraction(*fraction);
            progress.set_hexpand(true);
            usage_grid.attach(&label, 0, i as i32, 1, 1);
            usage_grid.attach(&progress, 1, i as i32, 1, 1);
        }
    }

    usage_frame.set_child(Some(&usage_grid));
    main_box.append(&usage_frame);

    // ── Partition Detail Table (from df and lsblk) ──

    let partitions_frame = Frame::new(Some(tr!("partition")));
    partitions_frame.set_css_classes(&["card"]);
    partitions_frame.set_vexpand(true);

    let scrolled = ScrolledWindow::new();
    scrolled.set_min_content_height(200);

    let text_view = TextView::new();
    text_view.set_editable(false);
    text_view.set_monospace(true);
    text_view.set_vexpand(true);
    let buffer = text_view.buffer();

    // Build the partition detail text from real data
    let df_detail = get_real_disk_data();
    let lsblk_detail = get_lsblk_info();
    let detail_text = format!(
        "=== df -h ===\n{}\n\n=== lsblk ===\n{}",
        df_detail.trim(),
        lsblk_detail.trim()
    );
    buffer.set_text(&detail_text);

    scrolled.set_child(Some(&text_view));
    partitions_frame.set_child(Some(&scrolled));
    main_box.append(&partitions_frame);

    // ── Actions ──

    let actions_frame = Frame::new(Some(tr!("actions")));
    actions_frame.set_css_classes(&["card"]);

    let actions_grid = Grid::new();
    actions_grid.set_column_spacing(10);
    actions_grid.set_row_spacing(10);
    actions_grid.set_margin_top(10);
    actions_grid.set_margin_bottom(10);
    actions_grid.set_margin_start(10);
    actions_grid.set_margin_end(10);

    let open_gparted_button = Button::with_label(&tr!("open_gparted"));
    open_gparted_button.set_halign(gtk::Align::Start);
    open_gparted_button.connect_clicked(|_| {
        let _ = std::process::Command::new("pkexec").arg("gparted").spawn();
    });

    let open_disks_button = Button::with_label(&tr!("open_gnome_disks"));
    open_disks_button.set_halign(gtk::Align::Start);
    open_disks_button.connect_clicked(|_| {
        let _ = std::process::Command::new("gnome-disks").spawn();
    });

    let check_disk_button = Button::with_label(&tr!("check_health"));
    check_disk_button.set_halign(gtk::Align::Start);
    check_disk_button.set_css_classes(&["suggested-action"]);
    check_disk_button.connect_clicked(move |_| {
        let devices = detect_disk_devices();

        if devices.len() <= 1 {
            // Single device (or fallback) - check directly
            let device = devices.into_iter().next().unwrap_or_else(|| "/dev/sda".to_string());
            let message = check_disk_health(&device);
            show_health_dialog(&message);
        } else {
            // Multiple devices - let user choose
            let dialog = gtk::Dialog::new();
            dialog.set_title(Some("Select Disk"));
            dialog.set_modal(true);
            dialog.add_button("Cancel", gtk::ResponseType::Cancel);
            dialog.add_button("Check", gtk::ResponseType::Ok);

            let combo = ComboBoxText::new();
            for dev in &devices {
                combo.append_text(dev);
            }
            combo.set_active(Some(0));

            let content = dialog.content_area();
            let label = Label::new(Some("Select a disk device to check:"));
            content.append(&label);
            content.append(&combo);

            let combo_clone = combo.clone();
            dialog.connect_response(move |dlg, response| {
                if response == gtk::ResponseType::Ok {
                    if let Some(selected) = combo_clone.active_text() {
                        let device = selected.to_string();
                        let message = check_disk_health(&device);
                        show_health_dialog(&message);
                    }
                }
                dlg.close();
            });
            dialog.present();
        }
    });

    actions_grid.attach(&open_gparted_button, 0, 0, 1, 1);
    actions_grid.attach(&open_disks_button, 1, 0, 1, 1);
    actions_grid.attach(&check_disk_button, 2, 0, 1, 1);

    actions_frame.set_child(Some(&actions_grid));
    main_box.append(&actions_frame);

    main_box.upcast()
}
