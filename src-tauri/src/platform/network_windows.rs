use std::process::Command;

pub(crate) fn get_wifi_ssid() -> Option<String> {
    let output = Command::new("netsh")
        .args(["wlan", "show", "interfaces"])
        .output()
        .ok()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_wifi_ssid(&stdout)
}

pub fn get_local_ipv4() -> Option<String> {
    local_ip_address::local_ip().ok().map(|ip| ip.to_string())
}

pub fn get_local_ipv6() -> Option<String> {
    let output = Command::new("ipconfig").output().ok()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_ipv6(&stdout)
}

fn parse_ipv6(output: &str) -> Option<String> {
    for line in output.lines() {
        let line = line.trim();
        if !line.contains("IPv6") {
            continue;
        }
        if let Some((_, value)) = line.split_once(':') {
            let value = value.trim();
            if value.is_empty() || value.to_lowercase().starts_with("fe80") {
                continue;
            }
            let ip = value.split('%').next().unwrap_or(value);
            return Some(ip.to_string());
        }
    }
    None
}

fn parse_wifi_ssid(output: &str) -> Option<String> {
    // Try to find SSID in a connected adapter block first
    let blocks: Vec<&str> = output.split("\n\n").collect();
    for block in &blocks {
        let is_connected = block.contains("已连接") || block.contains("connected");
        if !is_connected {
            continue;
        }
        for line in block.lines() {
            let line = line.trim();
            if let Some((key, value)) = line.split_once(':') {
                let key = key.trim();
                let value = value.trim();
                if key == "SSID" && !value.is_empty() {
                    return Some(value.to_string());
                }
            }
        }
    }
    // Fallback: return first SSID found regardless of connection state
    for line in output.lines() {
        let line = line.trim();
        if let Some((key, value)) = line.split_once(':') {
            let key = key.trim();
            let value = value.trim();
            if key == "SSID" && !value.is_empty() {
                return Some(value.to_string());
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_ssid_from_chinese_windows_netsh_output() {
        let output = "系统上有 1 个接口: \n\
            \n\
            \u{0020}   名称                   : WLAN\n\
            \u{0020}   说明            : MediaTek Wi-Fi 7 MT7925 Wireless LAN Card\n\
            \u{0020}   状态                  : 已连接\n\
            \u{0020}   SSID                   : TUST-5G\n\
            \u{0020}   AP BSSID               : a4:6d:a4:28:f2:95\n\
            \u{0020}   波段                   : 5 GHz\n";

        assert_eq!(parse_wifi_ssid(output), Some("TUST-5G".to_string()));
    }

    #[test]
    fn returns_none_when_no_ssid_present() {
        let output = "系统上有 1 个接口: \n\
            \n\
            \u{0020}   名称                   : WLAN\n\
            \u{0020}   状态                  : 已断开连接\n\
            \u{0020}   AP BSSID               : a4:6d:a4:28:f2:95\n";

        assert_eq!(parse_wifi_ssid(output), None);
    }

    #[test]
    fn returns_ssid_of_connected_adapter_when_multiple_exist() {
        let output = "系统上有 2 个接口: \n\
            \n\
            \u{0020}   名称                   : Wi-Fi 1\n\
            \u{0020}   状态                  : 已断开连接\n\
            \u{0020}   SSID                   : OLD_SSID\n\
            \n\
            \u{0020}   名称                   : Wi-Fi 2\n\
            \u{0020}   状态                  : 已连接\n\
            \u{0020}   SSID                   : TUST-5G\n";

        assert_eq!(parse_wifi_ssid(output), Some("TUST-5G".to_string()));
    }

    #[test]
    fn extracts_global_ipv6_from_ipconfig_output() {
        let output = "\n\
            无线局域网适配器 WLAN:\n\
            \u{0020}   IPv6 地址 . . . . . . . . . . . . : 2001:da8:a005:31a::4:3b82\n\
            \u{0020}   本地链接 IPv6 地址. . . . . . . . : fe80::762e:c0a4:6563:afa9%26\n\
            \u{0020}   IPv4 地址 . . . . . . . . . . . . : 10.59.16.97\n";

        assert_eq!(parse_ipv6(output), Some("2001:da8:a005:31a::4:3b82".to_string()));
    }

    #[test]
    fn returns_none_when_only_link_local_ipv6_present() {
        let output = "\n\
            无线局域网适配器 WLAN:\n\
            \u{0020}   本地链接 IPv6 地址. . . . . . . . : fe80::762e:c0a4:6563:afa9%26\n\
            \u{0020}   IPv4 地址 . . . . . . . . . . . . : 10.59.16.97\n";

        assert_eq!(parse_ipv6(output), None);
    }
}
