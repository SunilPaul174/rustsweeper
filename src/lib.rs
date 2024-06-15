use std::{
    cmp::{max,min},
    collections::HashMap,
    io::stdout,
    ops::ControlFlow,
    sync::{
        Arc,
        Mutex,
        atomic::{AtomicBool, Ordering},
    },
    process,
    thread,
};
use ansi_term::{
    ANSIGenericString,
    Color::{Black,RGB,White}
};
use crossterm::{
    cursor::{Hide,MoveTo,Show},
    ExecutableCommand,
    execute,
    event::{
        DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyEventKind, MouseButton, MouseEvent, MouseEventKind, read
    },
    style::ResetColor,
    terminal::{
        Clear, ClearType, disable_raw_mode, enable_raw_mode
    }
};
use dialoguer::{
    Input, MultiSelect, Select,
    theme::ColorfulTheme
};
use rand::prelude::SliceRandom;

#[derive(Debug, Copy, Clone)]
pub struct Cell {
    hidden: bool,
    element: char,
    flagged: bool,
    selected: bool,
}
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct MousePos {
    x: i32,
    y: i32,
}
impl MousePos {
    fn convert(&self, settings: &Settings) -> CellPos {
        let cell_pos = CellPos {
            x: ((self.x - settings.board_x_pos as i32) / 3)
                .max(0)
                .min(settings.width - 1),
            y: (self.y - settings.board_y_pos as i32)
                .max(0)
                .min(settings.height - 1),
        };
        cell_pos
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
struct CellPos {
    x: i32,
    y: i32,
}
impl CellPos {
    pub fn convert(&self, settings: &Settings) -> MousePos {
        MousePos {
            x: self.x + settings.board_x_pos as i32,
            y: self.y + settings.board_y_pos as i32,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Settings {
    mines: i32,
    width: i32,
    height: i32,
    input_type: InputType,
    bordered: bool,
    centered: bool,
    board_x_pos: u32,
    board_y_pos: u32,
    str_y_pos: u32,
    showing_board: bool,
}
impl Default for Settings {
    fn default() -> Self {
        Settings {
            mines: 10,
            width: 8,
            height: 8,
            input_type: InputType::Mouse,
            bordered: false,
            centered: true,
            board_x_pos: 0,
            board_y_pos: 0,
            str_y_pos: 0,
            showing_board: false,
        }
    }
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

fn clear(settings: &mut Settings) {
    execute!(stdout(), Clear(ClearType::All)).unwrap();
    print!("\x1B[2J\x1B[1;1H");
    execute!(stdout(), Clear(ClearType::All)).unwrap();
    print!("\x1B[2J\x1B[1;1H");
    settings.str_y_pos = 0;
    settings.showing_board = false;
}

fn center_board(settings: &mut Settings) {
    let terminal_size = get_terminal_size();
    if settings.centered {
        settings.board_x_pos = ((terminal_size.0 / 2 - (settings.width * 3) / 2).max(0)) as u32;
        settings.board_y_pos = ((terminal_size.1 / 2 - settings.height / 2).max(0)) as u32;
        if settings.bordered {
            settings.board_x_pos = (settings.board_x_pos as i32 - 1).max(1) as u32;
            settings.board_y_pos = (settings.board_y_pos as i32 - 1).max(1) as u32;
        }
    } else {
        if settings.bordered {
            settings.board_x_pos = 1;
            settings.board_y_pos = 1;
        } else {
            settings.board_x_pos = 0;
            settings.board_y_pos = 0;
        }
    }
}

fn get_settings(settings: &mut Settings) {
    let settings_options = vec!["Play", "Difficulty", "Controls", "Appearance", "Exit"];
    loop {
        let setting = Select::with_theme(&ColorfulTheme::default())
            .items(&settings_options)
            .interact()
            .unwrap();
        match setting {
            0 => break,
            1 => select_difficulty(settings),
            2 => select_input_type(settings),
            3 => get_appearance_settings(settings),
            4 => exit_gracefully(),
            _ => {}
        }
    }
}

fn display_board(board: &Vec<Vec<Cell>>, settings: &mut Settings) {
    disable_raw_mode().unwrap();
    clear(settings);
    let terminal_size = get_terminal_size();
    for y in 0..settings.height {
        for x in 0..settings.width {
            update_cell(&board, CellPos { x, y }, &settings);
        }
    }
    let mut tip_pos = (
        settings.board_x_pos as i32,
        settings.height + settings.board_y_pos as i32,
    );
    let mut y_limit = terminal_size.1;
    if settings.bordered {
        y_limit -= 1;
        tip_pos.1 += 1;
        tip_pos.0 -= 1;
        for j in 0..2 {
            for i in 0..settings.width {
                let move_to_x = settings.board_x_pos as i32 + i * 3;
                let mut move_to_y = settings.board_y_pos as i32 - 1;
                match j {
                    1 => move_to_y += settings.height + 1,
                    _ => {}
                }
                if move_to_x >= 0
                    && move_to_x < terminal_size.0
                    && move_to_y >= 0
                    && move_to_y < terminal_size.1
                {
                    stdout()
                        .execute(MoveTo(move_to_x as u16, move_to_y as u16))
                        .unwrap();
                    print!("{}", White.on(Black).paint("━━━"));
                }
            }
        }
        for j in 0..2 {
            for i in -1..settings.height + 1 {
                let mut move_to_x = settings.board_x_pos as i32 - 1;
                let move_to_y = settings.board_y_pos as i32 + i;
                match j {
                    1 => move_to_x += settings.width * 3 + 1,
                    _ => {}
                }
                if move_to_x >= 0
                    && move_to_x < terminal_size.0
                    && move_to_y >= 0
                    && move_to_y < terminal_size.1
                {
                    stdout()
                        .execute(MoveTo(move_to_x as u16, move_to_y as u16))
                        .unwrap();
                    let char: &str;
                    if i == -1 {
                        char = match j {
                            0 => "┏",
                            1 => "┓",
                            _ => "",
                        };
                    } else if i == settings.height {
                        char = match j {
                            0 => "┗",
                            1 => "┛",
                            _ => "",
                        };
                    } else {
                        char = "┃";
                    }
                    print!("{}", White.on(Black).paint(char));
                }
            }
        }
    }
    settings.showing_board = true;
    if tip_pos.1 < y_limit {
        if let InputType::Keyboard = settings.input_type {
            print_string(
                "WASD to move around, C to Click, F to Flag and ESC to exit to main menu. Use arrow keys to move board",
                settings,
            );
        } else {
            print_string(
                "Left Mouse Button to Click, F to Flag and ESC to exit to main menu. Use arrow keys to move board",
                settings,
            );
        }
    }
}

fn get_choice_from_user(
    mut board: &mut Vec<Vec<Cell>>,
    settings: Arc<Mutex<Settings>>,
    starting_pos: CellPos,
) -> (Choice, CellPos) {
    let mut cell_pos = starting_pos;
    let mut previous_select_pos = cell_pos;
    let settings_mutex: Arc<Mutex<Settings>> = Arc::clone(&settings);

    let mut mouse_pos = cell_pos.convert(&mut settings_mutex.lock().unwrap());
    let choice: Choice;
    let thread_flag = Arc::new(AtomicBool::new(false));
    let flag_clone = thread_flag.clone();
    let mut terminal_size = get_terminal_size();
    let (tx, rx) = std::sync::mpsc::channel::<Vec<Vec<Cell>>>();

    let cloned_mutex = Arc::clone(&settings);
    let handle = thread::spawn(move || {
        let mut board_copy: Option<Vec<Vec<Cell>>> = None;
        while !flag_clone.load(Ordering::Relaxed) {
            match rx.try_recv() {
                Ok(received_data) => {
                    board_copy = Some(received_data);
                }
                Err(_) => {}
            }
            let new_terminal_size = get_terminal_size();
            if terminal_size != new_terminal_size {
                clear(&mut cloned_mutex.lock().unwrap());
                terminal_size = new_terminal_size;
                if let Some(ref board_copy) = board_copy {
                    center_board(&mut cloned_mutex.lock().unwrap());
                    display_board(&board_copy, &mut cloned_mutex.lock().unwrap());
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
                let settings_guard = settings_mutex.lock().unwrap();
                if let InputType::Mouse = settings_guard.input_type {
                    choice = Choice::Click;
                    break;
                }
                drop(settings_guard);
            }
            Event::Mouse(MouseEvent { row, column, .. }) => {
                let mut settings_guard = settings_mutex.lock().unwrap();
                if let InputType::Mouse = settings_guard.input_type {
                    mouse_pos.x = column as i32;
                    mouse_pos.y = row as i32;
                    cell_pos = mouse_pos.convert(&mut settings_guard);
                }
                drop(settings_guard);
            }
            Event::Key(KeyEvent {
                           code: KeyCode::Char('a'),
                           kind: KeyEventKind::Press,
                           ..
                       }) => {
                let settings_guard = settings_mutex.lock().unwrap();
                if let InputType::Keyboard = settings_guard.input_type {
                    cell_pos.x = max(0, min(cell_pos.x - 1, settings_guard.width - 1));
                }
                drop(settings_guard);
            }
            Event::Key(KeyEvent {
                           code: KeyCode::Char('d'),
                           kind: KeyEventKind::Press,
                           ..
                       }) => {
                let settings_guard = settings_mutex.lock().unwrap();
                if let InputType::Keyboard = settings_guard.input_type {
                    cell_pos.x = max(0, min(cell_pos.x + 1, settings_guard.width - 1));
                }
                drop(settings_guard);
            }
            Event::Key(KeyEvent {
                           code: KeyCode::Char('w'),
                           kind: KeyEventKind::Press,
                           ..
                       }) => {
                let settings_guard = settings_mutex.lock().unwrap();
                if let InputType::Keyboard = settings_guard.input_type {
                    cell_pos.y = max(0, min(cell_pos.y - 1, settings_guard.height - 1));
                }
                drop(settings_guard);
            }
            Event::Key(KeyEvent {
                           code: KeyCode::Char('s'),
                           kind: KeyEventKind::Press,
                           ..
                       }) => {
                let settings_guard = settings_mutex.lock().unwrap();
                if let InputType::Keyboard = settings_guard.input_type {
                    cell_pos.y = max(0, min(cell_pos.y + 1, settings_guard.height - 1));
                }
                drop(settings_guard);
            }
            Event::Key(KeyEvent {
                           code: KeyCode::Char('c'),
                           kind: KeyEventKind::Press,
                           ..
                       }) => {
                let settings_guard = settings_mutex.lock().unwrap();
                if let InputType::Keyboard = settings_guard.input_type {
                    choice = Choice::Click;
                    break;
                }
                drop(settings_guard);
            }
            Event::Key(KeyEvent {
                           code: KeyCode::Char('f'),
                           kind: KeyEventKind::Press,
                           ..
                       }) => flag(&mut board, cell_pos, &settings_mutex.lock().unwrap()),
            Event::Key(KeyEvent {
                           code: KeyCode::Up,
                           kind: KeyEventKind::Press,
                           ..
                       }) => {
                let mut settings_guard = settings_mutex.lock().unwrap();
                settings_guard.board_y_pos = (settings_guard.board_y_pos as i32 - 1).max(0) as u32;
                display_board(board, &mut settings_guard);
                tx.send(board.clone()).unwrap();
                drop(settings_guard);
            }
            Event::Key(KeyEvent {
                           code: KeyCode::Down,
                           kind: KeyEventKind::Press,
                           ..
                       }) => {
                let mut settings_guard = settings_mutex.lock().unwrap();
                settings_guard.board_y_pos += 1;
                display_board(board, &mut settings_guard);
                tx.send(board.clone()).unwrap();
                drop(settings_guard);
            }
            Event::Key(KeyEvent {
                           code: KeyCode::Right,
                           kind: KeyEventKind::Press,
                           ..
                       }) => {
                let mut settings_guard = settings_mutex.lock().unwrap();
                settings_guard.board_x_pos += 1;
                display_board(board, &mut settings_guard);
                tx.send(board.clone()).unwrap();
                drop(settings_guard);
            }
            Event::Key(KeyEvent {
                           code: KeyCode::Left,
                           kind: KeyEventKind::Press,
                           ..
                       }) => {
                let mut settings_guard = settings_mutex.lock().unwrap();
                settings_guard.board_x_pos = (settings_guard.board_x_pos as i32 - 1).max(0) as u32;
                display_board(board, &mut settings_guard);
                tx.send(board.clone()).unwrap();
                drop(settings_guard);
            }
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
        if cell_pos != previous_select_pos {
            let mut settings_guard = settings_mutex.lock().unwrap();
            board[previous_select_pos.y as usize][previous_select_pos.x as usize].selected = false;
            update_cell(&board, previous_select_pos, &mut settings_guard);
            board[cell_pos.y as usize][cell_pos.x as usize].selected = true;
            update_cell(&board, cell_pos, &mut settings_guard);
            previous_select_pos = cell_pos;
            tx.send(board.clone()).unwrap();
            drop(settings_guard);
        }
    }
    disable_raw_mode().unwrap();
    stdout().execute(ResetColor).unwrap();
    thread_flag.store(true, Ordering::Relaxed);
    handle.join().unwrap();

    (choice, cell_pos)
}

fn place_mines(board: &mut Vec<Vec<Cell>>, settings: &Settings, starting_coords: CellPos) {
    let cell_amount = settings.width * settings.height;
    let mut indices: Vec<usize> = vec![];
    for i in 0..cell_amount as usize {
        let column_number = (i as i32) / settings.width;
        let row_number = (i as i32) % settings.width;
        if (starting_coords.y - column_number).abs() <= 1 {
            if (starting_coords.x - row_number).abs() <= 1 {
                continue;
            }
        }
        indices.push(i);
    }
    let choices: Vec<&usize> = indices
        .choose_multiple(&mut rand::thread_rng(), settings.mines as usize)
        .collect();
    for index in choices {
        let row_index = index / settings.width as usize;
        let column_index = index % settings.width as usize;
        board[row_index][column_index].element = 'M';
    }
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
fn get_terminal_size() -> (i32, i32) {
    let size = terminal_size::terminal_size().unwrap();
    (size.0 .0 as i32, size.1 .0 as i32)
}
fn game_play_loop_node(
    board: &mut Vec<Vec<Cell>>,
    settings: &mut Settings,
    choice: &Choice,
    cell_pos: CellPos,
    hidden_cells: &mut Vec<(usize, usize)>,
) -> ControlFlow<()> {
    match choice {
        Choice::Exit => {
            main_menu(settings.clone(), false);
        }
        Choice::Click => {
            let terminal_size = get_terminal_size();
            let event = event(cell_pos, board, &settings, hidden_cells);
            match event {
                Click::Dead => {
                    if terminal_size.1 > settings.height + 4 {
                        reveal_board(board, settings);
                    } else {
                        clear(settings);
                    }
                    print_string("You died.", settings);
                    return ControlFlow::Break(());
                }
                Click::Fine => {}
            }
            if won(hidden_cells) {
                if terminal_size.1 > settings.height + 4 {
                    reveal_board(board, settings);
                } else {
                    clear(settings);
                }
                print_string("You win!", settings);
                return ControlFlow::Break(());
            }
        }
    };
    ControlFlow::Continue(())
}
pub fn print_string(string: &str, settings: &mut Settings) {
    let mut string_x_pos = settings.board_x_pos as u16;
    if settings.bordered {
        string_x_pos = (string_x_pos as i32 - 1).max(0) as u16;
    }
    if settings.centered {
        string_x_pos = (string_x_pos as i32 - (string.len() / 2) as i32 + (settings.width * 3) / 2)
            .max(0) as u16;
    }
    let mut string_y_pos = settings.str_y_pos as u16;
    if settings.showing_board {
        string_y_pos +=
            (settings.board_y_pos as i32 + settings.height + settings.bordered as i32) as u16;
    }
    stdout()
        .execute(MoveTo(string_x_pos, string_y_pos))
        .unwrap();
    print!("{}", string);
    settings.str_y_pos += 1;
}

fn update_cell(board: &Vec<Vec<Cell>>, cell_pos: CellPos, settings: &Settings) {
    let mut x_pos: u16 = (cell_pos.x * 3) as u16;
    let mut y_pos: u16 = (cell_pos.y) as u16;
    x_pos += settings.board_x_pos as u16;
    y_pos += settings.board_y_pos as u16;
    let terminal_size = get_terminal_size();
    if x_pos as i32 >= terminal_size.0 {
        return;
    }
    if y_pos as i32 >= terminal_size.0 {
        return;
    }
    stdout().execute(MoveTo(x_pos, y_pos)).unwrap();
    display_cell(&board[cell_pos.y as usize][cell_pos.x as usize]);
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
            if i >= 0 && j >= 0 && i < settings.height && j < settings.width {
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
fn deobfuscate_board(
    board: &mut Vec<Vec<Cell>>,
    cell_pos: CellPos,
    settings: &Settings,
    hidden_cells: &mut Vec<(usize, usize)>,
) {
    let mut to_check = vec![];
    if board[cell_pos.y as usize][cell_pos.x as usize].element == '0' {
        to_check.push((cell_pos.y as usize, cell_pos.x as usize));
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
                        update_cell(
                            board,
                            CellPos {
                                x: j.2 as i32,
                                y: j.1 as i32,
                            },
                            settings,
                        );
                        hidden_cells.retain(|value| *value != (j.1, j.2));
                    } else if j.0 != '0' && j.0 != 'M' {
                        board[j.1][j.2].hidden = false;
                        update_cell(
                            board,
                            CellPos {
                                x: j.2 as i32,
                                y: j.1 as i32,
                            },
                            settings,
                        );
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
    cell_pos: CellPos,
    board: &mut Vec<Vec<Cell>>,
    settings: &Settings,
    hidden_cells: &mut Vec<(usize, usize)>,
) -> Click {
    let cell = board[cell_pos.y as usize][cell_pos.x as usize];
    if cell.flagged {
        return Click::Fine;
    }
    let cell_type = cell.element;
    if cell_type == 'M' {
        Click::Dead
    } else if cell_type != '0' {
        board[cell_pos.y as usize][cell_pos.x as usize].hidden = false;
        update_cell(board, cell_pos, settings);
        hidden_cells.retain(|value| *value != (cell_pos.y as usize, cell_pos.x as usize));
        Click::Fine
    } else {
        board[cell_pos.y as usize][cell_pos.x as usize].hidden = false;
        hidden_cells.retain(|value| *value != (cell_pos.y as usize, cell_pos.x as usize));
        deobfuscate_board(board, cell_pos, &settings, hidden_cells);
        Click::Fine
    }
}
fn flag(board: &mut Vec<Vec<Cell>>, cell_pos: CellPos, settings: &Settings) {
    board[cell_pos.y as usize][cell_pos.x as usize].flagged =
        !board[cell_pos.y as usize][cell_pos.x as usize].flagged;
    update_cell(&board, cell_pos, settings);
}
fn won(hidden_cells: &mut Vec<(usize, usize)>) -> bool {
    hidden_cells.is_empty()
}
fn get_appearance_settings(settings: &mut Settings) {
    let appearance_options = vec!["Centered", "Bordered"];
    let defaults = vec![settings.centered, settings.bordered];
    let mut theme = ColorfulTheme::default();
    theme.defaults_style = dialoguer::console::Style::new().red();
    let green_style = dialoguer::console::Style::new().green().bold();
    let black_style = dialoguer::console::Style::new().black();
    let checked_item_prefix = green_style.apply_to("✓".to_owned());
    let unchecked_item_prefix = black_style.apply_to("☐".to_owned());
    theme.checked_item_prefix = checked_item_prefix;
    theme.unchecked_item_prefix = unchecked_item_prefix;
    let appearance = MultiSelect::with_theme(&theme)
        .items(&appearance_options)
        .defaults(&defaults)
        .interact()
        .unwrap();
    settings.bordered = false;
    settings.centered = false;
    for i in appearance {
        match i {
            0 => settings.centered = true,
            1 => settings.bordered = true,
            _ => {}
        }
    }
    center_board(settings);
}
fn select_input_type(settings: &mut Settings) {
    let input_options = vec!["Mouse", "Keyboard"];
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
    let difficulty_options = vec!["Easy", "Normal", "Hard", "Custom"];
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
    stdout().execute(EnableMouseCapture).unwrap();
    stdout().execute(DisableMouseCapture).unwrap();
    stdout().execute(ResetColor).unwrap();
    stdout().execute(Show).unwrap();
    process::exit(0);
}
fn reveal_board(board: &mut Vec<Vec<Cell>>, settings: &Settings) {
    let mut cells_to_update: Vec<(i32, i32)> = vec![];
    for (x, i) in board.iter_mut().enumerate() {
        for (y, j) in i.iter_mut().enumerate() {
            if j.hidden || j.selected {
                cells_to_update.push((x as i32, y as i32));
            }
            j.hidden = false;
            j.selected = false;
        }
    }
    for cell in cells_to_update {
        update_cell(
            &board,
            CellPos {
                x: cell.1,
                y: cell.0,
            },
            settings,
        );
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

pub fn main_menu(mut settings: Settings, go_directly_to_game: bool) {
    clear(&mut settings);
    center_board(&mut settings);
    loop {
        if !go_directly_to_game {
            get_settings(&mut settings);
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
        clear(&mut settings);
        let mut cell_pos = CellPos {
            x: settings.width / 2,
            y: settings.height / 2,
        };
        board[cell_pos.y as usize][cell_pos.x as usize].selected = true;
        display_board(&board, &mut settings);
        let settings_mutex = Arc::new(Mutex::new(settings));
        let (mut choice, new_cell_pos) =
            get_choice_from_user(&mut board, Arc::clone(&settings_mutex), cell_pos);
        cell_pos = new_cell_pos;
        settings = *settings_mutex.lock().unwrap();
        place_mines(&mut board, &settings, cell_pos);
        place_numbers(&mut board, &settings);
        let mut hidden_cells = initialize_free_cells(&board);
        loop {
            if let ControlFlow::Break(_) = game_play_loop_node(
                &mut board,
                &mut settings,
                &choice,
                cell_pos,
                &mut hidden_cells,
            ) {
                break;
            }
            (choice, cell_pos) =
                get_choice_from_user(&mut board, Arc::clone(&settings_mutex), cell_pos);
        }
        let terminal_size = get_terminal_size();
        let options = vec!["Play Again", "Main Menu", "Exit"];
        let y_pos;
        if settings.showing_board {
            y_pos = (settings.board_y_pos + settings.height as u32 + 2 + settings.bordered as u32)
                as u16;
        } else {
            y_pos = settings.str_y_pos as u16;
            settings.str_y_pos = settings.str_y_pos + options.len() as u32;
        }
        if !settings.centered && y_pos < terminal_size.1 as u16 - options.len() as u16 {
            stdout().execute(MoveTo(0, y_pos)).unwrap();
        } else {
            stdout().execute(MoveTo(0, 0)).unwrap();
        }
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