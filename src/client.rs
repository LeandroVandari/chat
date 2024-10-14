use comunicacao::utilities;
use ratatui::style::{Color, Stylize};
use std::io::{prelude::*, BufReader};
use std::net::TcpStream;
use std::sync::{atomic, mpsc};
use std::thread;

static mut SERVER_EXITED: atomic::AtomicBool = atomic::AtomicBool::new(false);

pub fn run(meu_nome: String) {
    let mut ip_servidor = utilities::input("IP da sala a conectar: ")
        .trim()
        .to_string();
    let mut ip_sem_porta = ip_servidor.clone();
    ip_servidor.push_str(":7878");
    let mut conexao_servidor =
        TcpStream::connect(ip_servidor).expect("Não foi possível conectar ao servidor");

    conexao_servidor
        .write_all(meu_nome.as_bytes())
        .expect("Não foi possível enviar o nome de usuário ao servidor");
    let mut buffer: [u8; 2] = [0; 2];
    conexao_servidor
        .read_exact(&mut buffer)
        .expect("Servidor não mandou número da porta a conectar");
    let port = u16::from_be_bytes(buffer);
    let mut port_str = port.to_string();
    port_str.insert_str(0, ":");
    ip_sem_porta.push_str(&port_str);
    let conexao_receber = TcpStream::connect(ip_sem_porta).unwrap();
    conexao_servidor
        .set_nodelay(true)
        .expect("Esse era o problema :)");
    conexao_servidor.set_nonblocking(true).expect("Não sei");

    let (mensagem_tx, receber_mensagem) = mpsc::channel();
    let _receber_mensagens = thread::spawn(move || loop {
        let mut read_buffer = String::new();
        let mut buf_reader = BufReader::new(&conexao_receber);
        let amount = buf_reader
            .read_line(&mut read_buffer)
            .expect("Primeira mensagem deve ser o nome do usuário");
        if amount == 0 {
            unsafe { SERVER_EXITED.store(true, std::sync::atomic::Ordering::Relaxed) };
            return;
        }
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
        if *unsafe { SERVER_EXITED.get_mut() } == true {
            println!("{}", "Server disconnected! Exiting...".fg(Color::Red));
            break;
        }
        while let Ok(msg) = rec_msg.try_recv() {
            conexao_servidor
                .write_all(format!("{}", msg).as_bytes())
                .expect("Não consegui mandar sua mensagem");
        }

        while let Ok(msg) = receber_mensagem.try_recv() {
            println!("{msg}");
        }
    }
}
