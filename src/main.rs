use crossterm::{
    cursor::{Hide, MoveTo, RestorePosition, SavePosition, Show},
    event::{
        read, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyEventKind,
        MouseButton, MouseEvent, MouseEventKind,
    },
    execute,
    style::ResetColor,
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
    ExecutableCommand,
};
use dialoguer::{theme::ColorfulTheme, Input, Select};
use rand::seq::SliceRandom;
use std::{
    cmp::{max, min},
    collections::HashMap,
    io::stdout,
    ops::ControlFlow,
    process,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
};

use ansi_term::{
    ANSIGenericString,
    Colour::{Black, White, RGB},
};

fn clear() {
    execute!(stdout(), Clear(ClearType::All)).unwrap();
    print!("\x1B[2J\x1B[1;1H");
    execute!(stdout(), Clear(ClearType::All)).unwrap();
    print!("\x1B[2J\x1B[1;1H");
}

fn place_mines(board: &mut Vec<Vec<Cell>>, settings: &Settings, starting_coords: (i32, i32)) {
    let cell_amount = settings.width * settings.height;
    let mut indeces: Vec<usize> = vec![];
    for i in 0..cell_amount as usize {
        indeces.push(i);
    }
    let choices: Vec<&usize> = indeces
        .choose_multiple(&mut rand::thread_rng(), settings.mines as usize)
        .collect();
    let mut invalid_spots = get_around_cell_coord_only(
        [starting_coords.0 as usize, starting_coords.1 as usize],
        settings,
    );
    invalid_spots.push((starting_coords.0 as usize, starting_coords.0 as usize));
    for index in choices {
        let row_index = index / settings.width as usize;
        let column_index = index % settings.width as usize;
        if !(invalid_spots.contains(&(row_index, column_index))) {
            board[row_index][column_index].element = 'M';
        }
    }
}
fn display_board(board: &Vec<Vec<Cell>>, settings: &Settings) {
    disable_raw_mode().unwrap();
    clear();
    let terminal_size = get_terminal_size();
    for (i, row) in board.iter().enumerate() {
        for (_, cell) in row.iter().enumerate() {
            display_cell(cell);
        }
        if (i as i32 + 1) < terminal_size.1 {
            println!("");
        } else {
            print!("");
            stdout().execute(MoveTo(0, 0)).unwrap();
            break;
        }
    }
    if settings.height < terminal_size.1 {
        if let InputType::Keyboard = settings.input_type {
            print!("WASD to move around, C to Click, F to Flag and ESC to exit to main menu");
        } else {
            print!("Left Mouse Button to Click, F to Flag and ESC to exit to main menu");
        }
        stdout().execute(MoveTo(0, 0)).unwrap();
    }
}
fn display_cell(cell: &Cell) {
    let display_string;
    if cell.flagged == false {
        if cell.hidden == true {
            display_string = get_display_string('#', cell.selected);
        } else {
            if cell.element == '0' {
                display_string = get_display_string(' ', cell.selected);
            } else {
                display_string = get_display_string(cell.element, cell.selected);
            }
        }
    } else {
        display_string = get_display_string('⚑', cell.selected);
    }
    print!("{display_string}");
}
fn get_display_string(character: char, is_selected: bool) -> ANSIGenericString<'static, str> {
    let board_objects_map: HashMap<char, ANSIGenericString<'static, str>>;
    if !is_selected {
        board_objects_map = HashMap::from([
            ('M', RGB(0, 0, 0).on(White).bold().paint(" ✹ ")),
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
    } else {
        board_objects_map = HashMap::from([
            ('M', RGB(0, 0, 0).on(RGB(144, 238, 144)).bold().paint(" ✹ ")),
            (
                '1',
                RGB(6, 3, 255).on(RGB(144, 238, 144)).bold().paint(" 1 "),
            ),
            (
                '2',
                RGB(3, 122, 6).on(RGB(144, 238, 144)).bold().paint(" 2 "),
            ),
            (
                '3',
                RGB(254, 0, 0).on(RGB(144, 238, 144)).bold().paint(" 3 "),
            ),
            (
                '4',
                RGB(0, 0, 132).on(RGB(144, 238, 144)).bold().paint(" 4 "),
            ),
            (
                '5',
                RGB(130, 1, 2).on(RGB(144, 238, 144)).bold().paint(" 5 "),
            ),
            (
                '6',
                RGB(2, 127, 130).on(RGB(144, 238, 144)).bold().paint(" 6 "),
            ),
            ('7', RGB(0, 0, 0).on(RGB(144, 238, 144)).bold().paint(" 7 ")),
            (
                '8',
                RGB(125, 125, 125)
                    .on(RGB(144, 238, 144))
                    .bold()
                    .paint(" 8 "),
            ),
            ('#', Black.on(RGB(144, 238, 144)).bold().paint("   ")),
            ('⚑', White.on(RGB(144, 238, 144)).bold().paint(" ⚑ ")),
            (' ', White.on(RGB(144, 238, 144)).bold().paint("   ")),
        ]);
    }
    board_objects_map.get(&character).unwrap().clone()
}
fn get_around_cell(
    coords: [usize; 2],
    board: &Vec<Vec<Cell>>,
    settings: &Settings,
) -> Vec<(char, usize, usize)> {
    let mut cells: Vec<(char, usize, usize)> = vec![];
    let iterator = [coords[0] as i32, coords[1] as i32];
    for i in iterator[0] - 1..=iterator[0] + 1 {
        for j in iterator[1] - 1..=iterator[1] + 1 {
            if i >= 0 && j >= 0 && i < settings.height as i32 && j < settings.width as i32 {
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

fn get_around_cell_coord_only(coords: [usize; 2], settings: &Settings) -> Vec<(usize, usize)> {
    let mut cells: Vec<(usize, usize)> = vec![];
    let iterator = [coords[0] as i32, coords[1] as i32];
    for i in iterator[0] - 1..=iterator[0] + 1 {
        for j in iterator[1] - 1..=iterator[1] + 1 {
            if i >= 0 && j >= 0 && i < settings.height as i32 && j < settings.width as i32 {
                cells.push((i as usize, j as usize));
            }
        }
    }
    cells
}

fn place_numbers(board: &mut Vec<Vec<Cell>>, settings: &Settings) {
    let mut board_copy = board.clone();
    for (row_number, row) in board.iter().enumerate() {
        for (column_number, cell) in row.iter().enumerate() {
            let around = get_around_cell([row_number, column_number], &board, &settings);
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
    settings: &Settings,
    hidden_cells: &mut Vec<(usize, usize)>,
) {
    let mut to_check = vec![];
    if board[row_number][column_number].element == '0' {
        to_check.push((row_number, column_number));
    }
    let mut next_to_check: Vec<(usize, usize)> = Vec::new();
    let mut prev_checked: Vec<(usize, usize)> = Vec::new();
    while !to_check.is_empty() {
        for i in to_check.iter() {
            let around = get_around_cell([i.0, i.1], board, &settings);
            for j in around.iter() {
                let curr_cell = (j.1, j.2);
                if !prev_checked.contains(&curr_cell) {
                    prev_checked.push(curr_cell);
                    if j.0 == '0' {
                        next_to_check.push(curr_cell);
                        board[j.1][j.2].hidden = false;
                        update_cell(board, (j.1 as i32, j.2 as i32));
                        hidden_cells.retain(|value| *value != (j.1, j.2));
                    } else if j.0 != '0' && j.0 != 'M' {
                        board[j.1][j.2].hidden = false;
                        update_cell(board, (j.1 as i32, j.2 as i32));
                        hidden_cells.retain(|value| *value != (j.1, j.2));
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
    settings: &Settings,
    hidden_cells: &mut Vec<(usize, usize)>,
) -> Click {
    let cell = board[row_number as usize][column_number as usize];
    if cell.flagged {
        return Click::Fine;
    }
    let cell_type = cell.element;
    if cell_type == 'M' {
        Click::Dead
    } else if cell_type != '0' {
        board[row_number as usize][column_number as usize].hidden = false;
        update_cell(board, (row_number, column_number));
        hidden_cells.retain(|value| *value != (row_number as usize, column_number as usize));
        Click::Fine
    } else {
        board[row_number as usize][column_number as usize].hidden = false;
        hidden_cells.retain(|value| *value != (row_number as usize, column_number as usize));
        deobfuscate_board(
            board,
            row_number as usize,
            column_number as usize,
            &settings,
            hidden_cells,
        );
        Click::Fine
    }
}

fn flag(board: &mut Vec<Vec<Cell>>, row: i32, column: i32) {
    board[column as usize][row as usize].flagged = !board[column as usize][row as usize].flagged;
    update_cell(&board, (column, row));
}

fn won(hidden_cells: &mut Vec<(usize, usize)>) -> bool {
    hidden_cells.is_empty()
}
fn get_terminal_size() -> (i32, i32) {
    let size = terminal_size::terminal_size().unwrap();
    (size.0 .0 as i32, size.1 .0 as i32)
}
fn get_choice_from_user(
    mut board: &mut Vec<Vec<Cell>>,
    settings: &Settings,
    starting_coords: (i32, i32),
) -> (Choice, i32, i32) {
    let mut select_coords = (starting_coords.0, starting_coords.1);
    let mut previous_select_coords = select_coords;
    let choice: Choice;
    let thread_flag = Arc::new(AtomicBool::new(false));
    let flag_clone = thread_flag.clone();
    let mut terminal_size = get_terminal_size();
    let settings_copy = settings.clone();
    let (tx, rx) = std::sync::mpsc::channel();
    let handle = thread::spawn(move || {
        let mut board_copy: Option<Vec<Vec<Cell>>> = None;
        while !flag_clone.load(Ordering::Relaxed) {
            match rx.try_recv() {
                Ok(received_data) => board_copy = Some(received_data),
                Err(_) => {}
            }
            let new_terminal_size = get_terminal_size();
            if terminal_size != new_terminal_size {
                clear();
                terminal_size = new_terminal_size;
                if let Some(ref board_copy) = board_copy {
                    display_board(&board_copy, &settings_copy);
                }
            }
        }
    });

    stdout().execute(EnableMouseCapture).unwrap();
    loop {
        enable_raw_mode().unwrap();
        stdout().execute(Hide).unwrap();
        match read().unwrap() {
            Event::Mouse(MouseEvent {
                kind: MouseEventKind::Down(MouseButton::Left),
                ..
            }) => {
                if let InputType::Mouse = settings.input_type {
                    choice = Choice::Click;
                    break;
                }
            }
            Event::Mouse(MouseEvent { row, column, .. }) => {
                if let InputType::Mouse = settings.input_type {
                    select_coords.1 = max(0, min(column as i32 / 3, settings.width - 1));
                    select_coords.0 = max(0, min(row as i32, settings.height - 1));
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('a'),
                kind: KeyEventKind::Press,
                ..
            }) => {
                if let InputType::Keyboard = settings.input_type {
                    select_coords.1 = max(0, min(select_coords.1 - 1, settings.width - 1));
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('d'),
                kind: KeyEventKind::Press,
                ..
            }) => {
                if let InputType::Keyboard = settings.input_type {
                    select_coords.1 = max(0, min(select_coords.1 + 1, settings.width - 1));
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('w'),
                kind: KeyEventKind::Press,
                ..
            }) => {
                if let InputType::Keyboard = settings.input_type {
                    select_coords.0 = max(0, min(select_coords.0 - 1, settings.height - 1));
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('s'),
                kind: KeyEventKind::Press,
                ..
            }) => {
                if let InputType::Keyboard = settings.input_type {
                    select_coords.0 = max(0, min(select_coords.0 + 1, settings.height - 1));
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('c'),
                kind: KeyEventKind::Press,
                ..
            }) => {
                if let InputType::Keyboard = settings.input_type {
                    choice = Choice::Click;
                    break;
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('f'),
                kind: KeyEventKind::Press,
                ..
            }) => flag(&mut board, select_coords.1, select_coords.0),
            Event::Key(KeyEvent {
                code: KeyCode::Esc,
                kind: KeyEventKind::Press,
                ..
            }) => {
                choice = Choice::Exit;
                break;
            }
            _ => {}
        }
        if select_coords != previous_select_coords {
            board[previous_select_coords.0 as usize][previous_select_coords.1 as usize].selected =
                false;
            update_cell(&board, previous_select_coords);
            board[select_coords.0 as usize][select_coords.1 as usize].selected = true;
            update_cell(&board, select_coords);
            previous_select_coords = select_coords;
            tx.send(board.clone()).unwrap();
        }
    }
    disable_raw_mode().unwrap();
    stdout().execute(ResetColor).unwrap();
    thread_flag.store(true, Ordering::Relaxed);
    handle.join().unwrap();
    (choice, select_coords.0 as i32, select_coords.1 as i32)
}
fn update_cell(board: &Vec<Vec<Cell>>, pos: (i32, i32)) {
    stdout().execute(SavePosition).unwrap();
    stdout()
        .execute(MoveTo((pos.1 * 3) as u16, (pos.0) as u16))
        .unwrap();
    display_cell(&board[pos.0 as usize][pos.1 as usize]);
    stdout().execute(RestorePosition).unwrap();
}
fn get_settings(mut settings: Settings) -> Settings {
    let settings_options = vec!["Play", "Difficulty", "Controls", "Exit"];
    loop {
        let setting = Select::with_theme(&ColorfulTheme::default())
            .items(&settings_options)
            .interact()
            .unwrap();
        match setting {
            0 => break,
            1 => select_difficulty(&mut settings),
            2 => select_input_type(&mut settings),
            3 => exit_gracefully(),
            _ => {}
        }
    }
    settings
}
fn select_input_type(settings: &mut Settings) {
    let input_options = vec!["Mouse", "Keyboard"]; //Todo Add Custom diffiuclty
    let input_type = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select Input Type")
        .items(&input_options)
        .interact()
        .unwrap();
    let input_type = match input_type {
        0 => InputType::Mouse,
        1 => InputType::Keyboard,
        _ => InputType::Mouse,
    };
    settings.input_type = input_type;
}
fn select_difficulty(settings: &mut Settings) {
    let difficulty_options = vec!["Easy", "Normal", "Hard", "Custom"]; //Todo Add Custom diffiuclty
    let difficulty = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select Difficulty")
        .items(&difficulty_options)
        .interact()
        .unwrap();
    let difficulty = match difficulty {
        0 => Difficulty::Easy,
        1 => Difficulty::Normal,
        2 => Difficulty::Hard,
        3 => Difficulty::Custom,
        _ => Difficulty::Easy,
    };
    match difficulty {
        Difficulty::Easy => {
            settings.mines = 10;
            settings.width = 8;
            settings.height = 8;
        }
        Difficulty::Normal => {
            settings.mines = 40;
            settings.width = 16;
            settings.height = 16;
        }
        Difficulty::Hard => {
            settings.mines = 99;
            settings.width = 30;
            settings.height = 16;
        }
        Difficulty::Custom => {
            let size = terminal_size::terminal_size().unwrap();
            let width: u32 = Input::with_theme(&ColorfulTheme::default())
                .with_prompt(&format!("Board width (max: {})", size.0 .0 / 3))
                .validate_with(|x: &u32| {
                    if *x > size.0 .0 as u32 / 3 {
                        Err("Width entered exceeds the width of your terminal")
                    } else {
                        Ok(())
                    }
                })
                .interact()
                .unwrap();
            settings.width = width as i32;
            let height: u32 = Input::with_theme(&ColorfulTheme::default())
                .with_prompt(&format!("Board height (max: {})", size.1 .0 -2))
                .validate_with(|x: &u32| {
                    if *x > size.1 .0 as u32 - 2 {
                        Err("Height entered exceeds the height of your terminal and the instructions")
                    } else {
                        Ok(())
                    }
                })
                .interact()
                .unwrap();
            settings.height = height as i32;
            let mines: u32 = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("Mine amount")
                .validate_with(|x: &u32| {
                    if *x >= width * height {
                        Err("Mine amount cannot exceed board area")
                    } else {
                        Ok(())
                    }
                })
                .interact()
                .unwrap();
            settings.mines = mines as i32;
        }
    };
}
fn exit_gracefully() {
    disable_raw_mode().unwrap();
    stdout().execute(DisableMouseCapture).unwrap();
    stdout().execute(ResetColor).unwrap();
    stdout().execute(Show).unwrap();
    process::exit(0);
}
fn reveal_board(board: &mut Vec<Vec<Cell>>) {
    let mut updated_cells: Vec<(i32, i32)> = vec![];
    for (x, i) in board.iter_mut().enumerate() {
        for (y, j) in i.iter_mut().enumerate() {
            if j.hidden || j.selected {
                updated_cells.push((x as i32, y as i32));
            }
            j.hidden = false;
            j.selected = false;
        }
    }
    for cell in updated_cells {
        update_cell(&board, (cell.0 as i32, cell.1 as i32));
    }
}
fn initialize_free_cells(board: &Vec<Vec<Cell>>) -> Vec<(usize, usize)> {
    let mut hidden_cells: Vec<(usize, usize)> = vec![];
    for (row_number, row) in board.iter().enumerate() {
        for (cell_number, cell) in row.iter().enumerate() {
            if cell.element != 'M' {
                hidden_cells.push((row_number, cell_number));
            }
        }
    }
    hidden_cells
}
fn main_menu(mut settings: Settings, go_directly_to_game: bool) {
    clear();
    loop {
        if !go_directly_to_game {
            settings = get_settings(settings);
        }

        let mut board = vec![
            vec![
                Cell {
                    hidden: true,
                    element: '0',
                    flagged: false,
                    selected: false,
                };
                settings.width as usize
            ];
            settings.height as usize
        ];
        clear();
        let mut select_coords = (settings.height / 2 as i32, settings.width / 2 as i32);
        board[select_coords.0 as usize][select_coords.1 as usize].selected = true;
        display_board(&board, &settings);
        let (mut choice, mut row_number, mut column_number) =
            get_choice_from_user(&mut board, &settings, select_coords);
        place_mines(&mut board, &settings, (row_number, column_number));
        place_numbers(&mut board, &settings);
        let mut hidden_cells = initialize_free_cells(&board);
        loop {
            if let ControlFlow::Break(_) = game_play_loop_node(
                &mut board,
                settings,
                &mut select_coords,
                &choice,
                row_number,
                column_number,
                &mut hidden_cells,
            ) {
                break;
            }
            (choice, row_number, column_number) =
                get_choice_from_user(&mut board, &settings, select_coords);
        }
        let options = vec!["Play Again", "Main Menu", "Exit"];
        let choice = Select::with_theme(&ColorfulTheme::default())
            .items(&options)
            .interact()
            .unwrap();
        match choice {
            0 => main_menu(settings.clone(), true),
            1 => main_menu(settings.clone(), false),
            2 => exit_gracefully(),
            _ => {}
        }
    }
}

fn game_play_loop_node(
    board: &mut Vec<Vec<Cell>>,
    settings: Settings,
    select_coords: &mut (i32, i32),
    choice: &Choice,
    row_number: i32,
    column_number: i32,
    hidden_cells: &mut Vec<(usize, usize)>,
) -> ControlFlow<()> {
    select_coords.0 = row_number;
    select_coords.1 = column_number;
    match choice {
        Choice::Exit => {
            main_menu(settings.clone(), false);
        }
        Choice::Click => {
            let terminal_size = get_terminal_size();
            let event = event(row_number, column_number, board, &settings, hidden_cells);
            match event {
                Click::Dead => {
                    if terminal_size.1 > settings.height + 4 {
                        stdout()
                            .execute(MoveTo(0, (settings.height + 1) as u16))
                            .unwrap();
                        reveal_board(board);
                    } else {
                        clear();
                    }
                    println!("You died.");
                    return ControlFlow::Break(());
                }
                Click::Fine => {}
            }
            if won(hidden_cells) {
                if terminal_size.1 > settings.height + 4 {
                    stdout()
                        .execute(MoveTo(0, (settings.height + 1) as u16))
                        .unwrap();
                    reveal_board(board);
                } else {
                    clear();
                }
                println!("You win!");
                return ControlFlow::Break(());
            }
        }
    };
    ControlFlow::Continue(())
}
#[derive(Debug, Copy, Clone)]
struct Cell {
    hidden: bool,
    element: char,
    flagged: bool,
    selected: bool,
}
enum Choice {
    Click,
    Exit,
}
enum Difficulty {
    Easy,
    Normal,
    Hard,
    Custom,
}
#[derive(Debug, Clone, Copy)]
enum InputType {
    Mouse,
    Keyboard,
}

#[derive(PartialEq)]
enum Click {
    Dead,
    Fine,
}
#[derive(Debug, Clone, Copy)]
struct Settings {
    mines: i32,
    width: i32,
    height: i32,
    input_type: InputType,
}
impl Default for Settings {
    fn default() -> Self {
        Settings {
            mines: 10,
            width: 8,
            height: 8,
            input_type: InputType::Mouse,
        }
    }
}
fn main() {
    main_menu(Settings::default(), false);
}
