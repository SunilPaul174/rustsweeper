use crossterm::{
    event::{read, Event, KeyCode, KeyEvent, KeyEventKind},
    execute,
    style::ResetColor,
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
    ExecutableCommand,
};
use dialoguer::{theme::ColorfulTheme, Select};
use rand::seq::SliceRandom;
use std::{
    cmp::{max, min},
    collections::HashMap,
    io::stdout,
    process,
};

use ansi_term::{
    ANSIGenericString,
    Colour::{Black, White, RGB},
};

fn clear() {
    execute!(stdout(), Clear(ClearType::All)).unwrap();
    print!("\x1B[2J\x1B[1;1H");
}

fn mine_board(board: &mut Vec<Vec<Cell>>, settings: &Settings) {
    let cell_amount = settings.width * settings.height;
    let mut indeces: Vec<usize> = vec![];
    for i in 0..cell_amount as usize {
        indeces.push(i);
    }
    let choices: Vec<&usize> = indeces
        .choose_multiple(&mut rand::thread_rng(), settings.mines as usize)
        .collect();
    for index in choices {
        let row_index = index / settings.width as usize;
        let column_index = index % settings.width as usize;
        board[row_index][column_index].element = 'M';
    }
}
fn display_board(
    board: &Vec<Vec<Cell>>,
    board_objects_map: &HashMap<char, ANSIGenericString<'static, str>>,
    select_coords: Option<(i32, i32)>,
) {
    disable_raw_mode().unwrap();
    clear();
    for (i, row) in board.iter().enumerate() {
        for (j, cell) in row.iter().enumerate() {
            let mut display_string;
            if cell.flagged == false {
                if cell.hidden == true {
                    display_string = board_objects_map.get(&'#').expect("Fuck").clone();
                } else {
                    if cell.element == '0' {
                        display_string = board_objects_map.get(&' ').expect("Fuck").clone();
                    } else {
                        display_string =
                            board_objects_map.get(&cell.element).expect("Fuck").clone();
                    }
                }
            } else {
                display_string = board_objects_map.get(&'⚑').expect("Fuck").clone();
            }
            if let Some(select_coords) = select_coords {
                if i == select_coords.0 as usize && j == select_coords.1 as usize {
                    display_string = ansi_term::Colour::White
                        .on(ansi_term::Colour::RGB(144, 238, 144))
                        .paint("   ");
                }
            }
            print!("{display_string}");
        }
        println!("");
    }
    println!("WASD to move around, C to Click, F to FLag and ESC to exit");
}

fn get_around_cell(
    coords: [usize; 2],
    board: &Vec<Vec<Cell>>,
    boardsize: usize,
) -> Vec<(char, usize, usize)> {
    let mut cells: Vec<(char, usize, usize)> = vec![];
    let iterator = [coords[0] as i32, coords[1] as i32];
    for i in iterator[0] - 1..=iterator[0] + 1 {
        for j in iterator[1] - 1..=iterator[1] + 1 {
            if !(i == 0 && j == 0)
                && i >= 0
                && j >= 0
                && i < boardsize as i32
                && j < boardsize as i32
            {
                cells.push((
                    board[i as usize][j as usize].element,
                    i as usize,
                    j as usize,
                ));
            }
        }
    }
    cells
}

fn make_numbers(board: &mut Vec<Vec<Cell>>, settings: &Settings) {
    let mut board_copy = board.clone();
    for (row_number, row) in board.iter().enumerate() {
        for (column_number, cell) in row.iter().enumerate() {
            let around =
                get_around_cell([row_number, column_number], &board, settings.width as usize);
            let mut number = 0;
            for i in around.iter() {
                if i.0 == 'M' {
                    number += 1;
                }
            }
            if cell.element != 'M' {
                board_copy[row_number][column_number].element =
                    char::from_digit(number, 10).expect("Fuck");
            }
        }
    }
    *board = board_copy.clone();
}

fn deobfuscate_board(
    board: &mut Vec<Vec<Cell>>,
    row_number: usize,
    column_number: usize,
    boardsize: usize,
) {
    let mut to_check = vec![];
    if board[row_number][column_number].element == '0' {
        to_check.push((row_number, column_number));
    }
    let mut next_to_check: Vec<(usize, usize)> = Vec::new();
    let mut prev_checked: Vec<(usize, usize)> = Vec::new();
    while !to_check.is_empty() {
        for i in to_check.iter() {
            let around = get_around_cell([i.0, i.1], board, boardsize);
            for j in around.iter() {
                let curr_cell = (j.1, j.2);
                if !prev_checked.contains(&curr_cell) {
                    prev_checked.push(curr_cell);
                    if j.0 == '0' {
                        next_to_check.push(curr_cell);
                        board[j.1][j.2].hidden = false;
                    } else if j.0 != '0' && j.0 != 'M' {
                        board[j.1][j.2].hidden = false;
                    }
                }
            }
        }
        to_check = next_to_check.clone();
        next_to_check = vec![];
    }
}

fn event(
    row_number: i32,
    column_number: i32,
    board: &mut Vec<Vec<Cell>>,
    boardsize: usize,
) -> char {
    let temp_cell = board[row_number as usize][column_number as usize].element;
    if temp_cell == 'M' {
        'D'
    } else if temp_cell != '0' {
        board[row_number as usize][column_number as usize].hidden = false;
        'F'
    } else {
        board[row_number as usize][column_number as usize].hidden = false;
        deobfuscate_board(
            board,
            row_number as usize,
            column_number as usize,
            boardsize,
        );
        'F'
    }
}

fn flag(board: &mut Vec<Vec<Cell>>, row: usize, column: usize) {
    board[row][column].flagged = !board[row][column].flagged;
}

fn won(board: &Vec<Vec<Cell>>) -> bool {
    for i in board.iter() {
        for &j in i.iter() {
            if j.hidden == true && j.element != 'M' {
                return false;
            }
        }
    }
    true
}

fn get_choice_from_user(
    board: &Vec<Vec<Cell>>,
    board_objects_map: &HashMap<char, ANSIGenericString<'static, str>>,
    settings: &Settings,
    starting_coords: (i32, i32),
) -> (Choice, i32, i32) {
    let mut select_coords = (starting_coords.0, starting_coords.1);
    let choice: Choice;
    display_board(board, board_objects_map, Some(select_coords));
    loop {
        enable_raw_mode().unwrap();
        match read().unwrap() {
            Event::Key(KeyEvent {
                code: KeyCode::Char('a'),
                kind: KeyEventKind::Press,
                ..
            }) => {
                select_coords.1 = max(0, min(select_coords.1 - 1, settings.width - 1));
                display_board(board, board_objects_map, Some(select_coords));
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('d'),
                kind: KeyEventKind::Press,
                ..
            }) => {
                select_coords.1 = max(0, min(select_coords.1 + 1, settings.width - 1));
                display_board(board, board_objects_map, Some(select_coords));
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('w'),
                kind: KeyEventKind::Press,
                ..
            }) => {
                select_coords.0 = max(0, min(select_coords.0 - 1, settings.height - 1));
                display_board(board, board_objects_map, Some(select_coords));
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('s'),
                kind: KeyEventKind::Press,
                ..
            }) => {
                select_coords.0 = max(0, min(select_coords.0 + 1, settings.height - 1));
                display_board(board, board_objects_map, Some(select_coords));
            }
            Event::Key(KeyEvent {
                code: KeyCode::Esc,
                kind: KeyEventKind::Press,
                ..
            }) => {
                choice = Choice::Exit;
                break;
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('c'),
                kind: KeyEventKind::Press,
                ..
            }) => {
                choice = Choice::Click;
                break;
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('f'),
                kind: KeyEventKind::Press,
                ..
            }) => {
                choice = Choice::Flag;
                break;
            }
            _ => {}
        }
    }
    disable_raw_mode().unwrap();
    stdout().execute(ResetColor).unwrap();

    (choice, select_coords.0 as i32, select_coords.1 as i32)
}

#[derive(Copy, Clone)]
struct Cell {
    hidden: bool,
    element: char,
    flagged: bool,
}
enum Choice {
    Click,
    Flag,
    Exit,
}
enum Difficulty {
    Easy,
    Normal,
    Hard, //Custom(CustomDifficuly)
}
struct Settings {
    mines: i32,
    width: i32,
    height: i32,
}

fn main() {
    loop {
        let difficulty_options = vec!["Easy", "Normal", "Hard"]; //Todo Add Custom diffiuclty
        let difficulty = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select Difficulty")
            .items(&difficulty_options)
            .interact()
            .unwrap();
        let difficulty = match difficulty {
            0 => Difficulty::Easy,
            1 => Difficulty::Normal,
            2 => Difficulty::Hard,
            _ => Difficulty::Easy,
        };
        let settings = match difficulty {
            Difficulty::Easy => Settings {
                mines: 10,
                width: 8,
                height: 8,
            },
            Difficulty::Normal => Settings {
                mines: 40,
                width: 16,
                height: 16,
            },
            Difficulty::Hard => Settings {
                mines: 99,
                width: 30,
                height: 30,
            },
        };
        let boardsize = settings.width;
        let mut board = vec![
            vec![
                Cell {
                    hidden: true,
                    element: '0',
                    flagged: false
                };
                boardsize as usize
            ];
            boardsize as usize
        ];
        clear();
        let board_objects_map: HashMap<char, ANSIGenericString<'static, str>> = HashMap::from([
            ('M', RGB(0, 0, 0).on(White).bold().paint(" 🟐  ")),
            ('1', RGB(6, 3, 255).on(White).bold().paint(" 1 ")),
            ('2', RGB(3, 122, 6).on(White).bold().paint(" 2 ")),
            ('3', RGB(254, 0, 0).on(White).bold().paint(" 3 ")),
            ('4', RGB(0, 0, 132).on(White).bold().paint(" 4 ")),
            ('5', RGB(130, 1, 2).on(White).bold().paint(" 5 ")),
            ('6', RGB(2, 127, 130).on(White).bold().paint(" 6 ")),
            ('7', RGB(0, 0, 0).on(White).bold().paint(" 7 ")),
            ('8', RGB(125, 125, 125).on(White).bold().paint(" 8 ")),
            ('#', Black.on(Black).bold().paint("   ")),
            ('⚑', White.on(Black).bold().paint(" ⚑ ")),
            (' ', White.on(White).bold().paint("   ")),
        ]);
        mine_board(&mut board, &settings);
        make_numbers(&mut board, &settings);
        let mut select_coords = (settings.width / 2 as i32, settings.height / 2 as i32);
        loop {
            let (choice, row_number, column_number) =
                get_choice_from_user(&board, &board_objects_map, &settings, select_coords);
            select_coords.0 = row_number;
            select_coords.1 = column_number;
            match choice {
                Choice::Exit => {
                    disable_raw_mode().unwrap();
                    process::exit(0);
                }
                Choice::Click => {
                    let event = event(
                        row_number,
                        column_number,
                        &mut board,
                        boardsize.try_into().unwrap(),
                    );
                    if event == 'D' {
                        clear();
                        for i in board.iter_mut() {
                            for j in i.iter_mut() {
                                j.hidden = false;
                            }
                        }
                        display_board(&board, &board_objects_map, None);
                        println!("You died.");
                        return;
                    } else {
                        clear();
                    }
                    if won(&board) {
                        for i in board.iter_mut() {
                            for j in i.iter_mut() {
                                j.hidden = false;
                            }
                        }
                        display_board(&board, &board_objects_map, None);
                        println!("You win!");
                        return;
                    }
                }
                Choice::Flag => {
                    flag(&mut board, row_number as usize, column_number as usize);
                    clear();
                }
            };
        }
    }
}
