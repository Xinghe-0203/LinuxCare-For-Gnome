use std::env;
use std::sync::OnceLock;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Locale {
    ZhCN,
    EnUS,
    JaJP,
    KoKR,
}

impl Locale {
    #[allow(dead_code)]
    pub fn code(&self) -> &'static str {
        match self {
            Locale::ZhCN => "zh_CN",
            Locale::EnUS => "en_US",
            Locale::JaJP => "ja_JP",
            Locale::KoKR => "ko_KR",
        }
    }
}

#[allow(dead_code)]
pub static CURRENT_LOCALE: OnceLock<Locale> = OnceLock::new();

#[allow(dead_code)]
pub fn detect_locale() -> Locale {
    let locale_str = env::var("LC_MESSAGES")
        .or_else(|_| env::var("LC_ALL"))
        .or_else(|_| env::var("LANG"))
        .unwrap_or_else(|_| "en_US.UTF-8".to_string());

    let lower = locale_str.to_lowercase();
    if lower.starts_with("zh") {
        Locale::ZhCN
    } else if lower.starts_with("ja") {
        Locale::JaJP
    } else if lower.starts_with("ko") {
        Locale::KoKR
    } else {
        Locale::EnUS
    }
}

#[allow(dead_code)]
pub fn current_locale() -> Locale {
    *CURRENT_LOCALE.get_or_init(detect_locale)
}

#[macro_export]
macro_rules! tr {
    ($key:expr) => {
        $crate::i18n::tr($key)
    };
}

pub fn tr(key: &str) -> &'static str {
    let locale_name = current_locale_name();
    match locale_name.as_str() {
        "zh_CN" => zh_cn(key),
        "ja_JP" => ja_jp(key),
        "ko_KR" => ko_kr(key),
        _ => en_us(key),
    }
}

fn en_us(key: &str) -> &'static str {
    match key {
        // Navigation
        "desktop" => "Desktop",
        "system" => "System",
        "software" => "Software",
        "disk" => "Disk",
        "cleanup" => "Cleanup",
        "menu" => "Menu",
        "shortcuts" => "Shortcuts",
        "settings" => "Settings",

        // Actions
        "search" => "Search",
        "install" => "Install",
        "remove" => "Remove",
        "update" => "Update",
        "clean" => "Clean",
        "scan" => "Scan",
        "apply" => "Apply",
        "cancel" => "Cancel",
        "clear" => "Clear",
        "confirm" => "Confirm",
        "save" => "Save",
        "reset" => "Reset",
        "refresh" => "Refresh",
        "close" => "Close",
        "back" => "Back",
        "next" => "Next",
        "finish" => "Finish",

        // Modules
        "monitor" => "Monitor",
        "process" => "Process",
        "network" => "Network",
        "user" => "User",
        "startup" => "Startup",
        "service" => "Service",
        "log" => "Log",
        "backup" => "Backup",
        "driver" => "Driver",
        "firewall" => "Firewall",

        // App
        "help" => "Help",
        "about" => "About",
        "quit" => "Quit",
        "linuxcare" => "LinuxCare",
        "welcome" => "Welcome",
        "version" => "Version",

        // System
        "cpu" => "CPU",
        "memory" => "Memory",
        "disk_usage" => "Disk Usage",
        "kernel" => "Kernel",
        "uptime" => "Uptime",
        "temperature" => "Temperature",
        "load" => "Load",
        "hostname" => "Hostname",
        "architecture" => "Architecture",
        "shell" => "Shell",

        // Software
        "installed" => "Installed",
        "available" => "Available",
        "package" => "Package",
        "packages" => "Packages",
        "repository" => "Repository",
        "upgrade" => "Upgrade",
        "downgrade" => "Downgrade",
        "autoremove" => "Autoremove",

        // Disk
        "size" => "Size",
        "used" => "Used",
        "free" => "Free",
        "mount_point" => "Mount Point",
        "device" => "Device",
        "filesystem" => "Filesystem",
        "partition" => "Partition",

        // Cleanup
        "cache" => "Cache",
        "logs" => "Logs",
        "temp_files" => "Temp Files",
        "trash" => "Trash",
        "history" => "History",
        "clean_all" => "Clean All",

        // Desktop
        "wallpaper" => "Wallpaper",
        "icons" => "Icons",
        "theme" => "Theme",
        "appearance" => "Appearance",
        "font" => "Font",
        "resolution" => "Resolution",
        "taskbar" => "Taskbar",
        "widgets" => "Widgets",

        // Status
        "running" => "Running",
        "stopped" => "Stopped",
        "enabled" => "Enabled",
        "disabled" => "Disabled",
        "active" => "Active",
        "inactive" => "Inactive",
        "success" => "Success",
        "error" => "Error",
        "warning" => "Warning",
        "info" => "Info",
        "status" => "Status",

        // Misc
        "loading" => "Loading...",
        "no_data" => "No data",
        "enter_search_pattern" => "Please enter a search pattern",
        "please_wait" => "Please wait...",
        "operation_complete" => "Operation complete",
        "operation_failed" => "Operation failed",
        "are_you_sure" => "Are you sure?",
        "yes" => "Yes",
        "no" => "No",
        "ok" => "OK",
        "open" => "Open",
        "close_app" => "Close",
        "minimize" => "Minimize",
        "maximize" => "Maximize",

        // Desktop Settings
        "show_desktop_icons" => "Show desktop icons",
        "icon_size" => "Icon size",
        "wallpaper_mode" => "Wallpaper mode",
        "normal" => "Normal",
        "zoom" => "Zoom",
        "scale" => "Scale",
        "stretch" => "Stretch",
        "center" => "Center",
        "change_wallpaper" => "Change Wallpaper",
        "dynamic_workspaces" => "Dynamic workspaces",
        "num_workspaces" => "Number of workspaces",
        "power_management" => "Power Management",
        "auto_suspend" => "Auto suspend",
        "suspend_after" => "Suspend after (minutes)",
        "show_battery" => "Show battery percentage",
        "night_light" => "Night light",
        "fractional_scaling" => "Fractional scaling",
        "system_sounds" => "System sounds",
        "alert_sound" => "Alert sound",
        "open_gnome_settings" => "Open GNOME Settings",
        "system_info" => "System Info",
        "desktop_environment" => "Desktop Environment",

        // Package Manager
        "package_manager" => "Package Manager",
        "search_and_install" => "Search & Install",
        "installed_packages" => "Installed Packages",
        "auto_check_updates" => "Auto-check for updates",
        "check_updates" => "Check for Updates",
        "upgrade_all" => "Upgrade All",
        "search_packages" => "Search for packages...",

        // Disk Cleanup
        "disk_usage_overview" => "Disk Usage Overview",
        "open_gparted" => "Open GParted",
        "open_gnome_disks" => "Open GNOME Disks",
        "check_health" => "Check Disk Health",
        "cleanup_categories" => "Cleanup Categories",
        "apt_cache" => "APT cache",
        "snap_cache" => "Snap cache",
        "flatpak_cache" => "Flatpak cache",
        "log_files" => "Log files",
        "temp_files_short" => "Temporary files",
        "browser_cache" => "Browser cache",
        "total_space" => "Total Space to be Freed",
        "clean_selected" => "Clean Selected",
        "scan_again" => "Scan Again",

        // Duplicate Files
        "duplicate_files" => "Duplicate Files",
        "find_remove_duplicates" => "Find and remove duplicate files",
        "scan_duplicates" => "Scan for Duplicates",
        "remove_duplicates" => "Remove Duplicates",

        // Context Menu
        "context_menu" => "Context Menu (Right-click)",
        "nautilus_context" => "Nautilus context menu",
        "desktop_context" => "Desktop context menu",
        "custom_menu_items" => "Custom Menu Items",
        "add_custom_item" => "Add Custom Item",
        "edit_selected" => "Edit Selected",
        "remove_selected" => "Remove Selected",
        "add_to_desktop" => "Add to Desktop",
        "remove_from_desktop" => "Remove from Desktop",
        "app_menu" => "Application Menu",
        "add_to_menu" => "Add to Menu",
        "edit_menu" => "Edit Selected",
        "remove_from_menu" => "Remove from Menu",
        "reset_default" => "Reset to Default",
        "desktop_shortcuts" => "Desktop Shortcuts",
        "operations_output" => "Operations Output",

        // Process Manager
        "process_manager" => "Process Manager",
        "resource_summary" => "Resource Summary",
        "search_hint" => "Filter by name, PID, user, or command...",
        "process_list" => "Process List",
        "kill_pid" => "Kill PID",
        "kill_process" => "Kill Process",
        "set_priority" => "Set Priority",
        "set_nice_value" => "Set Nice Value",
        "pid_for_priority" => "PID for priority",
        "process_details" => "Process Details",

        // Network
        "network_management" => "Network Management",
        "wifi_management" => "WiFi Management",
        "scan_networks" => "Scan Networks",
        "connect" => "Connect",
        "disconnect" => "Disconnect",
        "available_wifi" => "Available WiFi Networks",
        "scanning" => "Scanning for WiFi networks...",
        "no_wifi" => "No WiFi networks found or WiFi not available.",
        "vpn_management" => "VPN Management",
        "vpn_connections" => "VPN Connections:",
        "no_vpn" => "No VPN connections configured",
        "add_vpn" => "Add VPN",
        "edit_vpn" => "Edit VPN",
        "remove_vpn" => "Remove VPN",
        "proxy_settings" => "Proxy Settings",
        "enable_proxy" => "Enable Proxy",
        "http_proxy" => "HTTP Proxy",
        "https_proxy" => "HTTPS Proxy",
        "socks_proxy" => "SOCKS Proxy",
        "no_proxy" => "No Proxy",
        "apply_proxy" => "Apply Proxy Settings",
        "dns_settings" => "DNS Settings",
        "current_dns" => "Current DNS Servers:",
        "no_dns" => "No DNS servers configured",
        "add_dns_server" => "Add DNS Server",
        "add_dns" => "Add DNS Server",
        "dns_reset" => "Reset to Default",
        "dns_reset_dhcp" => "DNS reset to DHCP defaults",
        "network_diagnostics" => "Network Diagnostics",
        "target_host" => "Target Host",
        "ping" => "Ping",
        "traceroute" => "Traceroute",
        "speed_test" => "Speed Test",
        "running_speed" => "Running speed test (downloading 10MB file)...",
        "diag_output" => "Diagnostics Output",
        "run_diag" => "Run a diagnostic tool to see results here...",
        "network_info" => "Network Information",
        "active_connections" => "Active Connections:",
        "no_active_conn" => "No active connections",
        "interfaces" => "Network Interfaces:",
        "no_interfaces" => "No interfaces found",
        "ip_config" => "IP Configuration:",
        "open_network_settings" => "Open Network Settings",

        // User Management
        "user_management" => "User Management",
        "current_user_info" => "Current User Info",
        "system_users" => "System Users",
        "add_user" => "Add User",
        "new_user" => "New User",
        "full_name" => "Full Name",
        "create_home_dir" => "Create Home Dir",
        "system_account" => "System Account",
        "create_user" => "Create User",
        "modify_user" => "Modify User",
        "new_shell" => "New Shell",
        "new_home_dir" => "New Home Dir",
        "new_full_name" => "New Full Name",
        "groups" => "Groups",
        "move_home" => "Move Home",
        "lock_account" => "Lock Account",
        "apply_changes" => "Apply Changes",
        "unlock_account" => "Unlock Account",
        "delete_user" => "Delete User",
        "group_management" => "Group Management",
        "new_group" => "New Group",
        "add_group" => "Add Group",
        "add_user_to_group" => "Add User to Group",
        "add_to_group" => "Add to Group",
        "remove_from_group" => "Remove from Group",
        "user_sessions" => "User Sessions",
        "change_password" => "Change Password",
        "change_shell" => "Change Shell",
        "refresh_all" => "Refresh All",
        "username" => "Username",
        "enter_username" => "Enter username",

        // Startup Apps
        "startup_apps" => "Startup Applications",
        "information" => "Information",
        "manage_autostart" => "Manage applications that start automatically at login.\nStartup entries are stored as .desktop files in ~/.config/autostart/",
        "actions" => "Actions",
        "app_name" => "Application Name:",
        "command" => "Command:",
        "add_entry" => "Add Entry",
        "file_name" => "File Name:",
        "enable" => "Enable",
        "disable" => "Disable",
        "output" => "Output",

        // Service Manager
        "service_manager" => "Service Manager",
        "filter_services" => "Filter Services",
        "filter_hint" => "Filter by service name...",
        "type" => "Type:",
        "service_list" => "Service List",
        "service_control" => "Service Control",
        "start" => "Start",
        "stop" => "Stop",
        "restart" => "Restart",
        "service_details" => "Service Details",
        "select_service" => "Select a service and click 'Status' or 'Logs' to view details.",

        // Log Viewer
        "log_viewer" => "Log Viewer",
        "log_source" => "Log Source",
        "system_log" => "System Log",
        "kernel_log" => "Kernel Log",
        "auth_log" => "Auth Log",
        "syslog" => "Syslog",
        "journalctl" => "Journalctl",
        "service_log" => "Service Log",
        "log_level" => "Log Level",
        "all" => "All",
        "emergency" => "Emergency",
        "alert" => "Alert",
        "critical" => "Critical",
        "err" => "Error",
        "notice" => "Notice",
        "debug" => "Debug",
        "time_range" => "Time Range",
        "last_hour" => "Last Hour",
        "last_day" => "Last Day",
        "last_week" => "Last Week",
        "last_month" => "Last Month",
        "lines" => "Lines:",
        "load_log" => "Load Log",
        "boot_logs" => "Boot Logs",
        "dmesg" => "dmesg",
        "loading_auth_log" => "Loading auth log...",
        "loading_boot_logs" => "Loading boot logs...",
        "loading_dmesg" => "Loading dmesg...",
        "loading_kernel_log" => "Loading kernel log...",
        "loading_syslog" => "Loading syslog...",

        // Backup
        "backup_management" => "Backup Management",
        "backup_tool" => "Backup Tool",
        "rsync_backup" => "rsync Backup",
        "timeshift_backup" => "Timeshift Backup",
        "borg_backup" => "Borg Backup",
        "backup_source" => "Backup Source:",
        "backup_dest" => "Backup Destination:",
        "start_backup" => "Start Backup",
        "start_restore" => "Start Restore",
        "backup_history" => "Backup History",
        "no_history" => "No backup history found.\n\nTip: Create your first backup using one of the backup methods below.",

        // Driver
        "driver_management" => "Driver Management",
        "hardware_detection" => "Hardware Detection",
        "pci_devices" => "PCI Devices",
        "usb_devices" => "USB Devices",
        "loaded_modules" => "Loaded Kernel Modules",
        "nvidia_driver" => "NVIDIA Driver",
        "amd_driver" => "AMD Driver",
        "intel_driver" => "Intel Driver",
        "open_drivers" => "Open Drivers",
        "proprietary_drivers" => "Proprietary Drivers",
        "install_driver" => "Install Driver",
        "remove_driver" => "Remove Driver",

        // Firewall
        "firewall_management" => "Firewall Management",
        "ufw_status" => "UFW Status",
        "firewall_rules" => "Firewall Rules",
        "add_rule" => "Add Rule",
        "delete_rule" => "Delete Rule",
        "app_profiles" => "Application Profiles",
        "ufw_enable" => "Enable UFW",
        "ufw_disable" => "Disable UFW",
        "ufw_reset" => "Reset UFW",

        
        // Optimizer
        "optimizer" => "Optimizer",
        "system_optimization" => "System Optimization",
        "scan_optimizations" => "Scan Optimizations",
        "apply_selected" => "Apply Selected",
        "kernel_tuning" => "Kernel Tuning",
        "service_management" => "Service Management",
        "swap_optimization" => "Swap Optimization",
        "filesystem_optimization" => "Filesystem Optimization",
        "network_optimization" => "Network Optimization",
        "power_optimization" => "Power Optimization",
        "memory_optimization" => "Memory Optimization",
        "current_value" => "Current Value",
        "recommended_value" => "Recommended Value",
        "category" => "Category",
        "optimization_items" => "Optimization Items",
        "optimization_results" => "Optimization Results",
        "apply_success" => "Applied successfully",
        "apply_failed" => "Failed to apply",
        "select_items" => "Select items to optimize",
        _ => "unknown",
    }
}

fn zh_cn(key: &str) -> &'static str {
    match key {
        // Navigation
        "desktop" => "桌面",
        "system" => "系统",
        "software" => "软件",
        "disk" => "磁盘",
        "cleanup" => "清理",
        "menu" => "菜单",
        "shortcuts" => "快捷键",
        "settings" => "设置",

        // Actions
        "search" => "搜索",
        "install" => "安装",
        "remove" => "移除",
        "update" => "更新",
        "clean" => "清理",
        "scan" => "扫描",
        "apply" => "应用",
        "cancel" => "取消",
        "clear" => "清除",
        "confirm" => "确认",
        "save" => "保存",
        "reset" => "重置",
        "refresh" => "刷新",
        "close" => "关闭",
        "back" => "返回",
        "next" => "下一步",
        "finish" => "完成",

        // Modules
        "monitor" => "监控",
        "process" => "进程",
        "network" => "网络",
        "user" => "用户",
        "startup" => "启动项",
        "service" => "服务",
        "log" => "日志",
        "backup" => "备份",
        "driver" => "驱动",
        "firewall" => "防火墙",

        // App
        "help" => "帮助",
        "about" => "关于",
        "quit" => "退出",
        "linuxcare" => "LinuxCare",
        "welcome" => "欢迎",
        "version" => "版本",

        // System
        "cpu" => "处理器",
        "memory" => "内存",
        "disk_usage" => "磁盘使用",
        "kernel" => "内核",
        "uptime" => "运行时间",
        "temperature" => "温度",
        "load" => "负载",
        "hostname" => "主机名",
        "architecture" => "架构",
        "shell" => "Shell",

        // Software
        "installed" => "已安装",
        "available" => "可用",
        "package" => "软件包",
        "packages" => "软件包",
        "repository" => "软件源",
        "upgrade" => "升级",
        "downgrade" => "降级",
        "autoremove" => "自动移除",

        // Disk
        "size" => "大小",
        "used" => "已用",
        "free" => "空闲",
        "mount_point" => "挂载点",
        "device" => "设备",
        "filesystem" => "文件系统",
        "partition" => "分区",

        // Cleanup
        "cache" => "缓存",
        "logs" => "日志",
        "temp_files" => "临时文件",
        "trash" => "回收站",
        "history" => "历史记录",
        "clean_all" => "全部清理",

        // Desktop
        "wallpaper" => "壁纸",
        "icons" => "图标",
        "theme" => "主题",
        "appearance" => "外观",
        "font" => "字体",
        "resolution" => "分辨率",
        "taskbar" => "任务栏",
        "widgets" => "小部件",

        // Status
        "running" => "运行中",
        "stopped" => "已停止",
        "enabled" => "已启用",
        "disabled" => "已禁用",
        "active" => "活跃",
        "inactive" => "未激活",
        "success" => "成功",
        "error" => "错误",
        "warning" => "警告",
        "info" => "信息",
        "status" => "状态",

        // Misc
        "loading" => "加载中...",
        "no_data" => "无数据",
        "enter_search_pattern" => "请输入搜索模式",
        "please_wait" => "请稍候...",
        "operation_complete" => "操作完成",
        "operation_failed" => "操作失败",
        "are_you_sure" => "确定吗？",
        "yes" => "是",
        "no" => "否",
        "ok" => "确定",
        "open" => "打开",
        "close_app" => "关闭",
        "minimize" => "最小化",
        "maximize" => "最大化",

        // Desktop Settings
        "show_desktop_icons" => "显示桌面图标",
        "icon_size" => "图标大小",
        "wallpaper_mode" => "壁纸模式",
        "normal" => "普通",
        "zoom" => "缩放",
        "scale" => "拉伸",
        "stretch" => "平铺",
        "center" => "居中",
        "change_wallpaper" => "更换壁纸",
        "dynamic_workspaces" => "动态工作区",
        "num_workspaces" => "工作区数量",
        "power_management" => "电源管理",
        "auto_suspend" => "自动挂起",
        "suspend_after" => "挂起时间（分钟）",
        "show_battery" => "显示电池百分比",
        "night_light" => "夜光模式",
        "fractional_scaling" => "分数缩放",
        "system_sounds" => "系统声音",
        "alert_sound" => "提示音",
        "open_gnome_settings" => "打开 GNOME 设置",
        "system_info" => "系统信息",
        "desktop_environment" => "桌面环境",

        // Package Manager
        "package_manager" => "软件包管理器",
        "search_and_install" => "搜索与安装",
        "installed_packages" => "已安装软件包",
        "auto_check_updates" => "自动检查更新",
        "check_updates" => "检查更新",
        "upgrade_all" => "全部升级",
        "search_packages" => "搜索软件包...",

        // Disk Cleanup
        "disk_usage_overview" => "磁盘使用概览",
        "open_gparted" => "打开 GParted",
        "open_gnome_disks" => "打开 GNOME 磁盘",
        "check_health" => "检查磁盘健康",
        "cleanup_categories" => "清理类别",
        "apt_cache" => "APT 缓存",
        "snap_cache" => "Snap 缓存",
        "flatpak_cache" => "Flatpak 缓存",
        "log_files" => "日志文件",
        "temp_files_short" => "临时文件",
        "browser_cache" => "浏览器缓存",
        "total_space" => "可释放空间",
        "clean_selected" => "清理选中",
        "scan_again" => "重新扫描",

        // Duplicate Files
        "duplicate_files" => "重复文件",
        "find_remove_duplicates" => "查找并删除重复文件",
        "scan_duplicates" => "扫描重复文件",
        "remove_duplicates" => "删除重复文件",

        // Context Menu
        "context_menu" => "右键菜单",
        "nautilus_context" => "Nautilus 右键菜单",
        "desktop_context" => "桌面右键菜单",
        "custom_menu_items" => "自定义菜单项",
        "add_custom_item" => "添加自定义项",
        "edit_selected" => "编辑选中项",
        "remove_selected" => "删除选中项",
        "add_to_desktop" => "添加到桌面",
        "remove_from_desktop" => "从桌面移除",
        "app_menu" => "应用程序菜单",
        "add_to_menu" => "添加到菜单",
        "edit_menu" => "编辑选中项",
        "remove_from_menu" => "从菜单移除",
        "reset_default" => "恢复默认",
        "desktop_shortcuts" => "桌面快捷键",
        "operations_output" => "操作输出",

        // Process Manager
        "process_manager" => "进程管理器",
        "resource_summary" => "资源概览",
        "search_hint" => "按名称、PID、用户或命令筛选...",
        "process_list" => "进程列表",
        "kill_pid" => "终止 PID",
        "kill_process" => "终止进程",
        "set_priority" => "设置优先级",
        "set_nice_value" => "设置 Nice 值",
        "pid_for_priority" => "PID",
        "process_details" => "进程详情",

        // Network
        "network_management" => "网络管理",
        "wifi_management" => "WiFi 管理",
        "scan_networks" => "扫描网络",
        "connect" => "连接",
        "disconnect" => "断开",
        "available_wifi" => "可用 WiFi 网络",
        "scanning" => "正在扫描 WiFi 网络...",
        "no_wifi" => "未找到 WiFi 网络或 WiFi 不可用。",
        "vpn_management" => "VPN 管理",
        "vpn_connections" => "VPN 连接:",
        "no_vpn" => "未配置 VPN 连接",
        "add_vpn" => "添加 VPN",
        "edit_vpn" => "编辑 VPN",
        "remove_vpn" => "删除 VPN",
        "proxy_settings" => "代理设置",
        "enable_proxy" => "启用代理",
        "http_proxy" => "HTTP 代理",
        "https_proxy" => "HTTPS 代理",
        "socks_proxy" => "SOCKS 代理",
        "no_proxy" => "不使用代理",
        "apply_proxy" => "应用代理设置",
        "dns_settings" => "DNS 设置",
        "current_dns" => "当前 DNS 服务器:",
        "no_dns" => "未配置 DNS 服务器",
        "add_dns_server" => "添加 DNS 服务器",
        "add_dns" => "添加 DNS 服务器",
        "dns_reset" => "恢复默认",
        "dns_reset_dhcp" => "DNS 已重置为 DHCP 默认值",
        "network_diagnostics" => "网络诊断",
        "target_host" => "目标主机",
        "ping" => "Ping",
        "traceroute" => "路由追踪",
        "speed_test" => "速度测试",
        "running_speed" => "正在运行速度测试（下载 10MB 文件）...",
        "diag_output" => "诊断输出",
        "run_diag" => "运行诊断工具查看结果...",
        "network_info" => "网络信息",
        "active_connections" => "活动连接:",
        "no_active_conn" => "无活动连接",
        "interfaces" => "网络接口:",
        "no_interfaces" => "未找到接口",
        "ip_config" => "IP 配置:",
        "open_network_settings" => "打开网络设置",

        // User Management
        "user_management" => "用户管理",
        "current_user_info" => "当前用户信息",
        "system_users" => "系统用户",
        "add_user" => "添加用户",
        "new_user" => "新用户",
        "full_name" => "全名",
        "create_home_dir" => "创建主目录",
        "system_account" => "系统账户",
        "create_user" => "创建用户",
        "modify_user" => "修改用户",
        "new_shell" => "新 Shell",
        "new_home_dir" => "新主目录",
        "new_full_name" => "新全名",
        "groups" => "组",
        "move_home" => "移动主目录",
        "lock_account" => "锁定账户",
        "apply_changes" => "应用更改",
        "unlock_account" => "解锁账户",
        "delete_user" => "删除用户",
        "group_management" => "组管理",
        "new_group" => "新组",
        "add_group" => "添加组",
        "add_user_to_group" => "将用户添加到组",
        "add_to_group" => "添加到组",
        "remove_from_group" => "从组中移除",
        "user_sessions" => "用户会话",
        "change_password" => "修改密码",
        "change_shell" => "修改 Shell",
        "refresh_all" => "刷新全部",
        "username" => "用户名",
        "enter_username" => "输入用户名",

        // Startup Apps
        "startup_apps" => "启动应用程序",
        "information" => "信息",
        "manage_autostart" => "管理登录时自动启动的应用程序。\n启动项存储为 ~/.config/autostart/ 中的 .desktop 文件",
        "actions" => "操作",
        "app_name" => "应用名称:",
        "command" => "命令:",
        "add_entry" => "添加条目",
        "file_name" => "文件名:",
        "enable" => "启用",
        "disable" => "禁用",
        "output" => "输出",

        // Service Manager
        "service_manager" => "服务管理器",
        "filter_services" => "筛选服务",
        "filter_hint" => "按服务名称筛选...",
        "type" => "类型:",
        "service_list" => "服务列表",
        "service_control" => "服务控制",
        "start" => "启动",
        "stop" => "停止",
        "restart" => "重启",
        "service_details" => "服务详情",
        "select_service" => "选择一个服务，点击[状态]或[日志]查看详情。",

        // Log Viewer
        "log_viewer" => "日志查看器",
        "log_source" => "日志来源",
        "system_log" => "系统日志",
        "kernel_log" => "内核日志",
        "auth_log" => "认证日志",
        "syslog" => "系统日志",
        "journalctl" => "Journalctl",
        "service_log" => "服务日志",
        "log_level" => "日志级别",
        "all" => "全部",
        "emergency" => "紧急",
        "alert" => "警告",
        "critical" => "严重",
        "err" => "错误",
        "notice" => "通知",
        "debug" => "调试",
        "time_range" => "时间范围",
        "last_hour" => "最近一小时",
        "last_day" => "最近一天",
        "last_week" => "最近一周",
        "last_month" => "最近一月",
        "lines" => "行数:",
        "load_log" => "加载日志",
        "boot_logs" => "启动日志",
        "dmesg" => "内核消息",
        "loading_auth_log" => "加载认证日志...",
        "loading_boot_logs" => "加载启动日志...",
        "loading_dmesg" => "加载内核消息...",
        "loading_kernel_log" => "加载内核日志...",
        "loading_syslog" => "加载系统日志...",

        // Backup
        "backup_management" => "备份管理",
        "backup_tool" => "备份工具",
        "rsync_backup" => "rsync 备份",
        "timeshift_backup" => "Timeshift 备份",
        "borg_backup" => "Borg 备份",
        "backup_source" => "备份源:",
        "backup_dest" => "备份目标:",
        "start_backup" => "开始备份",
        "start_restore" => "开始恢复",
        "backup_history" => "备份历史",
        "no_history" => "未找到备份历史。\n\n提示：使用以下备份方法创建您的第一个备份。",

        // Driver
        "driver_management" => "驱动管理",
        "hardware_detection" => "硬件检测",
        "pci_devices" => "PCI 设备",
        "usb_devices" => "USB 设备",
        "loaded_modules" => "已加载内核模块",
        "nvidia_driver" => "NVIDIA 驱动",
        "amd_driver" => "AMD 驱动",
        "intel_driver" => "Intel 驱动",
        "open_drivers" => "开源驱动",
        "proprietary_drivers" => "专有驱动",
        "install_driver" => "安装驱动",
        "remove_driver" => "移除驱动",

        // Firewall
        "firewall_management" => "防火墙管理",
        "ufw_status" => "UFW 状态",
        "firewall_rules" => "防火墙规则",
        "add_rule" => "添加规则",
        "delete_rule" => "删除规则",
        "app_profiles" => "应用配置文件",
        "ufw_enable" => "启用 UFW",
        "ufw_disable" => "禁用 UFW",
        "ufw_reset" => "重置 UFW",

        
        // Optimizer
        "optimizer" => "优化器",
        "system_optimization" => "系统优化",
        "scan_optimizations" => "扫描优化项",
        "apply_selected" => "应用选中项",
        "kernel_tuning" => "内核调优",
        "service_management" => "服务管理",
        "swap_optimization" => "交换分区优化",
        "filesystem_optimization" => "文件系统优化",
        "network_optimization" => "网络优化",
        "power_optimization" => "电源优化",
        "memory_optimization" => "内存优化",
        "current_value" => "当前值",
        "recommended_value" => "推荐值",
        "category" => "类别",
        "optimization_items" => "优化项目",
        "optimization_results" => "优化结果",
        "apply_success" => "应用成功",
        "apply_failed" => "应用失败",
        "select_items" => "选择要优化的项目",
        _ => "unknown",
    }
}

fn ja_jp(key: &str) -> &'static str {
    match key {
        // Navigation
        "desktop" => "デスクトップ",
        "system" => "システム",
        "software" => "ソフトウェア",
        "disk" => "ディスク",
        "cleanup" => "クリーンアップ",
        "menu" => "メニュー",
        "shortcuts" => "ショートカット",
        "settings" => "設定",

        // Actions
        "search" => "検索",
        "install" => "インストール",
        "remove" => "削除",
        "update" => "更新",
        "clean" => "クリーン",
        "scan" => "スキャン",
        "apply" => "適用",
        "cancel" => "キャンセル",
        "clear" => "クリア",
        "confirm" => "確認",
        "save" => "保存",
        "reset" => "リセット",
        "refresh" => "更新",
        "close" => "閉じる",
        "back" => "戻る",
        "next" => "次へ",
        "finish" => "完了",

        // Modules
        "monitor" => "モニター",
        "process" => "プロセス",
        "network" => "ネットワーク",
        "user" => "ユーザー",
        "startup" => "スタートアップ",
        "service" => "サービス",
        "log" => "ログ",
        "backup" => "バックアップ",
        "driver" => "ドライバー",
        "firewall" => "ファイアウォール",

        // App
        "help" => "ヘルプ",
        "about" => "概要",
        "quit" => "終了",
        "linuxcare" => "LinuxCare",
        "welcome" => "ようこそ",
        "version" => "バージョン",

        // System
        "cpu" => "CPU",
        "memory" => "メモリ",
        "disk_usage" => "ディスク使用量",
        "kernel" => "カーネル",
        "uptime" => "稼働時間",
        "temperature" => "温度",
        "load" => "負荷",
        "hostname" => "ホスト名",
        "architecture" => "アーキテクチャ",
        "shell" => "Shell",

        // Software
        "installed" => "インストール済み",
        "available" => "利用可能",
        "package" => "パッケージ",
        "packages" => "パッケージ",
        "repository" => "リポジトリ",
        "upgrade" => "アップグレード",
        "downgrade" => "ダウングレード",
        "autoremove" => "自動削除",

        // Disk
        "size" => "サイズ",
        "used" => "使用済み",
        "free" => "空き",
        "mount_point" => "マウントポイント",
        "device" => "デバイス",
        "filesystem" => "ファイルシステム",
        "partition" => "パーティション",

        // Cleanup
        "cache" => "キャッシュ",
        "logs" => "ログ",
        "temp_files" => "一時ファイル",
        "trash" => "ゴミ箱",
        "history" => "履歴",
        "clean_all" => "すべてクリーン",

        // Desktop
        "wallpaper" => "壁紙",
        "icons" => "アイコン",
        "theme" => "テーマ",
        "appearance" => "外観",
        "font" => "フォント",
        "resolution" => "解像度",
        "taskbar" => "タスクバー",
        "widgets" => "ウィジェット",

        // Status
        "running" => "実行中",
        "stopped" => "停止",
        "enabled" => "有効",
        "disabled" => "無効",
        "active" => "アクティブ",
        "inactive" => "非アクティブ",
        "success" => "成功",
        "error" => "エラー",
        "warning" => "警告",
        "info" => "情報",
        "status" => "ステータス",

        // Misc
        "loading" => "読み込み中...",
        "no_data" => "データなし",
        "enter_search_pattern" => "検索パターンを入力してください",
        "please_wait" => "お待ちください...",
        "operation_complete" => "操作完了",
        "operation_failed" => "操作失敗",
        "are_you_sure" => "よろしいですか？",
        "yes" => "はい",
        "no" => "いいえ",
        "ok" => "OK",
        "open" => "開く",
        "close_app" => "閉じる",
        "minimize" => "最小化",
        "maximize" => "最大化",

        // Desktop Settings
        "show_desktop_icons" => "デスクトップアイコンを表示",
        "icon_size" => "アイコンサイズ",
        "wallpaper_mode" => "壁紙モード",
        "normal" => "通常",
        "zoom" => "ズーム",
        "scale" => "スケール",
        "stretch" => "ストレッチ",
        "center" => "中央",
        "change_wallpaper" => "壁紙を変更",
        "dynamic_workspaces" => "動的ワークスペース",
        "num_workspaces" => "ワークスペース数",
        "power_management" => "電源管理",
        "auto_suspend" => "自動サスペンド",
        "suspend_after" => "サスペンドまでの時間（分）",
        "show_battery" => "バッテリー残量を表示",
        "night_light" => "ナイトライト",
        "fractional_scaling" => "小数スケーリング",
        "system_sounds" => "システムサウンド",
        "alert_sound" => "アラートサウンド",
        "open_gnome_settings" => "GNOME設定を開く",
        "system_info" => "システム情報",
        "desktop_environment" => "デスクトップ環境",

        // Package Manager
        "package_manager" => "パッケージマネージャー",
        "search_and_install" => "検索とインストール",
        "installed_packages" => "インストール済みパッケージ",
        "auto_check_updates" => "自動更新チェック",
        "check_updates" => "アップデートを確認",
        "upgrade_all" => "すべてアップグレード",
        "search_packages" => "パッケージを検索...",

        // Disk Cleanup
        "disk_usage_overview" => "ディスク使用量概要",
        "open_gparted" => "GPartedを開く",
        "open_gnome_disks" => "GNOMEディスクを開く",
        "check_health" => "ディスクヘルスチェック",
        "cleanup_categories" => "クリーンアップカテゴリ",
        "apt_cache" => "APTキャッシュ",
        "snap_cache" => "Snapキャッシュ",
        "flatpak_cache" => "Flatpakキャッシュ",
        "log_files" => "ログファイル",
        "temp_files_short" => "一時ファイル",
        "browser_cache" => "ブラウザキャッシュ",
        "total_space" => "解放可能な合計スペース",
        "clean_selected" => "選択をクリーン",
        "scan_again" => "再スキャン",

        // Duplicate Files
        "duplicate_files" => "重複ファイル",
        "find_remove_duplicates" => "重複ファイルを検索して削除",
        "scan_duplicates" => "重複をスキャン",
        "remove_duplicates" => "重複を削除",

        // Context Menu
        "context_menu" => "コンテキストメニュー（右クリック）",
        "nautilus_context" => "Nautilusコンテキストメニュー",
        "desktop_context" => "デスクトップコンテキストメニュー",
        "custom_menu_items" => "カスタムメニュー項目",
        "add_custom_item" => "カスタム項目を追加",
        "edit_selected" => "選択を編集",
        "remove_selected" => "選択を削除",
        "add_to_desktop" => "デスクトップに追加",
        "remove_from_desktop" => "デスクトップから削除",
        "app_menu" => "アプリケーションメニュー",
        "add_to_menu" => "メニューに追加",
        "edit_menu" => "選択を編集",
        "remove_from_menu" => "メニューから削除",
        "reset_default" => "デフォルトに戻す",
        "desktop_shortcuts" => "デスクトップショートカット",
        "operations_output" => "操作出力",

        // Process Manager
        "process_manager" => "プロセスマネージャー",
        "resource_summary" => "リソース概要",
        "search_hint" => "名前、PID、ユーザー、コマンドでフィルタ...",
        "process_list" => "プロセスリスト",
        "kill_pid" => "PIDをキル",
        "kill_process" => "プロセスをキル",
        "set_priority" => "優先度を設定",
        "set_nice_value" => "Nice値を設定",
        "pid_for_priority" => "PID",
        "process_details" => "プロセス詳細",

        // Network
        "network_management" => "ネットワーク管理",
        "wifi_management" => "WiFi管理",
        "scan_networks" => "ネットワークをスキャン",
        "connect" => "接続",
        "disconnect" => "切断",
        "available_wifi" => "利用可能なWiFiネットワーク",
        "scanning" => "WiFiネットワークをスキャン中...",
        "no_wifi" => "WiFiネットワークが見つからないか、WiFiが利用できません。",
        "vpn_management" => "VPN管理",
        "vpn_connections" => "VPN接続:",
        "no_vpn" => "VPN接続が設定されていません",
        "add_vpn" => "VPNを追加",
        "edit_vpn" => "VPNを編集",
        "remove_vpn" => "VPNを削除",
        "proxy_settings" => "プロキシ設定",
        "enable_proxy" => "プロキシを有効にする",
        "http_proxy" => "HTTPプロキシ",
        "https_proxy" => "HTTPSプロキシ",
        "socks_proxy" => "SOCKSプロキシ",
        "no_proxy" => "プロキシなし",
        "apply_proxy" => "プロキシ設定を適用",
        "dns_settings" => "DNS設定",
        "current_dns" => "現在のDNSサーバー:",
        "no_dns" => "DNSサーバーが設定されていません",
        "add_dns_server" => "DNSサーバーを追加",
        "add_dns" => "DNSサーバーを追加",
        "dns_reset" => "デフォルトに戻す",
        "dns_reset_dhcp" => "DNSがDHCPデフォルトにリセットされました",
        "network_diagnostics" => "ネットワーク診断",
        "target_host" => "ターゲットホスト",
        "ping" => "Ping",
        "traceroute" => "traceroute",
        "speed_test" => "速度テスト",
        "running_speed" => "速度テストを実行中（10MBファイルをダウンロード中）...",
        "diag_output" => "診断出力",
        "run_diag" => "診断ツールを実行して結果を表示...",
        "network_info" => "ネットワーク情報",
        "active_connections" => "アクティブな接続:",
        "no_active_conn" => "アクティブな接続なし",
        "interfaces" => "ネットワークインターフェース:",
        "no_interfaces" => "インターフェースが見つかりません",
        "ip_config" => "IP設定:",
        "open_network_settings" => "ネットワーク設定を開く",

        // User Management
        "user_management" => "ユーザー管理",
        "current_user_info" => "現在のユーザー情報",
        "system_users" => "システムユーザー",
        "add_user" => "ユーザーを追加",
        "new_user" => "新規ユーザー",
        "full_name" => "フルネーム",
        "create_home_dir" => "ホームディレクトリを作成",
        "system_account" => "システムアカウント",
        "create_user" => "ユーザーを作成",
        "modify_user" => "ユーザーを変更",
        "new_shell" => "新しいShell",
        "new_home_dir" => "新しいホームディレクトリ",
        "new_full_name" => "新しいフルネーム",
        "groups" => "グループ",
        "move_home" => "ホームを移動",
        "lock_account" => "アカウントをロック",
        "apply_changes" => "変更を適用",
        "unlock_account" => "アカウントのロック解除",
        "delete_user" => "ユーザーを削除",
        "group_management" => "グループ管理",
        "new_group" => "新規グループ",
        "add_group" => "グループを追加",
        "add_user_to_group" => "ユーザーをグループに追加",
        "add_to_group" => "グループに追加",
        "remove_from_group" => "グループから削除",
        "user_sessions" => "ユーザーセッション",
        "change_password" => "パスワードを変更",
        "change_shell" => "Shellを変更",
        "refresh_all" => "すべて更新",
        "username" => "ユーザー名",
        "enter_username" => "ユーザー名を入力",

        // Startup Apps
        "startup_apps" => "スタートアップアプリケーション",
        "information" => "情報",
        "manage_autostart" => "ログイン時に自動起動するアプリケーションを管理します。\nスタートアップエントリは ~/.config/autostart/ に .desktop ファイルとして保存されます",
        "actions" => "アクション",
        "app_name" => "アプリケーション名:",
        "command" => "コマンド:",
        "add_entry" => "エントリを追加",
        "file_name" => "ファイル名:",
        "enable" => "有効",
        "disable" => "無効",
        "output" => "出力",

        // Service Manager
        "service_manager" => "サービスマネージャー",
        "filter_services" => "サービスをフィルタ",
        "filter_hint" => "サービス名でフィルタ...",
        "type" => "タイプ:",
        "service_list" => "サービスリスト",
        "service_control" => "サービス制御",
        "start" => "開始",
        "stop" => "停止",
        "restart" => "再起動",
        "service_details" => "サービス詳細",
        "select_service" => "サービスを選択して「ステータス」または「ログ」をクリックして詳細を表示",

        // Log Viewer
        "log_viewer" => "ログビューアー",
        "log_source" => "ログソース",
        "system_log" => "システムログ",
        "kernel_log" => "カーネルログ",
        "auth_log" => "認証ログ",
        "syslog" => "syslog",
        "journalctl" => "Journalctl",
        "service_log" => "サービスログ",
        "log_level" => "ログレベル",
        "all" => "すべて",
        "emergency" => "緊急",
        "alert" => "警告",
        "critical" => "重大",
        "err" => "エラー",
        "notice" => "通知",
        "debug" => "デバッグ",
        "time_range" => "時間範囲",
        "last_hour" => "過去1時間",
        "last_day" => "過去1日",
        "last_week" => "過去1週間",
        "last_month" => "過去1ヶ月",
        "lines" => "行数:",
        "load_log" => "ログを読み込み",
        "boot_logs" => "ブートログ",
        "dmesg" => "dmesg",
        "loading_auth_log" => "認証ログを読み込み中...",
        "loading_boot_logs" => "ブートログを読み込み中...",
        "loading_dmesg" => "dmesgを読み込み中...",
        "loading_kernel_log" => "カーネルログを読み込み中...",
        "loading_syslog" => "syslogを読み込み中...",

        // Backup
        "backup_management" => "バックアップ管理",
        "backup_tool" => "バックアップツール",
        "rsync_backup" => "rsync バックアップ",
        "timeshift_backup" => "Timeshift バックアップ",
        "borg_backup" => "Borg バックアップ",
        "backup_source" => "バックアップソース:",
        "backup_dest" => "バックアップ先:",
        "start_backup" => "バックアップ開始",
        "start_restore" => "復元開始",
        "backup_history" => "バックアップ履歴",
        "no_history" => "バックアップ履歴が見つかりません。\n\nヒント：以下のバックアップ方法で最初のバックアップを作成してください。",

        // Driver
        "driver_management" => "ドライバー管理",
        "hardware_detection" => "ハードウェア検出",
        "pci_devices" => "PCIデバイス",
        "usb_devices" => "USBデバイス",
        "loaded_modules" => "ロード済みカーネルモジュール",
        "nvidia_driver" => "NVIDIAドライバー",
        "amd_driver" => "AMDドライバー",
        "intel_driver" => "Intelドライバー",
        "open_drivers" => "オープンドライバー",
        "proprietary_drivers" => "プロプライエタリードライバー",
        "install_driver" => "ドライバーをインストール",
        "remove_driver" => "ドライバーを削除",

        // Firewall
        "firewall_management" => "ファイアウォール管理",
        "ufw_status" => "UFWステータス",
        "firewall_rules" => "ファイアウォールルール",
        "add_rule" => "ルールを追加",
        "delete_rule" => "ルールを削除",
        "app_profiles" => "アプリプロファイル",
        "ufw_enable" => "UFWを有効にする",
        "ufw_disable" => "UFWを無効にする",
        "ufw_reset" => "UFWをリセット",

        
        // Optimizer
        "optimizer" => "オプティマイザー",
        "system_optimization" => "システム最適化",
        "scan_optimizations" => "最適化をスキャン",
        "apply_selected" => "選択を適用",
        "kernel_tuning" => "カーネルチューニング",
        "service_management" => "サービス管理",
        "swap_optimization" => "スワップ最適化",
        "filesystem_optimization" => "ファイルシステム最適化",
        "network_optimization" => "ネットワーク最適化",
        "power_optimization" => "電源最適化",
        "memory_optimization" => "メモリ最適化",
        "current_value" => "現在の値",
        "recommended_value" => "推奨値",
        "category" => "カテゴリ",
        "optimization_items" => "最適化項目",
        "optimization_results" => "最適化結果",
        "apply_success" => "適用成功",
        "apply_failed" => "適用失敗",
        "select_items" => "最適化する項目を選択",
        _ => "unknown",
    }
}

fn ko_kr(key: &str) -> &'static str {
    match key {
        // Navigation
        "desktop" => "데스크톱",
        "system" => "시스템",
        "software" => "소프트웨어",
        "disk" => "디스크",
        "cleanup" => "정리",
        "menu" => "메뉴",
        "shortcuts" => "바로가기",
        "settings" => "설정",

        // Actions
        "search" => "검색",
        "install" => "설치",
        "remove" => "제거",
        "update" => "업데이트",
        "clean" => "정리",
        "scan" => "스캔",
        "apply" => "적용",
        "cancel" => "취소",
        "clear" => "지우기",
        "confirm" => "확인",
        "save" => "저장",
        "reset" => "초기화",
        "refresh" => "새로고침",
        "close" => "닫기",
        "back" => "뒤로",
        "next" => "다음",
        "finish" => "완료",

        // Modules
        "monitor" => "모니터",
        "process" => "프로세스",
        "network" => "네트워크",
        "user" => "사용자",
        "startup" => "시작 프로그램",
        "service" => "서비스",
        "log" => "로그",
        "backup" => "백업",
        "driver" => "드라이버",
        "firewall" => "방화벽",

        // App
        "help" => "도움말",
        "about" => "정보",
        "quit" => "종료",
        "linuxcare" => "LinuxCare",
        "welcome" => "환영합니다",
        "version" => "버전",

        // System
        "cpu" => "CPU",
        "memory" => "메모리",
        "disk_usage" => "디스크 사용량",
        "kernel" => "커널",
        "uptime" => "가동 시간",
        "temperature" => "온도",
        "load" => "부하",
        "hostname" => "호스트 이름",
        "architecture" => "아키텍처",
        "shell" => "Shell",

        // Software
        "installed" => "설치됨",
        "available" => "사용 가능",
        "package" => "패키지",
        "packages" => "패키지",
        "repository" => "저장소",
        "upgrade" => "업그레이드",
        "downgrade" => "다운그레이드",
        "autoremove" => "자동 제거",

        // Disk
        "size" => "크기",
        "used" => "사용됨",
        "free" => "여유",
        "mount_point" => "마운트 지점",
        "device" => "장치",
        "filesystem" => "파일 시스템",
        "partition" => "파티션",

        // Cleanup
        "cache" => "캐시",
        "logs" => "로그",
        "temp_files" => "임시 파일",
        "trash" => "휴지통",
        "history" => "기록",
        "clean_all" => "모두 정리",

        // Desktop
        "wallpaper" => "배경화면",
        "icons" => "아이콘",
        "theme" => "테마",
        "appearance" => "외관",
        "font" => "글꼴",
        "resolution" => "해상도",
        "taskbar" => "작업 표시줄",
        "widgets" => "위젯",

        // Status
        "running" => "실행 중",
        "stopped" => "중지됨",
        "enabled" => "활성화됨",
        "disabled" => "비활성화됨",
        "active" => "활성",
        "inactive" => "비활성",
        "success" => "성공",
        "error" => "오류",
        "warning" => "경고",
        "info" => "정보",
        "status" => "상태",

        // Misc
        "loading" => "로딩 중...",
        "no_data" => "데이터 없음",
        "enter_search_pattern" => "검색 패턴을 입력하세요",
        "please_wait" => "잠시만 기다려 주세요...",
        "operation_complete" => "작업 완료",
        "operation_failed" => "작업 실패",
        "are_you_sure" => "확실합니까?",
        "yes" => "예",
        "no" => "아니오",
        "ok" => "확인",
        "open" => "열기",
        "close_app" => "닫기",
        "minimize" => "최소화",
        "maximize" => "최대화",

        // Desktop Settings
        "show_desktop_icons" => "바탕화면 아이콘 표시",
        "icon_size" => "아이콘 크기",
        "wallpaper_mode" => "배경화면 모드",
        "normal" => "일반",
        "zoom" => "확대",
        "scale" => "스케일",
        "stretch" => "늘리기",
        "center" => "가운데",
        "change_wallpaper" => "배경화면 변경",
        "dynamic_workspaces" => "동적 작업 공간",
        "num_workspaces" => "작업 공간 수",
        "power_management" => "전원 관리",
        "auto_suspend" => "자동 절전",
        "suspend_after" => "절전까지 (분)",
        "show_battery" => "배터리 잔량 표시",
        "night_light" => "야간 모드",
        "fractional_scaling" => "분수 스케일링",
        "system_sounds" => "시스템 사운드",
        "alert_sound" => "알림 사운드",
        "open_gnome_settings" => "GNOME 설정 열기",
        "system_info" => "시스템 정보",
        "desktop_environment" => "데스크톱 환경",

        // Package Manager
        "package_manager" => "패키지 관리자",
        "search_and_install" => "검색 및 설치",
        "installed_packages" => "설치된 패키지",
        "auto_check_updates" => "자동 업데이트 확인",
        "check_updates" => "업데이트 확인",
        "upgrade_all" => "모두 업그레이드",
        "search_packages" => "패키지 검색...",

        // Disk Cleanup
        "disk_usage_overview" => "디스크 사용량 개요",
        "open_gparted" => "GParted 열기",
        "open_gnome_disks" => "GNOME 디스크 열기",
        "check_health" => "디스크 상태 확인",
        "cleanup_categories" => "정리 카테고리",
        "apt_cache" => "APT 캐시",
        "snap_cache" => "Snap 캐시",
        "flatpak_cache" => "Flatpak 캐시",
        "log_files" => "로그 파일",
        "temp_files_short" => "임시 파일",
        "browser_cache" => "브라우저 캐시",
        "total_space" => "해제 가능한 총 공간",
        "clean_selected" => "선택 항목 정리",
        "scan_again" => "다시 스캔",

        // Duplicate Files
        "duplicate_files" => "중복 파일",
        "find_remove_duplicates" => "중복 파일 찾아서 삭제",
        "scan_duplicates" => "중복 스캔",
        "remove_duplicates" => "중복 삭제",

        // Context Menu
        "context_menu" => "상황별 메뉴 (오른쪽 클릭)",
        "nautilus_context" => "Nautilus 상황별 메뉴",
        "desktop_context" => "바탕화면 상황별 메뉴",
        "custom_menu_items" => "사용자 정의 메뉴 항목",
        "add_custom_item" => "사용자 정의 항목 추가",
        "edit_selected" => "선택 항목 편집",
        "remove_selected" => "선택 항목 삭제",
        "add_to_desktop" => "바탕화면에 추가",
        "remove_from_desktop" => "바탕화면에서 삭제",
        "app_menu" => "응용 프로그램 메뉴",
        "add_to_menu" => "메뉴에 추가",
        "edit_menu" => "선택 항목 편집",
        "remove_from_menu" => "메뉴에서 삭제",
        "reset_default" => "기본값으로 초기화",
        "desktop_shortcuts" => "바탕화면 바로가기",
        "operations_output" => "작업 출력",

        // Process Manager
        "process_manager" => "프로세스 관리자",
        "resource_summary" => "리소스 요약",
        "search_hint" => "이름, PID, 사용자 또는 명령으로 필터링...",
        "process_list" => "프로세스 목록",
        "kill_pid" => "PID 종료",
        "kill_process" => "프로세스 종료",
        "set_priority" => "우선순위 설정",
        "set_nice_value" => "Nice 값 설정",
        "pid_for_priority" => "PID",
        "process_details" => "프로세스 상세정보",

        // Network
        "network_management" => "네트워크 관리",
        "wifi_management" => "WiFi 관리",
        "scan_networks" => "네트워크 스캔",
        "connect" => "연결",
        "disconnect" => "연결 해제",
        "available_wifi" => "사용 가능한 WiFi 네트워크",
        "scanning" => "WiFi 네트워크 스캔 중...",
        "no_wifi" => "WiFi 네트워크를 찾을 수 없거나 WiFi를 사용할 수 없습니다.",
        "vpn_management" => "VPN 관리",
        "vpn_connections" => "VPN 연결:",
        "no_vpn" => "VPN 연결이 구성되지 않았습니다",
        "add_vpn" => "VPN 추가",
        "edit_vpn" => "VPN 편집",
        "remove_vpn" => "VPN 삭제",
        "proxy_settings" => "프록시 설정",
        "enable_proxy" => "프록시 활성화",
        "http_proxy" => "HTTP 프록시",
        "https_proxy" => "HTTPS 프록시",
        "socks_proxy" => "SOCKS 프록시",
        "no_proxy" => "프록시 없음",
        "apply_proxy" => "프록시 설정 적용",
        "dns_settings" => "DNS 설정",
        "current_dns" => "현재 DNS 서버:",
        "no_dns" => "DNS 서버가 구성되지 않았습니다",
        "add_dns_server" => "DNS 서버 추가",
        "add_dns" => "DNS 서버 추가",
        "dns_reset" => "기본값으로 초기화",
        "dns_reset_dhcp" => "DNS가 DHCP 기본값으로 초기화되었습니다",
        "network_diagnostics" => "네트워크 진단",
        "target_host" => "대상 호스트",
        "ping" => "Ping",
        "traceroute" => "traceroute",
        "speed_test" => "속도 테스트",
        "running_speed" => "속도 테스트 실행 중 (10MB 파일 다운로드)...",
        "diag_output" => "진단 출력",
        "run_diag" => "진단 도구를 실행하여 결과를 여기에서 확인하세요...",
        "network_info" => "네트워크 정보",
        "active_connections" => "활성 연결:",
        "no_active_conn" => "활성 연결 없음",
        "interfaces" => "네트워크 인터페이스:",
        "no_interfaces" => "인터페이스를 찾을 수 없습니다",
        "ip_config" => "IP 구성:",
        "open_network_settings" => "네트워크 설정 열기",

        // User Management
        "user_management" => "사용자 관리",
        "current_user_info" => "현재 사용자 정보",
        "system_users" => "시스템 사용자",
        "add_user" => "사용자 추가",
        "new_user" => "새 사용자",
        "full_name" => "전체 이름",
        "create_home_dir" => "홈 디렉토리 생성",
        "system_account" => "시스템 계정",
        "create_user" => "사용자 생성",
        "modify_user" => "사용자 수정",
        "new_shell" => "새 Shell",
        "new_home_dir" => "새 홈 디렉토리",
        "new_full_name" => "새 전체 이름",
        "groups" => "그룹",
        "move_home" => "홈 이동",
        "lock_account" => "계정 잠금",
        "apply_changes" => "변경 사항 적용",
        "unlock_account" => "계정 잠금 해제",
        "delete_user" => "사용자 삭제",
        "group_management" => "그룹 관리",
        "new_group" => "새 그룹",
        "add_group" => "그룹 추가",
        "add_user_to_group" => "그룹에 사용자 추가",
        "add_to_group" => "그룹에 추가",
        "remove_from_group" => "그룹에서 삭제",
        "user_sessions" => "사용자 세션",
        "change_password" => "비밀번호 변경",
        "change_shell" => "Shell 변경",
        "refresh_all" => "모두 새로고침",
        "username" => "사용자 이름",
        "enter_username" => "사용자 이름 입력",

        // Startup Apps
        "startup_apps" => "시작 프로그램",
        "information" => "정보",
        "manage_autostart" => "로그인 시 자동 시작되는 애플리케이션을 관리합니다.\n시작 항목은 ~/.config/autostart/에 .desktop 파일로 저장됩니다",
        "actions" => "작업",
        "app_name" => "앱 이름:",
        "command" => "명령:",
        "add_entry" => "항목 추가",
        "file_name" => "파일 이름:",
        "enable" => "활성화",
        "disable" => "비활성화",
        "output" => "출력",

        // Service Manager
        "service_manager" => "서비스 관리자",
        "filter_services" => "서비스 필터",
        "filter_hint" => "서비스 이름으로 필터링...",
        "type" => "유형:",
        "service_list" => "서비스 목록",
        "service_control" => "서비스 제어",
        "start" => "시작",
        "stop" => "중지",
        "restart" => "재시작",
        "service_details" => "서비스 상세정보",
        "select_service" => "서비스를 선택하고 '상태' 또는 '로그'를 클릭하여 상세정보를 확인하세요.",

        // Log Viewer
        "log_viewer" => "로그 뷰어",
        "log_source" => "로그 소스",
        "system_log" => "시스템 로그",
        "kernel_log" => "커널 로그",
        "auth_log" => "인증 로그",
        "syslog" => "syslog",
        "journalctl" => "Journalctl",
        "service_log" => "서비스 로그",
        "log_level" => "로그 레벨",
        "all" => "모두",
        "emergency" => "긴급",
        "alert" => "경고",
        "critical" => "치명적",
        "err" => "오류",
        "notice" => "알림",
        "debug" => "디버그",
        "time_range" => "시간 범위",
        "last_hour" => "지난 1시간",
        "last_day" => "지난 1일",
        "last_week" => "지난 1주",
        "last_month" => "지난 1개월",
        "lines" => "줄 수:",
        "load_log" => "로그 불러오기",
        "boot_logs" => "부팅 로그",
        "dmesg" => "dmesg",
        "loading_auth_log" => "인증 로그 로딩 중...",
        "loading_boot_logs" => "부팅 로그 로딩 중...",
        "loading_dmesg" => "dmesg 로딩 중...",
        "loading_kernel_log" => "커널 로그 로딩 중...",
        "loading_syslog" => "syslog 로딩 중...",

        // Backup
        "backup_management" => "백업 관리",
        "backup_tool" => "백업 도구",
        "rsync_backup" => "rsync 백업",
        "timeshift_backup" => "Timeshift 백업",
        "borg_backup" => "Borg 백업",
        "backup_source" => "백업 소스:",
        "backup_dest" => "백업 대상:",
        "start_backup" => "백업 시작",
        "start_restore" => "복원 시작",
        "backup_history" => "백업 기록",
        "no_history" => "백업 기록이 없습니다.\n\n팁: 아래 백업 방법으로 첫 번째 백업을 만드세요.",

        // Driver
        "driver_management" => "드라이버 관리",
        "hardware_detection" => "하드웨어 감지",
        "pci_devices" => "PCI 장치",
        "usb_devices" => "USB 장치",
        "loaded_modules" => "로드된 커널 모듈",
        "nvidia_driver" => "NVIDIA 드라이버",
        "amd_driver" => "AMD 드라이버",
        "intel_driver" => "Intel 드라이버",
        "open_drivers" => "오픈 소스 드라이버",
        "proprietary_drivers" => "독점 드라이버",
        "install_driver" => "드라이버 설치",
        "remove_driver" => "드라이버 삭제",

        // Firewall
        "firewall_management" => "방화벽 관리",
        "ufw_status" => "UFW 상태",
        "firewall_rules" => "방화벽 규칙",
        "add_rule" => "규칙 추가",
        "delete_rule" => "규칙 삭제",
        "app_profiles" => "앱 프로필",
        "ufw_enable" => "UFW 활성화",
        "ufw_disable" => "UFW 비활성화",
        "ufw_reset" => "UFW 초기화",

        
        // Optimizer
        "optimizer" => "옵티마이저",
        "system_optimization" => "시스템 최적화",
        "scan_optimizations" => "최적화 스캔",
        "apply_selected" => "선택 적용",
        "kernel_tuning" => "커널 튜닝",
        "service_management" => "서비스 관리",
        "swap_optimization" => "스왑 최적화",
        "filesystem_optimization" => "파일 시스템 최적화",
        "network_optimization" => "네트워크 최적화",
        "power_optimization" => "전원 최적화",
        "memory_optimization" => "메모리 최적화",
        "current_value" => "현재 값",
        "recommended_value" => "권장 값",
        "category" => "카테고리",
        "optimization_items" => "최적화 항목",
        "optimization_results" => "최적화 결과",
        "apply_success" => "적용 성공",
        "apply_failed" => "적용 실패",
        "select_items" => "최적화할 항목 선택",
        _ => "unknown",
    }
}

#[macro_export]
macro_rules! t {
    ($key:expr) => {
        $crate::i18n::tr($key)
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_locale() {
        let locale = detect_locale();
        assert!(matches!(
            locale,
            Locale::ZhCN | Locale::EnUS | Locale::JaJP | Locale::KoKR
        ));
    }

    #[test]
    fn test_tr_english() {
        let result = en_us("desktop");
        assert_eq!(result, "Desktop");
    }

    #[test]
    fn test_tr_chinese() {
        let result = zh_cn("desktop");
        assert_eq!(result, "桌面");
    }

    #[test]
    fn test_tr_japanese() {
        let result = ja_jp("desktop");
        assert_eq!(result, "デスクトップ");
    }

    #[test]
    fn test_tr_korean() {
        let result = ko_kr("desktop");
        assert_eq!(result, "데스크톱");
    }

    #[test]
    fn test_tr_unknown_key() {
        let result = tr("unknown_key");
        assert_eq!(result, "unknown");
    }
}

use std::sync::Mutex;

static DYNAMIC_LOCALE: Mutex<Option<String>> = Mutex::new(None);

pub fn set_locale(code: &str) {
    if let Ok(mut loc) = DYNAMIC_LOCALE.lock() {
        *loc = Some(code.to_string());
    }
}

pub fn current_locale_name() -> String {
    if let Ok(loc) = DYNAMIC_LOCALE.lock() {
        if let Some(ref code) = *loc {
            return code.clone();
        }
    }
    let locale_str = std::env::var("LC_MESSAGES")
        .or_else(|_| std::env::var("LC_ALL"))
        .or_else(|_| std::env::var("LANG"))
        .unwrap_or_else(|_| "en_US.UTF-8".to_string());
    let lower = locale_str.to_lowercase();
    if lower.starts_with("zh") { "zh_CN".to_string() }
    else if lower.starts_with("ja") { "ja_JP".to_string() }
    else if lower.starts_with("ko") { "ko_KR".to_string() }
    else { "en_US".to_string() }
}
