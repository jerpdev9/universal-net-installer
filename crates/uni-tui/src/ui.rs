//! Renders the dashboard (CPU/RAM/boot mode, disks, network interfaces,
//! connectivity) and, on top of it, the Wi-Fi scan/connect popups. OS
//! pick / download / install screens don't exist yet — those are later
//! phases.

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Flex, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, Borders, Cell, Clear, List, ListItem, ListState, Paragraph, Row, Table,
};

use uni_hardware::HardwareSnapshot;
use uni_network::{ConnectivityState, InterfaceKind};
use uni_storage::DiskKind;

use crate::app::{App, Screen};

pub fn draw(frame: &mut Frame, app: &App) {
    let root = Block::default()
        .title(" UNIVERSAL NET INSTALLER ")
        .title_alignment(ratatui::layout::Alignment::Center)
        .borders(Borders::ALL);
    let inner = root.inner(frame.area());
    frame.render_widget(root, frame.area());

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),
            Constraint::Min(6),
            Constraint::Length(1),
        ])
        .split(inner);

    draw_system_panel(frame, rows[0], app);

    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(rows[1]);

    draw_disks_panel(frame, columns[0], app.snapshot.as_ref());
    draw_network_panel(frame, columns[1], app);

    draw_status_bar(frame, rows[2], app);

    match app.screen {
        Screen::Dashboard => {}
        Screen::WifiList => draw_wifi_list_popup(frame, app),
        Screen::WifiPassword => draw_wifi_password_popup(frame, app),
    }
}

fn draw_system_panel(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default().title(" SYSTEM ").borders(Borders::ALL);

    let text = match app.snapshot.as_ref() {
        Some(s) => vec![
            Line::from(format!(
                "CPU:  {} ({} cores / {} threads, {})",
                s.cpu.model, s.cpu.physical_cores, s.cpu.logical_cores, s.cpu.architecture
            )),
            Line::from(format!(
                "RAM:  {}",
                uni_storage::format_size(s.memory.total_bytes)
            )),
            Line::from(format!(
                "Boot: {}   GPU: {}",
                match s.boot_mode {
                    uni_hardware::BootMode::Uefi => "UEFI",
                    uni_hardware::BootMode::Bios => "BIOS (legacy)",
                },
                if s.gpus.is_empty() {
                    "none detected".to_string()
                } else {
                    s.gpus
                        .iter()
                        .map(|g| format!("{} {}", g.vendor, g.model))
                        .collect::<Vec<_>>()
                        .join(", ")
                }
            )),
        ],
        None => vec![Line::from("Detecting hardware...")],
    };

    frame.render_widget(Paragraph::new(text).block(block), area);
}

fn draw_disks_panel(frame: &mut Frame, area: Rect, snapshot: Option<&HardwareSnapshot>) {
    let block = Block::default().title(" DISKS ").borders(Borders::ALL);

    let Some(snapshot) = snapshot else {
        frame.render_widget(Paragraph::new("Detecting...").block(block), area);
        return;
    };

    let header = Row::new(vec!["DEVICE", "TYPE", "SIZE", "MODEL", "STATE"])
        .style(Style::default().add_modifier(Modifier::BOLD));

    let rows = snapshot.disks.iter().map(|disk| {
        let kind = match disk.kind {
            DiskKind::Nvme => "NVMe",
            DiskKind::SataSsd => "SATA SSD",
            DiskKind::Hdd => "HDD",
            DiskKind::Usb => "USB",
            DiskKind::Unknown => "unknown",
        };
        let state = if disk.is_protected() {
            Cell::from("PROTECTED").style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
        } else {
            Cell::from("")
        };
        Row::new(vec![
            Cell::from(disk.path.clone()),
            Cell::from(kind),
            Cell::from(uni_storage::format_size(disk.size_bytes)),
            Cell::from(disk.model.clone().unwrap_or_else(|| "unknown".to_string())),
            state,
        ])
    });

    let widths = [
        Constraint::Length(14),
        Constraint::Length(10),
        Constraint::Length(10),
        Constraint::Min(16),
        Constraint::Length(10),
    ];

    let table = Table::new(rows, widths).header(header).block(block);
    frame.render_widget(table, area);
}

fn draw_network_panel(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default().title(" NETWORK ").borders(Borders::ALL);

    let Some(snapshot) = app.snapshot.as_ref() else {
        frame.render_widget(Paragraph::new("Detecting...").block(block), area);
        return;
    };

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(1)])
        .split(inner);

    let (label, color) = connectivity_label(app.connectivity);
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::raw("Internet: "),
            Span::styled(label, Style::default().fg(color)),
        ])),
        sections[0],
    );

    let header = Row::new(vec!["IFACE", "KIND", "STATE"])
        .style(Style::default().add_modifier(Modifier::BOLD));

    let rows = snapshot.interfaces.iter().map(|iface| {
        let kind = match iface.kind {
            InterfaceKind::Ethernet => "Ethernet",
            InterfaceKind::WiFi => "Wi-Fi",
            InterfaceKind::Loopback => "Loopback",
            InterfaceKind::Other => "Other",
        };
        let state = if iface.is_up {
            Cell::from("up").style(Style::default().fg(Color::Green))
        } else {
            Cell::from("down").style(Style::default().fg(Color::DarkGray))
        };
        Row::new(vec![
            Cell::from(iface.name.clone()),
            Cell::from(kind),
            state,
        ])
    });

    let widths = [
        Constraint::Length(10),
        Constraint::Length(10),
        Constraint::Length(6),
    ];
    let table = Table::new(rows, widths).header(header);
    frame.render_widget(table, sections[1]);
}

fn connectivity_label(state: Option<ConnectivityState>) -> (&'static str, Color) {
    match state {
        Some(ConnectivityState::Online) => ("Online", Color::Green),
        Some(ConnectivityState::Limited) => ("Limited", Color::Yellow),
        Some(ConnectivityState::Offline) => ("Offline", Color::Red),
        Some(ConnectivityState::Unknown) | None => ("Unknown", Color::DarkGray),
    }
}

fn draw_status_bar(frame: &mut Frame, area: Rect, app: &App) {
    if let Some(status) = &app.status {
        let color = if status.is_error {
            Color::Red
        } else {
            Color::Green
        };
        frame.render_widget(
            Paragraph::new(Line::from(vec![Span::styled(
                &status.text,
                Style::default().fg(color),
            )])),
            area,
        );
        return;
    }

    if let Some(err) = &app.error {
        frame.render_widget(
            Paragraph::new(Line::from(vec![Span::styled(
                format!("error: {err}"),
                Style::default().fg(Color::Red),
            )])),
            area,
        );
        return;
    }

    let hints = match app.screen {
        Screen::Dashboard => "[q] quit   [r] refresh   [w] wifi",
        Screen::WifiList => "[↑/↓] select   [Enter] connect   [r] rescan   [Esc] back",
        Screen::WifiPassword => "[Enter] connect   [Esc] cancel",
    };
    frame.render_widget(Paragraph::new(hints), area);
}

fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let horizontal = Layout::horizontal([Constraint::Length(width)]).flex(Flex::Center);
    let vertical = Layout::vertical([Constraint::Length(height)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}

fn draw_wifi_list_popup(frame: &mut Frame, app: &App) {
    let area = centered_rect(60, 14, frame.area());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(" SELECT WI-FI NETWORK ")
        .borders(Borders::ALL);

    if app.wifi_networks.is_empty() {
        frame.render_widget(
            Paragraph::new("No networks found. [r] to rescan.").block(block),
            area,
        );
        return;
    }

    let items: Vec<ListItem> = app
        .wifi_networks
        .iter()
        .map(|network| {
            let marker = if network.in_use { "* " } else { "  " };
            let lock = if network.security.trim() == "--" {
                " "
            } else {
                "🔒"
            };
            ListItem::new(format!(
                "{marker}{:<24} {:>3}% {lock}",
                network.ssid, network.signal
            ))
        })
        .collect();

    let mut state = ListState::default().with_selected(Some(app.wifi_selected));
    let list = List::new(items)
        .block(block)
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::Cyan),
        )
        .highlight_symbol("> ");

    frame.render_stateful_widget(list, area, &mut state);
}

fn draw_wifi_password_popup(frame: &mut Frame, app: &App) {
    let area = centered_rect(50, 5, frame.area());
    frame.render_widget(Clear, area);

    let ssid = app.wifi_pending_ssid.as_deref().unwrap_or("");
    let block = Block::default()
        .title(format!(" PASSWORD FOR {ssid} "))
        .borders(Borders::ALL);

    let masked: String = "*".repeat(app.password_input.chars().count());
    frame.render_widget(Paragraph::new(masked).block(block), area);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn connectivity_label_maps_every_state() {
        assert_eq!(
            connectivity_label(Some(ConnectivityState::Online)).0,
            "Online"
        );
        assert_eq!(
            connectivity_label(Some(ConnectivityState::Limited)).0,
            "Limited"
        );
        assert_eq!(
            connectivity_label(Some(ConnectivityState::Offline)).0,
            "Offline"
        );
        assert_eq!(connectivity_label(None).0, "Unknown");
    }
}
