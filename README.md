# Shell

Un shell minimaliste écrit en Rust, créé à des fins d'apprentissage.

## Fonctionnalités

- **Boucle interactive** avec prompt `>`
- **Exécution de commandes** avec arguments
- **Commandes intégrées (built-in)** :
  - `cd [chemin]` — Change de répertoire (répertoire home par défaut)
  - `exit` — Quitte le shell

## Compilation

```bash
cargo build
```

## Utilisation

```bash
cargo run
```

## Limitations actuelles

- Sur windows : les commandes intégrées à `cmd.exe` (`dir`, `echo`, `type`, etc.) ne fonctionnent pas directement
- Pas de support pour les pipes (`|`) ou redirections (`>`, `<`)
- Pas d'historique de commandes
- Pas d'autocomplétion

## Licence

Voir [LICENSE](LICENSE)
