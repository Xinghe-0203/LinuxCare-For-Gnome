use gtk::prelude::*;
use gtk::{Box, Frame, Grid, Label, Orientation, ProgressBar, ScrolledWindow, TextView};

use crate::tr;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;
use sysinfo::{Components, Networks, System};

fn format_bytes(bytes: u64) -> String {
    if bytes >= 1_073_741_824 {
        format!("{:.2} GB", bytes as f64 / 1_073_741_824.0)
    } else if bytes >= 1_048_576 {
        format!("{:.2} MB", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{} B", bytes)
    }
}

fn format_speed(bytes_per_sec: u64) -> String {
    if bytes_per_sec >= 1_073_741_824 {
        format!("{:.2} GB/s", bytes_per_sec as f64 / 1_073_741_824.0)
    } else if bytes_per_sec >= 1_048_576 {
        format!("{:.2} MB/s", bytes_per_sec as f64 / 1_048_576.0)
    } else if bytes_per_sec >= 1024 {
        format!("{:.1} KB/s", bytes_per_sec as f64 / 1024.0)
    } else {
        format!("{} B/s", bytes_per_sec)
    }
}

fn get_gpu_info() -> String {
    if let Ok(output) = std::process::Command::new("nvidia-smi")
        .args([
            "--query-gpu=name,memory.total,temperature.gpu,utilization.gpu",
            "--format=csv,noheader,nounits",
        ])
        .output()
    {
        if output.status.success() {
            let info = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !info.is_empty() {
                let parts: Vec<&str> = info.split(", ").collect();
                if parts.len() >= 4 {
                    return format!(
                        "GPU: {}\nMemory: {} MiB\nTemperature: {}°C\nUtilization: {}%",
                        parts[0].trim(),
                        parts[1].trim(),
                        parts[2].trim(),
                        parts[3].trim(),
                    );
                }
                return info;
            }
        }
    }

    if let Ok(output) = std::process::Command::new("lspci").output() {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                let lower = line.to_lowercase();
                if lower.contains("vga") || lower.contains("3d") || lower.contains("display") {
                    return line.trim().to_string();
                }
            }
        }
    }

    if let Ok(content) = std::fs::read_to_string("/sys/class/drm/card0/device/vendor") {
        let vendor = content.trim();
        let name = match vendor {
            "0x10de" => "NVIDIA",
            "0x1002" => "AMD",
            "0x8086" => "Intel",
            _ => "Unknown",
        };
        if let Ok(device) = std::fs::read_to_string("/sys/class/drm/card0/device/device") {
            return format!("{} GPU (Device: {})", name, device.trim());
        }
        return format!("{} GPU", name);
    }

    "No GPU information available".to_string()
}

fn get_temperatures() -> Vec<(String, f32)> {
    let mut temps: Vec<(String, f32)> = Vec::new();

    if let Ok(entries) = std::fs::read_dir("/sys/class/thermal/") {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("thermal_zone") {
                let temp_path = entry.path().join("temp");
                let type_path = entry.path().join("type");

                let temp_value = std::fs::read_to_string(&temp_path)
                    .ok()
                    .and_then(|s| s.trim().parse::<f32>().ok());

                let sensor_name = std::fs::read_to_string(&type_path)
                    .ok()
                    .map(|s| s.trim().to_string())
                    .unwrap_or_else(|| name);

                if let Some(temp) = temp_value {
                    temps.push((sensor_name, temp / 1000.0));
                }
            }
        }
    }

    let components = Components::new_with_refreshed_list();
    for component in components.iter() {
        let temp = component.temperature();
        let label = component.label().to_string();
        if !temps.iter().any(|(n, _)| n == &label) {
            temps.push((label, temp));
        }
    }

    temps
}

struct MonitorState {
    sys: System,
    networks: Networks,
    prev_net_rx: u64,
    prev_net_tx: u64,
    prev_disk_read: u64,
    prev_disk_write: u64,
    prev_time: Instant,
}

pub fn build_monitor_page() -> gtk::Widget {
    let main_box = Box::new(Orientation::Vertical, 0);
    main_box.set_margin_top(20);
    main_box.set_margin_bottom(20);
    main_box.set_margin_start(20);
    main_box.set_margin_end(20);

    let title = Label::new(Some(tr!("monitor")));
    title.set_css_classes(&["title-1"]);
    main_box.append(&title);

    // ── CPU Usage ──

    let cpu_frame = Frame::new(Some(tr!("cpu")));
    cpu_frame.set_css_classes(&["card"]);

    let cpu_grid = Grid::new();
    cpu_grid.set_column_spacing(12);
    cpu_grid.set_row_spacing(8);
    cpu_grid.set_margin_top(10);
    cpu_grid.set_margin_bottom(10);
    cpu_grid.set_margin_start(12);
    cpu_grid.set_margin_end(12);

    let cpu_progress = ProgressBar::new();
    cpu_progress.set_show_text(true);
    cpu_progress.set_text(Some("0%"));
    cpu_progress.set_hexpand(true);
    cpu_progress.set_valign(gtk::Align::Center);

    let cpu_info_label = Label::new(Some("Loading CPU info..."));
    cpu_info_label.set_halign(gtk::Align::Start);
    cpu_info_label.set_wrap(true);
    cpu_info_label.set_selectable(true);

    cpu_grid.attach(&cpu_progress, 0, 0, 2, 1);
    cpu_grid.attach(&cpu_info_label, 0, 1, 2, 1);

    cpu_frame.set_child(Some(&cpu_grid));
    main_box.append(&cpu_frame);

    // ── Memory Usage ──

    let mem_frame = Frame::new(Some(tr!("memory")));
    mem_frame.set_css_classes(&["card"]);

    let mem_grid = Grid::new();
    mem_grid.set_column_spacing(12);
    mem_grid.set_row_spacing(8);
    mem_grid.set_margin_top(10);
    mem_grid.set_margin_bottom(10);
    mem_grid.set_margin_start(12);
    mem_grid.set_margin_end(12);

    let ram_label = Label::new(Some("RAM:"));
    ram_label.set_halign(gtk::Align::Start);
    ram_label.set_css_classes(&["heading"]);

    let ram_progress = ProgressBar::new();
    ram_progress.set_show_text(true);
    ram_progress.set_text(Some("0%"));
    ram_progress.set_hexpand(true);

    let ram_info = Label::new(Some("Loading..."));
    ram_info.set_halign(gtk::Align::Start);
    ram_info.set_selectable(true);

    let swap_label = Label::new(Some("Swap:"));
    swap_label.set_halign(gtk::Align::Start);
    swap_label.set_css_classes(&["heading"]);

    let swap_progress = ProgressBar::new();
    swap_progress.set_show_text(true);
    swap_progress.set_text(Some("0%"));
    swap_progress.set_hexpand(true);

    let swap_info = Label::new(Some("Loading..."));
    swap_info.set_halign(gtk::Align::Start);
    swap_info.set_selectable(true);

    mem_grid.attach(&ram_label, 0, 0, 1, 1);
    mem_grid.attach(&ram_progress, 1, 0, 1, 1);
    mem_grid.attach(&ram_info, 2, 0, 1, 1);
    mem_grid.attach(&swap_label, 0, 1, 1, 1);
    mem_grid.attach(&swap_progress, 1, 1, 1, 1);
    mem_grid.attach(&swap_info, 2, 1, 1, 1);

    mem_frame.set_child(Some(&mem_grid));
    main_box.append(&mem_frame);

    // ── Disk I/O ──

    let disk_frame = Frame::new(Some(tr!("disk")));
    disk_frame.set_css_classes(&["card"]);

    let disk_grid = Grid::new();
    disk_grid.set_column_spacing(12);
    disk_grid.set_row_spacing(8);
    disk_grid.set_margin_top(10);
    disk_grid.set_margin_bottom(10);
    disk_grid.set_margin_start(12);
    disk_grid.set_margin_end(12);

    let disk_read_label = Label::new(Some("Read:"));
    disk_read_label.set_halign(gtk::Align::Start);
    let disk_read_value = Label::new(Some("0 B/s"));
    disk_read_value.set_halign(gtk::Align::Start);
    disk_read_value.set_hexpand(true);
    disk_read_value.set_selectable(true);

    let disk_write_label = Label::new(Some("Write:"));
    disk_write_label.set_halign(gtk::Align::Start);
    let disk_write_value = Label::new(Some("0 B/s"));
    disk_write_value.set_halign(gtk::Align::Start);
    disk_write_value.set_hexpand(true);
    disk_write_value.set_selectable(true);

    let disk_total_label = Label::new(Some("Total Usage:"));
    disk_total_label.set_halign(gtk::Align::Start);
    disk_total_label.set_css_classes(&["heading"]);
    let disk_total_value = Label::new(Some("Loading..."));
    disk_total_value.set_halign(gtk::Align::Start);
    disk_total_value.set_selectable(true);

    disk_grid.attach(&disk_read_label, 0, 0, 1, 1);
    disk_grid.attach(&disk_read_value, 1, 0, 1, 1);
    disk_grid.attach(&disk_write_label, 0, 1, 1, 1);
    disk_grid.attach(&disk_write_value, 1, 1, 1, 1);
    disk_grid.attach(&disk_total_label, 0, 2, 1, 1);
    disk_grid.attach(&disk_total_value, 1, 2, 1, 1);

    disk_frame.set_child(Some(&disk_grid));
    main_box.append(&disk_frame);

    // ── Network ──

    let net_frame = Frame::new(Some(tr!("network")));
    net_frame.set_css_classes(&["card"]);

    let net_grid = Grid::new();
    net_grid.set_column_spacing(12);
    net_grid.set_row_spacing(8);
    net_grid.set_margin_top(10);
    net_grid.set_margin_bottom(10);
    net_grid.set_margin_start(12);
    net_grid.set_margin_end(12);

    let net_down_label = Label::new(Some("Download:"));
    net_down_label.set_halign(gtk::Align::Start);
    let net_down_value = Label::new(Some("0 B/s"));
    net_down_value.set_halign(gtk::Align::Start);
    net_down_value.set_hexpand(true);
    net_down_value.set_selectable(true);

    let net_up_label = Label::new(Some("Upload:"));
    net_up_label.set_halign(gtk::Align::Start);
    let net_up_value = Label::new(Some("0 B/s"));
    net_up_value.set_halign(gtk::Align::Start);
    net_up_value.set_hexpand(true);
    net_up_value.set_selectable(true);

    let net_total_label = Label::new(Some("Total:"));
    net_total_label.set_halign(gtk::Align::Start);
    let net_total_value = Label::new(Some("Loading..."));
    net_total_value.set_halign(gtk::Align::Start);
    net_total_value.set_selectable(true);

    net_grid.attach(&net_down_label, 0, 0, 1, 1);
    net_grid.attach(&net_down_value, 1, 0, 1, 1);
    net_grid.attach(&net_up_label, 0, 1, 1, 1);
    net_grid.attach(&net_up_value, 1, 1, 1, 1);
    net_grid.attach(&net_total_label, 0, 2, 1, 1);
    net_grid.attach(&net_total_value, 1, 2, 1, 1);

    net_frame.set_child(Some(&net_grid));
    main_box.append(&net_frame);

    // ── GPU ──

    let gpu_frame = Frame::new(Some(tr!("driver")));
    gpu_frame.set_css_classes(&["card"]);

    let gpu_label = Label::new(Some("Loading GPU info..."));
    gpu_label.set_halign(gtk::Align::Start);
    gpu_label.set_wrap(true);
    gpu_label.set_selectable(true);
    gpu_label.set_margin_top(10);
    gpu_label.set_margin_bottom(10);
    gpu_label.set_margin_start(12);
    gpu_label.set_margin_end(12);

    gpu_frame.set_child(Some(&gpu_label));
    main_box.append(&gpu_frame);

    // ── Temperature ──

    let temp_frame = Frame::new(Some(tr!("temperature")));
    temp_frame.set_css_classes(&["card"]);

    let temp_scrolled = ScrolledWindow::new();
    temp_scrolled.set_min_content_height(80);
    temp_scrolled.set_max_content_height(200);

    let temp_text_view = TextView::new();
    temp_text_view.set_editable(false);
    temp_text_view.set_monospace(true);
    temp_text_view.set_wrap_mode(gtk::WrapMode::Word);
    temp_text_view.set_vexpand(true);

    let temp_buffer = temp_text_view.buffer();
    temp_buffer.set_text("Loading temperature data...");

    temp_scrolled.set_child(Some(&temp_text_view));
    temp_scrolled.set_margin_top(10);
    temp_scrolled.set_margin_bottom(10);
    temp_scrolled.set_margin_start(12);
    temp_scrolled.set_margin_end(12);

    temp_frame.set_child(Some(&temp_scrolled));
    main_box.append(&temp_frame);

    // ── Initialize system data ──

    let mut sys = System::new_all();
    sys.refresh_all();

    let networks = Networks::new_with_refreshed_list();

    let prev_net_rx: u64 = networks.iter().map(|(_, n)| n.received()).sum();
    let prev_net_tx: u64 = networks.iter().map(|(_, n)| n.transmitted()).sum();

    let cpus = sys.cpus();
    if let Some(cpu) = cpus.first() {
        let phys_cores = sys
            .physical_core_count()
            .map(|c| c.to_string())
            .unwrap_or_else(|| "unknown".to_string());
        cpu_info_label.set_text(&format!(
            "{}\n{} logical cores ({} physical) | {:.0} MHz",
            cpu.brand(),
            cpus.len(),
            phys_cores,
            cpu.frequency()
        ));
    }

    let gpu_info = get_gpu_info();
    gpu_label.set_text(&gpu_info);

    let temps = get_temperatures();
    if temps.is_empty() {
        temp_buffer.set_text("No temperature sensors found");
    } else {
        let temp_text: Vec<String> = temps
            .iter()
            .map(|(name, temp)| {
                let bar_len = (*temp as usize / 5).min(40);
                let bar: String = "█".repeat(bar_len);
                format!("{:<24} {:>6.1}°C  {}", name, temp, bar)
            })
            .collect();
        temp_buffer.set_text(&temp_text.join("\n"));
    }

    let (total_disk, used_disk): (u64, u64) = {
        let mut total: u64 = 0;
        let mut used: u64 = 0;
        if let Ok(output) = std::process::Command::new("df").arg("--output=size,used").arg("-B1").arg("/").output() {
            if let Ok(stdout) = String::from_utf8(output.stdout) {
                for line in stdout.lines().skip(1) {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        if let (Ok(t), Ok(u)) = (parts[0].parse::<u64>(), parts[1].parse::<u64>()) {
                            total = t;
                            used = u;
                        }
                    }
                }
            }
        }
        (total, used)
    };
    let disk_pct = if total_disk > 0 {
        used_disk as f64 / total_disk as f64 * 100.0
    } else {
        0.0
    };
    disk_total_value.set_text(&format!(
        "{} / {} ({:.1}%)",
        format_bytes(used_disk),
        format_bytes(total_disk),
        disk_pct
    ));

    // ── Set up periodic updates ──

    let state = Rc::new(RefCell::new(MonitorState {
        sys,
        networks,
        prev_net_rx,
        prev_net_tx,
        prev_disk_read: 0,
        prev_disk_write: 0,
        prev_time: Instant::now(),
    }));

    let cpu_progress_c = cpu_progress.clone();
    let cpu_info_c = cpu_info_label.clone();
    let ram_progress_c = ram_progress.clone();
    let ram_info_c = ram_info.clone();
    let swap_progress_c = swap_progress.clone();
    let swap_info_c = swap_info.clone();
    let disk_read_value_c = disk_read_value.clone();
    let disk_write_value_c = disk_write_value.clone();
    let disk_total_value_c = disk_total_value.clone();
    let net_down_value_c = net_down_value.clone();
    let net_up_value_c = net_up_value.clone();
    let net_total_value_c = net_total_value.clone();
    let gpu_label_c = gpu_label.clone();
    let temp_buffer_c = temp_buffer.clone();

    gtk::glib::timeout_add_seconds_local(2, move || {
        let mut st = state.borrow_mut();

        st.sys.refresh_all();
        st.networks.refresh();

        // ── CPU ──

        let cpu_usage = st.sys.global_cpu_usage();
        let fraction = (cpu_usage / 100.0).clamp(0.0, 1.0) as f64;
        cpu_progress_c.set_fraction(fraction);
        cpu_progress_c.set_text(Some(&format!("{:.1}%", cpu_usage)));

        let cpus = st.sys.cpus();
        if let Some(cpu) = cpus.first() {
            let phys_cores = st
                .sys
                .physical_core_count()
                .map(|c| c.to_string())
                .unwrap_or_else(|| "unknown".to_string());
            cpu_info_c.set_text(&format!(
                "{}\n{} logical cores ({} physical) | {:.0} MHz",
                cpu.brand(),
                cpus.len(),
                phys_cores,
                cpu.frequency()
            ));
        }

        // ── Memory ──

        let total_mem = st.sys.total_memory();
        let used_mem = st.sys.used_memory();
        let ram_frac = if total_mem > 0 {
            used_mem as f64 / total_mem as f64
        } else {
            0.0
        };
        ram_progress_c.set_fraction(ram_frac);
        ram_progress_c.set_text(Some(&format!("{:.1}%", ram_frac * 100.0)));
        ram_info_c.set_text(&format!(
            "{} / {}",
            format_bytes(used_mem),
            format_bytes(total_mem)
        ));

        let total_swap = st.sys.total_swap();
        let used_swap = st.sys.used_swap();
        let swap_frac = if total_swap > 0 {
            used_swap as f64 / total_swap as f64
        } else {
            0.0
        };
        swap_progress_c.set_fraction(swap_frac);
        swap_progress_c.set_text(Some(&format!("{:.1}%", swap_frac * 100.0)));
        if total_swap > 0 {
            swap_info_c.set_text(&format!(
                "{} / {}",
                format_bytes(used_swap),
                format_bytes(total_swap)
            ));
        } else {
            swap_info_c.set_text("No swap configured");
        }

        // ── Disk I/O ──

        let now = Instant::now();
        let elapsed = now.duration_since(st.prev_time).as_secs_f64();

        // Read disk I/O from /proc/diskstats
        if let Ok(content) = std::fs::read_to_string("/proc/diskstats") {
            let mut total_read: u64 = 0;
            let mut total_write: u64 = 0;
            for line in content.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 14 {
                    let name = parts[2];
                    // Skip partitions (only count whole disks like sda, nvme0n1)
                    if name.starts_with("sd") && name.len() == 3 {
                        // sda, sdb etc. - whole disk, keep
                    } else if name.starts_with("sd") {
                        continue; // sda1, sdb2 etc. - partition, skip
                    } else if name.starts_with("nvme") && name.contains("p") {
                        continue; // nvme0n1p1 etc. - partition, skip
                    } else if name.starts_with("loop") || name.starts_with("ram") {
                        continue; // loop/ram devices, skip
                    }
                    if let (Ok(r), Ok(w)) = (parts[5].parse::<u64>(), parts[9].parse::<u64>()) {
                        total_read += r * 512; // sectors to bytes
                        total_write += w * 512;
                    }
                }
            }
            if elapsed > 0.5 {
                let read_speed = ((total_read.saturating_sub(st.prev_disk_read)) as f64 / elapsed) as u64;
                let write_speed = ((total_write.saturating_sub(st.prev_disk_write)) as f64 / elapsed) as u64;
                disk_read_value_c.set_text(&format_speed(read_speed));
                disk_write_value_c.set_text(&format_speed(write_speed));
            }
            st.prev_disk_read = total_read;
            st.prev_disk_write = total_write;
        }

        let (total_disk, used_disk): (u64, u64) = {
            let mut total: u64 = 0;
            let mut used: u64 = 0;
            if let Ok(output) = std::process::Command::new("df").arg("--output=size,used").arg("-B1").arg("/").output() {
                if let Ok(stdout) = String::from_utf8(output.stdout) {
                    for line in stdout.lines().skip(1) {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 2 {
                            if let (Ok(t), Ok(u)) = (parts[0].parse::<u64>(), parts[1].parse::<u64>()) {
                                total = t;
                                used = u;
                            }
                        }
                    }
                }
            }
            (total, used)
        };
        let disk_pct = if total_disk > 0 {
            used_disk as f64 / total_disk as f64 * 100.0
        } else {
            0.0
        };
        disk_total_value_c.set_text(&format!(
            "{} / {} ({:.1}%)",
            format_bytes(used_disk),
            format_bytes(total_disk),
            disk_pct
        ));

        // ── Network ──

        let cur_net_rx: u64 = st.networks.iter().map(|(_, n)| n.received()).sum();
        let cur_net_tx: u64 = st.networks.iter().map(|(_, n)| n.transmitted()).sum();

        if elapsed > 0.5 {
            let down_speed = ((cur_net_rx.saturating_sub(st.prev_net_rx)) as f64 / elapsed) as u64;
            let up_speed = ((cur_net_tx.saturating_sub(st.prev_net_tx)) as f64 / elapsed) as u64;
            net_down_value_c.set_text(&format_speed(down_speed));
            net_up_value_c.set_text(&format_speed(up_speed));
        }

        net_total_value_c.set_text(&format!(
            "↓ {} total | ↑ {} total",
            format_bytes(cur_net_rx),
            format_bytes(cur_net_tx)
        ));

        st.prev_net_rx = cur_net_rx;
        st.prev_net_tx = cur_net_tx;
        st.prev_time = now;

        // ── GPU ──

        gpu_label_c.set_text(&get_gpu_info());

        // ── Temperature ──

        let temps = get_temperatures();
        if temps.is_empty() {
            temp_buffer_c.set_text("No temperature sensors found");
        } else {
            let temp_text: Vec<String> = temps
                .iter()
                .map(|(name, temp)| {
                    let bar_len = (*temp as usize / 5).min(40);
                    let bar: String = "█".repeat(bar_len);
                    format!("{:<24} {:>6.1}°C  {}", name, temp, bar)
                })
                .collect();
            temp_buffer_c.set_text(&temp_text.join("\n"));
        }

        gtk::glib::ControlFlow::Continue
    });

    main_box.upcast()
}
