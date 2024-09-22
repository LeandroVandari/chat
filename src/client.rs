use comunicacao::utilities;
use std::io::prelude::*;
use std::net::TcpStream;
use std::sync::mpsc;
use std::thread;

pub fn run() {
    let meu_nome = utilities::input("Seu nome de usuário: ");
    let mut ip_servidor = utilities::input("IP da sala a conectar: ")
        .trim()
        .to_string();
    ip_servidor.push_str(":7878");

    let mut conexao_servidor =
        TcpStream::connect(ip_servidor).expect("Não foi possível conectar ao servidor");
    conexao_servidor
        .write_all(meu_nome.as_bytes())
        .expect("Não foi possível enviar o nome de usuário ao servidor");
    conexao_servidor.set_nonblocking(true).expect("Não sei");
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

    let mut receive_buffer = String::new();
    loop {
        while let Ok(msg) = rec_msg.try_recv() {
            conexao_servidor
                .write_all(format!("{meu_nome}: {}", msg).as_bytes())
                .expect("Não consegui mandar sua mensagem");
        }

        let mensagem = conexao_servidor.read_to_string(&mut receive_buffer);

        if let Ok(tamanho) = mensagem {
            if tamanho >= 1 {
                println!("{receive_buffer}");
            }
        }

        receive_buffer.clear();
    }
}
