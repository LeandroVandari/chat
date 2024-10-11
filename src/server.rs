use comunicacao::utilities;
use local_ip_address::local_ip;
use std::{
    collections::VecDeque,
    io::{prelude::*, BufRead, BufReader},
    net::TcpListener,
    thread,
};
use colored::Colorize;

static mut MAX_ID: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

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
            let enviar_mensagens = TcpListener::bind("0.0.0.0:0").unwrap();
            let port_bytes = enviar_mensagens.local_addr().unwrap().port().to_be_bytes();
            stream.write_all(&port_bytes).expect("Não consegui enviar conexao para enviar mensagens");

            let (conexao_enviar, _addr) = enviar_mensagens.accept().unwrap();

            let client = Client::new(nome_outro, stream);

            let (mensagem_tx, receber_mensagem) = std::sync::mpsc::channel();
            let pessoa = client.pessoa.clone();
            let _receber_mensagens_do_cliente = thread::spawn(move || {
                let mut read_buffer = String::new();
                let mut buf_reader = BufReader::new(&client.conexao);
                loop {
                    let amount = buf_reader
                        .read_line(&mut read_buffer)
                        .expect("alguma mensagem");

                    if amount == 0 {
                        mensagem_tx.send(Message::new(pessoa.clone(), TipoMensagem::Saida)).expect("Comunicacao entre threads nao funciona");
                        return;
                    }
                    mensagem_tx
                        .send(Message::new(
                            pessoa.clone(),
                            TipoMensagem::Chat(read_buffer.clone()),
                        ))
                        .expect("Comunicacao entre threads nao funciona");
                    read_buffer.clear();
                }
            });

            tx.send((receber_mensagem, conexao_enviar, client.pessoa))
                .expect("Comunicação entre threads não funciona...");
            read_buffer.clear();
        }
    });

    let mut conexoes_receber = Vec::new();
    let mut conexoes_enviar = Vec::new();
    let mut mensagens = VecDeque::new();
    loop {
        while let Ok((receber_mensagem, client, pessoa)) = nova_conexao.try_recv() {
            mensagens.push_back(Message::new(pessoa.clone(), TipoMensagem::Entrada));
            conexoes_receber.push(receber_mensagem);
            conexoes_enviar.push((client, pessoa.id));
        }

        conexoes_receber.retain(|cliente| {if let Ok(msg) = cliente.try_recv() {
            if let TipoMensagem::Saida = msg.tipo {
                return false;
            }
            mensagens.push_back(msg);
            return true;
        }
    return false;});

        while let Some(msg) = mensagens.pop_front() {
            println!("{msg}");
            let msg_string = msg.to_string();
            let msg_bytes = msg_string.as_bytes();
            for conexao in &mut conexoes_enviar {
                if msg.autor.id != conexao.1 {
                    conexao
                        .0
                        .write_all(msg_bytes)
                        .expect("Não foi possível enviar a mensagem aos clientes");
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
struct Pessoa {
    nome: String,
    id: u32, //cor: (u8, u8, u8)
}

impl Pessoa {
    fn new(nome: String) -> Self {
        let id = unsafe { MAX_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed) };
        Self { nome, id }
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

#[derive(Debug)]
struct Message {
    autor: Pessoa,
    pub tipo: TipoMensagem,
}

#[derive(Debug)]
enum TipoMensagem {
    Entrada,
    Saida,
    Chat(String),
}

impl Message {
    fn new(autor: Pessoa, tipo: TipoMensagem) -> Self {
        Self { autor, tipo }
    }
}

impl std::fmt::Display for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.pessoa.nome)
    }
}

impl std::fmt::Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let texto = match &self.tipo {
            TipoMensagem::Entrada => format!("{} entrou no chat!...", self.autor.nome).bright_blue(),
            TipoMensagem::Saida => format!("{} saiu do chat...", self.autor.nome).red(),
            TipoMensagem::Chat(texto) => format!("{}: {}", self.autor.nome, texto.trim_end()).white(),
        };
        write!(f, "{texto}")
    }
}
