use std::io::{prelude::*, stdin, stdout};
use std::net::{TcpStream, ToSocketAddrs};

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
