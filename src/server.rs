use comunicacao::utilities;
use local_ip_address::local_ip;
use std::{
    collections::VecDeque,
    io::{prelude::*, BufRead, BufReader},
    net::TcpListener,
    thread,
};

use utilities::{Message, Pessoa, TipoMensagem};

pub fn run() {
    let meu_nome = utilities::input("Seu nome de usuário: ");
    let listener = TcpListener::bind("0.0.0.0:7878").expect("Não consegui abrir um servidor");
    println!(
        "IP da sala: {}",
        local_ip().expect("Não consegui pegar o seu IP")
    );

    let (tx, nova_conexao) = std::sync::mpsc::channel();
    let _receber_conexoes = thread::spawn(move || {
        let mut read_buffer = String::new();
        println!("Preparando para ouvir conexões... ");
        for stream in listener.incoming() {
            let mut stream = stream.unwrap();

            let mut buf_reader = BufReader::new(&mut stream);
            buf_reader
                .read_line(&mut read_buffer)
                .expect("Primeira mensagem deve ser o nome do usuário");
            let nome_outro = read_buffer.trim().to_string();

            stream.set_nodelay(true).expect("Grandes problemas");
            stream.set_nonblocking(true).expect("Agora é pra funcionar");

            let client = Client::new(nome_outro, stream);

            tx.send(client)
                .expect("Comunicação entre threads não funciona...");
            read_buffer.clear();
        }
    });

    let mut conexoes = Vec::new();
    let mut mensagens = VecDeque::new();
    let mut rec_buffer = Vec::new();
    loop {
        while let Ok(client) = nova_conexao.try_recv() {
            mensagens.push_back(Message::new(client.pessoa.clone(), TipoMensagem::Entrada));
            conexoes.push(client);
        }
        for cliente in &mut conexoes {
        if let Ok(amount) = cliente.conexao.read(&mut rec_buffer) {
                let txt = std::str::from_utf8(&rec_buffer[0..amount]).unwrap();
                let mensagem = Message::new(cliente.pessoa.clone(), TipoMensagem::Chat(txt.to_string()));
                mensagens.push_back(mensagem);
                rec_buffer.clear();
            }
        }

        while let Some(msg) = mensagens.pop_front() {
            println!("{msg}");
            let msg_string = msg.to_string();
            let msg_bytes = msg_string.as_bytes();
            for cliente in &mut conexoes {
                if msg.autor.id() != cliente.pessoa.id() {
                    cliente.conexao
                        .write_all(msg_bytes)
                        .expect("Não foi possível enviar a mensagem aos clientes");
                }
            }
        }
    }
}



struct Client {
    conexao: std::net::TcpStream,
    pessoa: Pessoa,
}

impl Client {
    fn new(nome: String, conexao: std::net::TcpStream) -> Self {
        let pessoa = Pessoa::new(nome);
        Self { pessoa, conexao }
    }
}

impl std::fmt::Display for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.pessoa.nome)
    }
}
