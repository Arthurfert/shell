use std::io::{self, Write, stdin};
use std::process::Command;

fn main() {
    loop {
        // Afficher le prompt
        print!("> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        stdin().read_line(&mut input).unwrap();

        let mut parts = input.trim().split_whitespace();
        let command = parts.next().unwrap_or("");
        let args: Vec<&str> = parts.collect();

        // Quitter si l'utilisateur tape "exit"
        if command == "exit" {
            break;
        }

        // Ignorer les lignes vides
        if command.is_empty() {
            continue;
        }

        // Exécuter la commande avec ses arguments
        let output = Command::new(command)
            .args(&args)
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