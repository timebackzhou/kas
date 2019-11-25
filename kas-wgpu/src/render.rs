// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License in the LICENSE-APACHE file or at:
//     https://www.apache.org/licenses/LICENSE-2.0

//! Widget rendering

use std::f32;

use kas::geom::{AxisInfo, Margins, Size, SizeRules};
use kas::{event, TkAction, TkWindow, Widget};

use crate::draw::DrawPipe;
use crate::theme::Theme;

/// Widget renderer
pub(crate) struct Widgets<T> {
    draw_pipe: DrawPipe,
    action: TkAction,
    pub(crate) ev_mgr: event::Manager,
    theme: T,
}

impl<T: Theme<DrawPipe>> Widgets<T> {
    pub fn new(
        device: &mut wgpu::Device,
        tex_format: wgpu::TextureFormat,
        size: Size,
        dpi_factor: f64,
        mut theme: T,
    ) -> Self {
        let draw_pipe = DrawPipe::new(device, tex_format, theme.get_fonts(), size);
        theme.set_dpi_factor(dpi_factor as f32);

        Widgets {
            draw_pipe,
            action: TkAction::None,
            ev_mgr: event::Manager::new(dpi_factor),
            theme,
        }
    }

    pub fn set_dpi_factor(&mut self, dpi_factor: f64) {
        self.ev_mgr.set_dpi_factor(dpi_factor);
        self.theme.set_dpi_factor(dpi_factor as f32);
        // Note: we rely on caller to resize widget
    }

    pub fn resize(&mut self, device: &wgpu::Device, size: Size) -> wgpu::CommandBuffer {
        self.draw_pipe.resize(device, size)
    }

    #[inline]
    pub fn pop_action(&mut self) -> TkAction {
        let action = self.action;
        self.action = TkAction::None;
        action
    }

    pub fn draw(
        &mut self,
        device: &mut wgpu::Device,
        frame_view: &wgpu::TextureView,
        win: &dyn kas::Window,
    ) -> wgpu::CommandBuffer {
        self.draw_iter(win.as_widget());
        self.draw_pipe.render(device, frame_view)
    }

    fn draw_iter(&mut self, widget: &dyn kas::Widget) {
        // draw widget; recurse over children
        self.draw_widget(widget);

        for n in 0..widget.len() {
            self.draw_iter(widget.get(n).unwrap());
        }
    }

    fn draw_widget(&mut self, widget: &dyn kas::Widget) {
        self.theme.draw(&mut self.draw_pipe, &self.ev_mgr, widget)
    }
}

impl<T: Theme<DrawPipe>> TkWindow for Widgets<T> {
    fn data(&self) -> &event::Manager {
        &self.ev_mgr
    }

    fn update_data(&mut self, f: &mut dyn FnMut(&mut event::Manager) -> bool) {
        if f(&mut self.ev_mgr) {
            self.send_action(TkAction::Redraw);
        }
    }

    fn size_rules(&mut self, widget: &dyn Widget, axis: AxisInfo) -> SizeRules {
        self.theme.size_rules(&mut self.draw_pipe, widget, axis)
    }

    fn margins(&self, widget: &dyn Widget) -> Margins {
        self.theme.margins(widget)
    }

    #[inline]
    fn redraw(&mut self, _: &dyn Widget) {
        self.send_action(TkAction::Redraw);
    }

    #[inline]
    fn send_action(&mut self, action: TkAction) {
        self.action = self.action.max(action);
    }
}
