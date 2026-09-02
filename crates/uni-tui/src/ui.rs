//! Renders the phase-1 dashboard: CPU/RAM/boot mode, disks and network
//! interfaces. No selectable menu exists yet (Wi-Fi connect / OS pick /
//! install are later phases) — this screen is read-only detection output.

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table};

use uni_hardware::HardwareSnapshot;
use uni_network::InterfaceKind;
use uni_storage::DiskKind;

use crate::app::App;

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

    draw_system_panel(frame, rows[0], app.snapshot.as_ref());

    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(rows[1]);

    draw_disks_panel(frame, columns[0], app.snapshot.as_ref());
    draw_network_panel(frame, columns[1], app.snapshot.as_ref());

    draw_status_bar(frame, rows[2], app);
}

fn draw_system_panel(frame: &mut Frame, area: Rect, snapshot: Option<&HardwareSnapshot>) {
    let block = Block::default().title(" SYSTEM ").borders(Borders::ALL);

    let text = match snapshot {
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

fn draw_network_panel(frame: &mut Frame, area: Rect, snapshot: Option<&HardwareSnapshot>) {
    let block = Block::default().title(" NETWORK ").borders(Borders::ALL);

    let Some(snapshot) = snapshot else {
        frame.render_widget(Paragraph::new("Detecting...").block(block), area);
        return;
    };

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
    let table = Table::new(rows, widths).header(header).block(block);
    frame.render_widget(table, area);
}

fn draw_status_bar(frame: &mut Frame, area: Rect, app: &App) {
    let text = if let Some(err) = &app.error {
        Line::from(vec![Span::styled(
            format!("error: {err}"),
            Style::default().fg(Color::Red),
        )])
    } else {
        Line::from("[q] quit   [r] refresh")
    };
    frame.render_widget(Paragraph::new(text), area);
}
