mod folder_utility;

use std::{path::PathBuf, str::FromStr};

use folder_utility::folder_strucure::{print_tree, get_folder_structure, FolderStructureOptionsBuilder};

fn main() {
    let path = match PathBuf::from_str("./") {
        Ok(p) => p,
        Err(_) => panic!("Couldn't parse path as path buffer"),
    };
    let options = FolderStructureOptionsBuilder::default()
        .show_empty_folder(false)
        // .exclude_by_filter(vec![Regex::new("target").unwrap()])
        .include_extension_only(vec!["rs".to_string(), "toml".to_string()])
        // .exclude_extension(vec![
        //     "o".to_string(), "exe".to_string(), "bin".to_string(), "lock".to_string(), "d".to_string()
        //     ])
        .build()
        .unwrap();
    let root = match get_folder_structure(&path, &options) {
        Ok(root) => root, 
        Err(_) => panic!("Couldn't create the folder tree")
    };
    print_tree(&root, &options);
}
