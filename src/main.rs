extern crate argparse;
extern crate rand;
extern crate termion;

use argparse::{ArgumentParser, Store, StoreTrue};
use rand::Rng;
use std::io::{stdout, Read, Write};
use std::thread;
use std::time::Duration;
use termion::{async_stdin, clear, color, cursor};
use termion::raw::IntoRawMode;

// Unit for game
type Unit = usize;

mod graphics {
    pub const BORDER_HORIZONTAL: &str = "═";
    pub const BORDER_VERTICAL: &str = "║";
    pub const TOP_LEFT_BORDER: &str = "╔";
    pub const TOP_RIGHT_BORDER: &str = "╗";
    pub const BOTTOM_LEFT_BORDER: &str = "╚";
    pub const BOTTOM_RIGHT_BORDER: &str = "╝";

    pub const GAME_START:      &str =  "╔══════════════════════════════╗\n\
                                        ║──   Welcome to rbreakout   ──║\n\
                                        ║──────────────────────────────║\n\
                                        ║       <space>  begin         ║\n\
                                        ║       q        quit          ║\n\
                                        ║                              ║\n\
                                        ║  Controls                    ║\n\
                                        ║       h    move left         ║\n\
                                        ║       l    move left         ║\n\
                                        ║       q    quit              ║\n\
                                        ║       r    reset             ║\n\
                                        ║                              ║\n\
                                        ╚══════════════════════════════╝";
    pub const GAME_OVER: &str = "╔═════════════════╗\n\
                                 ║──  GAME OVER  ──║\n\
                                 ║   r  replay     ║\n\
                                 ║   q  quit       ║\n\
                                 ╚═════════════════╝";
    pub const GAME_WIN: &str =
        "╔════════════════════╗\n\
         ║── CONGRATULATION ──║\n\
         ║     r  replay      ║\n\
         ║     q  quit        ║\n\
         ╚════════════════════╝";
    pub const BALL_GRAPHIC: &str = "●";
    pub const PEDDLE_GRAPHIC: &str = "════════════";
}

trait Drawable {
    fn write<W: Write>(&self, stdout: &mut W) {
        write!(
            stdout,
            "{}{}{}",
            color::Fg(self.get_color()),
            self.get_cursor_pos(),
            self.get_graphics(),
        ).unwrap();
    }
    fn clear<W: Write>(&self, stdout: &mut W) {
        write!(
            stdout,
            "{}{}{}",
            color::Bg(color::Reset),
            self.get_cursor_pos(),
            " ".repeat(Self::get_width() as usize)
        ).unwrap();
    }

    fn get_pos(&self) -> (Unit, Unit);
    fn x(&self) -> (Unit) {
        self.get_pos().0
    }
    fn y(&self) -> (Unit) {
        self.get_pos().1
    }
    fn get_cursor_pos(&self) -> cursor::Goto {
        let pos = self.get_pos();
        cursor::Goto(pos.0 as u16, pos.1 as u16)
    }

    fn get_color(&self) -> &color::Color;
    fn get_graphics(&self) -> String;

    fn get_width() -> Unit;
    fn get_height(&self) -> Unit {
        return 1 as Unit;
    }
}

struct Cell {
    pos: (Unit, Unit),
    color: Box<color::Color>,
}

impl Drawable for Cell {
    fn get_pos(&self) -> (Unit, Unit) {
        (self.pos.0, self.pos.1)
    }

    fn get_color(&self) -> &color::Color {
        self.color.as_ref()
    }
    fn get_graphics(&self) -> String {
        "█".repeat(Cell::get_width() as usize)
    }

    fn get_width() -> Unit {
        8 as Unit
    }
}

struct Ball {
    game_pos: (f32, f32),
    vel: (f32, f32),
}

impl Drawable for Ball {
    fn get_pos(&self) -> (Unit, Unit) {
        (
            self.game_pos.0.round() as Unit,
            self.game_pos.1.round() as Unit,
        )
    }

    fn get_color(&self) -> &color::Color {
        &color::Red
    }
    fn get_graphics(&self) -> String {
        graphics::BALL_GRAPHIC.to_string()
    }

    fn get_width() -> Unit {
        1
    }
}

impl Ball {
    fn update(&mut self, game_size: (Unit, Unit), player_pos: (Unit, Unit)) -> bool {
        if self.x() <= 2 || self.x() >= (game_size.0 as Unit - 1) {
            self.vel.0 *= -1f32;
        }
        if self.y() <= 2 {
            self.vel.1 *= -1f32;
        }

        if self.y() >= (game_size.1 as Unit - 1) {
            return false;
        }

        if (self.x() >= player_pos.0 && self.x() <= player_pos.0 + Peddle::get_width())
            && self.y() == player_pos.1
        {
            self.vel.1 *= -1f32;
            let xoffset = self.game_pos.0 - (player_pos.0 + Peddle::get_width() / 2 as Unit) as f32;
            self.vel.0 += 0.4 * (xoffset / (Peddle::get_width() / 2) as f32);
        }

        self.game_pos.0 += self.vel.0;
        self.game_pos.1 += self.vel.1;

        fn clamp(val: f32, min: f32, max: f32) -> f32 {
            val.max(min).min(max)
        }
        self.game_pos = (
            clamp(self.game_pos.0, 2f32, (game_size.0 - 1) as f32),
            clamp(self.game_pos.1, 2f32, (game_size.1 - 1) as f32),
        );

        true
    }

    fn collides_with<T: Drawable>(&self, target: &T) -> Option<Direction> {
        let target_x = target.x() as f32;
        let target_y = target.y() as f32;
        let target_width = T::get_width() as f32;

        let x = self.game_pos.0;
        let y = self.game_pos.1;

        fn in_range(val: f32, min: f32, max: f32) -> bool {
            val >= min && val <= max
        }

        if in_range(y, target_y, target_y + 1f32) {
            if in_range(x, target_x, target_x + 0.3) && self.vel.0 > 0f32 {
                return Some(Direction::LEFT);
            }
            if in_range(x, target_x + target_width - 0.3, target_x + target_width)
                && self.vel.0 < 0f32
            {
                return Some(Direction::LEFT);
            }
        }

        if in_range(x, target_x, target_x + target_width) {
            if in_range(y, target_y, target_y + 0.5f32) {
                return Some(Direction::DOWN);
            }
            if in_range(y, target_y + 0.5, target_y + 1f32) {
                return Some(Direction::UP);
            }
        }

        return None;
    }

    pub fn change_direction(&mut self, dir: Direction) {
        match dir {
            Direction::LEFT | Direction::RIGHT => self.vel.0 *= -1f32,
            Direction::UP | Direction::DOWN => self.vel.1 *= -1f32,
        }
    }
}

enum Direction {
    LEFT,
    RIGHT,
    UP,
    DOWN,
}

struct Peddle {
    pos: (Unit, Unit),
}

impl Drawable for Peddle {
    fn get_pos(&self) -> (Unit, Unit) {
        (self.pos.0, self.pos.1)
    }

    fn get_color(&self) -> &color::Color {
        &color::Red
    }
    fn get_graphics(&self) -> String {
        graphics::PEDDLE_GRAPHIC.to_string()
    }

    fn get_width() -> Unit {
        graphics::PEDDLE_GRAPHIC.chars().count() as Unit
    }
}

impl Peddle {
    pub fn move_in_dir(&mut self, dir: Direction, game_width: Unit) {
        match dir {
            Direction::LEFT => match self.pos.0.checked_sub(Self::get_speed()) {
                None => self.pos.0 = 2,
                _ => self.pos.0 -= Self::get_speed(),
            },
            Direction::RIGHT => self.pos.0 += Peddle::get_speed(),
            _ => panic!("Unexpected direction for peddle"),
        }
        // clamp position
        fn clamp(val: Unit, min: Unit, max: Unit) -> Unit {
            val.max(min).min(max)
        }
        self.pos.0 = clamp(self.pos.0, 2, game_width - Self::get_width());
    }

    pub fn get_speed() -> Unit {
        3 as Unit
    }
}

struct BreakoutGame<R, W> {
    stdin: R,
    stdout: W,

    ball: Ball,
    peddle: Peddle,
    cells: Vec<Cell>,

    height: Unit,
    width: Unit,
}

impl<R: Read, W: Write> BreakoutGame<R, W> {
    pub fn new(stdin: R, stdout: W, width: Unit, height: Unit) -> BreakoutGame<R, W> {
        let (ball, peddle, cells) = Self::get_start_values(width, height);
        BreakoutGame {
            width: width,
            height: height,
            stdin: stdin,
            stdout: stdout,
            ball: ball,
            peddle: peddle,
            cells: cells,
        }
    }

    pub fn get_start_values(width: Unit, height: Unit) -> (Ball, Peddle, Vec<Cell>) {
        let half_peddle_width = Peddle::get_width() / 2 as Unit;
        let peddle_pos = (
            (width as Unit / 2) - half_peddle_width,
            (height - 2) as Unit,
        );

        let ball_pos = ((width as f32) / 2f32 - 10f32, (height as f32) / 1.5f32);

        (
            Ball {
                game_pos: ball_pos,
                vel: (0.3, 0.3),
            },
            Peddle { pos: peddle_pos },
            Self::generate_cell_grid((width, height)),
        )
    }

    pub fn reset_game(&mut self) {
        let (ball, peddle, cells) = Self::get_start_values(self.width, self.height);
        self.ball = ball;
        self.peddle = peddle;
        self.cells = cells;

        write!(self.stdout, "{}{}", clear::All, cursor::Goto(1, 1),).unwrap();
        self.draw_game_borders();
        for cell in &mut self.cells {
            cell.write(&mut self.stdout);
        }
        self.stdout.flush().unwrap();
    }

    pub fn run(&mut self) {
        write!(self.stdout, "{}", cursor::Hide).unwrap();
        self.reset_game(); // Display dummy game scene

        if !self.start_screen() {
            return;
        }

        self.reset_game();
        loop {
            if !self.update() {
                break;
            }

            if !self.ball.update((self.width, self.height), self.peddle.pos) {
                if self.game_over_screen() {
                    self.reset_game();
                } else {
                    break;
                }
            }

            let mut to_kill = vec![];
            for (index, cell) in &mut self.cells.iter().enumerate() {
                let hit_dir = self.ball.collides_with(cell);
                match hit_dir {
                    None => { /***/ }
                    _ => {
                        to_kill.push(index);
                        self.ball.change_direction(hit_dir.unwrap());
                    }
                }
            }
            for i in to_kill {
                self.cells[i].clear(&mut self.stdout);
                self.cells.remove(i);
            }
            if self.cells.is_empty() {
                if self.game_won_screen() {
                    self.reset_game();
                } else {
                    break;
                }
            }

            self.ball.write(&mut self.stdout);
            self.peddle.write(&mut self.stdout);

            self.stdout.flush().unwrap();
            thread::sleep(Duration::from_millis(20));

            self.ball.clear(&mut self.stdout);
            self.peddle.clear(&mut self.stdout);
        }

        writeln!(self.stdout, "{}", cursor::Show).unwrap();
    }

    fn update(&mut self) -> bool {
        let mut key_bytes = [0];
        self.stdin.read(&mut key_bytes).unwrap();

        match key_bytes[0] {
            b'q' => return false,
            b'r' => self.reset_game(),
            b'h' | b'a' => self.peddle.move_in_dir(Direction::LEFT, self.width),
            b'l' | b'd' => self.peddle.move_in_dir(Direction::RIGHT, self.width),
            _ => {}
        }

        true
    }

    fn start_screen(&mut self) -> bool {
        self.yes_no_dialog(graphics::GAME_START, Box::new(color::Blue), ' ', 'q')
    }

    fn game_over_screen(&mut self) -> bool {
        self.yes_no_dialog(graphics::GAME_OVER, Box::new(color::Red), 'r', 'q')
    }

    fn game_won_screen(&mut self) -> bool {
        self.yes_no_dialog(graphics::GAME_WIN, Box::new(color::Green), 'r', 'q')
    }

    fn yes_no_dialog(
        &mut self,
        graphics: &str,
        color: Box<color::Color>,
        yes: char,
        no: char,
    ) -> bool {
        for (index, l) in graphics.lines().enumerate() {
            write!(
                self.stdout,
                "{}{}{}",
                color::Fg(color.as_ref()),
                cursor::Goto(
                    (self.width as u16 / 2) - l.chars().count() as u16 / 2,
                    (self.height as u16 / 2) + index as u16
                ),
                l
            ).unwrap();
        }
        self.stdout.flush().unwrap();

        let mut key_bytes = [0u8];
        loop {
            self.stdin.read(&mut key_bytes).unwrap();
            if key_bytes[0] == yes as u8 {
                return true;
            }
            if key_bytes[0] == no as u8 {
                return false;
            }
        }
    }

    fn draw_game_borders(&mut self) {
        let horizontal_border = graphics::BORDER_HORIZONTAL.repeat(self.width as usize - 2);

        write!(self.stdout, "{}", color::Fg(color::Blue)).unwrap();
        write!(
            self.stdout,
            "{}{}{}{}",
            cursor::Goto(1, 1),
            graphics::TOP_LEFT_BORDER,
            horizontal_border,
            graphics::TOP_RIGHT_BORDER
        ).unwrap();
        for y in 2..(self.height) as u16 {
            write!(
                self.stdout,
                "{}{}",
                cursor::Goto(1, y),
                graphics::BORDER_VERTICAL
            ).unwrap();
            write!(
                self.stdout,
                "{}{}",
                cursor::Goto(self.width as u16, y),
                graphics::BORDER_VERTICAL
            ).unwrap();
        }
        write!(
            self.stdout,
            "{}{}{}{}",
            cursor::Goto(1, self.height as u16),
            graphics::BOTTOM_LEFT_BORDER,
            horizontal_border,
            graphics::BOTTOM_RIGHT_BORDER
        ).unwrap();
    }

    fn generate_cell_grid(game_size: (Unit, Unit)) -> Vec<Cell> {
        let cell_width = Cell::get_width();
        let cell_margin = 0;
        let num_cells_horizontally = game_size.0 / (cell_margin + cell_width) - 2;
        let num_cells_vertically = game_size.1 / 3;

        let vec_capacity = (num_cells_vertically * num_cells_horizontally) as usize;
        let mut cells = Vec::with_capacity(vec_capacity);
        for cy in 0..num_cells_vertically {
            for cx in 0..num_cells_horizontally {
                let mut rng = rand::thread_rng();
                let c = rng.gen_range(0, 5);

                let xpos = cell_width + cx * (cell_width + cell_margin);
                let ypos = 4 + cy;
                cells.push(Cell {
                    pos: (xpos, ypos),
                    color: match c {
                        0 => Box::new(color::Red),
                        1 => Box::new(color::Green),
                        2 => Box::new(color::Blue),
                        4 => Box::new(color::Magenta),
                        _ => Box::new(color::Red),
                    },
                });
            }
        }
        return cells;
    }
}

fn init(width: Unit, height: Unit) {
    let stdout = stdout();
    let stdout = stdout.lock().into_raw_mode().unwrap();
    let stdin = async_stdin();
    let mut game = BreakoutGame::new(stdin, stdout, width, height);
    game.run();
}

fn main() {
    // Store default game size
    let mut width = Cell::get_width() * 13;
    let mut height = 30;

    let mut auto_scale_to_terminal = false;

    {
        // this block limits scope of borrows by ap.refer() method

        let mut ap = ArgumentParser::new();
        ap.set_description("A simple breakout clone built with rust, playable in the terminal");

        ap.refer(&mut width).add_option(
            &["-w", "--width"],
            Store,
            "Preferable game width greater than 32,
                 gets scaled according to the in-game cell width",
        );

        ap.refer(&mut height).add_option(
            &["-h", "--height"],
            Store,
            "Preferable game height greater than 20",
        );

        ap.refer(&mut auto_scale_to_terminal).add_option(
            &["-f", "--fill"],
            StoreTrue,
            "Fill game to current terminal size",
        );

        ap.parse_args_or_exit();
    }

    if auto_scale_to_terminal {
        let terminal_size = termion::terminal_size();
        match terminal_size {
            Ok(size) => {
                width = size.0 as usize;
                height = size.1 as usize;
            }
            Err(e) => println!("Failed to get terminal size with error: {}", e),
        }
    }

    if width < 32 {
        println!("The specified or computed width is too small!");
        return;
    }
    if height < 20 {
        println!("The specified or computed height is too small!");
        return;
    }

    width = (width / Cell::get_width()) * Cell::get_width();
    init(width, height);
}
