use std::env;
use std::io::{self, Write, stdin};
use std::path::Path;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use chrono::{DateTime, Local};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

/// Retourne le répertoire home de l'utilisateur
fn get_home_dir() -> String {
    if cfg!(target_os = "windows") {
        env::var("USERPROFILE").unwrap_or_else(|_| ".".to_string())
    } else {
        env::var("HOME").unwrap_or_else(|_| ".".to_string())
    }
}

/// Retourne le nom de l'utilisateur
fn get_username() -> String {
    if cfg!(target_os = "windows") {
        env::var("USERNAME").unwrap_or_else(|_| "user".to_string())
    } else {
        env::var("USER").unwrap_or_else(|_| "user".to_string())
    }
}

/// Vérifie si un fichier est exécutable
#[cfg(windows)]
fn is_executable(name: &str, _metadata: &std::fs::Metadata) -> bool {
    let name_lower = name.to_lowercase();
    name_lower.ends_with(".exe") 
        || name_lower.ends_with(".bat") 
        || name_lower.ends_with(".cmd") 
        || name_lower.ends_with(".ps1")
        || name_lower.ends_with(".com")
}

#[cfg(unix)]
fn is_executable(_name: &str, metadata: &std::fs::Metadata) -> bool {
    // Sur Unix, vérifier le bit d'exécution
    metadata.permissions().mode() & 0o111 != 0
}

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
                    get_home_dir(),
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
                // Parser les options et le chemin
                let mut show_details = false;
                let mut show_hidden = false;
                let mut path = ".";

                for arg in &args {
                    if arg.starts_with('-') {
                        if arg.contains('l') { show_details = true; }
                        if arg.contains('a') { show_hidden = true; }
                    } else {
                        path = arg;
                    }
                }

                match std::fs::read_dir(path) {
                    Ok(entries) => {
                        // Collecter les entrées pour calculer le total et aligner les colonnes
                        let mut file_entries: Vec<_> = entries.filter_map(|e| e.ok()).collect();
                        
                        // Trier par nom
                        file_entries.sort_by(|a, b| {
                            a.file_name().to_string_lossy().to_lowercase()
                                .cmp(&b.file_name().to_string_lossy().to_lowercase())
                        });

                        // Afficher l'en-tête si mode détaillé
                        if show_details {
                            // Calculer le total (en blocs de 512 octets, comme Linux)
                            let total: u64 = file_entries.iter()
                                .filter_map(|e| e.metadata().ok())
                                .map(|m| (m.len() + 511) / 512)
                                .sum();
                            println!("total {}", total);
                            println!("\x1B[4mPermissions  Lnk Owner        Size Modified     Name\x1B[0m");
                        }

                        for entry in &file_entries {
                            let file_name = entry.file_name();
                            let name = file_name.to_string_lossy();

                            // Ignorer les fichiers cachés (commençant par .) sauf si -a
                            if !show_hidden && name.starts_with('.') {
                                continue;
                            }

                            if show_details {
                                // Affichage détaillé style Linux
                                if let Ok(metadata) = entry.metadata() {
                                    // Permissions style Linux
                                    let is_dir = metadata.is_dir();
                                    let is_readonly = metadata.permissions().readonly();
                                    
                                    // Vérifier si le fichier est exécutable
                                    let is_exec = is_executable(&name, &metadata);
                                    
                                    let perms = if is_dir {
                                        if is_readonly { "dr-xr-xr-x" } else { "drwxr-xr-x" }
                                    } else if is_exec {
                                        if is_readonly { "-r-xr-xr-x" } else { "-rwxr-xr-x" }
                                    } else {
                                        if is_readonly { "-r--r--r--" } else { "-rw-r--r--" }
                                    };

                                    // Nombre de liens (simulé: 1 pour fichiers, 2+ pour dossiers)
                                    let links = if is_dir { 2 } else { 1 };

                                    // Propriétaire
                                    let owner = get_username();

                                    let size = metadata.len();
                                    
                                    // Date de modification
                                    let modified = if let Ok(time) = metadata.modified() {
                                        let datetime: DateTime<Local> = time.into();
                                        datetime.format("%b %e %H:%M").to_string()
                                    } else {
                                        "?".to_string()
                                    };

                                    // Couleurs ANSI
                                    let (color_start, color_end) = if is_dir {
                                        ("\x1B[1;34m", "\x1B[0m")  // Bleu gras pour dossiers
                                    } else if is_exec {
                                        ("\x1B[1;32m", "\x1B[0m")  // Vert gras pour exécutables
                                    } else {
                                        let name_lower = name.to_lowercase();
                                        if name_lower.ends_with(".zip") || name_lower.ends_with(".tar") || name_lower.ends_with(".gz") || name_lower.ends_with(".7z") || name_lower.ends_with(".rar") {
                                            ("\x1B[1;31m", "\x1B[0m")  // Rouge gras pour archives
                                        } else {
                                            ("", "")  // Pas de couleur
                                        }
                                    };

                                    println!("{} {:>2} {:<8} {:>8} {} {}{}{}",
                                        perms, links, owner, size, modified,
                                        color_start, name, color_end);
                                }
                            } else {
                                // Affichage simple avec couleurs
                                if let Ok(metadata) = entry.metadata() {
                                    if metadata.is_dir() {
                                        println!("\x1B[1;34m{}\x1B[0m", name);  // Bleu pour dossiers
                                    } else if is_executable(&name, &metadata) {
                                        println!("\x1B[1;32m{}\x1B[0m", name);  // Vert pour exécutables
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