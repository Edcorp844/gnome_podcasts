use adw::prelude::*;
use relm4::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct ProgressBar {
    fraction: Rc<RefCell<f64>>,
    interactive: bool,
}

#[derive(Debug)]
pub enum ProgressBarInput {
    SetFraction(f64),
    HandleScrub(f64),
}

#[derive(Debug)]
pub enum ProgressBarOutput {
    FractionChanged(f64),
}

// Initialization parameters configuration struct
pub struct ProgressBarInit {
    pub initial_fraction: f64,
    pub interactive: bool,
}

#[relm4::component(pub)]
impl Component for ProgressBar {
    type Init = ProgressBarInit;
    type Input = ProgressBarInput;
    type Output = ProgressBarOutput;
    type CommandOutput = ();

    view! {
        gtk::DrawingArea {
            set_hexpand: true,
            set_vexpand: true,

            // Attaching gesture controllers right inside the UI layout macro loop
            add_controller = gtk::GestureClick {
                set_button: gtk::gdk::BUTTON_PRIMARY,
                // Fires immediately when a user clicks anywhere on the bar track
                connect_pressed[sender, model] => move |gesture, _, x, _| {
                    if model.interactive {
                        if let Some(widget) = gesture.widget() {
                            let width = widget.width() as f64;
                            if width > 0.0 {
                                sender.input(ProgressBarInput::HandleScrub(x / width));
                            }
                        }
                    }
                }
            },

            add_controller = gtk::GestureDrag {
            set_button: gtk::gdk::BUTTON_PRIMARY,
            // Calculates continuous updates while pulling/dragging across the bar width
            connect_drag_update[sender, model] => move |gesture, offset_x, _| {
                if model.interactive {
                    if let Some(widget) = gesture.widget() {
                        let width = widget.width() as f64;

                        if let Some((start_x, _)) = gesture.start_point() {
                            let target_x = start_x + offset_x;
                            if width > 0.0 {
                                sender.input(ProgressBarInput::HandleScrub(target_x / width));
                            }
                        }
                    }
                }
            }
        }

        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let fraction_data = Rc::new(RefCell::new(init.initial_fraction.clamp(0.0, 1.0)));
        let model = ProgressBar {
            fraction: fraction_data.clone(),
            interactive: init.interactive,
        };

        let widgets = view_output!();

        root.set_draw_func(move |widget, cr, width, height| {
            let fraction = *fraction_data.borrow();

            let w = width as f64;
            let h = height as f64;

            // Dynamically scale the rounded track corner radius to the height of the bar
            let radius = h / 2.0;

            if w <= 0.0 || h <= 0.0 {
                return;
            }

            // A clean custom helper to trace a rounded rectangle path matching track layout bounds
            let draw_rounded_rect =
                |cairo_ctx: &gtk::cairo::Context, x: f64, y: f64, rect_w: f64, rect_h: f64| {
                    if rect_w <= 0.0 {
                        return;
                    }
                    cairo_ctx.new_sub_path();
                    // Top Right, Bottom Right, Bottom Left, Top Left corners arc segments tracing loop
                    cairo_ctx.arc(
                        x + rect_w - radius,
                        y + radius,
                        radius,
                        -std::f64::consts::PI / 2.0,
                        0.0,
                    );
                    cairo_ctx.arc(
                        x + rect_w - radius,
                        y + rect_h - radius,
                        radius,
                        0.0,
                        std::f64::consts::PI / 2.0,
                    );
                    cairo_ctx.arc(
                        x + radius,
                        y + rect_h - radius,
                        radius,
                        std::f64::consts::PI / 2.0,
                        std::f64::consts::PI,
                    );
                    cairo_ctx.arc(
                        x + radius,
                        y + radius,
                        radius,
                        std::f64::consts::PI,
                        3.0 * std::f64::consts::PI / 2.0,
                    );
                    cairo_ctx.close_path();
                };

            // 1. LOOK UP DYNAMIC SYSTEM ACCENT COLORS
            let context = widget.style_context();
            let accent_rgba = context
                .lookup_color("accent_color")
                .unwrap_or_else(|| context.color());

            // 2. DRAW BACKGROUND PROGRESS TRACK BAR (Muted background track rail)
            cr.set_source_rgba(
                accent_rgba.red() as f64,
                accent_rgba.green() as f64,
                accent_rgba.blue() as f64,
                0.15,
            );
            draw_rounded_rect(&cr, 0.0, 0.0, w, h);
            let _ = cr.fill();

            // 3. DRAW FOREGROUND FILLED ACTIVE BAR
            if fraction > 0.0 {
                // Ensure active fill width doesn't squeeze smaller than the drawing radius diameter bounds
                let fill_width = (w * fraction).max(radius * 2.0);

                cr.set_source_rgba(
                    accent_rgba.red() as f64,
                    accent_rgba.green() as f64,
                    accent_rgba.blue() as f64,
                    accent_rgba.alpha() as f64,
                );
                draw_rounded_rect(&cr, 0.0, 0.0, fill_width, h);
                let _ = cr.fill();
            }
        });

        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        _widgets: &mut Self::Widgets,
        message: Self::Input,
        sender: ComponentSender<Self>,
        root: &Self::Root,
    ) {
        match message {
            ProgressBarInput::SetFraction(f) => {
                if f.is_nan() || f.is_infinite() {
                    return;
                }
                let clamped = f.clamp(0.0, 1.0);

                if let Ok(mut guard) = self.fraction.try_borrow_mut() {
                    *guard = clamped;
                }
                root.queue_draw();
            }
            ProgressBarInput::HandleScrub(pct) => {
                if pct.is_nan() || pct.is_infinite() {
                    return;
                }
                let clamped = pct.clamp(0.0, 1.0);

                if let Ok(mut guard) = self.fraction.try_borrow_mut() {
                    *guard = clamped;
                }
                root.queue_draw();

                // Broadcast the scrub position change event instantly to the parent component
                let _ = sender.output(ProgressBarOutput::FractionChanged(clamped));
            }
        }
    }
}
