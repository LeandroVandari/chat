use ratatui::{
    style::{Color, Stylize},
    text::Line,
};
use std::io::{prelude::*, stdin, stdout};
use std::net::TcpStream;

pub fn input(mensagem: &str) -> String {
    let mut meu_nome = String::new();
    print!("{mensagem}");

    let _ = stdout().flush();
    stdin()
        .read_line(&mut meu_nome)
        .expect("Não foi possível ler seu texto :(");

    meu_nome
}

pub struct Conexao {
    pub nome: String,
    pub conexao: TcpStream,
}

impl Conexao {
    pub fn new(nome: String, conexao: TcpStream) -> Self {
        Self { nome, conexao }
    }
}

pub static mut MAX_ID: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

#[derive(Clone, Debug)]
pub struct Pessoa {
    pub nome: String,
    pub id: u32,
    pub cor: (f64, f64, f64),
}

impl Pessoa {
    pub fn new(nome: String) -> Self {
        let id = unsafe { MAX_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed) };
        Self {
            nome,
            id,
            cor: (
                rand::random::<f64>() * 360.0,
                rand::random::<f64>() * 100.0,
                rand::random::<f64>() * 100.0,
            ),
        }
    }
}

pub struct Client {
    pub conexao: std::net::TcpStream,
    pub pessoa: Pessoa,
}

impl Client {
    pub fn new(nome: String, conexao: std::net::TcpStream) -> Self {
        let pessoa = Pessoa::new(nome);
        Self { pessoa, conexao }
    }
}

#[derive(Debug, Clone)]
pub struct Message {
    pub autor: Pessoa,
    pub tipo: TipoMensagem,
}

#[derive(Debug, Clone)]
pub enum TipoMensagem {
    Entrada,
    Saida,
    Chat(String),
}

impl Message {
    pub fn new(autor: Pessoa, tipo: TipoMensagem) -> Self {
        Self { autor, tipo }
    }

    pub fn formatted(&self) -> Line {
        let (h, s, l) = self.autor.cor;
        match self.tipo.clone() {
            TipoMensagem::Entrada => Line::from(vec![
                self.autor.nome.clone().fg(Color::from_hsl(h, s, l)),
                " entrou no chat!...".fg(Color::LightBlue),
            ]),
            TipoMensagem::Saida => Line::from(vec![
                self.autor.nome.clone().fg(Color::from_hsl(h, s, l)),
                " saiu no chat!...".fg(Color::Red),
            ]),
            TipoMensagem::Chat(texto) => Line::from(vec![
                self.autor.nome.clone().fg(Color::from_hsl(h, s, l)),
                ratatui::text::Span::raw(": "),
                ratatui::text::Span::raw(texto.as_str().trim_end().to_string()),
            ]),
        }
    }
}

impl std::fmt::Display for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.pessoa.nome)
    }
}

impl std::fmt::Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (h, s, l) = self.autor.cor;
        let texto = match &self.tipo {
            TipoMensagem::Entrada => {
                format!(
                    "{} {}",
                    self.autor.nome.clone().fg(Color::from_hsl(h, s, l)),
                    "entrou no chat!...".fg(Color::LightBlue)
                )
            }
            TipoMensagem::Saida => format!(
                "{} {}",
                self.autor.nome.clone().fg(Color::from_hsl(h, s, l)),
                "saiu do chat...".fg(Color::Red)
            ),
            TipoMensagem::Chat(texto) => {
                format!(
                    "{}: {}",
                    self.autor.nome.clone().fg(Color::from_hsl(h, s, l)),
                    texto.trim_end()
                )
            }
        };
        write!(f, "{texto}")
    }
}
