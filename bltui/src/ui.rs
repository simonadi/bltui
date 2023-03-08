use std::{collections::HashSet, time::Duration};

use crossterm::{
    execute,
    terminal::{disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use log::trace;
use tokio::time::Interval;
use tui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, StatefulWidget, Widget},
    Frame,
};

use crate::{
    devices::{Device, Devices, MacAddress},
    settings::DisplaySettings,
    AppState,
};

lazy_static! {
    static ref INPUT_TICK_RATE: Duration = Duration::from_millis(300);
}

type Backend = tui::backend::CrosstermBackend<std::io::Stdout>;
type Terminal = tui::Terminal<Backend>;

//  ------------------------------ Utils ------------------------------

fn clear_area(area: Rect, buf: &mut Buffer) {
    for x in area.left()..area.right() {
        for y in area.top()..area.bottom() {
            buf.get_mut(x, y).reset();
        }
    }
}

fn get_device_order(devices: &Devices, show_unknown: bool) -> Vec<MacAddress> {
    trace!("get_device_order");
    // Connected devices should be floated at the top
    let result = devices
        .iter()
        .filter(|device| device.connected)
        .map(|device| device.address)
        .chain(
            devices
                .iter()
                .filter(|device| !device.connected)
                .filter(|device| device.paired)
                .map(|device| device.address),
        )
        .chain(
            devices
                .iter()
                .filter(|device| !device.connected)
                .filter(|device| !device.paired)
                .filter(|device| device.name.is_some() || show_unknown)
                .map(|device| device.address),
        )
        .collect();
    trace!("get_device_order: done");
    result
}

pub fn blue_box(title: Option<String>) -> Block<'static> {
    let block = Block::default()
        .style(Style::default().fg(Color::Black))
        .borders(Borders::all())
        .border_style(Style::default().fg(Color::Blue));

    if let Some(title_str) = title {
        block.title(Span::styled(title_str, Style::default().fg(Color::White)))
    } else {
        block
    }
}

struct FrameChunks {
    title: Rect,
    devices: Rect,
    device_details: Rect,
    logger: Rect,
    commands: Rect,
    popup: Rect,
}

impl FrameChunks {
    fn from_frame(frame: &Frame<Backend>) -> FrameChunks {
        let size = frame.size();

        let vertical_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(5),
                Constraint::Length(3),
            ])
            .split(size);

        let middle_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
            .split(vertical_chunks[1]);

        let middle_right_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(middle_chunks[1]);

        let popup = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(20),
                Constraint::Percentage(60),
                Constraint::Percentage(20),
            ])
            .split(
                Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Percentage(45),
                        Constraint::Length(6),
                        Constraint::Percentage(50),
                    ])
                    .split(size)[1],
            )[1];

        FrameChunks {
            title: vertical_chunks[0],
            devices: middle_chunks[0],
            device_details: middle_right_chunks[0],
            logger: middle_right_chunks[1],
            commands: vertical_chunks[2],
            popup,
        }
    }
}

//  ------------------------------ Popup Widgets ------------------------------

pub trait Popup<R>: Widget {
    /// Returns true if the popup should be closed
    fn read(&self) -> R;
}

/// agent event -> popup
/// only variation is whether there's an input or not.
/// confirm and cancel always mean the same thing
/// enter -> confirm -> send Ok with either input value or empty
/// esc -> cancel -> respond with a BluezError
/// popup holds its input state
/// should also be closed if agent receives a cancel request

// if popup handles keypress it needs to return bool to orchestrator to know if it should be deleted.

// trait forces use of dyn
// could be 1 struct with a constructor that allows adding an input or not
// commands are the same
// either confirm cancel and handle_other or just handle_keypress
// responder type known at compile time

struct PopupInput {
    content: String,
    nbrs_only: bool,
}

impl PopupInput {
    fn new(nbrs_only: bool) -> PopupInput {
        PopupInput {
            content: String::new(),
            nbrs_only,
        }
    }
}

// struct Popup<R> {
//     msg: String,
//     tx: Responder<R>,
//     input: Option<PopupInput>,
// }

// impl Popup<R> {
//     fn new() -> Popup<R> {

//     }

//     fn add_input(self, numbers_only: bool) -> Popup<R> {
//         self
//     }

//     fn handle_keypress(&self, keycode: KeyCode) -> bool {
//         match keycode {
//             KeyCode::Enter => {
//                 true
//             }
//             KeyCode::Esc => {
//                 true
//             }
//             _ => {
//                 if self.input.is_some() {

//                 }
//             }
//         }
//     }
// }

// impl Widget for Popup<R> {
//     fn render(self, area: Rect, buf: &mut Buffer) {

//     }
// }

struct InputBox {
    ticker: Interval,
    content: String,
    numbers_only: bool,
}

impl InputBox {
    fn new(numbers_only: bool) -> InputBox {
        InputBox {
            ticker: tokio::time::interval(*INPUT_TICK_RATE),
            content: String::new(),
            numbers_only,
        }
    }
}

impl Widget for InputBox {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let content = self.content + if true { "_" } else { " " };
        Paragraph::new(content)
            .style(Style::default().fg(Color::White))
            .block(blue_box(None))
            .render(area, buf);
    }
}

impl Popup<u32> for InputBox {
    fn read(&self) -> u32 {
        self.content.parse().unwrap_or(0)
    }
}

impl Popup<String> for InputBox {
    fn read(&self) -> String {
        self.content.clone()
    }
}

pub struct InputPopup {
    title: String,
    message: String,
    input_box: InputBox,
}

impl InputPopup {
    pub fn new(title: String, message: String, numbers_only: bool) -> InputPopup {
        InputPopup {
            title,
            message,
            input_box: InputBox::new(numbers_only),
        }
    }
}

impl Widget for InputPopup {
    fn render(self, area: Rect, buf: &mut Buffer) {
        clear_area(area, buf);
        todo!();
    }
}

// impl Popup for InputPopup {
//     fn handle_keypress(&mut self, key: crossterm::event::KeyEvent) -> bool {
//         match key.code {
//             KeyCode::Char(c) => {
//                 if self.input_box.numbers_only && !c.is_ascii_digit() {
//                     return false;
//                 }

//                 self.input_box.content.push(c);
//             }
//             KeyCode::Backspace => {
//                 self.input_box.content.pop();
//             }
//             KeyCode::Enter => {
//                 return true;
//             }
//             KeyCode::Esc => {
//                 return true;
//             }
//             _ => {}
//         }

//         false
//     }
// }

pub struct ConfirmationPopup {
    title: String,
    message: String,
}

impl ConfirmationPopup {
    pub fn new(title: String, message: String) -> ConfirmationPopup {
        ConfirmationPopup { title, message }
    }
}

impl Widget for ConfirmationPopup {
    fn render(self, area: Rect, buf: &mut Buffer) {
        clear_area(area, buf);
        todo!();
    }
}

impl Popup<()> for ConfirmationPopup {
    fn read(&self) {}
}

struct PopupCommands {}

impl PopupCommands {
    fn new() -> PopupCommands {
        PopupCommands {}
    }
}

impl Widget for PopupCommands {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let text = vec![Spans::from("Enter: Confirm"), Spans::from("Esc: Cancel")];

        Paragraph::new(text)
            .style(Style::default().fg(Color::White))
            .block(blue_box(None))
            .render(area, buf);
    }
}

// impl Popup for ConfirmationPopup {
//     fn handle_keypress(&mut self, key: crossterm::event::KeyEvent) -> bool {
//         match key.code {
//             KeyCode::Enter => {
//                 return true;
//             }
//             KeyCode::Esc => {
//                 return true;
//             }
//             _ => {}
//         }

//         false
//     }
// }

// struct RequestPincodePopup {
//     device: Device,
//     input_box: InputBox,
// }

// impl RequestPincodePopup {
//     fn new(device: Device) -> RequestPincodePopup {
//         RequestPincodePopup {
//             device,
//             input_box: InputBox::new(false),
//         }
//     }
// }

// impl Widget for RequestPincodePopup {
//     fn render(self, area: Rect, buf: &mut tui::buffer::Buffer) {
//         clear_area(area, buf);
//         todo!();
//     }
// }

// impl Popup for RequestPincodePopup {
//     fn handle_keypress(&mut self, key: crossterm::event::KeyEvent) -> bool {
//         match key.code {
//             KeyCode::Char(c) => {
//                 if self.input_box.numbers_only && !c.is_ascii_digit() {
//                     return false;
//                 }

//                 self.input_box.content.push(c);
//             }
//             KeyCode::Backspace => {
//                 self.input_box.content.pop();
//             }
//             KeyCode::Enter => {
//                 return true;
//             }
//             _ => {}
//         }

//         false
//     }
// }

// struct DisplayPincodePopup {
//     device: Device,
//     pincode: String,
// }

// impl DisplayPincodePopup {
//     fn new(device: Device, pincode: String) -> DisplayPincodePopup {
//         DisplayPincodePopup { device, pincode }
//     }
// }

// impl Widget for DisplayPincodePopup {
//     fn render(self, area: Rect, buf: &mut tui::buffer::Buffer) {
//         clear_area(area, buf);

//         todo!();
//     }
// }

// struct RequestPasskeyPopup {
//     device: Device,
//     input_box: InputBox,
// }

// impl RequestPasskeyPopup {
//     fn new(device: Device) -> RequestPasskeyPopup {
//         RequestPasskeyPopup {
//             device,
//             input_box: InputBox::new(true),
//         }
//     }
// }

// impl Widget for RequestPasskeyPopup {
//     fn render(self, area: Rect, buf: &mut tui::buffer::Buffer) {
//         clear_area(area, buf);
//         todo!();
//     }
// }

// struct DisplayPasskeyPopup {
//     passkey: String,
// }

// impl DisplayPasskeyPopup {
//     fn new(passkey: String) -> DisplayPasskeyPopup {
//         DisplayPasskeyPopup { passkey }
//     }
// }

// impl Widget for DisplayPasskeyPopup {
//     fn render(self, area: Rect, buf: &mut tui::buffer::Buffer) {
//         clear_area(area, buf);
//         todo!();
//     }
// }

// struct RequestConfirmationPopup {
//     passkey: u32,
// }

// impl RequestConfirmationPopup {
//     fn new(passkey: u32) -> RequestConfirmationPopup {
//         RequestConfirmationPopup { passkey }
//     }
// }

// impl Widget for RequestConfirmationPopup {
//     fn render(self, area: Rect, buf: &mut tui::buffer::Buffer) {
//         clear_area(area, buf);
//         todo!();
//     }
// }

// struct RequestAuthorizationPopup {
//     device: Device,
// }

// impl RequestAuthorizationPopup {
//     fn new(device: Device) -> RequestAuthorizationPopup {
//         RequestAuthorizationPopup { device }
//     }
// }

// impl Widget for RequestAuthorizationPopup {
//     fn render(self, area: Rect, buf: &mut tui::buffer::Buffer) {
//         clear_area(area, buf);
//         todo!();
//     }
// }

// struct AuthorizeServicePopup {
//     device: Device,
//     service: Uuid,
// }

// impl AuthorizeServicePopup {
//     fn new(device: Device, service: Uuid) -> AuthorizeServicePopup {
//         AuthorizeServicePopup { device, service }
//     }
// }

// impl Widget for AuthorizeServicePopup {
//     fn render(self, area: Rect, buf: &mut tui::buffer::Buffer) {
//         clear_area(area, buf);
//         todo!();
//     }
// }

// #[Derive(Popup)]
// enum Popup {
//     #[input(typ = String)]
//     RequestPincode,
// }

// fn init_popup() -> Popup {
//     Popup::RequestPincode
// }

//  ------------------------------ Main Widgets ------------------------------
struct DevicesList {
    devices: Devices,
    selected_device: Option<MacAddress>,
    device_order: Vec<MacAddress>,
    connecting: HashSet<MacAddress>,
    disconnecting: HashSet<MacAddress>,
}

impl DevicesList {
    fn new(
        devices: Devices,
        selected_device: Option<MacAddress>,
        device_order: Vec<MacAddress>,
        connecting: HashSet<MacAddress>,
        disconnecting: HashSet<MacAddress>,
    ) -> DevicesList {
        DevicesList {
            devices,
            selected_device,
            device_order,
            connecting,
            disconnecting,
        }
    }
}

impl Widget for DevicesList {
    fn render(self, area: Rect, buf: &mut tui::buffer::Buffer) {
        let mut list_state = ListState::default();
        if let Some(selected_address) = &self.selected_device {
            list_state.select(
                self.device_order
                    .iter()
                    .position(|address| address == selected_address),
            );
        }
        let list = List::new(
            self.device_order
                .iter()
                .map(|address| self.devices.get_by_mac_address(address).unwrap())
                .map(|device| {
                    ListItem::new(Text::from(vec![Spans::from(vec![
                        Span::from(if let Some(name) = device.name.clone() {
                            name
                        } else {
                            format!("Unknown ({})", device.address)
                        }),
                        Span::styled(
                            if self.connecting.contains(&device.address) {
                                " (Connecting)"
                            } else {
                                ""
                            },
                            Style::default().fg(Color::Yellow),
                        ),
                        Span::styled(
                            if self.disconnecting.contains(&device.address) {
                                " (Disconnecting)"
                            } else {
                                ""
                            },
                            Style::default().fg(Color::Yellow),
                        ),
                        Span::styled(
                            if device.connected { " (Connected)" } else { "" },
                            Style::default().fg(Color::Green),
                        ),
                        Span::styled(
                            if device.paired { " (Paired)" } else { "" },
                            Style::default().fg(Color::Green),
                        ),
                    ])]))
                })
                .collect::<Vec<ListItem>>(),
        )
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().bg(Color::White).fg(Color::Black))
        .highlight_symbol("->")
        .block(blue_box(Some(format!(
            "Devices ({}/{})",
            {
                if let Some(selected_device) = self.selected_device {
                    self.device_order
                        .iter()
                        .position(|mac| mac == &selected_device)
                        .unwrap_or(0)
                        + 1
                } else {
                    0
                }
            },
            self.device_order.len()
        ))));
        StatefulWidget::render(list, area, buf, &mut list_state);
    }
}

struct DeviceDetails {
    device: Option<Device>,
}

impl DeviceDetails {
    fn new(device: Option<Device>) -> DeviceDetails {
        DeviceDetails { device }
    }
}

impl Widget for DeviceDetails {
    fn render(self, area: Rect, buf: &mut tui::buffer::Buffer) {
        let device_details_str = if let Some(device) = self.device {
            Text::from(vec![
                Spans::from(Span::styled(
                    format!("{}", device),
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                )),
                Spans::from(Span::raw(format!("Address: {}", device.address))),
                Spans::from(Span::raw(format!("Paired: {}", device.paired))),
                Spans::from(Span::raw(format!("Connected: {}", device.connected))),
                Spans::from(Span::raw(format!("RSSI: {}", device.rssi.unwrap_or(0)))),
                Spans::from(Span::raw(format!(
                    "Tx power : {} dBm",
                    device.tx_power.unwrap_or(0)
                ))),
            ])
        } else {
            Text::from("No device selected")
        };

        Paragraph::new(device_details_str)
            .block(blue_box(None))
            .render(area, buf);
    }
}

struct Logger {}

impl Logger {
    fn new() -> Logger {
        Logger {}
    }
}

impl Widget for Logger {
    fn render(self, area: Rect, buf: &mut tui::buffer::Buffer) {
        trace!("Rendering logger");
        tui_logger::TuiLoggerWidget::default()
            .block(blue_box(None))
            .style_error(Style::default().fg(Color::Red))
            .style_warn(Style::default().fg(Color::Yellow))
            .style_info(Style::default().fg(Color::White))
            .style_debug(Style::default().fg(Color::Gray))
            .style_trace(Style::default().fg(Color::Gray))
            .output_level(Some(tui_logger::TuiLoggerLevelOutput::Long))
            .output_file(false)
            .output_target(false)
            .output_line(true)
            .output_timestamp(Some("%F %H:%M:%S%.3f".to_string()))
            .render(area, buf);
    }
}

struct Title {}

impl Title {
    fn new() -> Title {
        Title {}
    }
}

impl Widget for Title {
    fn render(self, area: Rect, buf: &mut tui::buffer::Buffer) {
        Paragraph::new("bltui")
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Center)
            .block(blue_box(None))
            .render(area, buf);
    }
}

struct MainCommands {
    scanning: bool,
    show_unknwown: bool,
}

impl MainCommands {
    fn new(state: &AppState, show_unknwown: bool) -> MainCommands {
        MainCommands {
            scanning: state.scanning,
            show_unknwown,
        }
    }
}

impl Widget for MainCommands {
    fn render(self, area: Rect, buf: &mut tui::buffer::Buffer) {
        Paragraph::new(Spans::from(vec![
            Span::raw("⇵: move through devices   "),
            Span::raw(format!(
                "s: {}   ",
                if self.scanning {
                    "stop scanning"
                } else {
                    "start scanning"
                }
            )),
            Span::raw("c: connect   "),
            Span::raw("d: disconnect   "),
            Span::raw("q: quit"),
        ]))
        .style(Style::default().fg(Color::White))
        .block(blue_box(None))
        .render(area, buf);
    }
}

//  ------------------------------ App Renderer ------------------------------

pub struct AppRenderer {
    terminal: Terminal,
    device_order: Vec<MacAddress>,
}

impl AppRenderer {
    pub fn initialize_terminal() -> AppRenderer {
        let mut stdout = std::io::stdout();
        execute!(stdout, EnterAlternateScreen).unwrap();
        crossterm::terminal::enable_raw_mode().unwrap();
        let backend = tui::backend::CrosstermBackend::new(stdout);
        let mut terminal = tui::Terminal::new(backend).unwrap();
        terminal.clear().unwrap();
        AppRenderer {
            terminal,
            device_order: vec![],
        }
    }

    pub fn render(&mut self, state: &AppState, display_settings: &DisplaySettings) {
        trace!("Rendering app");
        self.device_order = get_device_order(&state.devices, display_settings.show_unknown);

        self.terminal
            .draw(|frame| {
                let chunks = FrameChunks::from_frame(frame);

                frame.render_widget(Title::new(), chunks.title);

                trace!("Rendered title");

                frame.render_widget(
                    DevicesList::new(
                        state.devices.clone(),
                        state.selected_device,
                        self.device_order.clone(),
                        state.connecting.clone(),
                        state.disconnecting.clone(),
                    ),
                    chunks.devices,
                );

                trace!("Rendered devices list");

                frame.render_widget(
                    DeviceDetails::new(state.selected_device.map(|mac| {
                        state
                            .devices
                            .get_by_mac_address(&mac)
                            .expect("Selected device not found in devices list")
                    })),
                    chunks.device_details,
                );

                trace!("Rendered device details");

                // frame.render_widget(Logger::new(), chunks.logger);

                trace!("Rendered logger");

                frame.render_widget(
                    MainCommands::new(state, display_settings.show_unknown),
                    chunks.commands,
                );

                trace!("Rendered commands");

                // if let Some(popup) = state.popup {
                //     popup.render(chunks.popup, self.terminal.current_buffer_mut());
                // }
            })
            .unwrap();
        trace!("App rendered");
    }

    pub fn get_next_device(&mut self, selected_device: &MacAddress) -> Option<MacAddress> {
        self.device_order
            .iter()
            .position(|mac| mac == selected_device)
            .and_then(|pos| {
                if pos == self.device_order.len() - 1 {
                    None
                } else {
                    Some(self.device_order[pos + 1])
                }
            })
    }

    pub fn get_previous_device(&mut self, selected_device: &MacAddress) -> Option<MacAddress> {
        self.device_order
            .iter()
            .position(|mac| mac == selected_device)
            .and_then(|pos| {
                if pos == 0 {
                    None
                } else {
                    Some(self.device_order[pos - 1])
                }
            })
    }

    pub fn get_first_device(&mut self) -> Option<MacAddress> {
        self.device_order.first().cloned()
    }

    pub fn get_last_device(&mut self) -> Option<MacAddress> {
        self.device_order.last().cloned()
    }

    pub fn clear(&mut self) {
        disable_raw_mode().unwrap();
        execute!(self.terminal.backend_mut(), LeaveAlternateScreen).unwrap();
    }
}
