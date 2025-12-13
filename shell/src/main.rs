use std::env;
use std::io::{self, Write, stdin};
use std::path::Path;
use std::process::{Child, Command};
use std::sync::{Arc, Mutex};

fn main() {
    // Stocker le processus enfant en cours pour pouvoir le tuer avec Ctrl+C
    let child: Arc<Mutex<Option<Child>>> = Arc::new(Mutex::new(None));
    let child_clone = Arc::clone(&child);

    // Configurer le handler Ctrl+C
    ctrlc::set_handler(move || {
        let mut child_guard = child_clone.lock().unwrap();
        if let Some(ref mut process) = *child_guard {
            // Tuer le processus enfant en cours
            let _ = process.kill();
            println!("\nProcessus interrompu.");
        }
    }).expect("Erreur lors de la configuration du handler Ctrl+C");

    loop {
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

            // Exécuter la commande avec ses arguments
            _ => {
                match Command::new(command).args(&args).spawn() {
                    Ok(process) => {
                        // Stocker le processus pour que Ctrl+C puisse le tuer
                        {
                            let mut child_guard = child.lock().unwrap();
                            *child_guard = Some(process);
                        }

                        // Attendre que le processus se termine
                        {
                            let mut child_guard = child.lock().unwrap();
                            if let Some(ref mut p) = *child_guard {
                                let _ = p.wait();
                            }
                            *child_guard = None;
                        }
                    }
                    Err(e) => {
                        eprintln!("Erreur: {}", e);
                    }
                }
            }
        }
    }
}