use std::sync::Mutex;

use cursive::{
    direction::Orientation,
    menu,
    view::{Nameable, Resizable, ScrollStrategy, Scrollable},
    views::{Dialog, FixedLayout, Layer, OnLayoutView, ScrollView, TextView, TextContentRef, TextContent},
    Printer, Rect, Vec2, View, event::Key, utils::span::SpannedString, theme::Style, Cursive,
};

use ringbuffer::{ConstGenericRingBuffer, RingBuffer, RingBufferExt, RingBufferWrite};

// static buffer : ConstGenericRingBuffer<&str, 2> = ConstGenericRingBuffer::<_, 2>::new();
const BUFFER_SIZE: usize = 64;

struct ScrollableBufferView {
    buffer: Mutex<ConstGenericRingBuffer<String, BUFFER_SIZE>>,
}

impl View for ScrollableBufferView {
    fn draw(&self, printer: &Printer<'_, '_>) {
        let mut buffer = self.buffer.lock().unwrap();
        let start = buffer.len().saturating_sub(printer.size.y);

        // for i in 0..
        let mut y = 0;
        for i in start..buffer.len() {
            let i = i as isize;
            let line = buffer.get(i).unwrap();
            printer.print_line(Orientation::Horizontal, (0, y), line.len(), line);
            y += 1;
        }
        // while let Some(line) = buffer.get_absolute(start + y) {
        //     printer.print_line(Orientation::Horizontal, (0, y), line.len(), line.as_str());
        //     y += 1;
        //     // start += 1;
        //     // println!("test {}", start);
        // }
    }

    fn required_size(&mut self, constraint: Vec2) -> Vec2 {
        Vec2::new(constraint.x, self.buffer.lock().unwrap().len())
    }
}

pub trait StatusBarExt {
    fn status_bar(&mut self, content: impl Into<SpannedString<Style>>) -> TextContent;
    fn get_status_bar_content(&mut self) -> TextContentRef;
    fn set_status_bar_content(&mut self, content: impl Into<SpannedString<Style>>);
}

impl StatusBarExt for Cursive {

    /// Create a new status bar, set to the given content.
    fn status_bar(&mut self, content: impl Into<SpannedString<Style>>) -> TextContent {
        let text_content = TextContent::new(content);
        self.screen_mut()
        .add_transparent_layer(
            OnLayoutView::new(
                FixedLayout::new().child(
                    Rect::from_point(Vec2::zero()),
                    Layer::new(
                        TextView::new_with_content(text_content.clone()).with_name("status"),
                    )
                    .full_width(),
                ),
                |layout, size| {
                    let rect = Rect::from_size((0, size.y - 1), (size.x, 1));
                    layout.set_child_position(0, rect);
                    layout.layout(size);
                },
            )
            .full_screen(),
        );
        text_content
    }

    fn get_status_bar_content(&mut self) -> TextContentRef {
        self.call_on_name("status", |text_view: &mut TextView| {
            text_view.get_content()
        })
        .expect("get_status")
    }

    fn set_status_bar_content(&mut self, content: impl Into<SpannedString<Style>>) {
        self.call_on_name("status", |text_view: &mut TextView| {
            text_view.set_content(content);
        })
        .expect("set_status")
    }

}

fn main() {
    let mut buffer = ConstGenericRingBuffer::<String, BUFFER_SIZE>::new();
    buffer.push("Hello".to_string());
    buffer.push("World".to_string());
    for i in 0..100 {
        buffer.push(format!("test {}", i));
    }
    buffer.push("Hello".to_string());
    buffer.push("World".to_string());

    // Creates the cursive root - required for every application.
    let mut siv = cursive::crossterm();

    siv.add_global_callback('q', |s| s.quit());

    let view = ScrollableBufferView {
        buffer: Mutex::new(buffer),
    }
    .scrollable()
    .scroll_y(true)
    .scroll_x(false)
    .scroll_strategy(ScrollStrategy::StickToBottom);
    // siv.add_layer(view.fixed_height(20));
    // siv.add_layer(
    // DialogAround::new(
    //     Dialog::around(view)
    //         .title("ScrollableBufferView")
    //         .button("Quit", |s| s.quit())
    //         .fixed_size(size),
    // )
    // );

    // siv.screen_mut().add_transparent_layer(
    //     OnLayoutView::new(
    //         FixedLayout::new().child(
    //             Rect::from_point(Vec2::zero()),
    //             Layer::new(TextView::new("Status: unknown").with_name("status")).full_width(),
    //         ),
    //         |layout, size| {
    //             // We could also keep the status bar at the top instead.
    //             layout.set_child_position(0, Rect::from_size((0, size.y - 1), (size.x, 1)));
    //             layout.layout(size);
    //         },
    //     )
    //     .full_screen(),
    // );
    let height = siv.screen_size().y;
    println!("height {}", height);
    siv.add_layer(TextView::new("test").full_width().fixed_height(height - 1));
    siv.status_bar("test");


    // siv.add_layer(view);
    // siv.menubar().
    //     .add_subtree(
    //         "Help",
    //         menu::Tree::new()
    //             .subtree(
    //                 "Help",
    //                 menu::Tree::new()
    //                     .leaf("General", |s| s.add_layer(Dialog::info("Help message!")))
    //                     .leaf("Online", |s| {
    //                         let text = "Google it yourself!\n\
    //                                     Kids, these days...";
    //                         s.add_layer(Dialog::info(text))
    //                     }),
    //             )
    //             .leaf("About", |s| s.add_layer(Dialog::info("Cursive v0.0.0"))),
    //     )
    //     .add_delimiter()
    //     .add_leaf("Quit", |s| s.quit());

    // siv.add_global_callback(Key::Esc, |s| s.select_menubar());

    // siv.add_layer(Dialog::text("Hit <Esc> to show the menu!"));

    // Starts the event loop.
    siv.run();
}
