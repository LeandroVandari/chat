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
    conexao_servidor.set_nodelay(true).expect("Esse era o problema :)");
    conexao_servidor.set_nonblocking(true).expect("Não sei");
    conexao_servidor
        .write_all(meu_nome.as_bytes())
        .expect("Não foi possível enviar o nome de usuário ao servidor");

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
    let mut rec_buffer = Vec::new();
    loop {
        while let Ok(msg) = rec_msg.try_recv() {
            conexao_servidor
                .write_all(format!("{meu_nome}: {}", msg).as_bytes())
                .expect("Não consegui mandar sua mensagem");
        }

        if let Ok(amount) = conexao_servidor.read(&mut rec_buffer) {
            let txt = std::str::from_utf8(&rec_buffer[0..amount]).unwrap();
            println!("{txt}");
            rec_buffer.clear();
        }
    }
}
