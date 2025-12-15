use std::env;
use std::io::{self, Write, stdin};
use std::path::Path;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

fn main() {
    // Flag pour indiquer si Ctrl+C a été pressé
    let interrupted = Arc::new(AtomicBool::new(false));
    let interrupted_clone = Arc::clone(&interrupted);

    // Configurer le handler Ctrl+C
    ctrlc::set_handler(move || {
        interrupted_clone.store(true, Ordering::SeqCst);
        println!("\nProcessus interrompu.");
    }).expect("Erreur lors de la configuration du handler Ctrl+C");

    loop {
        // Réinitialiser le flag
        interrupted.store(false, Ordering::SeqCst);

        // Afficher le prompt
        print!("> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if stdin().read_line(&mut input).is_err() {
            continue;
        }

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

            "clear" => {
                // \x1B[2J = efface l'écran visible
                // \x1B[3J = efface le buffer de défilement (scrollback)
                // \x1B[H  = remet le curseur en haut à gauche
                print!("\x1B[2J\x1B[3J\x1B[H");
                io::stdout().flush().unwrap();
            },

            "ls" => {
                // Lister les fichiers du répertoire courant
                let path = args.first().map_or(".", |x| *x);
                match std::fs::read_dir(path) {
                    Ok(entries) => {
                        for entry in entries {
                            if let Ok(entry) = entry {
                                let file_name = entry.file_name();
                                let name = file_name.to_string_lossy();
                                
                                // Ajouter / à la fin si c'est un dossier
                                if let Ok(file_type) = entry.file_type() {
                                    if file_type.is_dir() {
                                        println!("{}/", name);
                                    } else {
                                        println!("{}", name);
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Erreur: {}", e);
                    }
                }
            },

            // Exécuter la commande avec ses arguments
            _ => {
                match Command::new(command).args(&args).spawn() {
                    Ok(mut child) => {
                        // Attendre que le processus se termine
                        // Ctrl+C automatiquement envoyé au processus enfant
                        let _ = child.wait();
                    }
                    Err(e) => {
                        eprintln!("Erreur: {}", e);
                    }
                }
            }
        }
    }
}