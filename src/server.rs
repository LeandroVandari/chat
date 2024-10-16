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
            let enviar_mensagens = TcpListener::bind("0.0.0.0:0").unwrap();
            let port_bytes = enviar_mensagens.local_addr().unwrap().port().to_be_bytes();
            stream
                .write_all(&port_bytes)
                .expect("Não consegui enviar conexao para enviar mensagens");

        

            let (mut conexao_enviar, _addr) = enviar_mensagens.accept().unwrap();
            conexao_enviar.set_nodelay(true).expect("aonteuhaoneuh");

            let mut client = Client::new(nome_outro, stream);
            let client_json = serde_json::to_string(&client.pessoa).unwrap();
            conexao_enviar.write_all(&(client_json.len() as u16).to_be_bytes()).unwrap();
            conexao_enviar.write_all(client_json.as_bytes()).unwrap();
            client.conexao.set_nonblocking(true).unwrap();
            let (mensagem_tx, receber_mensagem) = std::sync::mpsc::channel();
            let pessoa = client.pessoa.clone();
            let _receber_mensagens_do_cliente = thread::spawn(move || {
                let mut read_buffer = vec![0;2usize.pow(20)];
                loop {
                    
                    if let Ok(amount) = client.conexao.read(&mut read_buffer)
                         {

                    if amount == 0 {
                        mensagem_tx
                            .send(Message::new(pessoa.clone(), TipoMensagem::Saida))
                            .expect("Comunicacao entre threads nao funciona");
                        break;
                    }
                    let message_size = u32::from_be_bytes(unsafe {*((read_buffer[0..4].as_ptr()) as *const [u8;4])}) as usize;
                    let texto = String::from_utf8(read_buffer[4..message_size+4].to_vec()).unwrap();
                    mensagem_tx
                        .send(Message::new(
                            pessoa.clone(),
                            TipoMensagem::Chat(texto),
                        ))
                        .expect("Comunicacao entre threads nao funciona");
                    read_buffer.clear();
                        }
                }
            });

            tx.send((receber_mensagem, conexao_enviar, client.pessoa))
                .expect("Comunicação entre threads não funciona...");
            read_buffer.clear();
        }
    });

    let mut conexoes_receber = Vec::new();
    let mut conexoes_enviar = Vec::new();
    let mut mensagens = VecDeque::new();
    let mut chat_window = ChatWindow::new(Some(local_ip().expect("Não foi possível pegar o IP")));
    loop {
        match chat_window.draw() {
            TerminalMessage::Tick => (),
            TerminalMessage::Quit => return,
            TerminalMessage::SendMessage(msg) => {
                mensagens.push_back(Message::new(eu.clone(), TipoMensagem::Chat(msg)))
            }
            TerminalMessage::Command(command) => ()
        }

        while let Ok((receber_mensagem, client, pessoa)) = nova_conexao.try_recv() {
            mensagens.push_back(Message::new(pessoa.clone(), TipoMensagem::Entrada));
            conexoes_receber.push(receber_mensagem);
            conexoes_enviar.push((client, pessoa.id));
        }

        conexoes_receber = conexoes_receber
            .into_iter()
            .enumerate()
            .filter(|(idx, cliente)| {
                if let Ok(msg) = cliente.try_recv() {
                    let tipo = msg.tipo.clone();
                    mensagens.push_back(msg);
                    if let TipoMensagem::Saida = tipo {
                        conexoes_enviar.remove(*idx);
                        return false;
                    }
                }
                return true;
            })
            .map(|(_, cliente)| cliente)
            .collect();

        while let Some(msg) = mensagens.pop_front() {
            chat_window.receive_message(msg.clone());
            for conexao in &mut conexoes_enviar {
                if msg.autor.id != conexao.1 {
                    let mut msg_json = serde_json::to_string(&msg).unwrap();
                    let msg_bytes = msg_json.as_bytes();
    
                    conexao
                        .0
                        .write_all(&[msg_bytes.len() as u8])
                        .expect("Não foi possível enviar a mensagem aos clientes");
                }
            }
        }
    }
}
