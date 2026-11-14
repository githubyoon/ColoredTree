use colored::Colorize;
use std::{env, path::{Path, PathBuf}};
use std::process::exit;
use walkdir::WalkDir;
use std::io::{Write, stdout};

struct Config { root: PathBuf, depth: usize, hidden: bool, system: bool, symlink: bool}

fn get_args() -> Config {
    let args: Vec<String> = env::args().collect();
    let mut cfg = Config { root: PathBuf::from("."), depth: 1, hidden: false, system: false, symlink: false};
    let mut i = 1;

    if args.len() > 1 {
        if matches!(args[1].as_str(), "-h" | "-help") {
            print_help();
        }
    }


    while i < args.len() {
        match args[i].as_str() {
            "-H" | "-Hidden" => {
                cfg.hidden = true
            } "-d" | "-depth" => {
                i += 1;
                cfg.depth = args.get(i).and_then(|s| s.parse().ok()).unwrap_or(1);
            } "-s" | "-system" => {
                cfg.system = true
            } "-sym" | "-symlnk" => {
                cfg.symlink = true
            }
            other if other.starts_with('-') => print_help(),
            other => cfg.root = PathBuf::from(other)
        }
        i += 1;
    }
    cfg
}

fn print_help() {
    println!("Add Color in tree.\n
Usage: ColoredTree [PATH] [OPTIONS]\n
Options:\n\
\t-d, -depth [depth]\t Maximum depth of directories\n\
\t-h, -help\t\t Show this help message\n\
\t-H, -Hidden\t\t Show hidden directory\n\
\t-s, -system\t\t Show system directory\n\
\t-sym, -symlink\t\t Show symbolic links\n\
");
    exit(0);
}

fn is_hidden(entry: &walkdir::DirEntry) -> bool {
    let name = entry.file_name().to_str().unwrap_or("");

    if name.starts_with('.') {
        return true;
    }
           use std::os::windows::fs::MetadataExt;
        if let Ok(meta) = entry.metadata() {
            if (meta.file_attributes() & 0x2) != 0 {
                return true;
            }
        }
    false
}

fn is_system_path(path: &Path) -> bool {
    let path_str = path.to_string_lossy().to_lowercase();

        if path_str.contains(r"c:\windows")
            || path_str.contains(r"c:\programdata")
            || path_str.contains(r"$recycle.bin")
            || path_str.contains(r"system volume information") {
            return true;
    }
    false
}

fn is_system(entry: &walkdir::DirEntry) -> bool {
    let name = entry.path();

    if is_system_path(name) {
        return true;
    }

        use std::os::windows::fs::MetadataExt;

        if let Ok(meta) = entry.metadata() {
            let attr = meta.file_attributes();
            if (attr & 0x4) != 0 {
                return true;
            }
        }

    false
}


fn print_tree(root: &Path, cfg: &Config) -> walkdir::Result<()> {
    let mut entries = Vec::new();
    let mut it = WalkDir::new(root)
        .min_depth(1)
        .max_depth(cfg.depth)
        .follow_links(cfg.symlink)
        .into_iter();

    while let Some(entry_res) = it.next() {
        match entry_res {
            Ok(entry) => {
                let is_h = !cfg.hidden && entry.depth() > 0 && is_hidden(&entry);
                let is_s = !cfg.system && entry.depth() > 0 && is_system(&entry);

                if is_h || is_s {
                    if entry.file_type().is_dir() {
                        it.skip_current_dir();
                    }
                    continue;
                }
                entries.push(entry);
            }
            Err(_) => continue,
        }
    }

    let mut indexed_entries: Vec<_> = entries.into_iter().map(|e| {
        let parent_str = e.path().parent().unwrap_or(Path::new("")).to_string_lossy().to_lowercase();
        let file_str = e.file_name().to_string_lossy().to_lowercase();
        let sort_key = format!("{parent_str}\0{file_str}");
        (sort_key, e)
    }).collect();

    indexed_entries.sort_by(|a, b| a.0.cmp(&b.0));
    let final_entries: Vec<_> = indexed_entries.into_iter().map(|(_, e)| e).collect();

    let mut is_last_stack = vec![false; cfg.depth + 2];

    for (index, entry) in final_entries.iter().enumerate() {
        let depth = entry.depth();
        if depth == 0 { continue; }


        let is_last = if index + 1 < final_entries.len() {
            let next_entry = &final_entries[index + 1];
            next_entry.path().parent() != entry.path().parent()
        } else {
            true
        };

        is_last_stack[depth] = is_last;


        let mut prefix = String::new();
        for level in 1..depth {
            if is_last_stack[level] {
                prefix.push_str("    ");
            } else {
                prefix.push_str("│   ");
            }
        }
        let prefix = prefix.bright_black();


        let connector = if is_last {
            "└── ".bright_black()
        } else {
            "├── ".bright_black()
        };

        let name = entry.file_name().to_string_lossy();

        let is_sym = entry.path().symlink_metadata()
            .map(|m| m.file_type().is_symlink())
            .unwrap_or(false);

        if is_sym {
            if cfg.symlink {
                println!("{prefix}{connector}{}", name.bold().magenta());
            } continue;
        } else if entry.file_type().is_dir() {
            println!("{prefix}{connector}{}", name.bold().blue());
        } else {
            println!("{prefix}{connector}{}", name);
        }
    }

    Ok(())
}

fn main() {
    let cfg = get_args();
    let root = &cfg.root;

    let label = root.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| root.display().to_string());

    println!("{}", label.bold().cyan());
    let _ = stdout().flush();

    if let Err(error) = print_tree(root, &cfg) {
        eprintln!("Invalid Path: {error}");
        exit(1);
    }
}
