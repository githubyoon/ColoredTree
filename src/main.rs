use colored::Colorize;
use std::{env,path::{Path ,PathBuf}};
use std::process::exit;
use walkdir::WalkDir;

fn main() {
    let root = env::args_os()
        .nth(2)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));

    let label = root.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| root.display().to_string());

    println!("{}", label.bold().cyan());

    if let Err(error) = print_tree(&root) {
        eprintln!("Invalid Path: {error}");
        exit(1);
    }
}

fn print_tree(root: &Path) -> walkdir::Result<()> {
    let entries: Vec<_> = WalkDir::new(root)
        .min_depth(1)
        .sort_by(|a, b| {
            a.file_name()
                .to_string_lossy()
                .to_lowercase()
                .cmp(&b.file_name().to_string_lossy().to_lowercase())
        })
        .into_iter()
        .collect::<Result<Vec<_>, _>>()?;

    for (index, entry) in entries.iter().enumerate() {
        let depth = entry.depth();

        let is_last = !entries[index + 1..]
            .iter()
            .any(|next| next.depth() >= depth);

        let prefix = (1..depth)
            .map(|level| {
                if entries[index + 1..]
                    .iter()
                    .any(|next| next.depth() == level)
                {
                    "│   "
                } else {
                    "    "
                }
            })
            .collect::<String>()
            .bright_black();

        let connector = if is_last {
            "└── ".bright_black()
        } else {
            "├── ".bright_black()
        };

        let name = entry.file_name().to_string_lossy();

        if entry.file_type().is_dir() {
            println!("{prefix}{connector}{}", name.bold().blue());
        } else {
            println!("{prefix}{connector}{}", name.ansi_color(15));
        }
    }

    Ok(())
}