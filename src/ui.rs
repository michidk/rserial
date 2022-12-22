// use bytes::Bytes;
use color_eyre::eyre::{Result};
use crossbeam_channel::{Receiver, Sender};
use cursive::event::{EventResult, Key};
use cursive::traits::With;
use cursive::view::Nameable;
use cursive::views::{EditView, ScrollView};
use cursive::{
    direction::Orientation,
    event::Event,
    theme::{Palette, Theme},
    view::{Resizable, ScrollStrategy},
    views::{LinearLayout, TextView},
    Cursive, Printer, Vec2, View,
};
use cursive::{CursiveExt, CursiveRunnable};
use ringbuffer::{ConstGenericRingBuffer, RingBuffer, RingBufferExt, RingBufferWrite};
use std::sync::{Arc, Mutex};
use std::thread;
use crate::Bytes;

struct BufferView<const CAP: usize> {
    pub buffer: Arc<Mutex<ConstGenericRingBuffer<Bytes, CAP>>>,
}

impl<const CAP: usize> BufferView<CAP> {
    pub fn new(buffer: Arc<Mutex<ConstGenericRingBuffer<Bytes, CAP>>>) -> Self {
        BufferView { buffer }
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
            let line = std::str::from_utf8(line).unwrap();
            printer.print_line(Orientation::Horizontal, (0, y), line.len(), line);
        }
    }

    fn required_size(&mut self, constraint: Vec2) -> Vec2 {
        Vec2::new(constraint.x, self.buffer.lock().unwrap().len())
    }
}

const BUFFER_SIZE: usize = 64;

pub fn start(rx: Receiver<Bytes>, tx: Sender<Bytes>) -> Result<()> {
    // RC would be enough here?
    let buffer = Arc::new(Mutex::new(
        ConstGenericRingBuffer::<Bytes, BUFFER_SIZE>::new(),
    ));

    // TODO: apparently the style is lost when transmitting the bytes over a channel
    let text = ansi_term::Colour::Red.bold().paint("Press CTRL+c to exit.");
    let text = text.as_bytes().to_vec();
    buffer.lock().unwrap().push(text);
    let mut console = cursive::crossterm()
        .use_custom_theme()
        .install_exit_callback();

    let sbv = BufferView::new(buffer.clone()).scrollable();

    let buffer_clone = buffer.clone();

    let mut command_line = EditView::new();
    command_line = command_line.on_submit_mut(move |cursive, text| {
        let bytes = (format!("{}\n", text)).as_bytes().to_vec();
        buffer_clone.clone().lock().unwrap().push(bytes.clone());
        tx.send(bytes).expect("Could not send bytes to channel");
        cursive.call_on_name("command_line", |v: &mut EditView| v.set_content(""));
    });

    console.add_layer(
        LinearLayout::vertical()
            .child(sbv.full_screen().with_name("buffer"))
            .child(command_line.with_name("command_line"))
            .child(TextView::new("testetesehnte").full_width()),
    );

    let cb_sink = console.cb_sink().clone();
    thread::spawn(move || loop {
        while let Ok(bytes) = rx.recv() {
            buffer.lock().unwrap().push(bytes);

            // TODO: make error handling more readable
            match cb_sink.send(Box::new(|s: &mut Cursive| {
                match s.call_on_name(
                    "buffer",
                    |v: &mut ScrollView<BufferView<BUFFER_SIZE>>| match v.on_event(Event::Refresh) {
                        EventResult::Consumed(_) => {}
                        EventResult::Ignored => {
                            log::error!("Error refreshing view: {}", v.type_name())
                        }
                    },
                ) {
                    Some(_) => {}
                    None => log::error!("Error refreshing view: buffer"),
                }
            })) {
                Ok(_) => {}
                Err(e) => log::error!("Error refreshing view: {}", e),
            }
        }
    });

    console.run_crossterm()?;
    Ok(())
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
        self.add_global_callback(Key::Esc, Cursive::quit);
        self
    }
}
