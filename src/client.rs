use comunicacao::utilities;
use std::io::{prelude::*, BufReader};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc;
use std::thread;

pub fn run() {
    let meu_nome = utilities::input("Seu nome de usuário: ");
    let mut ip_servidor = utilities::input("IP da sala a conectar: ")
        .trim()
        .to_string();
    ip_servidor.push_str(":7878");
    let listener_receber_mensagens =
        TcpListener::bind("0.0.0.0:7878").expect("Não consigo receber mensagens do servidor");
    let mut conexao_servidor =
        TcpStream::connect(ip_servidor).expect("Não foi possível conectar ao servidor");
    conexao_servidor
        .write_all(meu_nome.as_bytes())
        .expect("Não foi possível enviar o nome de usuário ao servidor");
    conexao_servidor.set_nodelay(true).expect("Esse era o problema :)");
    conexao_servidor.set_nonblocking(true).expect("Não sei");

    let (conexao_receber, _) = listener_receber_mensagens.accept().unwrap();
    let (mensagem_tx, receber_mensagem) = mpsc::channel();
    let _receber_mensagens = thread::spawn(move || loop {
        let mut read_buffer = String::new();
        let mut buf_reader = BufReader::new(&conexao_receber);
        buf_reader
            .read_line(&mut read_buffer)
            .expect("Primeira mensagem deve ser o nome do usuário");
        mensagem_tx
            .send(read_buffer.clone())
            .expect("Comunicacao entre threads nao funciona");
        read_buffer.clear();
    });

    println!("Conectado com sucesso!\n");

    let (tx, rec_msg) = mpsc::channel();
    let _ = thread::spawn(move || {
        let mut msg = String::new();

        loop {
            std::io::stdin().read_line(&mut msg).unwrap();
            let _ = tx.send(msg.clone());
            msg.clear();
        }
    });

    loop {
        while let Ok(msg) = rec_msg.try_recv() {
            conexao_servidor
                .write_all(format!("{meu_nome}: {}", msg).as_bytes())
                .expect("Não consegui mandar sua mensagem");
        }

        while let Ok(msg) = receber_mensagem.try_recv() {
            println!("{msg}");
        }
    }
}
