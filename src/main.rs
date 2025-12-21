use raylib::prelude::*;
use std::fs::File;
use std::io::{BufRead, BufReader};

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
                cards.push(Flashcard {
                    question,
                    answer,
                });
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

fn main() {
    let cards = match load_flashcards("cards.csv") {
        Ok(cards) if !cards.is_empty() => cards,
        Ok(_) => {
            eprintln!("Error: cards.csv is empty or contains no valid flashcards");
            return;
        }
        Err(e) => {
            eprintln!("Error loading cards.csv: {}", e);
            return;
        }
    };

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

        // Drawing
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::from_hex("2C3E50").unwrap());

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
            
            if let Some(ref font) = custom_font {
                // Approximate text width for custom font
                let approx_width = (line.len() as f32 * font_size_smaller * 0.5) as f32;
                let x = 400.0 - approx_width / 2.0;
                d.draw_text_ex(font, line, Vector2::new(x, y), font_size, 1.0, text_color);
            } else {
                let text_width = d.measure_text(line, 28);
                let x = 400 - text_width / 2;
                d.draw_text(line, x, y as i32, font_size as i32, text_color);
            }
        }

        // Draw status indicator
        let status_text = if game.is_flipped { "ANSWER" } else { "QUESTION" };
        if let Some(ref font) = custom_font {
            d.draw_text_ex(font, status_text, Vector2::new(350.0, 470.0), font_size_smaller, 1.0, Color::from_hex("95A5A6").unwrap());
        } else {
            d.draw_text(status_text, 350, 470, 20, Color::from_hex("95A5A6").unwrap());
        }

        // Draw card counter
        let counter = format!("Card {} / {}", game.current_index + 1, game.cards.len());
        if let Some(ref font) = custom_font {
            d.draw_text_ex(font, &counter, Vector2::new(350.0, 500.0), font_size_smaller, 1.0, Color::from_hex("95A5A6").unwrap());
        } else {
            d.draw_text(&counter, 350, 500, font_size_smaller as i32, Color::from_hex("95A5A6").unwrap());
        }

        // Draw instructions
        if let Some(ref font) = custom_font {
            d.draw_text_ex(font, "SPACE/UP: Flip  |  LEFT/RIGHT: Navigate", Vector2::new(220.0, 550.0), font_size_smaller, 1.0, Color::from_hex("7F8C8D").unwrap());
        } else {
            d.draw_text("SPACE/UP: Flip  |  LEFT/RIGHT: Navigate", 220, 550, font_size_smaller as i32, Color::from_hex("7F8C8D").unwrap());
        }
    }
}

// Add to Cargo.toml:
// [dependencies]
// raylib = "5.0"