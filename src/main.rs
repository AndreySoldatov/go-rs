#![windows_subsystem = "windows"]

use std::{fs::write, fs::read_to_string};

use serde::{Serialize, Deserialize};
use macroquad::{prelude::*, audio::{load_sound, play_sound, set_sound_volume}};

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum BoardCellOption{
    Black,
    White,
    None
}

#[derive(Serialize, Deserialize)]
struct GoBoard{
    size: usize,
    board: Vec<Vec<BoardCellOption>>,
    captured_black: usize,
    captured_white: usize
}

impl GoBoard {
    fn new(size: usize) -> Self {
        GoBoard { 
            size, 
            board: vec![vec![BoardCellOption::None; size]; size],
            captured_black: 0,
            captured_white: 0
        }
    }

    fn load_from_file(path: &str) -> Self {
        serde_json::from_str(read_to_string(path).unwrap().as_str()).unwrap()
    }

    fn set(& mut self, x: usize, y: usize, piece: BoardCellOption) {
        if x < self.size && y < self.size {
            self.board[y][x] = piece;
            self.update(x, y);
        }
    }

    fn update(& mut self, x: usize, y: usize) {
        let c = Cluster::from(self, x, y);
        if !c.has_liberties(self) {
            self.clear_cluster(&c);
        }

        if x.wrapping_sub(1) < self.size {
            let c = Cluster::from(self, x.wrapping_sub(1), y);
            if !c.has_liberties(self) {
                self.clear_cluster(&c);
            }
        }
        if x + 1 < self.size {
            let c = Cluster::from(self, x + 1, y);
            if !c.has_liberties(self) {
                self.clear_cluster(&c);
            }
        }
        if y.wrapping_sub(1) < self.size {
            let c = Cluster::from(self, x, y.wrapping_sub(1));
            if !c.has_liberties(self) {
                self.clear_cluster(&c);
            }
        }

        if y + 1 < self.size {
            let c = Cluster::from(self, x, y + 1);
            if !c.has_liberties(self) {
                self.clear_cluster(&c);
            }
        }
    }

    fn clear_cluster(&mut self, c: &Cluster) {
        match c.color {
            BoardCellOption::Black => {
                self.captured_white += c.pieces.len()
            }, 
            BoardCellOption::White => {
                self.captured_black += c.pieces.len()
            },
            _ => {}
        }
        for p in &c.pieces {
            self.board[p[1]][p[0]] = BoardCellOption::None;
        }
    }

    fn has_liberties(&self, x: usize, y: usize) -> bool {
        self.value(x + 1, y) || 
        self.value(x.wrapping_sub(1), y) || 
        self.value(x, y + 1) || 
        self.value(x, y.wrapping_sub(1))
    }

    fn value(&self, x: usize, y: usize) -> bool {
        x < self.size && y < self.size && self.board[y][x] == BoardCellOption::None
    }

    fn save_to_file(&self, path: &str) {
        write(path, serde_json::to_string(self).unwrap()).unwrap();
    }
}

struct Cluster {
    pieces: Vec<[usize; 2]>,
    color: BoardCellOption
}

impl Cluster {
    fn from(board: &GoBoard, x: usize, y: usize) -> Self {
        let mut cl = Cluster { 
            pieces: vec![[x, y]], 
            color: board.board[y][x]
        };

        cl.next_piece(board, x, y.wrapping_sub(1));
        cl.next_piece(board, x.wrapping_sub(1), y);
        cl.next_piece(board, x + 1, y);
        cl.next_piece(board, x, y + 1);

        cl
    }

    fn next_piece(&mut self, board: &GoBoard, x: usize, y: usize) {
        if x < board.size && y < board.size {
            if board.board[y][x] == self.color && board.board[y][x] != BoardCellOption::None {
                if !self.pieces.contains(&[x, y]) {
                    self.pieces.push([x, y]);
                }

                if !self.pieces.contains(&[x, y.wrapping_sub(1)]) { 
                    self.next_piece(board, x, y.wrapping_sub(1));
                }
                if !self.pieces.contains(&[x.wrapping_sub(1), y]) { 
                    self.next_piece(board, x.wrapping_sub(1), y);
                }
                if !self.pieces.contains(&[x + 1, y]) {
                    self.next_piece(board, x + 1, y);
                }
                if !self.pieces.contains(&[x, y + 1]) { 
                    self.next_piece(board, x, y + 1);
                }
            }
        }
    }

    fn has_liberties(&self, board: &GoBoard) -> bool {
        for p in &self.pieces {
            if board.has_liberties(p[0], p[1]) {
                return true;
            }
        }
        false
    }
}

struct Theme {
    background_color: Color,
    foreground_color: Color
}

impl Default for Theme {
    fn default() -> Self {
        Theme { 
            background_color: Color::from_rgba(0, 0, 0, 255), 
            foreground_color: Color::from_rgba(255, 255, 255, 255) 
        }
    }
}

struct GoBoardUi {
    size: f32,
    data: GoBoard,
    board_theme: Theme,
    piece_theme: Theme
}

impl GoBoardUi {
    fn new(size: usize) -> Self {
        GoBoardUi {
            size: 30.,
            data: GoBoard::new(size), 
            board_theme: Theme { 
                background_color: Color::from_rgba(75, 107, 88, 255), 
                foreground_color: Color::from_rgba(255, 255, 255, 255) 
            }, 
            piece_theme: Theme::default() 
        }
    }

    fn draw(&self, font: &Font) {

        let board_width = self.size * (self.data.size.wrapping_sub(1)) as f32;
        let board_height = self.size * (self.data.size.wrapping_sub(1)) as f32;

        let start = Vec2::new(
            screen_width() * 0.5 - board_width * 0.5,
            screen_height() * 0.5 - board_height * 0.5,
        );

        clear_background(self.board_theme.background_color);
        for i in 0..self.data.size {
            draw_text_ex(
                (i + 1).to_string().as_str(),
                start.x - self.size * 1.3,
                start.y + self.size * i as f32 + self.size * 0.25, 
                TextParams { 
                    font: *font,
                    font_size: (self.size * 0.8) as u16,
                    color: self.board_theme.foreground_color,
                    ..Default::default()
                }
            );

            draw_line(
                start.x,
                start.y + self.size * i as f32, 
                start.x + board_width,
                start.y + self.size * i as f32, 
                self.size * 0.05, 
                self.board_theme.foreground_color
            );

            draw_text_ex(
                (i + 1).to_string().as_str(),
                start.x + self.size * i as f32 - self.size * 0.25,
                start.y - self.size * 0.7,
                TextParams { 
                    font: *font,
                    font_size: (self.size * 0.8) as u16,
                    color: self.board_theme.foreground_color,
                    ..Default::default()
                }
            );

            draw_line(
                start.x + self.size * i as f32,
                start.y, 
                start.x + self.size * i as f32,
                start.y + board_height, 
                self.size * 0.05, 
                self.board_theme.foreground_color
            );
        }

        for y in 0..self.data.board.len() {
            for x in 0..self.data.board[y].len() {
                match &self.data.board[y][x] {
                    BoardCellOption::Black => {
                        draw_circle(
                            start.x + self.size * x as f32, 
                            start.y + self.size * y as f32, 
                            self.size * 0.5,
                            self.piece_theme.background_color
                        );
                    },
                    BoardCellOption::White => {
                        draw_circle(
                            start.x + self.size * x as f32, 
                            start.y + self.size * y as f32, 
                            self.size * 0.5, 
                            self.piece_theme.foreground_color
                        );
                    },
                    BoardCellOption::None => {}
                }
            }   
        }

        let go_cursor_pos = Vec2::new(mouse_position().0 - start.x, mouse_position().1 - start.y);

        if go_cursor_pos.x > 0. && go_cursor_pos.y > 0. && go_cursor_pos.x <= board_width && go_cursor_pos.y <= board_height {
            draw_circle_lines(
                start.x + ((go_cursor_pos.x / (board_width + self.size as f32)) * self.data.size as f32).round() * self.size,
                start.y + ((go_cursor_pos.y / (board_height + self.size as f32)) * self.data.size as f32).round() * self.size,
                self.size * 0.5,
                5.0,
                Color::from_rgba(255, 20, 40, 50)
            );
        }

        draw_text_ex(
            format!("White captured: {} Black captured: {}", self.data.captured_white, self.data.captured_black).as_str(), 
            start.x, 
            start.y + board_height + board_width * 0.1, 
            TextParams { 
                font: *font, 
                font_size: ((self.size * 0.8) as u16).min((screen_width() / 25.) as u16),
                color: self.board_theme.foreground_color,
                ..Default::default()
            }
        );
    }

    fn update(& mut self) {
        if screen_width() >= screen_height() {
            self.size = screen_height() / (self.data.size + 4) as f32;
        } else {
            self.size = screen_width() / (self.data.size + 4) as f32;
        }

        let board_width = self.size * (self.data.size.wrapping_sub(1)) as f32;
        let board_height = self.size * (self.data.size.wrapping_sub(1)) as f32;

        let start = Vec2::new(
            screen_width() * 0.5 - board_width * 0.5,
            screen_height() * 0.5 - board_height * 0.5,
        );

        let go_cursor_pos = Vec2::new(mouse_position().0 - start.x, mouse_position().1 - start.y);

        if is_mouse_button_pressed(MouseButton::Left) {
            self.data.set(
                ((go_cursor_pos.x / (board_width + self.size as f32)) * self.data.size as f32).round() as usize,
                ((go_cursor_pos.y / (board_height + self.size as f32)) * self.data.size as f32).round() as usize,
                BoardCellOption::Black
            );
        }
        else if is_mouse_button_pressed(MouseButton::Right) {
            self.data.set(
                ((go_cursor_pos.x / (board_width + self.size as f32)) * self.data.size as f32).round() as usize,
                ((go_cursor_pos.y / (board_height + self.size as f32)) * self.data.size as f32).round() as usize,
                BoardCellOption::White
            );
        }
        else if is_mouse_button_pressed(MouseButton::Middle) {
            self.data.set(
                ((go_cursor_pos.x / (board_width + self.size as f32)) * self.data.size as f32).round() as usize,
                ((go_cursor_pos.y / (board_height + self.size as f32)) * self.data.size as f32).round() as usize,
                BoardCellOption::None
            );
        }

        if is_key_pressed(KeyCode::S) {
            self.data.save_to_file("save.gs");
        }
    }
}

fn window_conf() -> Conf {
    Conf { 
        window_title: String::from("Go"), 
        window_width: 800, 
        window_height: 800,
        sample_count: 16,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut volume = 1.0;

    let music = load_sound("music.ogg").await.unwrap();

    play_sound(
        music, 
        macroquad::audio::PlaySoundParams { 
            looped: true, 
            volume
        }
    );

    let font = load_ttf_font("font_regular.ttf").await.unwrap();

    let args = std::env::args().collect::<Vec<String>>();

    let mut go_game: GoBoardUi;

    if args.len() < 2 {
        go_game = GoBoardUi::new(19);
    } else if let Ok(num) = args[1].parse::<usize>() {
        go_game = GoBoardUi::new(num);
    }
    else {
        let board = GoBoard::load_from_file(args[1].as_str());
        go_game = GoBoardUi {
            data: board,
            size: 30.,
            board_theme: Theme { 
                background_color: Color::from_rgba(75, 107, 88, 255), 
                foreground_color: Color::from_rgba(255, 255, 255, 255) 
            }, 
            piece_theme: Theme::default() 
        };
    }

    let mut fade_time = 0.0;

    loop {
        let delta = get_frame_time();

        go_game.update();

        go_game.draw(&font);

        if mouse_wheel().1.abs() > 0. && fade_time < 0.001 {
            fade_time += 3.0;
        }

        fade_time = (fade_time - delta).max(0.0);

        volume += mouse_wheel().1 * 0.0008333;
        volume = volume.max(0.0).min(1.0);

        set_sound_volume(music, volume);

        if fade_time > 0. {
            draw_text_ex(format!("{:.1}", volume).as_str(), screen_width() - screen_height() * 0.1, screen_height()  - screen_height() * 0.05, 
                TextParams { 
                    font, 
                    font_size: (go_game.size * 0.8) as u16,
                    color: go_game.board_theme.foreground_color,
                    ..Default::default()
                }
            );
        }

        next_frame().await
    }
}