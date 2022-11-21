use std::fmt::{self, Display};
use std::io::Write;
use std::{io::stdout, thread::sleep, time::Duration};

use clap::{command, Parser, Subcommand};
use color_eyre::eyre::{eyre, Result};
use crossterm::{cursor::*, csi, Command};
use crossterm::terminal::SetSize;
use crossterm::{
    event::{Event, EventStream, KeyCode, KeyEvent, KeyModifiers},
    execute,
    style::{Print, ResetColor},
    terminal::{self, Clear, ClearType},
};

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
        write!(f, csi!("{};{}r"), self.0, self.1)
    }
}

impl Display for SetMargin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.write_ansi(f)
    }
}

fn main() -> Result<()> {
    let _raw_mode = RawModeGuard::new()?;
    let mut stdout = stdout();

    let (width, height) = terminal::size()?;
    execute!(
        stdout,
        Print("\n"),
        SavePosition,
        SetMargin(0, height - 1),
        RestorePosition,
        MoveUp(1)
    )?;

    for i in 0..20 {
        sleep(Duration::from_millis(50));
        execute!(
            stdout,
            SavePosition,
            Print("Hello wordl!"),
            MoveTo(0, height),
            Print("bottom bar"),
            RestorePosition,
            Print("\n")
        )?;
    }

    execute!(
        stdout,
        SavePosition,
        SetMargin(0, height),
        MoveTo(0, height),
        Clear(ClearType::CurrentLine),
        RestorePosition,
        ResetColor,
    )?;

    Ok(())
}
