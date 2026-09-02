//! Types produced by a Wi-Fi scan and by a connectivity check.

#[derive(Debug, Clone, serde::Serialize)]
pub struct WifiNetwork {
    pub ssid: String,
    /// 0-100.
    pub signal: u8,
    /// Raw `nmcli` security string, e.g. `WPA2`, `--` for open networks.
    pub security: String,
    pub in_use: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum ConnectivityState {
    Online,
    Limited,
    Offline,
    Unknown,
}

/// Parses one `nmcli -t -f SSID,SIGNAL,SECURITY,IN-USE device wifi list`
/// line. `nmcli` terse output escapes literal `:` inside a field as `\:`.
pub fn split_nmcli_fields(line: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut chars = line.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '\\' if chars.peek() == Some(&':') => {
                current.push(':');
                chars.next();
            }
            ':' => {
                fields.push(std::mem::take(&mut current));
            }
            other => current.push(other),
        }
    }
    fields.push(current);
    fields
}

/// Parses the full stdout of `nmcli -t -f SSID,SIGNAL,SECURITY,IN-USE
/// device wifi list`.
pub fn parse_wifi_scan(output: &str) -> Vec<WifiNetwork> {
    output
        .lines()
        .filter(|l| !l.trim().is_empty())
        .filter_map(|line| {
            let fields = split_nmcli_fields(line);
            let ssid = fields.first()?.clone();
            if ssid.is_empty() {
                return None;
            }
            let signal = fields.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
            let security = fields.get(2).cloned().unwrap_or_else(|| "--".to_string());
            let in_use = fields.get(3).map(|s| s == "*").unwrap_or(false);
            Some(WifiNetwork {
                ssid,
                signal,
                security,
                in_use,
            })
        })
        .collect()
}

/// Parses `nmcli networking connectivity check` output (`full`, `limited`,
/// `portal`, `none`, `unknown`).
pub fn parse_connectivity(output: &str) -> ConnectivityState {
    match output.trim() {
        "full" => ConnectivityState::Online,
        "limited" | "portal" => ConnectivityState::Limited,
        "none" => ConnectivityState::Offline,
        _ => ConnectivityState::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_plain_fields() {
        assert_eq!(
            split_nmcli_fields("MyWifi:82:WPA2:*"),
            vec!["MyWifi", "82", "WPA2", "*"]
        );
    }

    #[test]
    fn unescapes_colon_in_ssid() {
        assert_eq!(
            split_nmcli_fields(r"Home\:Network:55:WPA2:"),
            vec!["Home:Network", "55", "WPA2", ""]
        );
    }

    #[test]
    fn parses_scan_output_and_skips_empty_ssids() {
        let output = "MyWifi:82:WPA2:*\n:40:--:\nGuest:60:--:\n";
        let networks = parse_wifi_scan(output);
        assert_eq!(networks.len(), 2);
        assert_eq!(networks[0].ssid, "MyWifi");
        assert_eq!(networks[0].signal, 82);
        assert!(networks[0].in_use);
        assert_eq!(networks[1].security, "--");
        assert!(!networks[1].in_use);
    }

    #[test]
    fn parses_connectivity_states() {
        assert_eq!(parse_connectivity("full\n"), ConnectivityState::Online);
        assert_eq!(parse_connectivity("portal"), ConnectivityState::Limited);
        assert_eq!(parse_connectivity("none"), ConnectivityState::Offline);
        assert_eq!(parse_connectivity("weird"), ConnectivityState::Unknown);
    }
}
