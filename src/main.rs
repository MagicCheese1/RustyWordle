use std::ascii::AsciiExt;
use std::fs::File;
use std::{io, string};
use std::cmp::PartialEq;
use std::io::{stdout, BufRead, Write};
use std::thread::current;
use crossterm::{execute, style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor}, ExecutableCommand, event, queue, terminal, cursor};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use crossterm::terminal::ClearType;
use rand::seq::{SliceRandom};

#[derive(Copy, Clone, PartialEq, Debug)]
enum LetterColor {
    Black,
    Gray = 0x333333,
    Yellow = 0xadad07,
    Green = 0x176002,
}

impl LetterColor {
    fn to_crossterm_color(&self) -> crossterm::style::Color {
        match self {
            LetterColor::Black => crossterm::style::Color::Black,
            LetterColor::Gray => crossterm::style::Color::Rgb { r: 0x33, g: 0x33, b: 0x33 },
            LetterColor::Yellow => crossterm::style::Color::Rgb { r: 0xad, g: 0xad, b: 0x07 },
            LetterColor::Green => crossterm::style::Color::Rgb { r: 0x17, g: 0x60, b: 0x02 },
        }
    }
}
#[derive(Copy, Clone, Debug)]
struct Letter {
    character: char,
    color: LetterColor,
}

impl Letter {
    fn new(character: char, color: LetterColor) -> Result<Letter, &'static str> {
        if (character.is_ascii_alphabetic() || character == '_') {
            Ok(Letter { character, color })
        } else {
            Err("character must be ascii alphabetic or underscore")
        }
    }
}

fn main() {
    let mut guess_words = Vec::<String>::new();
    let mut solution_word = String::new();

    //Load words lists

    {
        let file = File::open("./wordle-Ta.txt");
        io::BufReader::new(file.unwrap()).lines().for_each(|word| { guess_words.push(word.unwrap()) });
    }

    {
        let mut solution_words = Vec::<String>::new();
        let file = File::open("./wordle-La.txt");
        io::BufReader::new(file.unwrap()).lines().for_each(|word| { solution_words.push(word.unwrap()) });
        solution_word = solution_words.choose(&mut rand::thread_rng()).unwrap().clone();
        guess_words.append(&mut solution_words);
    }
    
    let mut board = vec![vec![Letter::new('_', LetterColor::Black).unwrap(); 5]; 6];
    let mut current_line = 0;
    let mut current_position = 0;

    let keyboard_chars = ['Q', 'W', 'E', 'R', 'T', 'Y', 'U', 'I', 'O', 'P', 'A', 'S', 'D', 'F', 'G', 'H', 'J', 'K', 'L', 'Z', 'X', 'C', 'V', 'B', 'N', 'M'];
    let mut keyboard_letters = Vec::new();
    for character in keyboard_chars {
        let letter = Letter::new(character, LetterColor::Black).unwrap();
        keyboard_letters.push(letter);
    }
    let mut stdout = io::stdout();
    loop {
        execute!(stdout,
            terminal::EnterAlternateScreen).unwrap();
        terminal::enable_raw_mode().unwrap();
        queue!(stdout,
            ResetColor,
            terminal::Clear(ClearType::All),
            cursor::MoveTo(1, 1));
        execute!(stdout, cursor::MoveTo(16, 16), Print(&solution_word)).unwrap();

        // Draw Board
        for (i, line) in board.iter().enumerate() {
            for (j, position) in line.iter().enumerate() {
                queue!(stdout,
                    cursor::MoveTo((j*2 + 6) as u16,
                        (i*2 + 1) as u16),
                    SetBackgroundColor(position.color.to_crossterm_color()),
                    SetForegroundColor(Color::White),
                    Print(position.character.to_string()),
                    ResetColor);
            }
        }
        queue!(stdout,
            cursor::Show,
            cursor::MoveToNextLine(2),
            Print(" ".to_string()));

        //Draw Keyboard
        for letter in &keyboard_letters {
            queue!(stdout,SetForegroundColor(Color::White),
                 SetBackgroundColor(letter.color.to_crossterm_color()),
                 Print(letter.character.to_string()),
                ResetColor);
            queue!(stdout,
                Print(" ".to_string()));
            match letter.character {
                'P' | 'M' => { queue!(stdout,Print("\n  ".to_string())); }
                'L' => { queue!(stdout,Print("\n   ".to_string())); }
                _ => {}
            }
        }
        queue!(stdout,
                    cursor::MoveTo((current_position*2 + 6) as u16, (current_line*2 + 1) as u16));

        stdout.flush().unwrap();

        // Handle Input
        let pressed_char = read_key().unwrap();
        if let KeyCode::Char(c) = pressed_char {
            if c.is_ascii_alphabetic() && current_position <= 4 {
                board[current_line][current_position] = Letter::new(c.to_ascii_uppercase(), LetterColor::Black).unwrap();
                current_position += 1;
            }
        } else if pressed_char == KeyCode::Esc {
            break;
        } else if pressed_char == KeyCode::Enter && current_position > 4 && is_valid_word(&board[current_line], &guess_words) {
            //Color board
            let mut _solution_word = solution_word.clone();
            for (i, letter) in board[current_line].iter_mut().enumerate() {
                if _solution_word.chars().nth(i).unwrap() == letter.character {
                    _solution_word.replace_range(i..(i + 1), "_");
                    *letter = Letter::new(letter.character, LetterColor::Green).unwrap();
                    
                    for keyboard_letter in keyboard_letters.iter_mut() {
                        if letter.character == keyboard_letter.character {
                            *keyboard_letter = Letter::new(keyboard_letter.character, LetterColor::Green).unwrap();
                            break;
                        }
                    }
                }
            }
            for (i, letter) in board[current_line].iter_mut().enumerate() {
                if _solution_word.contains(letter.character) && letter.color != LetterColor::Green {
                    _solution_word = _solution_word.replacen(letter.character, "_", 1);
                    *letter = Letter::new(letter.character, LetterColor::Yellow).unwrap();
                    for keyboard_letter in keyboard_letters.iter_mut() {
                        if letter.character == keyboard_letter.character {
                            if keyboard_letter.color == LetterColor::Green {
                                break;
                            }
                            *keyboard_letter = Letter::new(keyboard_letter.character, LetterColor::Yellow).unwrap();
                            break;
                        }
                    }
                }
            }
            for (i, letter) in board[current_line].iter_mut().enumerate() {
                if letter.color == LetterColor::Black {
                    *letter = Letter::new(letter.character, LetterColor::Gray).unwrap();
                    for keyboard_letter in keyboard_letters.iter_mut() {
                        if letter.character == keyboard_letter.character {
                            match keyboard_letter.color {
                                LetterColor::Black => {*keyboard_letter = Letter::new(keyboard_letter.character, LetterColor::Gray).unwrap();}
                                _ => {}
                            }
                            break;
                        }
                    }
                }
            }
            current_line += 1;
            current_position = 0;
        } else if pressed_char == KeyCode::Backspace && current_position > 0 {
            board[current_line][current_position - 1] = Letter::new('_', LetterColor::Black).unwrap();
            current_position -= 1;
        }
    }
    execute!(stdout,
        terminal::LeaveAlternateScreen).unwrap();
}

fn is_valid_word(sliced_word: &Vec<Letter>, word_list: &Vec<String>) -> bool {
    let word = sliced_word.into_iter().map(|letter| letter.character).collect::<String>();
    word_list.contains(&word)
}

pub fn read_key() -> std::io::Result<KeyCode> {
    loop {
        if let Event::Key(KeyEvent { code, kind: KeyEventKind::Press, .. }) = event::read()? {
            return Ok(code);
        }
    }
}
