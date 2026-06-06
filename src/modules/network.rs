use crate::tr;
use crate::utils::spawn_bg;
use gtk::glib;
use gtk::prelude::*;
use gtk::{
    Box, Button, Entry, Frame, Grid, Label, Orientation, ScrolledWindow, Switch, TextView,
};

fn run_nmcli(args: &[&str]) -> String {
    std::process::Command::new("nmcli")
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

fn run_cmd_async(cmd: &str, args: Vec<String>, callback: impl FnOnce(String) + 'static) {
    let cmd = cmd.to_string();
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let output = std::process::Command::new(&cmd)
            .args(&args)
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
            .unwrap_or_else(|e| format!("Error: {}", e));
        let _ = tx.send(output);
    });
    let mut callback = Some(callback);
    glib::timeout_add_local(std::time::Duration::from_millis(50), move || {
        match rx.try_recv() {
            Ok(output) => {
                if let Some(cb) = callback.take() {
                    cb(output);
                }
                glib::ControlFlow::Break
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => glib::ControlFlow::Continue,
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                if let Some(cb) = callback.take() {
                    cb(String::from("Error: channel disconnected"));
                }
                glib::ControlFlow::Break
            }
        }
    });
}

fn get_active_connection_name() -> Option<String> {
    let output = run_nmcli(&["-t", "-f", "NAME,TYPE", "connection", "show", "--active"]);
    for line in output.lines() {
        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() >= 2 && parts[1] == "802-11-wireless" {
            return Some(parts[0].to_string());
        }
    }
    // Fallback: first active connection
    output
        .lines()
        .next()
        .and_then(|l| l.split(':').next())
        .map(|s| s.to_string())
}

fn parse_proxy_url(url: &str) -> (String, String) {
    let stripped = url
        .strip_prefix("http://")
        .or_else(|| url.strip_prefix("https://"))
        .or_else(|| url.strip_prefix("socks5://"))
        .or_else(|| url.strip_prefix("socks4://"))
        .unwrap_or(url);
    if let Some((host, port)) = stripped.rsplit_once(':') {
        (host.to_string(), port.to_string())
    } else {
        (stripped.to_string(), String::new())
    }
}

fn gsettings_set(schema: &str, key: &str, value: &str) {
    let _ = std::process::Command::new("gsettings")
        .args(["set", schema, key, value])
        .output();
}

fn write_proxy_gsettings(
    enabled: bool,
    http_host: &str,
    http_port: &str,
    https_host: &str,
    https_port: &str,
    socks_host: &str,
    socks_port: &str,
) {
    gsettings_set(
        "org.gnome.system.proxy",
        "mode",
        if enabled { "manual" } else { "none" },
    );
    if enabled {
        gsettings_set(
            "org.gnome.system.proxy.http",
            "host",
            http_host,
        );
        gsettings_set("org.gnome.system.proxy.http", "port", http_port);
        gsettings_set(
            "org.gnome.system.proxy.https",
            "host",
            https_host,
        );
        gsettings_set("org.gnome.system.proxy.https", "port", https_port);
        gsettings_set(
            "org.gnome.system.proxy.socks",
            "host",
            socks_host,
        );
        gsettings_set("org.gnome.system.proxy.socks", "port", socks_port);
    }
}

fn get_wifi_status() -> String {
    let output = run_nmcli(&["-t", "-f", "TYPE,STATE,NAME", "connection", "show", "--active"]);
    for line in output.lines() {
        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() >= 3 && parts[0] == "802-11-wireless" {
            return format!("Connected to: {}", parts[2]);
        }
    }
    "No WiFi connection".to_string()
}

fn get_active_connections() -> Vec<String> {
    let output = run_nmcli(&["-t", "-f", "NAME,TYPE,DEVICE", "connection", "show", "--active"]);
    output
        .lines()
        .filter(|l| !l.is_empty())
        .map(|l| l.to_string())
        .collect()
}

fn get_vpn_connections() -> Vec<String> {
    let output = run_nmcli(&["-t", "-f", "NAME,TYPE", "connection", "show"]);
    output
        .lines()
        .filter(|l| l.contains("vpn") || l.contains("tun") || l.contains("wireguard"))
        .map(|l| {
            let parts: Vec<&str> = l.split(':').collect();
            if !parts.is_empty() {
                parts[0].to_string()
            } else {
                l.to_string()
            }
        })
        .collect()
}

fn _get_all_connections() -> Vec<String> {
    let output = run_nmcli(&["-t", "-f", "NAME,TYPE", "connection", "show"]);
    output
        .lines()
        .filter(|l| !l.is_empty())
        .map(|l| {
            let parts: Vec<&str> = l.split(':').collect();
            if parts.len() >= 2 {
                format!("{} ({})", parts[0], parts[1])
            } else {
                l.to_string()
            }
        })
        .collect()
}

fn get_network_interfaces() -> Vec<String> {
    let output = run_nmcli(&["-t", "-f", "DEVICE,TYPE,STATE,CONNECTION", "device", "status"]);
    output
        .lines()
        .filter(|l| !l.is_empty())
        .map(|l| l.to_string())
        .collect()
}

fn get_ip_info() -> String {
    let output = run_nmcli(&[
        "-t",
        "-f",
        "IP4.ADDRESS,IP4.GATEWAY,IP4.DNS",
        "device",
        "show",
    ]);
    if output.trim().is_empty() {
        return "No IP information available".to_string();
    }
    output.trim().to_string()
}

fn get_dns_servers() -> Vec<String> {
    let output = run_nmcli(&["-t", "-f", "IP4.DNS", "device", "show"]);
    let mut dns = Vec::new();
    for line in output.lines() {
        // Match IP4.DNS[1]:, IP4.DNS[2]:, etc.
        if let Some(idx) = line.find("IP4.DNS[") {
            if let Some(colon_pos) = line[idx..].find(':') {
                let server = line[idx + colon_pos + 1..].trim();
                if !server.is_empty() {
                    dns.push(server.to_string());
                }
            }
        }
    }
    dns.dedup();
    dns
}

fn get_proxy_info() -> (String, String, String) {
    let http_proxy = std::env::var("http_proxy")
        .or_else(|_| std::env::var("HTTP_PROXY"))
        .unwrap_or_default();
    let https_proxy = std::env::var("https_proxy")
        .or_else(|_| std::env::var("HTTPS_PROXY"))
        .unwrap_or_default();
    let socks_proxy = std::env::var("all_proxy")
        .or_else(|_| std::env::var("ALL_PROXY"))
        .unwrap_or_default();
    (http_proxy, https_proxy, socks_proxy)
}

pub fn build_network_page() -> gtk::Widget {
    let main_box = Box::new(Orientation::Vertical, 20);
    main_box.set_margin_top(20);
    main_box.set_margin_bottom(20);
    main_box.set_margin_start(20);
    main_box.set_margin_end(20);

    let title = Label::new(Some(tr!("network_management")));
    title.set_css_classes(&["title-1"]);
    main_box.append(&title);

    // ── WiFi Management ──

    let wifi_frame = Frame::new(Some(tr!("wifi_management")));
    wifi_frame.set_css_classes(&["card"]);

    let wifi_grid = Grid::new();
    wifi_grid.set_column_spacing(12);
    wifi_grid.set_row_spacing(10);
    wifi_grid.set_margin_top(10);
    wifi_grid.set_margin_bottom(10);
    wifi_grid.set_margin_start(12);
    wifi_grid.set_margin_end(12);

    let wifi_status_label = Label::new(Some(&get_wifi_status()));
    wifi_status_label.set_halign(gtk::Align::Start);
    wifi_status_label.set_selectable(true);

    let wifi_scan_button = Button::with_label(tr!("scan_networks"));
    wifi_scan_button.set_halign(gtk::Align::Start);
    wifi_scan_button.set_css_classes(&["suggested-action"]);

    let wifi_connect_button = Button::with_label(tr!("connect"));
    wifi_connect_button.set_halign(gtk::Align::Start);
    wifi_connect_button.set_css_classes(&["suggested-action"]);

    let wifi_disconnect_button = Button::with_label(tr!("disconnect"));
    wifi_disconnect_button.set_halign(gtk::Align::Start);
    wifi_disconnect_button.set_css_classes(&["destructive-action"]);

    wifi_grid.attach(&wifi_status_label, 0, 0, 4, 1);
    wifi_grid.attach(&wifi_scan_button, 0, 1, 1, 1);
    wifi_grid.attach(&wifi_connect_button, 1, 1, 1, 1);
    wifi_grid.attach(&wifi_disconnect_button, 2, 1, 1, 1);

    wifi_frame.set_child(Some(&wifi_grid));
    main_box.append(&wifi_frame);

    // ── WiFi Scan Results ──

    let wifi_scan_frame = Frame::new(Some(tr!("available_wifi")));
    wifi_scan_frame.set_css_classes(&["card"]);

    let wifi_scan_scrolled = ScrolledWindow::new();
    wifi_scan_scrolled.set_min_content_height(150);
    wifi_scan_scrolled.set_max_content_height(250);

    let wifi_scan_text = TextView::new();
    wifi_scan_text.set_editable(false);
    wifi_scan_text.set_monospace(true);
    wifi_scan_text.set_vexpand(true);

    let wifi_scan_buffer = wifi_scan_text.buffer();
    wifi_scan_buffer.set_text("Click 'Scan Networks' to find available WiFi networks...");

    wifi_scan_scrolled.set_child(Some(&wifi_scan_text));
    wifi_scan_scrolled.set_margin_top(10);
    wifi_scan_scrolled.set_margin_bottom(10);
    wifi_scan_scrolled.set_margin_start(12);
    wifi_scan_scrolled.set_margin_end(12);

    wifi_scan_frame.set_child(Some(&wifi_scan_scrolled));
    main_box.append(&wifi_scan_frame);

    // ── VPN Management ──

    let vpn_frame = Frame::new(Some(tr!("vpn_management")));
    vpn_frame.set_css_classes(&["card"]);

    let vpn_grid = Grid::new();
    vpn_grid.set_column_spacing(12);
    vpn_grid.set_row_spacing(10);
    vpn_grid.set_margin_top(10);
    vpn_grid.set_margin_bottom(10);
    vpn_grid.set_margin_start(12);
    vpn_grid.set_margin_end(12);

    let vpn_list_label = Label::new(Some(tr!("vpn_connections")));
    vpn_list_label.set_halign(gtk::Align::Start);
    vpn_list_label.set_css_classes(&["heading"]);

    let vpn_connections = get_vpn_connections();
    let vpn_text = if vpn_connections.is_empty() {
        "No VPN connections configured".to_string()
    } else {
        vpn_connections.join("\n")
    };
    let vpn_list_value = Label::new(Some(&vpn_text));
    vpn_list_value.set_halign(gtk::Align::Start);
    vpn_list_value.set_selectable(true);
    vpn_list_value.set_wrap(true);

    let vpn_add_button = Button::with_label(tr!("add_vpn"));
    vpn_add_button.set_halign(gtk::Align::Start);
    vpn_add_button.set_css_classes(&["suggested-action"]);

    let vpn_edit_button = Button::with_label(tr!("edit_vpn"));
    vpn_edit_button.set_halign(gtk::Align::Start);

    let vpn_remove_button = Button::with_label(tr!("remove_vpn"));
    vpn_remove_button.set_halign(gtk::Align::Start);
    vpn_remove_button.set_css_classes(&["destructive-action"]);

    vpn_grid.attach(&vpn_list_label, 0, 0, 4, 1);
    vpn_grid.attach(&vpn_list_value, 0, 1, 4, 1);
    vpn_grid.attach(&vpn_add_button, 0, 2, 1, 1);
    vpn_grid.attach(&vpn_edit_button, 1, 2, 1, 1);
    vpn_grid.attach(&vpn_remove_button, 2, 2, 1, 1);

    vpn_frame.set_child(Some(&vpn_grid));
    main_box.append(&vpn_frame);

    // ── Proxy Settings ──

    let proxy_frame = Frame::new(Some(tr!("proxy_settings")));
    proxy_frame.set_css_classes(&["card"]);

    let proxy_grid = Grid::new();
    proxy_grid.set_column_spacing(12);
    proxy_grid.set_row_spacing(10);
    proxy_grid.set_margin_top(10);
    proxy_grid.set_margin_bottom(10);
    proxy_grid.set_margin_start(12);
    proxy_grid.set_margin_end(12);

    let (http_proxy, https_proxy, socks_proxy) = get_proxy_info();

    let proxy_enable_label = Label::new(Some(tr!("enable_proxy")));
    proxy_enable_label.set_halign(gtk::Align::Start);
    let proxy_switch = Switch::new();
    proxy_switch.set_active(!http_proxy.is_empty());
    proxy_switch.set_halign(gtk::Align::Start);

    let http_label = Label::new(Some(tr!("http_proxy")));
    http_label.set_halign(gtk::Align::Start);
    let http_entry = Entry::new();
    http_entry.set_text(&http_proxy);
    http_entry.set_hexpand(true);
    http_entry.set_placeholder_text(Some("http://proxy:port"));

    let https_label = Label::new(Some(tr!("https_proxy")));
    https_label.set_halign(gtk::Align::Start);
    let https_entry = Entry::new();
    https_entry.set_text(&https_proxy);
    https_entry.set_hexpand(true);
    https_entry.set_placeholder_text(Some("https://proxy:port"));

    let socks_label = Label::new(Some(tr!("socks_proxy")));
    socks_label.set_halign(gtk::Align::Start);
    let socks_entry = Entry::new();
    socks_entry.set_text(&socks_proxy);
    socks_entry.set_hexpand(true);
    socks_entry.set_placeholder_text(Some("socks5://proxy:port"));

    let no_proxy_label = Label::new(Some(tr!("no_proxy")));
    no_proxy_label.set_halign(gtk::Align::Start);
    let no_proxy_entry = Entry::new();
    no_proxy_entry.set_hexpand(true);
    no_proxy_entry.set_placeholder_text(Some("localhost,127.0.0.1"));

    let apply_proxy_button = Button::with_label(tr!("apply_proxy"));
    apply_proxy_button.set_halign(gtk::Align::Start);
    apply_proxy_button.set_css_classes(&["suggested-action"]);

    proxy_grid.attach(&proxy_enable_label, 0, 0, 1, 1);
    proxy_grid.attach(&proxy_switch, 1, 0, 1, 1);
    proxy_grid.attach(&http_label, 0, 1, 1, 1);
    proxy_grid.attach(&http_entry, 1, 1, 2, 1);
    proxy_grid.attach(&https_label, 0, 2, 1, 1);
    proxy_grid.attach(&https_entry, 1, 2, 2, 1);
    proxy_grid.attach(&socks_label, 0, 3, 1, 1);
    proxy_grid.attach(&socks_entry, 1, 3, 2, 1);
    proxy_grid.attach(&no_proxy_label, 0, 4, 1, 1);
    proxy_grid.attach(&no_proxy_entry, 1, 4, 2, 1);
    proxy_grid.attach(&apply_proxy_button, 0, 5, 3, 1);

    proxy_frame.set_child(Some(&proxy_grid));
    main_box.append(&proxy_frame);

    // ── DNS Settings ──

    let dns_frame = Frame::new(Some(tr!("dns_settings")));
    dns_frame.set_css_classes(&["card"]);

    let dns_grid = Grid::new();
    dns_grid.set_column_spacing(12);
    dns_grid.set_row_spacing(10);
    dns_grid.set_margin_top(10);
    dns_grid.set_margin_bottom(10);
    dns_grid.set_margin_start(12);
    dns_grid.set_margin_end(12);

    let dns_servers_label = Label::new(Some(tr!("current_dns")));
    dns_servers_label.set_halign(gtk::Align::Start);
    dns_servers_label.set_css_classes(&["heading"]);

    let dns_servers = get_dns_servers();
    let dns_text = if dns_servers.is_empty() {
        "No DNS servers configured".to_string()
    } else {
        dns_servers.join("\n")
    };
    let dns_servers_value = Label::new(Some(&dns_text));
    dns_servers_value.set_halign(gtk::Align::Start);
    dns_servers_value.set_selectable(true);

    let dns_add_label = Label::new(Some(tr!("add_dns_server")));
    dns_add_label.set_halign(gtk::Align::Start);
    let dns_add_entry = Entry::new();
    dns_add_entry.set_hexpand(true);
    dns_add_entry.set_placeholder_text(Some("8.8.8.8"));

    let dns_apply_button = Button::with_label(tr!("add_dns"));
    dns_apply_button.set_halign(gtk::Align::Start);
    dns_apply_button.set_css_classes(&["suggested-action"]);

    let dns_reset_button = Button::with_label(tr!("dns_reset"));
    dns_reset_button.set_halign(gtk::Align::Start);

    dns_grid.attach(&dns_servers_label, 0, 0, 3, 1);
    dns_grid.attach(&dns_servers_value, 0, 1, 3, 1);
    dns_grid.attach(&dns_add_label, 0, 2, 1, 1);
    dns_grid.attach(&dns_add_entry, 1, 2, 2, 1);
    dns_grid.attach(&dns_apply_button, 0, 3, 1, 1);
    dns_grid.attach(&dns_reset_button, 1, 3, 1, 1);

    dns_frame.set_child(Some(&dns_grid));
    main_box.append(&dns_frame);

    // ── Network Diagnostics ──

    let diag_frame = Frame::new(Some(tr!("network_diagnostics")));
    diag_frame.set_css_classes(&["card"]);

    let diag_grid = Grid::new();
    diag_grid.set_column_spacing(12);
    diag_grid.set_row_spacing(10);
    diag_grid.set_margin_top(10);
    diag_grid.set_margin_bottom(10);
    diag_grid.set_margin_start(12);
    diag_grid.set_margin_end(12);

    let diag_host_label = Label::new(Some(tr!("target_host")));
    diag_host_label.set_halign(gtk::Align::Start);
    let diag_host_entry = Entry::new();
    diag_host_entry.set_hexpand(true);
    diag_host_entry.set_placeholder_text(Some("8.8.8.8"));
    diag_host_entry.set_text("8.8.8.8");

    let ping_button = Button::with_label(tr!("ping"));
    ping_button.set_halign(gtk::Align::Start);
    ping_button.set_css_classes(&["suggested-action"]);

    let traceroute_button = Button::with_label(tr!("traceroute"));
    traceroute_button.set_halign(gtk::Align::Start);

    let speedtest_button = Button::with_label(tr!("speed_test"));
    speedtest_button.set_halign(gtk::Align::Start);
    speedtest_button.set_css_classes(&["suggested-action"]);

    diag_grid.attach(&diag_host_label, 0, 0, 1, 1);
    diag_grid.attach(&diag_host_entry, 1, 0, 3, 1);
    diag_grid.attach(&ping_button, 0, 1, 1, 1);
    diag_grid.attach(&traceroute_button, 1, 1, 1, 1);
    diag_grid.attach(&speedtest_button, 2, 1, 1, 1);

    diag_frame.set_child(Some(&diag_grid));
    main_box.append(&diag_frame);

    // ── Diagnostics Output ──

    let diag_output_frame = Frame::new(Some(tr!("diag_output")));
    diag_output_frame.set_css_classes(&["card"]);

    let diag_output_scrolled = ScrolledWindow::new();
    diag_output_scrolled.set_min_content_height(200);
    diag_output_scrolled.set_max_content_height(350);

    let diag_output_text = TextView::new();
    diag_output_text.set_editable(false);
    diag_output_text.set_monospace(true);
    diag_output_text.set_vexpand(true);

    let diag_output_buffer = diag_output_text.buffer();
    diag_output_buffer.set_text(tr!("run_diag"));

    diag_output_scrolled.set_child(Some(&diag_output_text));
    diag_output_scrolled.set_margin_top(10);
    diag_output_scrolled.set_margin_bottom(10);
    diag_output_scrolled.set_margin_start(12);
    diag_output_scrolled.set_margin_end(12);

    diag_output_frame.set_child(Some(&diag_output_scrolled));
    main_box.append(&diag_output_frame);

    // ── Network Info ──

    let info_frame = Frame::new(Some(tr!("network_info")));
    info_frame.set_css_classes(&["card"]);

    let info_grid = Grid::new();
    info_grid.set_column_spacing(12);
    info_grid.set_row_spacing(10);
    info_grid.set_margin_top(10);
    info_grid.set_margin_bottom(10);
    info_grid.set_margin_start(12);
    info_grid.set_margin_end(12);

    let active_conns_label = Label::new(Some(tr!("active_connections")));
    active_conns_label.set_halign(gtk::Align::Start);
    active_conns_label.set_css_classes(&["heading"]);

    let active_conns = get_active_connections();
    let active_conns_text = if active_conns.is_empty() {
        "No active connections".to_string()
    } else {
        active_conns.join("\n")
    };
    let active_conns_value = Label::new(Some(&active_conns_text));
    active_conns_value.set_halign(gtk::Align::Start);
    active_conns_value.set_selectable(true);
    active_conns_value.set_wrap(true);

    let interfaces_label = Label::new(Some(tr!("interfaces")));
    interfaces_label.set_halign(gtk::Align::Start);
    interfaces_label.set_css_classes(&["heading"]);

    let interfaces = get_network_interfaces();
    let interfaces_text = if interfaces.is_empty() {
        "No interfaces found".to_string()
    } else {
        interfaces.join("\n")
    };
    let interfaces_value = Label::new(Some(&interfaces_text));
    interfaces_value.set_halign(gtk::Align::Start);
    interfaces_value.set_selectable(true);
    interfaces_value.set_wrap(true);

    let ip_label = Label::new(Some(tr!("ip_config")));
    ip_label.set_halign(gtk::Align::Start);
    ip_label.set_css_classes(&["heading"]);

    let ip_info = get_ip_info();
    let ip_value = Label::new(Some(&ip_info));
    ip_value.set_halign(gtk::Align::Start);
    ip_value.set_selectable(true);
    ip_value.set_wrap(true);

    info_grid.attach(&active_conns_label, 0, 0, 2, 1);
    info_grid.attach(&active_conns_value, 0, 1, 2, 1);
    info_grid.attach(&interfaces_label, 0, 2, 2, 1);
    info_grid.attach(&interfaces_value, 0, 3, 2, 1);
    info_grid.attach(&ip_label, 0, 4, 2, 1);
    info_grid.attach(&ip_value, 0, 5, 2, 1);

    info_frame.set_child(Some(&info_grid));
    main_box.append(&info_frame);

    // ── Refresh Button ──

    let refresh_frame = Frame::new(Some(tr!("actions")));
    refresh_frame.set_css_classes(&["card"]);

    let refresh_grid = Grid::new();
    refresh_grid.set_column_spacing(12);
    refresh_grid.set_row_spacing(10);
    refresh_grid.set_margin_top(10);
    refresh_grid.set_margin_bottom(10);
    refresh_grid.set_margin_start(12);
    refresh_grid.set_margin_end(12);

    let refresh_button = Button::with_label(tr!("refresh_all"));
    refresh_button.set_halign(gtk::Align::Start);
    refresh_button.set_css_classes(&["suggested-action"]);

    let open_settings_button = Button::with_label(tr!("open_network_settings"));
    open_settings_button.set_halign(gtk::Align::Start);

    refresh_grid.attach(&refresh_button, 0, 0, 1, 1);
    refresh_grid.attach(&open_settings_button, 1, 0, 1, 1);

    refresh_frame.set_child(Some(&refresh_grid));
    main_box.append(&refresh_frame);

    // ── Button handlers using thread + idle callback (non-blocking) ──

    {
        let wifi_scan_buffer = wifi_scan_buffer.clone();
        let wifi_status_label = wifi_status_label.clone();

        wifi_scan_button.connect_clicked(move |_| {
            let buffer = wifi_scan_buffer.clone();
            let status_label = wifi_status_label.clone();
            buffer.set_text(tr!("scanning"));
            run_cmd_async(
                "nmcli",
                vec![
                    "-t".into(),
                    "-f".into(),
                    "SSID,SIGNAL,SECURITY".into(),
                    "device".into(),
                    "wifi".into(),
                    "list".into(),
                ],
                move |output| {
                    if output.trim().is_empty() {
                        buffer.set_text(tr!("no_wifi"));
                    } else {
                        let mut lines: Vec<String> = output
                            .lines()
                            .filter(|l| !l.is_empty())
                            .map(|l| {
                                let parts: Vec<&str> = l.split(':').collect();
                                if parts.len() >= 3 {
                                    format!(
                                        "{:<30} Signal: {:<5} Security: {}",
                                        parts[0], parts[1], parts[2]
                                    )
                                } else {
                                    l.to_string()
                                }
                            })
                            .collect();
                        lines.insert(
                            0,
                            format!(
                                "{:<30} {:<5}      {}",
                                "SSID", "SIGNAL", "SECURITY"
                            ),
                        );
                        lines.insert(1, "─".repeat(50));
                        buffer.set_text(&lines.join("\n"));
                    }
                    status_label.set_text(&get_wifi_status());
                },
            );
        });
    }

    {
        let wifi_status_label = wifi_status_label.clone();
        wifi_connect_button.connect_clicked(move |btn| {
            let status_label = wifi_status_label.clone();
            let window = btn
                .root()
                .and_then(|r| r.downcast::<gtk::Window>().ok());

            let dialog = gtk::Dialog::with_buttons(
                Some("Connect to WiFi"),
                window.as_ref(),
                gtk::DialogFlags::MODAL | gtk::DialogFlags::USE_HEADER_BAR,
                &[
                    ("Cancel", gtk::ResponseType::Cancel),
                    ("Connect", gtk::ResponseType::Accept),
                ],
            );
            dialog.set_default_response(gtk::ResponseType::Accept);

            let content = dialog.content_area();
            content.set_spacing(10);
            content.set_margin_top(10);
            content.set_margin_bottom(10);
            content.set_margin_start(20);
            content.set_margin_end(20);

            let ssid_label = Label::new(Some("SSID:"));
            ssid_label.set_halign(gtk::Align::Start);
            let ssid_entry = Entry::new();
            ssid_entry.set_hexpand(true);
            ssid_entry.set_placeholder_text(Some("Network name"));

            let pass_label = Label::new(Some("Password:"));
            pass_label.set_halign(gtk::Align::Start);
            let pass_entry = Entry::new();
            pass_entry.set_hexpand(true);
            pass_entry.set_placeholder_text(Some("WiFi password"));

            let grid = Grid::new();
            grid.set_column_spacing(12);
            grid.set_row_spacing(8);
            grid.attach(&ssid_label, 0, 0, 1, 1);
            grid.attach(&ssid_entry, 1, 0, 1, 1);
            grid.attach(&pass_label, 0, 1, 1, 1);
            grid.attach(&pass_entry, 1, 1, 1, 1);
            content.append(&grid);

            let ssid_entry_clone = ssid_entry.clone();
            let pass_entry_clone = pass_entry.clone();
            dialog.connect_response(move |dlg, response| {
                if response == gtk::ResponseType::Accept {
                    let ssid = ssid_entry_clone.text().to_string();
                    let password = pass_entry_clone.text().to_string();
                    if !ssid.is_empty() {
                        let status = status_label.clone();
                        status.set_text(&format!("Connecting to {}...", ssid));
                        let args = if password.is_empty() {
                            vec![
                                "device".into(),
                                "wifi".into(),
                                "connect".into(),
                                ssid,
                            ]
                        } else {
                            vec![
                                "device".into(),
                                "wifi".into(),
                                "connect".into(),
                                ssid,
                                "password".into(),
                                password,
                            ]
                        };
                        run_cmd_async("nmcli", args, move |output| {
                            if output.contains("successfully") {
                                status.set_text(&format!("Connected: {}", output.trim()));
                            } else {
                                status.set_text(&format!("Connection result: {}", output.trim()));
                            }
                        });
                    }
                }
                dlg.close();
            });

            dialog.show();
        });
    }

    {
        let wifi_status_label = wifi_status_label.clone();
        wifi_disconnect_button.connect_clicked(move |_| {
            let status_label = wifi_status_label.clone();
            status_label.set_text("Disconnecting...");
            run_cmd_async(
                "nmcli",
                vec![
                    "-t".into(),
                    "-f".into(),
                    "NAME,TYPE".into(),
                    "connection".into(),
                    "show".into(),
                    "--active".into(),
                ],
                move |output| {
                    let mut wifi_name: Option<String> = None;
                    for line in output.lines() {
                        if line.contains("802-11-wireless") {
                            wifi_name = Some(
                                line.split(':').next().unwrap_or("").to_string(),
                            );
                            break;
                        }
                    }
                    if let Some(name) = wifi_name {
                        run_cmd_async(
                            "nmcli",
                            vec!["connection".into(), "down".into(), name],
                            move |_| {
                                status_label.set_text(&get_wifi_status());
                            },
                        );
                    } else {
                        status_label.set_text(&get_wifi_status());
                    }
                },
            );
        });
    }

    {
        let diag_host_entry = diag_host_entry.clone();
        let diag_output_buffer = diag_output_buffer.clone();
        ping_button.connect_clicked(move |_| {
            let host = diag_host_entry.text().to_string();
            let buffer = diag_output_buffer.clone();
            buffer.set_text(&format!("Pinging {}...", host));
            let buffer2 = buffer.clone();
            run_cmd_async(
                "nmcli",
                vec![
                    "-t".into(),
                    "-f".into(),
                    "general".into(),
                    "networking".into(),
                    "status".into(),
                ],
                move |nm_output| {
                    let nm_status = nm_output.trim().to_string();
                    let buffer3 = buffer2.clone();
                    run_cmd_async(
                        "ping",
                        vec!["-c".into(), "4".into(), host],
                        move |ping_output| {
                            let result = format!(
                                "NetworkManager: {}\n\nPing results:\n{}",
                                nm_status, ping_output
                            );
                            buffer3.set_text(&result);
                        },
                    );
                },
            );
        });
    }

    {
        let diag_host_entry = diag_host_entry.clone();
        let diag_output_buffer = diag_output_buffer.clone();
        traceroute_button.connect_clicked(move |_| {
            let host = diag_host_entry.text().to_string();
            let buffer = diag_output_buffer.clone();
            buffer.set_text(&format!("Traceroute to {}...", host));
            run_cmd_async(
                "traceroute",
                vec!["-m".into(), "15".into(), host],
                move |output| {
                    buffer.set_text(&format!("Traceroute results:\n{}", output));
                },
            );
        });
    }

    {
        let diag_output_buffer = diag_output_buffer.clone();
        speedtest_button.connect_clicked(move |_| {
            let buffer = diag_output_buffer.clone();
            buffer.set_text(tr!("running_speed"));
            run_cmd_async(
                "curl",
                vec![
                    "--connect-timeout".into(),
                    "10".into(),
                    "--max-time".into(),
                    "30".into(),
                    "-o".into(),
                    "/dev/null".into(),
                    "-w".into(),
                    "%{speed_download}".into(),
                    "http://speedtest.tele2.net/10MB.zip".into(),
                ],
                move |output| {
                    let speed: f64 = output.trim().parse().unwrap_or(0.0);
                    let result = format!(
                        "Speed Test Results:\n\
                         Download speed: {:.2} MB/s",
                        speed / 1_048_576.0,
                    );
                    buffer.set_text(&result);
                },
            );
        });
    }

    {
        let apply_proxy_button_c = apply_proxy_button.clone();
        let http_entry_c = http_entry.clone();
        let https_entry_c = https_entry.clone();
        let socks_entry_c = socks_entry.clone();
        let no_proxy_entry_c = no_proxy_entry.clone();
        let proxy_switch_c = proxy_switch.clone();
        apply_proxy_button_c.connect_clicked(move |_| {
            let http = http_entry_c.text().to_string();
            let https = https_entry_c.text().to_string();
            let socks = socks_entry_c.text().to_string();
            let no_proxy = no_proxy_entry_c.text().to_string();
            let enabled = proxy_switch_c.is_active();

            // Write environment variables (as before)
            let env_vars = [
                ("http_proxy", &http),
                ("HTTP_PROXY", &http),
                ("https_proxy", &https),
                ("HTTPS_PROXY", &https),
                ("all_proxy", &socks),
                ("ALL_PROXY", &socks),
                ("no_proxy", &no_proxy),
                ("NO_PROXY", &no_proxy),
            ];

            if enabled {
                for (key, value) in &env_vars {
                    if !value.is_empty() {
                        std::env::set_var(key, value);
                    }
                }
            } else {
                for (key, _) in &env_vars {
                    std::env::remove_var(key);
                }
            }

            // Persist proxy settings via gsettings
            let (http_host, http_port) = parse_proxy_url(&http);
            let (https_host, https_port) = parse_proxy_url(&https);
            let (socks_host, socks_port) = parse_proxy_url(&socks);
            write_proxy_gsettings(
                enabled,
                &http_host,
                &http_port,
                &https_host,
                &https_port,
                &socks_host,
                &socks_port,
            );
        });
    }

    {
        let dns_add_entry = dns_add_entry.clone();
        let dns_servers_value = dns_servers_value.clone();
        dns_apply_button.connect_clicked(move |_| {
            let server = dns_add_entry.text().to_string();
            if server.is_empty() {
                return;
            }
            let dns_servers_value_c = dns_servers_value.clone();
            let dns_add_entry_c = dns_add_entry.clone();
            let server_c = server.clone();
            spawn_bg(
                move || {
                    if let Some(conn) = get_active_connection_name() {
                        run_nmcli(&["connection", "modify", &conn, "+ipv4.dns", &server_c]);
                        run_nmcli(&["connection", "up", &conn]);
                    }
                },
                move |_| {
                    dns_servers_value_c.set_text(&format!(
                        "{}\n{}",
                        dns_servers_value_c.text(),
                        server
                    ));
                    dns_add_entry_c.set_text("");
                },
            );
        });
    }

    {
        let dns_servers_value = dns_servers_value.clone();
        dns_reset_button.connect_clicked(move |_| {
            let dns_servers_value_c = dns_servers_value.clone();
            spawn_bg(
                move || {
                    let output = run_nmcli(&["-t", "-f", "DEVICE,TYPE", "device", "status"]);
                    let interfaces: Vec<&str> = output
                        .lines()
                        .filter(|l| l.contains("ethernet") || l.contains("wireless"))
                        .filter_map(|l| l.split(':').next())
                        .collect();

                    for iface in &interfaces {
                        run_nmcli(&["device", "modify", iface, "ipv4.ignore-auto-dns=no"]);
                    }
                    let new_dns = get_dns_servers();
                    if new_dns.is_empty() {
                        tr!("dns_reset_dhcp").to_string()
                    } else {
                        new_dns.join("\n")
                    }
                },
                move |text| {
                    dns_servers_value_c.set_text(&text);
                },
            );
        });
    }

    {
        vpn_add_button.connect_clicked(move |_| {
            let _ = std::process::Command::new("nm-connection-editor").spawn();
        });
    }

    {
        vpn_edit_button.connect_clicked(move |_| {
            let _ = std::process::Command::new("nm-connection-editor").spawn();
        });
    }

    {
        let vpn_list_value = vpn_list_value.clone();
        vpn_remove_button.connect_clicked(move |btn| {
            let vpn_connections = get_vpn_connections();
            if vpn_connections.is_empty() {
                vpn_list_value.set_text("No VPN connections to remove.");
                return;
            }

            let window = btn.root().and_then(|r| r.downcast::<gtk::Window>().ok());
            let dialog = gtk::Dialog::with_buttons(
                Some("Remove VPN Connection"),
                window.as_ref(),
                gtk::DialogFlags::MODAL,
                &[
                    ("Cancel", gtk::ResponseType::Cancel),
                    ("Remove", gtk::ResponseType::Accept),
                ],
            );
            dialog.set_default_response(gtk::ResponseType::Accept);

            let content = dialog.content_area();
            content.set_spacing(10);
            content.set_margin_top(10);
            content.set_margin_bottom(10);
            content.set_margin_start(20);
            content.set_margin_end(20);

            let select_label = Label::new(Some("Select VPN to remove:"));
            select_label.set_halign(gtk::Align::Start);
            content.append(&select_label);

            let combo = gtk::ComboBoxText::new();
            for name in &vpn_connections {
                combo.append_text(name);
            }
            combo.set_active(Some(0));
            content.append(&combo);

            let vpn_list_value_c = vpn_list_value.clone();
            let combo_c = combo.clone();
            dialog.connect_response(move |dlg, response| {
                if response == gtk::ResponseType::Accept {
                    if let Some(conn_name) = combo_c.active_text() {
                        let vpn_list_value_cc = vpn_list_value_c.clone();
                        let conn_name_c = conn_name.to_string();
                        let conn_name_cc = conn_name_c.clone();
                        spawn_bg(
                            move || run_nmcli(&["connection", "delete", &conn_name_c]),
                            move |output| {
                                if output.contains("successfully") || output.trim().is_empty() {
                                    vpn_list_value_cc.set_text(&format!("Removed VPN: {}", conn_name_cc));
                                } else {
                                    vpn_list_value_cc.set_text(&format!("Error removing VPN: {}", output.trim()));
                                }
                            },
                        );
                    }
                }
                dlg.close();
            });

            dialog.show();
        });
    }

    {
        open_settings_button.connect_clicked(move |_| {
            let _ = std::process::Command::new("gnome-control-center").arg("network").spawn();
        });
    }

    // ── Refresh button handler ──

    {
        let wifi_status_label = wifi_status_label.clone();
        let active_conns_value = active_conns_value.clone();
        let interfaces_value = interfaces_value.clone();
        let ip_value = ip_value.clone();
        let dns_servers_value = dns_servers_value.clone();
        refresh_button.connect_clicked(move |_| {
            let wifi_status_label_c = wifi_status_label.clone();
            let active_conns_value_c = active_conns_value.clone();
            let interfaces_value_c = interfaces_value.clone();
            let ip_value_c = ip_value.clone();
            let dns_servers_value_c = dns_servers_value.clone();
            spawn_bg(
                move || {
                    let wifi = get_wifi_status();
                    let conns = get_active_connections();
                    let ifaces = get_network_interfaces();
                    let ip = get_ip_info();
                    let dns = get_dns_servers();
                    (wifi, conns, ifaces, ip, dns)
                },
                move |(wifi, conns, ifaces, ip, dns)| {
                    wifi_status_label_c.set_text(&wifi);
                    let conns_text = if conns.is_empty() {
                        "No active connections".to_string()
                    } else {
                        conns.join("\n")
                    };
                    active_conns_value_c.set_text(&conns_text);
                    let ifaces_text = if ifaces.is_empty() {
                        "No interfaces found".to_string()
                    } else {
                        ifaces.join("\n")
                    };
                    interfaces_value_c.set_text(&ifaces_text);
                    ip_value_c.set_text(&ip);
                    let dns_text = if dns.is_empty() {
                        "No DNS servers configured".to_string()
                    } else {
                        dns.join("\n")
                    };
                    dns_servers_value_c.set_text(&dns_text);
                },
            );
        });
    }

    // ── Periodic refresh ──

    {
        let wifi_status_label_c = wifi_status_label.clone();
        let active_conns_value_c = active_conns_value.clone();
        let interfaces_value_c = interfaces_value.clone();
        let ip_value_c = ip_value.clone();

        glib::timeout_add_seconds_local(10, move || {
            let wifi_c = wifi_status_label_c.clone();
            let conn_c = active_conns_value_c.clone();
            let iface_c = interfaces_value_c.clone();
            let ip_c = ip_value_c.clone();
            spawn_bg(
                move || {
                    let wifi = get_wifi_status();
                    let conns = get_active_connections();
                    let ifaces = get_network_interfaces();
                    let ip = get_ip_info();
                    (wifi, conns, ifaces, ip)
                },
                move |(wifi, conns, ifaces, ip)| {
                    wifi_c.set_text(&wifi);
                    let conns_text = if conns.is_empty() {
                        "No active connections".to_string()
                    } else {
                        conns.join("\n")
                    };
                    conn_c.set_text(&conns_text);
                    let ifaces_text = if ifaces.is_empty() {
                        "No interfaces found".to_string()
                    } else {
                        ifaces.join("\n")
                    };
                    iface_c.set_text(&ifaces_text);
                    ip_c.set_text(&ip);
                },
            );
            glib::ControlFlow::Continue
        });
    }

    main_box.upcast()
}
