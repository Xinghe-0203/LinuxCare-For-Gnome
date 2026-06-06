use gtk::glib;
use gtk::prelude::*;
use gtk::{
    Adjustment, Box, Button, Frame, Grid, Label, Orientation, ScrolledWindow, SearchEntry,
    SpinButton, TextView,
};

use crate::tr;
use std::cell::RefCell;
use std::rc::Rc;
use sysinfo::{Pid, ProcessRefreshKind, System};

#[allow(dead_code)]
fn format_memory(bytes: u64) -> String {
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

fn get_status_label(process: &sysinfo::Process) -> String {
    match process.status() {
        sysinfo::ProcessStatus::Run => "Running".to_string(),
        sysinfo::ProcessStatus::Sleep => "Sleeping".to_string(),
        sysinfo::ProcessStatus::Stop => "Stopped".to_string(),
        sysinfo::ProcessStatus::Zombie => "Zombie".to_string(),
        sysinfo::ProcessStatus::Idle => "Idle".to_string(),
        sysinfo::ProcessStatus::Tracing => "Tracing".to_string(),
        sysinfo::ProcessStatus::Dead => "Dead".to_string(),
        sysinfo::ProcessStatus::Wakekill => "Wakekill".to_string(),
        sysinfo::ProcessStatus::Waking => "Waking".to_string(),
        sysinfo::ProcessStatus::Parked => "Parked".to_string(),
        sysinfo::ProcessStatus::UninterruptibleDiskSleep => "Disk Sleep".to_string(),
        sysinfo::ProcessStatus::LockBlocked => "Lock Blocked".to_string(),
        sysinfo::ProcessStatus::Unknown(st) => format!("Unknown({})", st),
    }
}

fn cmd_to_string(cmd: &[std::ffi::OsString]) -> String {
    cmd.iter()
        .map(|s| s.to_string_lossy().to_string())
        .collect::<Vec<_>>()
        .join(" ")
}

fn username_from_uid(uid: u32) -> String {
    use nix::unistd::Uid;
    nix::unistd::User::from_uid(Uid::from_raw(uid))
        .ok()
        .flatten()
        .map(|u| u.name.to_string())
        .unwrap_or_else(|| "-".to_string())
}

fn collect_process_data(
    sys: &System,
    filter: &str,
) -> Vec<(String, String, String, String, String, String)> {
    let filter_lower = filter.to_lowercase();
    let mut procs: Vec<(String, String, String, String, String, String)> = sys
        .processes()
        .iter()
        .filter(|(_, p)| {
            if filter_lower.is_empty() {
                return true;
            }
            let name = p.name().to_string_lossy().to_lowercase();
            let cmd = cmd_to_string(p.cmd()).to_lowercase();
            let user = p
                .user_id()
                .map(|uid| username_from_uid(**uid).to_lowercase())
                .unwrap_or_default();
            let pid_str = p.pid().to_string();
            name.contains(&filter_lower)
                || cmd.contains(&filter_lower)
                || user.contains(&filter_lower)
                || pid_str.contains(&filter_lower)
        })
        .map(|(_, p)| {
            let pid = p.pid().to_string();
            let name = p.name().to_string_lossy().to_string();
            let cpu = format!("{:.1}", p.cpu_usage());
            let mem = format!("{:.1}", p.memory() as f64 / 1_048_576.0);
            let user = p
                .user_id()
                .map(|uid| username_from_uid(**uid))
                .unwrap_or_else(|| "-".to_string());
            let status = get_status_label(p);
            (pid, name, cpu, mem, user, status)
        })
        .collect();

    procs.sort_by(|a, b| {
        let cpu_a: f64 = a.2.parse().unwrap_or(0.0);
        let cpu_b: f64 = b.2.parse().unwrap_or(0.0);
        cpu_b.partial_cmp(&cpu_a).unwrap_or(std::cmp::Ordering::Equal)
    });

    procs
}

fn format_process_list(procs: &[(String, String, String, String, String, String)]) -> String {
    if procs.is_empty() {
        return "No processes found.".to_string();
    }

    let header = format!(
        "{:<8} {:<24} {:>6} {:>10} {:<16} {:<12}",
        "PID", "NAME", "CPU%", "MEM(MB)", "USER", "STATUS"
    );
    let separator = "─".repeat(header.len());

    let mut lines = vec![header, separator];
    for (pid, name, cpu, mem, user, status) in procs {
        let display_name = if name.chars().count() > 24 {
            let truncated: String = name.chars().take(23).collect();
            format!("{}…", truncated)
        } else {
            name.clone()
        };
        lines.push(format!(
            "{:<8} {:<24} {:>6} {:>10} {:<16} {:<12}",
            pid, display_name, cpu, mem, user, status
        ));
    }
    lines.join("\n")
}

#[allow(dead_code)]
fn _get_process_details(sys: &System, pid_str: &str) -> String {
    let pid: Pid = match pid_str.parse() {
        Ok(p) => p,
        Err(_) => return "Invalid PID.".to_string(),
    };

    let Some(process) = sys.process(pid) else {
        return format!("Process {} not found.", pid_str);
    };

    let mut lines: Vec<String> = Vec::new();
    lines.push(format!("PID:          {}", process.pid()));
    lines.push(format!(
        "Name:         {}",
        process.name().to_string_lossy()
    ));
    lines.push(format!("Command:      {}", cmd_to_string(process.cmd())));
    lines.push(format!(
        "Executable:   {}",
        process
            .exe()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| "-".to_string())
    ));
    lines.push(format!(
        "Status:       {}",
        get_status_label(process)
    ));
    lines.push(format!("CPU Usage:    {:.1}%", process.cpu_usage()));
    lines.push(format!(
        "Memory:       {}",
        format_memory(process.memory())
    ));
    lines.push(format!(
        "Virtual Mem:  {}",
        format_memory(process.virtual_memory())
    ));
    lines.push(format!(
        "User ID:      {}",
        process
            .user_id()
            .map(|uid| uid.to_string())
            .unwrap_or_else(|| "-".to_string())
    ));
    lines.push(format!(
        "Group ID:     {}",
        process
            .group_id()
            .map(|gid| gid.to_string())
            .unwrap_or_else(|| "-".to_string())
    ));
    lines.push(format!(
        "Parent PID:   {}",
        process
            .parent()
            .map(|p| p.to_string())
            .unwrap_or_else(|| "-".to_string())
    ));
    lines.push(format!("Run Time:     {}s", process.run_time()));

    lines.join("\n")
}

fn get_status_summary(sys: &System) -> String {
    let mut running = 0u64;
    let mut sleeping = 0u64;
    let mut stopped = 0u64;
    let mut zombie = 0u64;
    let mut other = 0u64;

    for (_, p) in sys.processes() {
        match p.status() {
            sysinfo::ProcessStatus::Run => running += 1,
            sysinfo::ProcessStatus::Sleep => sleeping += 1,
            sysinfo::ProcessStatus::Stop => stopped += 1,
            sysinfo::ProcessStatus::Zombie => zombie += 1,
            _ => other += 1,
        }
    }

    let total = running + sleeping + stopped + zombie + other;

    format!(
        "Total: {}   Running: {}   Sleeping: {}   Stopped: {}   Zombie: {}   Other: {}",
        total, running, sleeping, stopped, zombie, other
    )
}

pub fn build_process_page() -> gtk::Widget {
    let main_box = Box::new(Orientation::Vertical, 20);
    main_box.set_margin_top(20);
    main_box.set_margin_bottom(20);
    main_box.set_margin_start(20);
    main_box.set_margin_end(20);

    let title = Label::new(Some(tr!("process")));
    title.set_css_classes(&["title-1"]);
    main_box.append(&title);

    // ── Resource Usage Summary ──

    let summary_frame = Frame::new(Some(tr!("resource_summary")));
    summary_frame.set_css_classes(&["card"]);

    let summary_label = Label::new(Some(tr!("loading")));
    summary_label.set_halign(gtk::Align::Start);
    summary_label.set_margin_top(10);
    summary_label.set_margin_bottom(10);
    summary_label.set_margin_start(12);
    summary_label.set_margin_end(12);
    summary_label.set_selectable(true);

    summary_frame.set_child(Some(&summary_label));
    main_box.append(&summary_frame);

    // ── Search ──

    let search_frame = Frame::new(Some(tr!("search")));
    search_frame.set_css_classes(&["card"]);

    let search_entry = SearchEntry::new();
    search_entry.set_placeholder_text(Some(tr!("search_hint")));
    search_entry.set_hexpand(true);
    search_entry.set_margin_top(10);
    search_entry.set_margin_bottom(10);
    search_entry.set_margin_start(12);
    search_entry.set_margin_end(12);

    search_frame.set_child(Some(&search_entry));
    main_box.append(&search_frame);

    // ── Process List ──

    let list_frame = Frame::new(Some(tr!("process_list")));
    list_frame.set_css_classes(&["card"]);

    let list_scrolled = ScrolledWindow::new();
    list_scrolled.set_min_content_height(300);
    list_scrolled.set_vexpand(true);

    let process_text_view = TextView::new();
    process_text_view.set_editable(false);
    process_text_view.set_monospace(true);
    process_text_view.set_wrap_mode(gtk::WrapMode::None);
    process_text_view.set_left_margin(8);
    process_text_view.set_top_margin(8);

    let list_buffer = process_text_view.buffer();
    list_buffer.set_text(&tr!("loading"));

    list_scrolled.set_child(Some(&process_text_view));
    list_scrolled.set_margin_top(10);
    list_scrolled.set_margin_bottom(10);
    list_scrolled.set_margin_start(12);
    list_scrolled.set_margin_end(12);

    list_frame.set_child(Some(&list_scrolled));
    main_box.append(&list_frame);

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

    let kill_pid_label = Label::new(Some(tr!("kill_pid")));
    kill_pid_label.set_halign(gtk::Align::Start);

    let kill_pid_entry = SearchEntry::new();
    kill_pid_entry.set_placeholder_text(Some(tr!("kill_pid")));
    kill_pid_entry.set_hexpand(true);

    let kill_button = Button::with_label(&tr!("kill_process"));
    kill_button.set_css_classes(&["destructive-action"]);

    let kill_pid_entry_c = kill_pid_entry.clone();
    kill_button.connect_clicked(move |_| {
        let pid_str = kill_pid_entry_c.text().to_string();
        if let Ok(pid) = pid_str.parse::<u32>() {
            use nix::sys::signal::{kill, Signal};
            use nix::unistd::Pid as NixPid;
            match kill(NixPid::from_raw(pid as i32), Signal::SIGTERM) {
                Ok(()) => {
                    kill_pid_entry_c.set_text("");
                }
                Err(e) => {
                    kill_pid_entry_c.set_text(&format!("Error: {}", e));
                }
            }
        }
    });

    let refresh_button = Button::with_label(&tr!("refresh"));
    refresh_button.set_css_classes(&["suggested-action"]);

    let priority_label = Label::new(Some(tr!("set_priority")));
    priority_label.set_halign(gtk::Align::Start);

    let priority_adj = Adjustment::new(0.0, -20.0, 19.0, 1.0, 5.0, 0.0);
    let priority_spin = SpinButton::new(Some(&priority_adj), 1.0, 0);
    priority_spin.set_halign(gtk::Align::Start);

    let nice_button = Button::with_label(&tr!("set_nice_value"));
    nice_button.set_css_classes(&["flat"]);

    let nice_pid_entry = SearchEntry::new();
    nice_pid_entry.set_placeholder_text(Some(tr!("pid_for_priority")));
    nice_pid_entry.set_hexpand(true);

    let priority_adj_c = priority_adj.clone();
    let nice_pid_entry_c = nice_pid_entry.clone();
    nice_button.connect_clicked(move |_| {
        let pid_str = nice_pid_entry_c.text().to_string();
        if let Ok(pid) = pid_str.parse::<u32>() {
            let nice_val = priority_adj_c.value() as i32;
            let _ = std::process::Command::new("renice")
                .args([
                    &nice_val.to_string(),
                    "-p",
                    &pid.to_string(),
                ])
                .output();
        }
    });

    actions_grid.attach(&kill_pid_label, 0, 0, 1, 1);
    actions_grid.attach(&kill_pid_entry, 1, 0, 1, 1);
    actions_grid.attach(&kill_button, 2, 0, 1, 1);
    actions_grid.attach(&refresh_button, 3, 0, 1, 1);
    actions_grid.attach(&priority_label, 0, 1, 1, 1);
    actions_grid.attach(&priority_spin, 1, 1, 1, 1);
    actions_grid.attach(&nice_pid_entry, 2, 1, 1, 1);
    actions_grid.attach(&nice_button, 3, 1, 1, 1);

    actions_frame.set_child(Some(&actions_grid));
    main_box.append(&actions_frame);

    // ── Process Details ──

    let details_frame = Frame::new(Some(tr!("process_details")));
    details_frame.set_css_classes(&["card"]);

    let details_scrolled = ScrolledWindow::new();
    details_scrolled.set_min_content_height(200);
    details_scrolled.set_max_content_height(350);

    let details_text_view = TextView::new();
    details_text_view.set_editable(false);
    details_text_view.set_monospace(true);
    details_text_view.set_wrap_mode(gtk::WrapMode::WordChar);
    details_text_view.set_left_margin(8);
    details_text_view.set_top_margin(8);

    let details_buffer = details_text_view.buffer();
    details_buffer.set_text("Click on a process PID in the list above to view details.");

    details_scrolled.set_child(Some(&details_text_view));
    details_scrolled.set_margin_top(10);
    details_scrolled.set_margin_bottom(10);
    details_scrolled.set_margin_start(12);
    details_scrolled.set_margin_end(12);

    details_frame.set_child(Some(&details_scrolled));
    main_box.append(&details_frame);

    // ── State and refresh logic ──

    let sys = System::new_with_specifics(
        sysinfo::RefreshKind::everything().with_processes(
            ProcessRefreshKind::new().with_cpu().with_memory(),
        ),
    );

    let state = Rc::new(RefCell::new(sys));

    let list_buffer_c = list_buffer.clone();
    let summary_label_c = summary_label.clone();
    let search_entry_c = search_entry.clone();
    let refresh_button_c = refresh_button.clone();
    let state_c = state.clone();

    let update_process_list = move || {
        let st = state_c.borrow();
        let filter = search_entry_c.text().to_string();
        let procs = collect_process_data(&st, &filter);
        list_buffer_c.set_text(&format_process_list(&procs));
        summary_label_c.set_text(&get_status_summary(&st));
    };

    update_process_list();

    {
        let state_c2 = state.clone();
        let list_buffer_c = list_buffer.clone();
        let summary_label_c = summary_label.clone();
        let search_entry_c = search_entry.clone();
        refresh_button_c.connect_clicked(move |_| {
            {
                let mut st = state_c2.borrow_mut();
                st.refresh_processes_specifics(
                    sysinfo::ProcessesToUpdate::All,
                    ProcessRefreshKind::new().with_cpu().with_memory(),
                );
            }
            let st = state_c2.borrow();
            let filter = search_entry_c.text().to_string();
            let procs = collect_process_data(&st, &filter);
            list_buffer_c.set_text(&format_process_list(&procs));
            summary_label_c.set_text(&get_status_summary(&st));
        });
    }

    {
        let update_fn = update_process_list.clone();
        search_entry.connect_activate(move |_| {
            update_fn();
        });
    }

    // ── Periodic refresh ──

    let state_c3 = state.clone();
    let list_buffer_c2 = list_buffer.clone();
    let summary_label_c2 = summary_label.clone();
    let search_entry_c2 = search_entry.clone();

    glib::timeout_add_seconds_local(3, move || {
        {
            let mut st = state_c3.borrow_mut();
            st.refresh_processes_specifics(
                sysinfo::ProcessesToUpdate::All,
                ProcessRefreshKind::new().with_cpu().with_memory(),
            );
        }

        let st = state_c3.borrow();
        let filter = search_entry_c2.text().to_string();
        let procs = collect_process_data(&st, &filter);
        list_buffer_c2.set_text(&format_process_list(&procs));
        summary_label_c2.set_text(&get_status_summary(&st));

        glib::ControlFlow::Continue
    });

    main_box.upcast()
}
