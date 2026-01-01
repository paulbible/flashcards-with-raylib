use std::fs;
use std::path::{Path, PathBuf};

pub struct DeckManager {
    deck_files: Vec<String>,      // List of all deck filenames
    current_deck_index: usize,    // Which deck is currently active
    deck_folder: String,          // Path to the folder
}

impl DeckManager {
    /// Creates a new DeckManager by reading all CSV files from the specified folder
    pub fn new(folder: &str) -> Result<Self, std::io::Error> {
        let path = Path::new(folder);
        
        // Check if folder exists
        if !path.exists() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Folder '{}' does not exist", folder)
            ));
        }

        if !path.is_dir() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("'{}' is not a directory", folder)
            ));
        }

        // Read all .csv files from the directory
        let mut deck_files = Vec::new();
        
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let file_path = entry.path();
            
            // Only include .csv files
            if file_path.is_file() {
                if let Some(extension) = file_path.extension() {
                    if extension == "csv" {
                        if let Some(filename) = file_path.file_name() {
                            if let Some(filename_str) = filename.to_str() {
                                deck_files.push(filename_str.to_string());
                            }
                        }
                    }
                }
            }
        }

        // Sort alphabetically for consistent ordering
        deck_files.sort();

        // Check if any decks were found
        if deck_files.is_empty() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("No CSV files found in '{}'", folder)
            ));
        }

        Ok(DeckManager {
            deck_files,
            current_deck_index: 0,
            deck_folder: folder.to_string(),
        })
    }

    /// Returns the full path to the current deck file
    pub fn get_current_deck_path(&self) -> String {
        let filename = &self.deck_files[self.current_deck_index];
        format!("{}/{}", self.deck_folder, filename)
    }

    /// Cycles to the next deck (wraps around to the beginning)
    pub fn next_deck(&mut self) {
        if !self.deck_files.is_empty() {
            self.current_deck_index = (self.current_deck_index + 1) % self.deck_files.len();
        }
    }

    /// Cycles to the previous deck (wraps around to the end)
    pub fn prev_deck(&mut self) {
        if !self.deck_files.is_empty() {
            if self.current_deck_index == 0 {
                self.current_deck_index = self.deck_files.len() - 1;
            } else {
                self.current_deck_index -= 1;
            }
        }
    }

    /// Returns the current deck name without path or extension
    pub fn get_current_deck_name(&self) -> &str {
        let filename = &self.deck_files[self.current_deck_index];
        
        // Remove .csv extension if present
        if let Some(name_without_ext) = filename.strip_suffix(".csv") {
            name_without_ext
        } else {
            filename
        }
    }

    /// Returns a formatted string showing current deck position (e.g., "Deck 2/5")
    pub fn get_deck_counter(&self) -> String {
        format!("Deck {} / {}", self.current_deck_index + 1, self.deck_files.len())
    }

    /// Returns the total number of decks available
    pub fn total_decks(&self) -> usize {
        self.deck_files.len()
    }

    /// Returns true if there is more than one deck available
    pub fn has_multiple_decks(&self) -> bool {
        self.deck_files.len() > 1
    }

    /// Formats the deck name to be more human-readable
    /// Converts underscores to spaces and capitalizes words
    pub fn get_formatted_deck_name(&self) -> String {
        let name = self.get_current_deck_name();
        
        // Replace underscores with spaces
        let with_spaces = name.replace('_', " ");
        
        // Capitalize first letter of each word
        with_spaces
            .split_whitespace()
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
}
