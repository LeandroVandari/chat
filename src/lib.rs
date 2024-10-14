pub mod utilities;

use std::{io::Stdout, net::IpAddr, time::Duration};

use crossterm::event::{self, KeyCode, KeyEventKind};
use ratatui::{
    self,
    layout::{Constraint, Layout},
    prelude::{CrosstermBackend, *},
    style::Stylize,
    widgets::{block::Title, Block, Paragraph, Wrap},
    Terminal,
};

pub struct ChatWindow {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    ip: Option<IpAddr>,
    messages: Vec<utilities::Message>,
    tick: u32,
}

impl ChatWindow {
    pub fn new(ip: Option<IpAddr>) -> Self {
        let mut terminal = ratatui::init();
        terminal.clear().expect("Can't clear terminal");

        Self {
            terminal,
            ip,
            messages: vec![],
            tick: 0,
        }
    }

    pub fn draw(&mut self) -> bool {
        self.terminal
            .draw(|frame| {
                let [message_area, send_area] =
                    Layout::vertical([Constraint::Min(3), Constraint::Length(3)])
                        .areas(frame.area());
                let messages = Block::bordered().title("Messages".bold());
                let mut send_message = Block::bordered().title("Enter your message: ".bold());
                if let Some(ip) = self.ip {
                    send_message = send_message.title(
                        Title::from(format!("IP da sala: {}", ip))
                            .alignment(Alignment::Right)
                            .position(ratatui::widgets::block::Position::Bottom),
                    )
                }
                // eprintln!("{:?}", self.messages.concat());
                let texts =
                    Paragraph::new(self.messages.iter().map(|message|message.formatted()).collect::<Vec<Line>>())
                    .block(messages)
                    .left_aligned()
                    .wrap(Wrap { trim: true });
                frame.render_widget(texts, message_area);
                frame.render_widget(send_message, send_area);
            })
            .expect("Ratatui não funcionou");
        if let Ok(true) = event::poll(Duration::from_millis(100)) {
        if let event::Event::Key(key) = event::read().unwrap() {
            if key.kind == KeyEventKind::Press && key.code == KeyCode::Esc {
                return false;
            }
        }}
        self.tick += 1;

        true
    }

    pub fn receive_message(&mut self, message: utilities::Message ) {
        self.messages.push(message);
    }
}

impl Drop for ChatWindow {
    fn drop(&mut self) {
        ratatui::restore();
    }
}

