mod client;
mod server;

use comunicacao::utilities::input;

fn main() {
    let mut modo = Modo::from_string(input("(C)liente ou (S)ervidor? Padrão: Cliente"));
    while let Err(e) =  modo {
        println!("Erro: {e}");
        modo = Modo::from_string(input("(C)liente ou (S)ervidor? Padrão: Cliente"));
    };

    let modo = modo.unwrap();

    match modo {
        Modo::Cliente => client::run(),
        Modo::Servidor => server::run(),
    }

}


enum Modo {
    Cliente,
    Servidor
}

impl Modo {
    fn from_string(string: String) -> Result<Self, &'static str> {
        if string.to_ascii_lowercase().starts_with("s") {
            Ok(Self::Servidor)
        }
        else if string.to_ascii_lowercase().starts_with("c") || string.is_empty() {
            Ok(Self::Cliente)
        } else {
            Err("Modo inválido")
        }

    }
}