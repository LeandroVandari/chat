use interfaces;
use local_ip_address::local_ip;
use std::io::{prelude::*, stdin, stdout, BufReader};
use std::net::TcpListener;
use std::net::TcpStream;

fn main() {
    println!("{:?}", local_ip());
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
                addr.set_port(7879);
                if let Ok(mut stream) = TcpStream::connect(addr) {
                    let mut read_buffer = String::new();
                    let mut buf_reader = BufReader::new(&mut stream);

                    buf_reader
                        .read_line(&mut read_buffer)
                        .expect("Primeira mensagem deveria ser o nome de usuário");
                    let nome_outro = read_buffer.trim().to_string();

                    stream.write(meu_nome.as_bytes()).unwrap();
                    println!("Conexão com {}!", nome_outro);
                    conexoes.push((nome_outro, stream));
                }
            }
        }
    }

    loop {
        let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
        for stream in listener.incoming() {
            let mut stream = stream.unwrap();

            let mut read_buffer = String::new();

            stream.write(&meu_nome.as_bytes()).unwrap();
            let mut buf_reader = BufReader::new(&mut stream);
            buf_reader
                .read_line(&mut read_buffer)
                .expect("Primeira mensagem deveria ser o nome de usuário");
            let nome_outro = read_buffer.trim().to_string();
            println!("Conexão com {}!", nome_outro);
            conexoes.push((nome_outro, stream));
        }
    }
}
