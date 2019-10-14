// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License in the LICENSE-APACHE file or at:
//     https://www.apache.org/licenses/LICENSE-2.0

//! Counter example (simple button)
#![feature(proc_macro_hygiene)]

use kas::control::TextButton;
use kas::text::Label;
use kas::event::Response;
use kas::macros::make_widget;
use kas::HasText;
use kas::{SimpleWindow, TkWidget};

#[derive(Debug)]
enum Message {
    Decr,
    Incr,
}

fn main() -> Result<(), winit::error::OsError> {
    let buttons = make_widget!(
        container(horizontal) => Message;
        struct {
            #[widget] _ = TextButton::new_on("−", || Message::Decr),
            #[widget] _ = TextButton::new_on("+", || Message::Incr),
        }
    );
    let window = SimpleWindow::new(make_widget!(
        container(vertical) => ();
        struct {
            #[widget] display: Label = Label::from("0"),
            #[widget(handler = handle_button)] buttons -> Message = buttons,
            counter: usize = 0,
        }
        impl {
            fn handle_button(&mut self, tk: &mut dyn TkWidget, msg: Message)
                -> Response<()>
            {
                match msg {
                    Message::Decr => {
                        self.counter = self.counter.saturating_sub(1);
                        self.display.set_text(tk, self.counter.to_string());
                    }
                    Message::Incr => {
                        self.counter = self.counter.saturating_add(1);
                        self.display.set_text(tk, self.counter.to_string());
                    }
                };
                Response::None
            }
        }));

    let mut toolkit = kas_rgx::Toolkit::new();
    toolkit.add(window)?;
    toolkit.run()
}