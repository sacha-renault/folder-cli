//! Tree-like Directory Visualization
//! 
//! This module provides functionality to generate and display directory structures
//! in a tree-like format, similar to the Unix `tree` command.
//! 
//! # Features
//! - Filter files by extension (include or exclude)
//! - Filter items using regex patterns
//! - Control visibility of empty folders
//! - Sort items (folders before files, alphabetically within types)

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

/// Represents an item in the file system, either a file or a folder
#[derive(Debug, PartialEq)]
pub enum Item {
    /// A file with its name
    File(String),

    /// A folder with its name, contained items, and a flag indicating if it contains any files
    /// The bool flag indicates whether this folder contains any terminal files (directly or indirectly)
    Folder(String, Vec<Item>, Option<bool>)
}

/// Possible errors that can occur during folder structure processing
impl From<std::io::Error> for FsError {
    fn from(_: std::io::Error) -> Self {
        FsError::IoError
    }
}

/// Expected configuration structure for folder traversal options
///
/// # Fields
/// * `show_empty_folder` - Whether to include empty folders in the output
/// * `exclude_extension` - List of file extensions to exclude
/// * `include_extension_only` - List of file extensions to exclusively include
/// * `exclude_by_filter` - List of regex patterns for excluding items
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

/// Validates the configuration options for folder structure.
///
/// # Errors
///
/// Returns an error if both `exclude_extension` and `include_extension_only` are non-empty,
/// as these options are mutually exclusive.
///
/// Returns `Ok(())` if the validation passes.
impl FolderStructureOptionsBuilder {
    fn validate(&self) -> Result<(), String> {
        if !self.exclude_extension.as_ref().unwrap_or(&vec![]).is_empty() 
            && !self.include_extension_only.as_ref().unwrap_or(&vec![]).is_empty() {
            return Err("Cannot specify both exclude_extension and include_extension_only".to_string());
        }
        Ok(())
    }
}

/// Gets the complete folder structure starting from the given path
///
/// # Arguments
/// * `path` - The starting path to generate the structure from
/// * `options` - Configuration options for filtering and display
///
/// # Returns
/// * `FsResult<Item>` - The resulting folder structure or an error
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

/// Prints the complete folder structure as a tree
///
/// # Arguments
/// * `root` - The root item of the structure
/// * `option` - Configuration options for display
pub fn print_tree(root: &Item, option: &FolderStructureOptions) {
    print_structure(root, "", true, option);
}

/// Determines if a file should be included based on extension filters
///
/// # Arguments
/// * `file_name` - Name of the file to check
/// * `options` - Filter options containing include/exclude patterns
///
/// # Returns
/// * `bool` - True if the file should be included
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

/// Determines if an item should be included based on name filters
///
/// # Arguments
/// * `item_name` - Name of the item to check
/// * `options` - Filter options containing regex patterns
///
/// # Returns
/// * `bool` - True if the item should be included
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

/// Updates the has_terminal_file flag for all folders in the structure
///
/// # Arguments
/// * `item` - The item to update
///
/// # Returns
/// * `bool` - True if this item or any of its children contain a terminal file
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

/// Extracts the name from a path
///
/// # Arguments
/// * `path` - The path to extract the name from
///
/// # Returns
/// * `String` - The extracted name
fn get_path_name(path: &PathBuf) -> String {
    path.file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned()
}

/// Processes a file item
///
/// # Arguments
/// * `name` - Name of the file
/// * `options` - Configuration options for filtering
///
/// # Returns
/// * `FsResult<Item>` - The file item or a filtered error
fn handle_file(name: String, options: &FolderStructureOptions) -> FsResult<Item> {
    if should_include_file(&name, options) {
        Ok(Item::File(name))
    } else {
        Err(FsError::Filtered)
    }
}

/// Processes a directory and its contents
///
/// # Arguments
/// * `path` - Path to the directory
/// * `options` - Configuration options for filtering
///
/// # Returns
/// * `FsResult<Vec<Item>>` - Vector of processed items or an error
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

/// Determines if an entry should be skipped during processing
///
/// # Arguments
/// * `path` - Path to the entry
/// * `options` - Configuration options for filtering
///
/// # Returns
/// * `bool` - True if the entry should be skipped
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

/// Comparison function for sorting items
///
/// # Arguments
/// * `a` - First item to compare
/// * `b` - Second item to compare
///
/// # Returns
/// * `Ordering` - The ordering relationship between the items
fn sort_items(a: &Item, b: &Item) -> Ordering {
    match (a, b) {
        (Item::Folder(name1, ..), Item::Folder(name2, ..)) => name1.cmp(name2),
        (Item::Folder(..), Item::File(..)) => Ordering::Less,
        (Item::File(..), Item::Folder(..)) => Ordering::Greater,
        (Item::File(name1), Item::File(name2)) => name1.cmp(name2),
    }
}

/// Creates a folder item with the given contents
///
/// # Arguments
/// * `path` - Path to the folder
/// * `name` - Name of the folder
/// * `items` - Contents of the folder
/// * `options` - Configuration options
///
/// # Returns
/// * `FsResult<Item>` - The folder item or an error
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

/// Prints a single item in the structure with proper formatting
///
/// # Arguments
/// * `item` - The item to print
/// * `prefix` - Current line prefix for proper tree formatting
/// * `is_last` - Whether this is the last item in its level
/// * `option` - Configuration options for display
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