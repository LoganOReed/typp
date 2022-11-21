use crate::Document;
use crate::Row;
use crate::Terminal;
use std::env;
use termion::event::Key;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Default)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

pub struct Editor {
    should_quit: bool,
    terminal: Terminal,
    cursor_position: Position,
    offset: Position,
    document: Document,
}

impl Editor {
    // default constructor
    pub fn default() -> Self {
        let args: Vec<String> = env::args().collect();
        let document = if args.len() > 1 {
            let file_name = &args[1];
            Document::open(&file_name).unwrap_or_default()
        } else {
            Document::default()
        };

        Self {
            should_quit: false,
            terminal: Terminal::default().expect("Failed to init terminal."),
            document,
            cursor_position: Position::default(),
            offset: Position::default(),
        }
    }

    // main loop
    pub fn run(&mut self) {
        loop {
            if let Err(error) = self.refresh_screen() {
                die(&error);
            }
            if self.should_quit {
                break;
            }
            if let Err(error) = self.process_keypress() {
                die(&error);
            }
        }
    }
    fn process_keypress(&mut self) -> Result<(), std::io::Error> {
        let pressed_key = Terminal::read_key()?;
        match pressed_key {
            Key::Ctrl('q') => self.should_quit = true,
            Key::Char('k')
            | Key::Char('j')
            | Key::Char('h')
            | Key::Char('l')
            | Key::Char('H')
            | Key::Char('L')
            | Key::Char('M')
            | Key::Char('0')
            | Key::Char('$') => self.move_cursor(pressed_key),
            _ => (),
        }
        self.scroll();
        Ok(())
    }

    fn scroll(&mut self) {
        let Position { x, y } = self.cursor_position;
        let width = self.terminal.size().width as usize;
        let height = self.terminal.size().height as usize;
        let padding = height / 3;
        let mut offset = &mut self.offset;
        // keep a 1/3 of the screen as padding when scrolling
        // like vim
        if y == 0 {}
        if y < offset.y + padding {
            offset.y = y.saturating_sub(padding);
        } else if y >= offset.y.saturating_add(height - padding) {
            offset.y = y.saturating_sub(height - padding);
        }
        if x < offset.x {
            offset.x = x;
        } else if x >= offset.x.saturating_add(width) {
            offset.x = x.saturating_sub(width).saturating_add(1);
        }
    }

    fn move_cursor(&mut self, key: Key) {
        let Position { mut y, mut x } = self.cursor_position;
        let size = self.terminal.size();
        let height = self.document.len();
        let width = size.width.saturating_sub(1) as usize;
        match key {
            Key::Char('k') => y = y.saturating_sub(1),
            Key::Char('j') => {
                if y < height {
                    y = y.saturating_add(1);
                }
            }
            Key::Char('h') => x = x.saturating_sub(1),
            Key::Char('l') => {
                if x < width {
                    x = x.saturating_add(1);
                }
            }
            Key::Char('H') => y = 0,
            Key::Char('L') => y = height,
            Key::Char('M') => y = height / 2,
            Key::Char('0') => x = 0,
            Key::Char('$') => x = width,

            _ => (),
        }
        self.cursor_position = Position { x, y }
    }

    fn refresh_screen(&self) -> Result<(), std::io::Error> {
        // handles crashing mid screen refresh
        Terminal::cursor_hide();
        Terminal::cursor_position(&Position::default());
        if self.should_quit {
            Terminal::clear_screen();
            println!("Goodbye.\r");
        } else {
            self.draw_rows();
            Terminal::cursor_position(&Position {
                x: self.cursor_position.x.saturating_sub(self.offset.x),
                y: self.cursor_position.y.saturating_sub(self.offset.y),
            });
        }
        Terminal::cursor_show();
        Terminal::flush()
    }

    pub fn draw_row(&self, row: &Row) {
        let width = self.terminal.size().width as usize;
        let start = self.offset.x;
        let end = self.offset.x + width;
        let row = row.render(start, end);
        println!("{}\r", row)
    }

    fn draw_rows(&self) {
        let height = self.terminal.size().height;
        for terminal_row in 0..height - 1 {
            Terminal::clear_current_line();
            if let Some(row) = self.document.row(terminal_row as usize + self.offset.y) {
                self.draw_row(row);
            } else if self.document.is_empty() && terminal_row == height / 3 {
                self.draw_welcome_message();
            } else {
                println!("~\r");
            }
        }
    }

    fn draw_welcome_message(&self) {
        let mut welcome_message = format!("TyPP -- version {}", VERSION);
        let width = self.terminal.size().width as usize;
        let buf = width.saturating_sub(welcome_message.len()) / 2;
        let spaces = " ".repeat(buf.saturating_sub(1));
        welcome_message = format!("~{}{}", spaces, welcome_message);
        welcome_message.truncate(width);
        println!("{}\r", welcome_message);
    }
}

fn die(e: &std::io::Error) {
    panic!("Problem reading input: {:?}", e);
}
