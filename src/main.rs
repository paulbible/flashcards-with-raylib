use raylib::prelude::*;
use std::fs::File;
use std::io::{BufRead, BufReader};

use crate::utils::DeckManager;

mod utils;

#[derive(Clone)]
struct Flashcard {
    question: String,
    answer: String,
}

struct FlashcardGame {
    cards: Vec<Flashcard>,
    current_index: usize,
    is_flipped: bool,
}

impl FlashcardGame {
    fn new(cards: Vec<Flashcard>) -> Self {
        FlashcardGame {
            cards,
            current_index: 0,
            is_flipped: false,
        }
    }

    fn next_card(&mut self) {
        if self.current_index < self.cards.len() - 1 {
            self.current_index += 1;
            self.is_flipped = false;
        }
    }

    fn prev_card(&mut self) {
        if self.current_index > 0 {
            self.current_index -= 1;
            self.is_flipped = false;
        }
    }

    fn flip(&mut self) {
        self.is_flipped = !self.is_flipped;
    }

    fn get_current_text(&self) -> &str {
        if let Some(card) = self.cards.get(self.current_index) {
            if self.is_flipped {
                &card.answer
            } else {
                &card.question
            }
        } else {
            ""
        }
    }
}

fn parse_csv_line(line: &str) -> Option<(String, String)> {
    let mut fields = Vec::new();
    let mut current_field = String::new();
    let mut in_quotes = false;
    let mut chars = line.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '"' => {
                if in_quotes && chars.peek() == Some(&'"') {
                    // Escaped quote (two quotes in a row)
                    current_field.push('"');
                    chars.next();
                } else {
                    // Toggle quote mode
                    in_quotes = !in_quotes;
                }
            }
            ',' if !in_quotes => {
                // Field separator
                fields.push(current_field.trim().to_string());
                current_field.clear();
            }
            _ => {
                current_field.push(c);
            }
        }
    }

    // Add the last field
    fields.push(current_field.trim().to_string());

    // Return first two fields as question and answer
    if fields.len() >= 2 {
        Some((fields[0].clone(), fields[1].clone()))
    } else {
        None
    }
}

fn load_flashcards(filename: &str) -> Result<Vec<Flashcard>, std::io::Error> {
    let file = File::open(filename)?;
    let reader = BufReader::new(file);
    let mut cards = Vec::new();

    for line in reader.lines() {
        let line = line?;

        if let Some((question, answer)) = parse_csv_line(&line) {
            if !question.is_empty() && !answer.is_empty() {
                cards.push(Flashcard { question, answer });
            }
        }
    }

    Ok(cards)
}

fn wrap_text(text: &str, max_width: i32, font_size: i32) -> Vec<String> {
    let words: Vec<&str> = text.split_whitespace().collect();
    let mut lines = Vec::new();
    let mut current_line = String::new();
    let approx_char_width = font_size / 2;

    for word in words {
        let test_line = if current_line.is_empty() {
            word.to_string()
        } else {
            format!("{} {}", current_line, word)
        };

        if (test_line.len() as i32 * approx_char_width) > max_width {
            if !current_line.is_empty() {
                lines.push(current_line);
                current_line = word.to_string();
            } else {
                lines.push(word.to_string());
            }
        } else {
            current_line = test_line;
        }
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }

    lines
}


fn draw_text_centered(
    d: &mut RaylibDrawHandle,
    custom_font: &Option<Font>,
    text: &str,
    center_x: i32,
    y: i32,
    font_size: f32,
    color: Color,
) {
    let font_size_i = font_size as i32;
    let y_f = y as f32;
    
    if let Some(font) = custom_font {
        // Use custom font
        let text_dimensions = font.measure_text(text, font_size, 0.0);
        let x = center_x as f32 - text_dimensions.x / 2.0;
        d.draw_text_ex(font, text, Vector2::new(x, y_f), font_size, 1.0, color);
    } else {
        // Use default font
        let text_width = d.measure_text(text, font_size_i);
        let x = center_x - text_width / 2;
        d.draw_text(text, x, y, font_size_i, color);
    }
}

fn try_load_cards(filename: &str) -> Option<Vec<Flashcard>> {
    match load_flashcards(filename) {
        Ok(cards) if !cards.is_empty() => Some(cards),
        Ok(_) => {
            eprintln!("Error: cards.csv is empty or contains no valid flashcards");
            None
        }
        Err(e) => {
            eprintln!("Error loading cards.csv: {}", e);
            None
        }
    }
}

fn update_decks(decks: &DeckManager, game: &mut FlashcardGame) {
    match try_load_cards(&decks.get_current_deck_path()){
        Some(cards) => *game = FlashcardGame::new(cards),
        None => panic!("Failed to load cards")
    }
}


fn main() {
    let mut decks: utils::DeckManager = utils::DeckManager::new("./flashcard_decks").unwrap();
    //let cards = match load_flashcards("cards.csv") {
    let maybe_cards = try_load_cards(&decks.get_current_deck_path());
    if maybe_cards.is_none() {
        panic!("Could not load cards from folder");
    }

    let cards = maybe_cards.unwrap();
   

    let mut game = FlashcardGame::new(cards);

    let (mut rl, thread) = raylib::init()
        .size(800, 600)
        .title("Flashcard Game")
        .build();

    rl.set_target_fps(60);

    // Load a font - first try to load from file, if not available use default
    let custom_font = rl.load_font(&thread, "font.ttf").ok();
    let font_size: f32 = 40.0;
    let font_size_smaller: f32 = 35.0;

    let signifier_color = Color::from_hex("95A5A6").unwrap();

    while !rl.window_should_close() {
        // Input handling
        if rl.is_key_pressed(KeyboardKey::KEY_SPACE) || rl.is_key_pressed(KeyboardKey::KEY_UP) {
            game.flip();
        }
        if rl.is_key_pressed(KeyboardKey::KEY_RIGHT) {
            game.next_card();
        }
        if rl.is_key_pressed(KeyboardKey::KEY_LEFT) {
            game.prev_card();
        }
        if rl.is_key_pressed(KeyboardKey::KEY_A) {
            decks.prev_deck();
            update_decks(&decks, &mut game);
        }
        if rl.is_key_pressed(KeyboardKey::KEY_D) {
            decks.next_deck();
            update_decks(&decks, &mut game);
        }

        // Drawing
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::from_hex("2C3E50").unwrap());

        // Draw Current Deck title.
        let deck_title = format!("Deck: {}", decks.get_current_deck_name());
        let y = 25.0;

        draw_text_centered(&mut d, &custom_font, &deck_title, 400, 25, font_size, Color::WHITE);

        // Draw card background
        let card_rect = Rectangle::new(100.0, 100.0, 600.0, 350.0);
        let card_color = if game.is_flipped {
            Color::from_hex("3498DB").unwrap()
        } else {
            Color::from_hex("ECF0F1").unwrap()
        };
        d.draw_rectangle_rounded(card_rect, 0.05, 10, card_color);

        // Draw card border
        d.draw_rectangle_rounded_lines(card_rect, 0.05, 10, Color::from_hex("34495E").unwrap());

        // Draw text
        let text = game.get_current_text();
        let wrapped_lines = wrap_text(text, 550, font_size as i32);
        let line_height = (font_size + 5.0) as i32;
        let total_height = wrapped_lines.len() as i32 * line_height;
        let start_y = 275 - (total_height / 2);

        let text_color = if game.is_flipped {
            Color::WHITE
        } else {
            Color::from_hex("2C3E50").unwrap()
        };

        for (i, line) in wrapped_lines.iter().enumerate() {
            let y = start_y as f32 + (i as f32 * line_height as f32);
            draw_text_centered(&mut d, &custom_font, &line, 400, y as i32, font_size, text_color);
        }

        // Draw status indicator
        let status_text = if game.is_flipped {
            "ANSWER"
        } else {
            "QUESTION"
        };
        draw_text_centered(&mut d, &custom_font, &status_text, 400, 470, font_size_smaller, signifier_color);
        
        // Draw card counter
        let counter = format!("Card {} / {}", game.current_index + 1, game.cards.len());
        draw_text_centered(&mut d, &custom_font, &counter, 400, 500, font_size_smaller, signifier_color);

        // Draw instructions
        let message = "SPACE/UP: Flip  |  LEFT/RIGHT: Navigate";
        draw_text_centered(&mut d, &custom_font, &message, 400, 550, font_size_smaller, signifier_color);

    }
}