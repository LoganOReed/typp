use crate::Terminal;
use termion::event::Key;

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct Editor {
    should_quit: bool,
    terminal: Terminal,
}

impl Editor {
    // default constructor
    pub fn default() -> Self {
        Self {
            should_quit: false,
            terminal: Terminal::default().expect("Failed to init terminal."),
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
            _ => (),
        }
        Ok(())
    }

    fn refresh_screen(&self) -> Result<(), std::io::Error> {
        // handles crashing mid screen refresh
        Terminal::cursor_hide();
        Terminal::cursor_position(0, 0);
        if self.should_quit {
            Terminal::clear_screen();
            println!("Goodbye.\r");
        } else {
            self.draw_rows();
            Terminal::cursor_position(0, 0);
        }
        Terminal::cursor_show();
        Terminal::flush()
    }

    fn draw_rows(&self) {
        let height = self.terminal.size().height;
        for row in 0..height - 1 {
            Terminal::clear_current_line();
            if row == height / 3 {
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
