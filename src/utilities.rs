use std::io::{prelude::*, stdin, stdout};

static mut MAX_ID: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

pub fn input(mensagem: &str) -> String {
    let mut meu_nome = String::new();
    print!("{mensagem}");

    let _ = stdout().flush();
    stdin()
        .read_line(&mut meu_nome)
        .expect("Não foi possível ler seu texto :(");

    meu_nome
}

#[derive(Debug)]
pub struct Message {
    pub autor: Pessoa,
    pub tipo: TipoMensagem,
}

#[derive(Debug)]
pub enum TipoMensagem {
    Entrada,
    Saida,
    Chat(String),
}

impl Message {
    pub fn new(autor: Pessoa, tipo: TipoMensagem) -> Self {
        Self { autor, tipo }
    }
}


impl std::fmt::Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let texto = match &self.tipo {
            TipoMensagem::Entrada => format!("{} entrou no chat!...", self.autor.nome),
            TipoMensagem::Saida => format!("{} saiu do chat...", self.autor.nome),
            TipoMensagem::Chat(texto) => format!("{}: {texto}", self.autor.nome),
        };
        write!(f, "{texto}")
    }
}

#[derive(Clone, Debug)]
pub struct Pessoa {
    pub nome: String,
    id: u32, //cor: (u8, u8, u8)
}

impl Pessoa {
    pub fn new(nome: String) -> Self {
        let id = unsafe { MAX_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed) };
        Self { nome, id }
    }

    pub fn id(&self) -> u32 {
        self.id
    }
}