use std::env;
use std::io::{self, Write, stdin};
use std::path::Path;
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

        // Ignorer les lignes vides
        if command.is_empty() {
            continue;
        }
        
        match command {
            "cd" => {
                // Par défaut aller au répertoire home si aucun argument
                let new_dir = args.first().map_or(
                    env::var("USERPROFILE").unwrap_or_else(|_| ".".to_string()),
                    |x| x.to_string()
                );
                let root = Path::new(&new_dir);
                if let Err(e) = env::set_current_dir(&root) {
                    eprintln!("{}", e);
                }
            },

            "exit" => return,

            // Exécuter la commande avec ses arguments
            _ => {
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
    }
}