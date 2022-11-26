use std::sync::Mutex;

use cursive::views::ScrollView;
use cursive::{CursiveRunnable};
use cursive::traits::With;
use cursive::{
    direction::Orientation,
    event::Event,
    theme::{Palette, Theme},
    view::{Resizable, ScrollStrategy},
    views::{LinearLayout, TextView},
    Cursive, Printer, Vec2, View,
};

use ringbuffer::{ConstGenericRingBuffer, RingBuffer, RingBufferExt, RingBufferWrite};

const BUFFER_SIZE: usize = 64;

struct BufferView<const CAP: usize> {
    buffer: Mutex<ConstGenericRingBuffer<String, CAP>>,
}

impl<const CAP: usize> BufferView<CAP> {
    pub fn new(buffer: ConstGenericRingBuffer<String, CAP>) -> Self {
        BufferView {
            buffer: Mutex::new(buffer),
        }
    }

    pub fn scrollable(self) -> ScrollView<Self> {
        ScrollView::new(self)
            .scroll_y(true)
            .scroll_x(false)
            .scroll_strategy(ScrollStrategy::StickToBottom)
    }
}

impl<const CAP: usize> View for BufferView<CAP> {
    fn draw(&self, printer: &Printer<'_, '_>) {
        let buffer = self.buffer.lock().unwrap();
        let start = buffer.len().saturating_sub(printer.size.y);

        for y in start..buffer.len() {
            let line = buffer.get(y as isize).unwrap();
            printer.print_line(Orientation::Horizontal, (0, y), line.len(), line);
        }
    }

    fn required_size(&mut self, constraint: Vec2) -> Vec2 {
        Vec2::new(constraint.x, self.buffer.lock().unwrap().len())
    }
}

fn main() {
    let mut buffer = ConstGenericRingBuffer::<String, BUFFER_SIZE>::new();

    buffer.push(
        ansi_term::Colour::Red
            .bold()
            .paint("Press CTRL+c to exit.")
            .to_string(),
    );

    let mut console = cursive::crossterm().use_custom_theme().install_exit_callback();

    let sbv = BufferView::new(buffer).scrollable();
    console.add_layer(
        LinearLayout::vertical()
            .child(sbv.full_screen())
            .child(TextView::new("testetesehnte").full_width()),
    );

    console.run();
}

pub trait CustomThemeExt {
    fn use_custom_theme(self) -> CursiveRunnable;
}

impl CustomThemeExt for CursiveRunnable {
    fn use_custom_theme(mut self) -> CursiveRunnable {
        self.set_theme(Theme {
            shadow: false,
            borders: cursive::theme::BorderStyle::None,
            palette: Palette::default().with(|palette| {
                use cursive::theme::BaseColor::*;
                use cursive::theme::Color::TerminalDefault;
                use cursive::theme::PaletteColor::*;

                palette[Background] = TerminalDefault;
                palette[View] = TerminalDefault;
                palette[Primary] = White.dark();
                palette[TitlePrimary] = Blue.light();
                palette[Secondary] = Blue.light();
                palette[Highlight] = Blue.dark();
            }),
        });
        self
    }
}

pub trait ExitCallbackExt {
    fn install_exit_callback(self) -> CursiveRunnable;
}

impl ExitCallbackExt for CursiveRunnable {
    fn install_exit_callback(mut self) -> CursiveRunnable {
        self.add_global_callback(Event::CtrlChar('c'), Cursive::quit);
        self
    }
}
