use std::fmt::{self, Display};
use std::io::Write;
use std::ops::Sub;
use std::{io::stdout, thread::sleep, time::Duration};

use color_eyre::eyre::{eyre, Result};
use crossterm::style::SetBackgroundColor;
use crossterm::{csi, cursor::*, queue, Command, QueueableCommand, execute};
use crossterm::{
    style::*,
    terminal::{self, Clear, ClearType},
};
use unicode_width::UnicodeWidthStr;


// queue commands inside a command
macro_rules! queuec {
    ($f:expr $(, $command:expr)* $(,)?) => {{
        $($command.write_ansi($f)?;)*
    }}
}


// inspired by esp-rs/espflash
struct RawModeGuard;

impl RawModeGuard {
    pub fn new() -> Result<Self> {
        crossterm::terminal::enable_raw_mode()?;
        Ok(RawModeGuard)
    }
}

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        if let Err(e) = crossterm::terminal::disable_raw_mode() {
            eprintln!("Failed to disable raw mode: {}", e);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SetMargin(pub u16, pub u16);

impl Command for SetMargin {
    fn write_ansi(&self, f: &mut impl fmt::Write) -> fmt::Result {
        // set scroll region (this will place the cursor in the top left)
        // sets the first line of content and the last line of content
        write!(f, csi!("{};{}r"), self.0, self.1)
    }
}

impl Display for SetMargin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.write_ansi(f)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Statusbar(u16, String);

impl Command for Statusbar {
    fn write_ansi(&self, f: &mut impl fmt::Write) -> fmt::Result {

        let w = self.0 as usize;

        let left_text = self.1.clone();
        let right_text = "this is a long text  ";

        let left_length = UnicodeWidthStr::width(left_text.as_str());
        let right_length = UnicodeWidthStr::width(right_text);

        queuec!(
            f,
            SetBackgroundColor(Color::Blue),
            SetForegroundColor(Color::White),
            Print(format!("{:<l$}{:>r$}", left_text, right_text, l = w - (left_length + right_length), r = right_length)),
            ResetColor,
        );

        Ok(())
    }
}

impl Display for Statusbar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.write_ansi(f)
    }
}

struct FancyTerm<W: std::io::Write> {
    write: W,
    original_height: u16,
}

impl<W: std::io::Write> FancyTerm<W> {
    pub fn new(mut write: W) -> Result<Self> {
        let (width, height) = terminal::size()?;

        // clear
        for _ in 0..height.saturating_sub(1u16) {
            queue!(write, Print("\n"))?;
        }
        write.flush()?;

        execute!(
            write,
            // SavePosition,
            MoveTo(0,0),
            SetMargin(2, height - 1),
            // RestorePosition,
            // MoveUp(1)
            MoveTo(0, 1),
        )?;

        Ok(Self {
            write,
            original_height: height,
        })
    }

    pub fn write(&mut self, text: String) -> Result<()> {
        let (width, height) = terminal::size()?;

        execute!(
            self.write,
            // print text
            SavePosition,
            Print(text),
            RestorePosition,
            Print("\n"),
            // render statusbars
            SavePosition,
            MoveTo(0, 0),
            Statusbar(width, "top".to_string()),
            MoveTo(0, height),
            Statusbar(width, "bottom".to_string()),
            RestorePosition
        )?;
        Ok(())
    }

    fn clear(&mut self) -> Result<()> {
        let (_, height) = terminal::size()?;
        execute!(
            self.write,
            SavePosition,
            SetMargin(0, height),
            // delete top statusbar
            MoveTo(0, 0),
            Clear(ClearType::CurrentLine),
            // delete bottom statusbar
            MoveTo(0, self.original_height),
            Clear(ClearType::CurrentLine),
            RestorePosition,
            ResetColor,
        )?;
        Ok(())
    }
}

impl<W: std::io::Write> Drop for FancyTerm<W> {
    fn drop(&mut self) {
        self.clear().expect("Cannot clear terminal");
    }
}

fn main() -> Result<()> {
    let _raw_mode = RawModeGuard::new()?;

    let mut out = stdout();
    let mut _term = FancyTerm::new(&mut out)?;

    // let mut stdoutt = stdout();

    for i in 0..30 {
        sleep(Duration::from_millis(150));
        _term.write(format!("test: {}", i))?;
    }

    Ok(())
}
