use comunicacao::{
    utilities::{Client, Message, Pessoa, TipoMensagem},
    ChatWindow, TerminalMessage,
};
use local_ip_address::local_ip;
use std::{
    collections::VecDeque,
    io::{prelude::*, BufRead, BufReader},
    net::TcpListener,
    thread,
};

pub fn run(meu_nome: String) {
    let eu = Pessoa::new(meu_nome);
    let listener = TcpListener::bind("0.0.0.0:7878").expect("Não consegui abrir um servidor");
    let (tx, nova_conexao) = std::sync::mpsc::channel();
    let _receber_conexoes = thread::spawn(move || {
        let mut read_buffer = String::new();
        for stream in listener.incoming() {
            let mut stream = stream.unwrap();
            let mut buf_reader = BufReader::new(&mut stream);
            buf_reader
                .read_line(&mut read_buffer)
                .expect("Primeira mensagem deve ser o nome do usuário");
            let nome_outro = read_buffer.trim().to_string();

    
            let mut client = Client::new(nome_outro, stream);
            let client_json = serde_json::to_string(&client.pessoa).unwrap();
            client.conexao.write_all(&(client_json.len() as u16).to_be_bytes()).unwrap();
            client.conexao.write_all(client_json.as_bytes()).unwrap();
            client.conexao.set_nonblocking(true).unwrap();
            

            tx.send(client)
                .expect("Comunicação entre threads não funciona...");
            read_buffer.clear();
        }
    });

    let mut read_buffer = vec![0;2usize.pow(20)];
    let mut conexoes = Vec::new();
    let mut mensagens = VecDeque::new();
    let mut chat_window = ChatWindow::new(Some(local_ip().expect("Não foi possível pegar o IP")));
    let mut idx_remove = Vec::new();
    loop {
        match chat_window.draw() {
            TerminalMessage::Tick => (),
            TerminalMessage::Quit => return,
            TerminalMessage::SendMessage(msg) => {
                mensagens.push_back(Message::new(eu.clone(), TipoMensagem::Chat(msg)))
            }
            TerminalMessage::Command(command) => ()
        }

        while let Ok(client) = nova_conexao.try_recv() {
            mensagens.push_back(Message::new(client.pessoa.clone(), TipoMensagem::Entrada));
            conexoes.push(client);
        }
        for (idx, client) in conexoes.iter_mut().enumerate() {
            if let Ok(amount) = client.conexao.read(&mut read_buffer)
                         {

                    if amount == 0 {
                        mensagens.push_back(
                            Message::new(client.pessoa.clone(), TipoMensagem::Saida));
                        idx_remove.push(idx);
                        continue;
                    }
                    let message_size = u32::from_be_bytes(unsafe {*((read_buffer[0..4].as_ptr()) as *const [u8;4])}) as usize;
                    let texto = String::from_utf8(read_buffer[4..message_size+4].to_vec()).unwrap();
                    mensagens.push_back(
                        Message::new(
                            client.pessoa.clone(),
                            TipoMensagem::Chat(texto),
                        ));
                    read_buffer.clear();
                        }
        }
        for idx in &idx_remove {
            conexoes.remove(*idx);
        }
        idx_remove.clear();


        while let Some(msg) = mensagens.pop_front() {
            chat_window.receive_message(msg.clone());
            for client in &mut conexoes {
                if msg.autor.id != client.pessoa.id {
                    let msg_json = serde_json::to_string(&msg).unwrap();
                    let msg_bytes = msg_json.as_bytes();
    
                    client.conexao
                        .write_all(&[&(msg_bytes.len() as u32).to_be_bytes(), msg_bytes].concat())
                        .expect("Não foi possível enviar a mensagem aos clientes");
                }
            }
        }
    }
}
