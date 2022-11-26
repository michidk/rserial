use std::borrow::Cow;
use std::fmt::{self, Display};
use std::{io::stdout, thread::sleep, time::Duration};

use color_eyre::eyre::Result;
use crossterm::style::SetBackgroundColor;
use crossterm::{csi, cursor::*, execute, queue, Command};
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TermSize(u16, u16);

impl TermSize {
    pub fn width(&self) -> u16 {
        self.0
    }
    pub fn height(&self) -> u16 {
        self.1
    }
}

impl From<(u16, u16)> for TermSize {
    fn from((width, height): (u16, u16)) -> Self {
        Self(width, height)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Position {
    Top,
    Bottom,
}

impl Position {
    fn to_command(self, TermSize(_, height): &TermSize) -> MoveTo {
        match self {
            Self::Top => MoveTo(0, 0),
            Self::Bottom => MoveTo(0, *height),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Statusbar<'a>(&'a TermSize, Position, Cow<'a, str>);

impl<'a> Statusbar<'a> {
    pub fn new(term_size: &'a TermSize, position: Position, text: impl Into<Cow<'a, str>>) -> Self {
        Self(term_size, position, text.into())
    }

    pub fn top(term_size: &'a TermSize, text: impl Into<Cow<'a, str>>) -> Self {
        Self::new(term_size, Position::Top, text.into())
    }

    pub fn bottom(term_size: &'a TermSize, text: impl Into<Cow<'a, str>>) -> Self {
        Self::new(term_size, Position::Bottom, text.into())
    }
}

impl Command for Statusbar<'_> {
    fn write_ansi(&self, f: &mut impl fmt::Write) -> fmt::Result {
        let w = self.0.width() as usize;
        let position = &self.1;

        let left_text = &self.2;
        let right_text = "this is a long text  ";

        let left_length = UnicodeWidthStr::width(left_text.as_ref());
        let right_length = UnicodeWidthStr::width(right_text);

        let margin_left = w - (left_length + right_length);
        let margin_right = right_length;

        queuec!(
            f,
            SavePosition,
            position.to_command(self.0),
            SetBackgroundColor(Color::Blue),
            SetForegroundColor(Color::White),
            Print(format!(
                "{:<l$}{:>r$}",
                left_text,
                right_text,
                l = margin_left,
                r = margin_right
            )),
            ResetColor,
            RestorePosition,
        );

        Ok(())
    }
}

impl Display for Statusbar<'_> {
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
        let (_, height) = terminal::size()?;

        // clear
        for _ in 0..height.saturating_sub(1u16) {
            queue!(write, Print("\n"))?;
        }
        write.flush()?;

        execute!(
            write,
            // SavePosition,
            MoveTo(0, 0),
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
        let term_size = terminal::size()?.into();

        execute!(
            self.write,
            // print text
            SavePosition,
            Print(text),
            RestorePosition,
            Print("\n"),
            // render statusbars
            // Statusbar::top(&term_size, "top"),
            Statusbar::bottom(&term_size, "bottom"),
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
