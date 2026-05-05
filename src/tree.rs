use std::fs;
use std::path::Path;

const BOLD_BLUE: &str = "\x1b[1;34m";
const RESET: &str = "\x1b[0m";

pub fn print_tree(root: &Path, subfolder: Option<&str>) {
    let dir = if let Some(sub) = subfolder {
        root.join(sub)
    } else {
        root.to_path_buf()
    };

    if !dir.exists() {
        eprintln!("Error: {} is not in the password store.", dir.display());
        return;
    }

    let header = subfolder.unwrap_or("Password Store");
    println!("{}{}{}", BOLD_BLUE, header, RESET);
    print_subtree(&dir, "");
}

fn print_subtree(dir: &Path, prefix: &str) {
    let mut entries = match fs::read_dir(dir) {
        Ok(rd) => rd
            .filter_map(|e| e.ok())
            .filter(|e| !e.file_name().to_string_lossy().starts_with('.'))
            .collect::<Vec<_>>(),
        Err(_) => return,
    };

    entries.sort_by_key(|e| e.file_name());

    let count = entries.len();
    for (i, entry) in entries.iter().enumerate() {
        let is_last = i == count - 1;
        let connector = if is_last { "└── " } else { "├── " };
        let cont = if is_last { "    " } else { "│   " };
        let path = entry.path();
        let raw_name = entry.file_name();
        let name = raw_name.to_string_lossy();

        if path.is_dir() {
            println!("{}{}{}{}{}", prefix, connector, BOLD_BLUE, name, RESET);
            print_subtree(&path, &format!("{}{}", prefix, cont));
        } else if let Some(stem) = name.strip_suffix(".gpg") {
            println!("{}{}{}", prefix, connector, stem);
        }
    }
}
