mod modules;
mod utils;
mod i18n;

use adw::prelude::*;
use adw::{Application, ApplicationWindow};
use gtk::{Box, ComboBoxText, HeaderBar, Label, Orientation, ScrolledWindow, Stack, StackSidebar};

use i18n::{set_locale, current_locale_name, tr};

fn build_ui(app: &Application) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("LinuxCare - Linux System Manager")
        .default_width(1200)
        .default_height(800)
        .build();

    let main_box = Box::new(Orientation::Vertical, 0);

    // Header bar with window controls (minimize/maximize/close)
    let header_bar = HeaderBar::new();
    main_box.append(&header_bar);

    // Language selector bar
    let lang_bar = Box::new(Orientation::Horizontal, 8);
    lang_bar.set_margin_top(4);
    lang_bar.set_margin_bottom(4);
    lang_bar.set_margin_start(12);
    lang_bar.set_margin_end(12);
    lang_bar.set_css_classes(&["toolbar"]);

    let spacer = Box::new(Orientation::Horizontal, 0);
    spacer.set_hexpand(true);

    let lang_label = Label::new(Some("语言/Language:"));
    lang_label.set_margin_end(4);

    let lang_combo = ComboBoxText::new();
    lang_combo.append_text("中文");
    lang_combo.append_text("English");
    lang_combo.append_text("日本語");
    lang_combo.append_text("한국어");

    match current_locale_name().as_str() {
        "zh_CN" => lang_combo.set_active(Some(0)),
        "ja_JP" => lang_combo.set_active(Some(2)),
        "ko_KR" => lang_combo.set_active(Some(3)),
        _ => lang_combo.set_active(Some(1)),
    }

    lang_bar.append(&spacer);
    lang_bar.append(&lang_label);
    lang_bar.append(&lang_combo);

    main_box.append(&lang_bar);

    // Main content: sidebar + stack
    let content_box = Box::new(Orientation::Horizontal, 0);
    content_box.set_vexpand(true);
    content_box.set_hexpand(true);

    let stack = Stack::new();
    stack.set_transition_type(gtk::StackTransitionType::Crossfade);

    let pages: Vec<(&str, &str, &str)> = vec![
        ("monitor", tr!("monitor"), "utilities-system-monitor-symbolic"),
        ("process", tr!("process"), "system-system-monitor-symbolic"),
        ("desktop", tr!("desktop"), "computer-symbolic"),
        ("system", tr!("system"), "system-settings-symbolic"),
        ("network", tr!("network"), "network-wireless-symbolic"),
        ("software", tr!("software"), "application-x-executable-symbolic"),
        ("disk", tr!("disk"), "drive-harddisk-symbolic"),
        ("cleanup", tr!("cleanup"), "edit-clear-symbolic"),
        ("startup", tr!("startup"), "system-run-symbolic"),
        ("service", tr!("service"), "emblem-system-symbolic"),
        ("user", tr!("user"), "avatar-default-symbolic"),
        ("driver", tr!("driver"), "input-gaming-symbolic"),
        ("firewall", tr!("firewall"), "security-high-symbolic"),
        ("backup", tr!("backup"), "document-save-symbolic"),
        ("logview", tr!("log"), "text-x-generic-symbolic"),
        ("menu", tr!("menu"), "view-list-symbolic"),
        ("shortcut", tr!("shortcuts"), "preferences-desktop-keyboard-symbolic"),
        ("optimizer", tr!("optimizer"), "system-tweaks-symbolic"),
    ];

    for (name, title, icon) in pages {
        let widget = match name {
            "monitor" => modules::monitor::build_monitor_page(),
            "process" => modules::process::build_process_page(),
            "desktop" => modules::desktop::build_desktop_page(),
            "system" => modules::system::build_system_page(),
            "network" => modules::network::build_network_page(),
            "software" => modules::software::build_software_page(),
            "disk" => modules::disk::build_disk_page(),
            "cleanup" => modules::cleanup::build_cleanup_page(),
            "startup" => modules::startup::build_startup_page(),
            "service" => modules::service::build_service_page(),
            "user" => modules::user::build_user_page(),
            "driver" => modules::driver::build_driver_page(),
            "firewall" => modules::firewall::build_firewall_page(),
            "backup" => modules::backup::build_backup_page(),
            "logview" => modules::logview::build_logview_page(),
            "menu" => modules::menu::build_menu_page(),
            "shortcut" => modules::shortcut::build_shortcut_page(),
            "optimizer" => modules::optimizer::build_optimizer_page(),
            _ => unreachable!(),
        };
        let scrolled = ScrolledWindow::new();
        scrolled.set_child(Some(&widget));
        scrolled.set_vexpand(true);
        scrolled.set_hexpand(true);
        let page = stack.add_titled(&scrolled, Some(name), title);
        page.set_icon_name(icon);
    }

    let sidebar = StackSidebar::new();
    sidebar.set_stack(&stack);
    sidebar.set_width_request(200);

    content_box.append(&sidebar);
    content_box.append(&stack);

    main_box.append(&content_box);

    window.set_content(Some(&main_box));

    // Language switcher handler
    let app_ref = app.clone();
    lang_combo.connect_changed(move |combo| {
        let idx = combo.active().unwrap_or(1);
        let code = match idx {
            0 => "zh_CN",
            1 => "en_US",
            2 => "ja_JP",
            3 => "ko_KR",
            _ => "en_US",
        };
        set_locale(code);

        for w in app_ref.windows() {
            w.close();
        }
        build_ui(&app_ref);
    });

    window.present();
}

fn main() {
    env_logger::init();

    let app = Application::builder()
        .application_id("com.linuxcare.app")
        .build();

    app.connect_activate(build_ui);
    app.run();
}
