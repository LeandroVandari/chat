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

    

    println!("Conectado com sucesso!\n");

    let mut chat_window = ChatWindow::new(None);
    let mut message = None;
    let mut read_buffer = vec![0;2usize.pow(20)];
    loop {
        match chat_window.draw() {
            TerminalMessage::Tick => (),
            TerminalMessage::Quit => return,
            TerminalMessage::SendMessage(msg) => message = Some(msg),
            TerminalMessage::Command(command) => ()
        }
        
        if let Ok(amount) = servidor.conexao.read(&mut read_buffer) {
        if amount == 0 {
            println!("{}", "Server disconnected! Exiting...".fg(Color::Red));
            break;
        }
        let message_size = u32::from_be_bytes(unsafe {*((read_buffer[0..4].as_ptr()) as *const [u8;4])}) as usize;
        let texto = String::from_utf8(read_buffer[4..message_size+4].to_vec()).unwrap();
        let msg = serde_json::from_str(&texto).unwrap();
        chat_window.receive_message(msg);
        }
        if let Some(ref msg) = message {
            chat_window.receive_message(Message::new(
                eu.clone(),
                utilities::TipoMensagem::Chat(msg.clone()),
            ));
            let message_bytes = msg.as_bytes();
            servidor.conexao
                .write_all(&[&(message_bytes.len() as u32).to_be_bytes(), message_bytes].concat())
                .expect("Não consegui mandar sua mensagem");
        }

        message = None;
    }
}


struct Servidor {
    conexao: TcpStream,

}
impl Servidor {
fn new(ip: IpAddr, meu_nome: String) -> (Self, Pessoa)  
{
    let conectar = SocketAddr::new(ip, 7878);
    let mut conexao =
        TcpStream::connect(conectar).expect("Não foi possível conectar ao servidor");

    conexao
        .write_all(meu_nome.as_bytes())
        .expect("Não foi possível enviar o nome de usuário ao servidor");



    

    let mut tamanho_eu_buf = [0; 2];
    conexao.read_exact(&mut tamanho_eu_buf).unwrap();
    let mut eu_buf = vec![0; u16::from_be_bytes(tamanho_eu_buf) as usize];
    conexao.read_exact(&mut eu_buf).unwrap();
    let eu = serde_json::from_str(&String::from_utf8(eu_buf).unwrap()).unwrap();
    conexao.set_nodelay(true).unwrap();
    conexao.set_nonblocking(true).unwrap();
    (Self {conexao}, eu)
}

}