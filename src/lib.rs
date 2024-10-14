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
use utilities::Message;

pub struct ChatWindow {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    ip: Option<IpAddr>,
    messages: Vec<Message>,
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
                let mut sum = 0;
                let mut lines_vec = self
                    .messages
                    .iter()
                    .rev()
                    .take(message_area.height as usize - 1)
                    .map(|msg| {
                        let line = msg.formatted();
                        (
                            {let line_string = line.spans
                                .iter()
                                .fold(String::new(), |mut acc, span| {acc.push_str(&span.content); acc});

                            let words = line_string.split_ascii_whitespace();
                            let bigger_than_line = words.clone().enumerate().filter(|(idx, word)| word.len() > (message_area.height as usize - 2));
                            if bigger_than_line.clone().peekable().peek().is_none() {
                                line.spans.iter().fold(0, |acc, span| acc + span.content.len())
                                .div_ceil(message_area.width as usize - 2)
                            }
                            else {
                            let mut last_bigger_idx: isize = -1;
                            let words_vec = words.collect::<Vec<_>>();
                            let mut lines_amount = 0;
                            for (idx, word) in bigger_than_line {
                                let lines_before = words_vec[(last_bigger_idx+1) as usize..idx].iter().fold(0, |acc, word| acc + word.len() + 1).div_ceil(message_area.width as usize - 2);
                                lines_amount += lines_before;
                                lines_amount += word.len().div_ceil(message_area.width as usize - 2);
                                last_bigger_idx = idx as isize;
                            }
                            lines_amount}

                            },


                            line,
                        )
                    })
                    .take_while(|(len, _line)| {
                        sum += len;
                        sum <= (message_area.height - 2) as usize
                    })
                    .map(|(_len, line)| line)
                    .collect::<Vec<Line>>();
                lines_vec.reverse();

                // eprintln!("{:?}", self.messages.concat());
                let texts = Paragraph::new(lines_vec)
                    .block(messages)
                    .left_aligned()
                    .wrap(Wrap { trim: true });

                frame.render_widget(texts, message_area);
                frame.render_widget(send_message, send_area);
            })
            .expect("Ratatui não funcionou");
        if let Ok(true) = event::poll(Duration::from_millis(10)) {
            if let event::Event::Key(key) = event::read().unwrap() {
                if key.kind == KeyEventKind::Press && key.code == KeyCode::Esc {
                    return false;
                }
            }
        }
        self.tick += 1;

        true
    }

    pub fn receive_message(&mut self, message: utilities::Message) {
        self.messages.push(message);
    }
}

impl Drop for ChatWindow {
    fn drop(&mut self) {
        ratatui::restore();
    }
}
