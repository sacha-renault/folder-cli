
use std::path::PathBuf;
use std::fs;
use std::cmp::Ordering;

use derive_builder::Builder;
use regex::Regex;

type FsResult<T> = Result<T, FsError>;

// Custom error type to avoid using std::io::Error
#[derive(Debug)]
pub enum FsError {
    IoError,
    Filtered,
    EmptyFolder,
}

#[derive(Debug, PartialEq)]
pub enum Item {
    File(String),
    Folder(String, Vec<Item>, Option<bool>)
}

#[derive(Builder)]
#[builder(build_fn(validate = "Self::validate"))]
pub struct FolderStructureOptions {
    #[builder(default = "Vec::new()")]
    exclude_extension: Vec<String>,

    #[builder(default = "Vec::new()")]
    exclude_by_filter: Vec<Regex>,

    #[builder(default = "Vec::new()")]
    include_extension_only: Vec<String>,

    #[builder(default = "false")]
    show_empty_folder: bool,
}

impl FolderStructureOptionsBuilder {
    fn validate(&self) -> Result<(), String> {
        if !self.exclude_extension.as_ref().unwrap_or(&vec![]).is_empty() 
            && !self.include_extension_only.as_ref().unwrap_or(&vec![]).is_empty() {
            return Err("Cannot specify both exclude_extension and include_extension_only".to_string());
        }
        Ok(())
    }
}

fn should_include_file(file_name: &str, options: &FolderStructureOptions) -> bool {
    // If both vectors are empty, include all files
    if options.exclude_extension.is_empty() && options.include_extension_only.is_empty() {
        return true;
    }

    // If exclude_extension is not empty, exclude files with matching extensions
    if !options.exclude_extension.is_empty() {
        return !options.exclude_extension.iter()
            .any(|ext| file_name.ends_with(ext));
    }

    // If include_extension_only is not empty, only include files with matching extensions
    if !options.include_extension_only.is_empty() {
        return options.include_extension_only.iter()
            .any(|ext| file_name.ends_with(ext));
    }

    true
}

fn should_include_item(item_name: &str, options: &FolderStructureOptions) -> bool {
    // If both vectors are empty, include all files
    if options.exclude_by_filter.is_empty() {
        return true;
    }

    // Check if any regex pattern matches
    !options.exclude_by_filter
        .iter()
        .any(|re| re.is_match(item_name))
}

impl From<std::io::Error> for FsError {
    fn from(_: std::io::Error) -> Self {
        FsError::IoError
    }
}

pub fn get_folder_structure(path: &PathBuf, options: &FolderStructureOptions) -> FsResult<Item> {
    let name = get_path_name(path);

    if path.is_file() {
        return handle_file(name, options);
    }

    let items = process_directory(path, options)?;
    let mut folder = create_folder_item(path, name, items, options)?;
    update_has_terminal_file(&mut folder);
    Ok(folder)
}

fn update_has_terminal_file(item: &mut Item) -> bool {
    match item {
        Item::File(_) => true,
        Item::Folder(_, items, has_terminal) => {
            let contains_terminal = items.iter_mut().any(|item| update_has_terminal_file(item));
            *has_terminal = Some(contains_terminal);
            contains_terminal
        }
    }
}

fn get_path_name(path: &PathBuf) -> String {
    path.file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned()
}

fn handle_file(name: String, options: &FolderStructureOptions) -> FsResult<Item> {
    if should_include_file(&name, options) {
        Ok(Item::File(name))
    } else {
        Err(FsError::Filtered)
    }
}

fn process_directory(path: &PathBuf, options: &FolderStructureOptions) -> FsResult<Vec<Item>> {
    let mut items = Vec::new();
    
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        
        if should_skip_entry(&path, options) {
            continue;
        }

        match get_folder_structure(&path, options) {
            Ok(item) => items.push(item),
            Err(FsError::Filtered) | Err(FsError::EmptyFolder) => continue,
            Err(e) => return Err(e),
        }
    }

    items.sort_by(sort_items);
    Ok(items)
}

fn should_skip_entry(path: &PathBuf, options: &FolderStructureOptions) -> bool {
    let file_name = path.file_name()
        .and_then(|n| n.to_str());
    
    match file_name {
        Some(name) => {
            name.starts_with('.') || !should_include_item(name, options)
        }
        None => true
    }
}

fn sort_items(a: &Item, b: &Item) -> Ordering {
    match (a, b) {
        (Item::Folder(name1, ..), Item::Folder(name2, ..)) => name1.cmp(name2),
        (Item::Folder(..), Item::File(..)) => Ordering::Less,
        (Item::File(..), Item::Folder(..)) => Ordering::Greater,
        (Item::File(name1), Item::File(name2)) => name1.cmp(name2),
    }
}

fn create_folder_item(path: &PathBuf, name: String, items: Vec<Item>, options: &FolderStructureOptions) -> FsResult<Item> {
    if items.is_empty() && !options.show_empty_folder {
        return Err(FsError::EmptyFolder);
    }

    let folder_name = if path.as_os_str() == "." { 
        ".".to_string() 
    } else { 
        name 
    };

    Ok(Item::Folder(folder_name, items, None))
}

// Helper function to print the structure
fn print_structure(item: &Item, prefix: &str, is_last: bool, option: &FolderStructureOptions) {
    let marker = if is_last { "└── " } else { "├── " };
    let next_prefix = if is_last { "    " } else { "│   " };

    match item {
        Item::File(name) => {
            println!("{}{}{}", prefix, marker, name);
        }
        Item::Folder(name, items, has_terminal_file) => {
            // Skip empty folders if show_empty_folder is false
            if !option.show_empty_folder && !has_terminal_file.unwrap_or(false) {
                return;
            }

            // Print the current folder with proper prefix
            if prefix.is_empty() {
                // Root folder case
                println!("{}", name);
            } else {
                println!("{}{}{}/", prefix, marker, name);
            }
            
            // Set up the prefix for children
            let new_prefix = if prefix.is_empty() {
                // For root's children
                String::from("    ")
            } else {
                // For nested children
                format!("{}{}", prefix, next_prefix)
            };
            
            // Print all children
            for (i, item) in items.iter().enumerate() {
                print_structure(item, &new_prefix, i == items.len() - 1, option);
            }
        }
    }
}

// Helper function to start the printing from root
pub fn print_tree(root: &Item, option: &FolderStructureOptions) {
    print_structure(root, "", true, option);
}