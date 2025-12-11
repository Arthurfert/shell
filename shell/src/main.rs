use std::io::{self, Write, stdin};
use std::process::Command;

fn main() {
    loop {
        // Afficher le prompt
        print!("> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        stdin().read_line(&mut input).unwrap();

        let command = input.trim();

        // Quitter si l'utilisateur tape "exit"
        if command == "exit" {
            break;
        }

        // Ignorer les lignes vides
        if command.is_empty() {
            continue;
        }

        // Exécuter la commande via cmd.exe sur Windows
        let output = Command::new("cmd")
            .args(["/C", command])
            .output();

        match output {
            Ok(result) => {
                print!("{}", String::from_utf8_lossy(&result.stdout));
                eprint!("{}", String::from_utf8_lossy(&result.stderr));
            }
            Err(e) => {
                eprintln!("Erreur: {}", e);
            }
        }
    }
}