use std::{collections::VecDeque, io::{BufRead, BufReader}, net::TcpListener, thread};
use local_ip_address::local_ip;
use comunicacao::utilities;

pub fn run() {
    let meu_nome = utilities::input("Seu nome de usuário: ");
    let listener = TcpListener::bind("0.0.0.0:7878").expect("Não consegui abrir um servidor");
    println!("IP da sala: {}", local_ip().expect("Não consegui pegar o seu IP"));

    let (tx, nova_conexao) = std::sync::mpsc::channel();
    let new_connections_thread = thread::spawn(move || {

        let mut read_buffer = String::new();
        for stream in listener.incoming() {
            let mut stream = stream.unwrap();
            let mut buf_reader = BufReader::new(&mut stream);

            buf_reader.read_line(&mut read_buffer).expect("Primeira mensagem deve ser o nome do usuário");

            let nome_outro = read_buffer.trim().to_string();

            tx.send((nome_outro, stream)).expect("Comunicação entre threads não funciona...");
            read_buffer.clear();
        }
    });

    let mut conexoes = Vec::new();
    let mut mensagens = VecDeque::new();
    loop {
        while let Ok((nome, stream)) = nova_conexao.try_recv() {
            mensagens.push_back(format!("{nome} entrou na conversa!"));
            conexoes.push(stream);
        }
    }
}
