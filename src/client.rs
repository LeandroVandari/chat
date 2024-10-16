use comunicacao::utilities::{Message, Pessoa};
use comunicacao::{utilities, ChatWindow, TerminalMessage};
use ratatui::style::{Color, Stylize};
use std::io::{prelude::*, BufReader};
use std::net::{IpAddr, SocketAddr, TcpStream};
use std::sync::{atomic, mpsc};
use std::thread;

static mut SERVER_EXITED: atomic::AtomicBool = atomic::AtomicBool::new(false);

pub fn run(meu_nome: String) {
    let ip_servidor: std::net::IpAddr = utilities::input("IP da sala a conectar: ").trim().parse().expect("Endereço IP inválido");
    let (mut servidor, eu) = Servidor::new(ip_servidor, meu_nome);


    let (mensagem_tx, receber_mensagem) = mpsc::channel();
    let _receber_mensagens = thread::spawn(move || loop {
        let mut read_buffer = String::new();
        let mut buf_reader = BufReader::new(&servidor.conexao_receber);
        let amount = buf_reader
            .read_line(&mut read_buffer)
            .expect("Alguma coisa");
        if amount == 0 {
            unsafe { SERVER_EXITED.store(true, std::sync::atomic::Ordering::Relaxed) };
            return;
        }
        read_buffer.pop();
        let msg = serde_json::from_str(&read_buffer).unwrap();
        mensagem_tx
            .send(msg)
            .expect("Comunicacao entre threads nao funciona");
        read_buffer.clear();
    });

    println!("Conectado com sucesso!\n");

    let mut chat_window = ChatWindow::new(None);
    let mut message = None;
    loop {
        match chat_window.draw() {
            TerminalMessage::Tick => (),
            TerminalMessage::Quit => return,
            TerminalMessage::SendMessage(msg) => message = Some(msg),
            TerminalMessage::Command(command) => ()
        }
        if *unsafe { SERVER_EXITED.get_mut() } == true {
            println!("{}", "Server disconnected! Exiting...".fg(Color::Red));
            break;
        }
        if let Some(ref msg) = message {
            chat_window.receive_message(Message::new(
                eu.clone(),
                utilities::TipoMensagem::Chat(msg.clone()),
            ));
            let message_bytes = msg.as_bytes();
            servidor.conexao_enviar
                .write_all(&[&(message_bytes.len() as u32).to_be_bytes(), message_bytes].concat())
                .expect("Não consegui mandar sua mensagem");
        }

        while let Ok(msg) = receber_mensagem.try_recv() {
            chat_window.receive_message(msg);
        }
        message = None;
    }
}


struct Servidor {
    conexao_enviar: TcpStream,
    conexao_receber: TcpStream
}
impl Servidor {
fn new(ip: IpAddr, meu_nome: String) -> (Self, Pessoa)  
{
    let conectar = SocketAddr::new(ip, 7878);
    let mut conexao_enviar =
        TcpStream::connect(conectar).expect("Não foi possível conectar ao servidor");

    conexao_enviar
        .write_all(meu_nome.as_bytes())
        .expect("Não foi possível enviar o nome de usuário ao servidor");

    let mut buffer: [u8; 2] = [0; 2];
    conexao_enviar
        .read_exact(&mut buffer)
        .expect("Servidor não mandou número da porta a conectar");
    let port = u16::from_be_bytes(buffer);
    conexao_enviar.set_nodelay(true).unwrap();
    conexao_enviar.set_nonblocking(true).unwrap();

    let mut conexao_receber = TcpStream::connect(SocketAddr::new(ip, port)).unwrap();

    let mut tamanho_eu_buf = [0; 2];
    conexao_receber.read_exact(&mut tamanho_eu_buf).unwrap();
    let mut eu_buf = vec![0; u16::from_be_bytes(tamanho_eu_buf) as usize];
    conexao_receber.read_exact(&mut eu_buf).unwrap();
    let eu = serde_json::from_str(&String::from_utf8(eu_buf).unwrap()).unwrap();

    (Self {conexao_enviar, conexao_receber}, eu)
}

}