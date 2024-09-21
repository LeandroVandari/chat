use interfaces;
use std::io::{self, prelude::*, stdin, stdout, BufReader};
use std::net::TcpListener;
use std::net::TcpStream;
use std::sync::mpsc;
use std::thread;

fn main() {
    let mut meu_nome = String::new();
    print!("Seu nome de usuário: ");
    let _ = stdout().flush();
    stdin()
        .read_line(&mut meu_nome)
        .expect("Não foi possível ler seu texto :(");
    let mut conexoes = Vec::new();
    //println!("My addr: {}", local_ip().unwrap());

    let interfaces = interfaces::Interface::get_all().unwrap();
    for interface in interfaces {
        for address in &interface.addresses {
            if let Some(mut addr) = address.addr {
                addr.set_port(7880);
                if let Ok(mut stream) = TcpStream::connect(addr) {
                    let mut read_buffer = String::new();
                    let mut buf_reader = BufReader::new(&mut stream);

                    buf_reader
                        .read_line(&mut read_buffer)
                        .expect("Primeira mensagem deveria ser o nome de usuário");
                    let nome_outro = read_buffer.trim().to_string();

                    stream.write(meu_nome.as_bytes()).unwrap();
                    stream.set_nonblocking(true).unwrap();
                    let conexao = Conexao::new(nome_outro, stream);
                    println!("Conexão com {}!", conexao.nome);

                    conexoes.push(conexao);
                }
            }
        }
    }
    let (tx, rec_msg) = mpsc::channel();
    let reader_thread = thread::spawn(move || {
        let mut msg = String::new();

        loop {
            stdin().read_line(&mut msg).unwrap();
            let _ = tx.send(msg.clone());
            msg.clear();
        }
    });
    loop {
        let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
        listener.set_nonblocking(true).expect("Sei lá");

        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    let mut read_buffer = String::new();

                    stream.write(&meu_nome.as_bytes()).unwrap();
                    let mut buf_reader = BufReader::new(&mut stream);
                    buf_reader
                        .read_line(&mut read_buffer)
                        .expect("Primeira mensagem deveria ser o nome de usuário");
                    stream.set_nonblocking(true).expect("Droga");
                    let nome_outro = read_buffer.trim().to_string();
                    let conexao = Conexao::new(nome_outro, stream);
                    println!("Conexão com {}!", conexao.nome);
                    conexoes.push(conexao);
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => break,
                Err(e) => panic!("Error: {e}"),
            }
        }

        for conexao in &mut conexoes {
            let mut receber_mensagens = String::new();
            let mensagem = conexao.conexao.read_to_string(&mut receber_mensagens);
            println!("{mensagem:?}");
            if let Ok(tamanho) = mensagem {
                if tamanho >= 1 {
                    println!("{}: {}", conexao.nome, receber_mensagens);
                }
            }
            while let Ok(msg) = rec_msg.try_recv() {
                println!("Mandando {msg}");
                conexao.conexao.write(&mut msg.as_bytes()).unwrap();
            }
        }
    }
}

struct Conexao {
    nome: String,
    conexao: TcpStream,
}

impl Conexao {
    pub fn new(nome: String, conexao: TcpStream) -> Self {
        Self { nome, conexao }
    }
}
