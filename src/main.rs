mod folder_utility;

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::str::FromStr;
use regex::Regex;

use folder_utility::folder_strucure::{print_tree, get_folder_structure, FolderStructureOptionsBuilder};

#[derive(Parser)]
#[command(name = "fs-tools")]
#[command(about = "File system utility tools", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Display directory structure as a tree
    Tree {
        /// Directory path to start from
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Show empty folders
        #[arg(long, short)]
        show_empty: bool,

        /// File extensions to include (comma-separated)
        #[arg(long, value_delimiter = ',')]
        include: Option<Vec<String>>,

        /// File extensions to exclude (comma-separated)
        #[arg(long, value_delimiter = ',')]
        exclude: Option<Vec<String>>,

        /// Regex patterns to exclude (comma-separated)
        #[arg(long, value_delimiter = ',')]
        exclude_pattern: Option<Vec<String>>,
    },
}

fn main() {
    let cli_args = Cli::parse();
    match cli_args.command {
        Commands::Tree { 
            path, 
            show_empty, 
            include, 
            exclude, 
            exclude_pattern 
        } => {
            let mut options_builder = FolderStructureOptionsBuilder::default();
            options_builder.show_empty_folder(show_empty);

            if let Some(include_ext) = include {
                options_builder.include_extension_only(
                    include_ext.iter()
                        .map(|s| s.trim_start_matches('.').to_string())
                        .collect()
                );
            }

            if let Some(exclude_ext) = exclude {
                options_builder.exclude_extension(
                    exclude_ext.iter()
                        .map(|s| s.trim_start_matches('.').to_string())
                        .collect()
                );
            }

            if let Some(patterns) = exclude_pattern {
                let regexes = patterns.iter()
                    .filter_map(|p| match Regex::new(p) {
                        Ok(re) => Some(re),
                        Err(e) => {
                            eprintln!("Invalid regex pattern '{}': {}", p, e);
                            None
                        }
                    })
                    .collect::<Vec<_>>();
                
                options_builder.exclude_by_filter(regexes);
            }

            let options = match options_builder.build() {
                Ok(opt) => opt,
                Err(e) => {
                    eprintln!("Error building options: {}", e);
                    return;
                }
            };

            match get_folder_structure(&path, &options) {
                Ok(root) => print_tree(&root, &options),
                Err(e) => eprintln!("Error creating folder tree: {:?}", e),
            }
        },
        _ => eprint!("Unknown cmd")
    }
}
