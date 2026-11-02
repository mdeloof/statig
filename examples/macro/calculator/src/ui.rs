use eframe::egui::{self, Button, Label, Response, RichText, Style, Visuals, WidgetText};
use eframe::egui::{Layout, Ui};
use eframe::emath::Align;
use eframe::epaint::{FontFamily, FontId};
use statig::blocking::InitializedStateMachine;

use crate::state::{Calculator, Event, Operator};

pub struct App(pub InitializedStateMachine<Calculator>);

impl std::ops::Deref for App {
    type Target = InitializedStateMachine<Calculator>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for App {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl App {
    /// Render the display
    fn display(&self, ui: &mut Ui) -> Response {
        let text = &self.display;

        ui.add_sized((ui.available_width(), 40.0), |ui: &mut Ui| {
            ui.with_layout(Layout::right_to_left(Align::LEFT), |ui: &mut Ui| {
                ui.add(Label::new(rich_text(text)))
            })
            .response
        })
    }

    /// Render the numpad
    fn numpad(&mut self, ui: &mut Ui) -> Response {
        let button_spacing = 4.0;
        let button_width = (ui.available_width() - 2.0 * button_spacing) / 3.0;
        let button_heigth = (ui.available_height() - 2.0 * button_spacing) / 3.0;

        ui.spacing_mut().item_spacing = (button_spacing, button_spacing).into();
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.add_sized((button_width, button_heigth), Button::new(rich_text("7")))
                    .clicked()
                    .then(|| self.handle_with_context(&Event::Digit { digit: 7 }, &mut ()));
                ui.add_sized((button_width, button_heigth), Button::new(rich_text("8")))
                    .clicked()
                    .then(|| self.handle_with_context(&Event::Digit { digit: 8 }, &mut ()));
                ui.add_sized((button_width, button_heigth), Button::new(rich_text("9")))
                    .clicked()
                    .then(|| self.handle_with_context(&Event::Digit { digit: 9 }, &mut ()));
            });
            ui.horizontal(|ui| {
                ui.add_sized((button_width, button_heigth), Button::new(rich_text("4")))
                    .clicked()
                    .then(|| self.handle_with_context(&Event::Digit { digit: 4 }, &mut ()));
                ui.add_sized((button_width, button_heigth), Button::new(rich_text("5")))
                    .clicked()
                    .then(|| self.handle_with_context(&Event::Digit { digit: 5 }, &mut ()));
                ui.add_sized((button_width, button_heigth), Button::new(rich_text("6")))
                    .clicked()
                    .then(|| self.handle_with_context(&Event::Digit { digit: 6 }, &mut ()));
            });
            ui.horizontal(|ui| {
                ui.add_sized((button_width, button_heigth), Button::new(rich_text("1")))
                    .clicked()
                    .then(|| self.handle_with_context(&Event::Digit { digit: 1 }, &mut ()));
                ui.add_sized((button_width, button_heigth), Button::new(rich_text("2")))
                    .clicked()
                    .then(|| self.handle_with_context(&Event::Digit { digit: 2 }, &mut ()));
                ui.add_sized((button_width, button_heigth), Button::new(rich_text("3")))
                    .clicked()
                    .then(|| self.handle_with_context(&Event::Digit { digit: 3 }, &mut ()));
            })
        })
        .response
    }

    /// Render the operands
    fn operands(&mut self, ui: &mut Ui) -> Response {
        let button_spacing = 4.0;
        let button_width = ui.available_width();
        let button_height = (ui.available_height() - 3.0 * button_spacing) / 4.0;

        ui.spacing_mut().item_spacing = (button_spacing, button_spacing).into();
        ui.vertical(|ui| {
            ui.add_sized((button_width, button_height), Button::new(rich_text("÷")))
                .clicked()
                .then(|| {
                    self.handle_with_context(
                        &Event::Operator {
                            operator: Operator::Div,
                        },
                        &mut (),
                    )
                });
            ui.add_sized((button_width, button_height), Button::new(rich_text("×")))
                .clicked()
                .then(|| {
                    self.handle_with_context(
                        &Event::Operator {
                            operator: Operator::Mul,
                        },
                        &mut (),
                    )
                });
            ui.add_sized((button_width, button_height), Button::new(rich_text("-")))
                .clicked()
                .then(|| {
                    self.handle_with_context(
                        &Event::Operator {
                            operator: Operator::Sub,
                        },
                        &mut (),
                    )
                });
            ui.add_sized((button_width, button_height), Button::new(rich_text("+")))
                .clicked()
                .then(|| {
                    self.handle_with_context(
                        &Event::Operator {
                            operator: Operator::Add,
                        },
                        &mut (),
                    )
                });
        })
        .response
    }

    /// Render the controls
    fn control(&mut self, ui: &mut Ui) -> Response {
        let button_spacing = 4.0;
        let button_width = (ui.available_width() - 2.0 * button_spacing) / 3.0;
        let button_width_2x = button_width * 2.0 + button_spacing;
        let button_height = ui.available_height();

        ui.spacing_mut().item_spacing = (button_spacing, button_spacing).into();
        ui.horizontal(|ui| {
            ui.add_sized(
                (button_width_2x, button_height),
                Button::new(rich_text("CE")),
            )
            .clicked()
            .then(|| self.handle_with_context(&Event::Ce, &mut ()));
            ui.add_sized((button_width, button_height), Button::new(rich_text("AC")))
                .clicked()
                .then(|| self.handle_with_context(&Event::Ac, &mut ()));
        })
        .response
    }

    /// Render the zero, point and equal buttons.
    fn zero_point_eq(&mut self, ui: &mut Ui) -> Response {
        let button_spacing = 4.0;
        let button_width = (ui.available_width() - 3.0 * button_spacing) / 4.0;
        let button_width_2x = button_width * 2.0 + button_spacing;
        let button_height = ui.available_height();

        ui.spacing_mut().item_spacing = (button_spacing, button_spacing).into();
        ui.horizontal(|ui| {
            ui.add_sized(
                (button_width_2x, button_height),
                Button::new(rich_text("0")),
            )
            .clicked()
            .then(|| self.handle_with_context(&Event::Digit { digit: 0 }, &mut ()));
            ui.add_sized((button_width, button_height), Button::new(rich_text(",")))
                .clicked()
                .then(|| self.handle_with_context(&Event::Point, &mut ()));
            ui.add_sized((button_width, button_height), Button::new(rich_text("=")))
                .clicked()
                .then(|| self.handle_with_context(&Event::Equal, &mut ()));
        })
        .response
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        ctx.set_style(Style {
            visuals: Visuals::dark(),
            ..Default::default()
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            self.display(ui);

            let button_spacing = 4.0;
            let button_width = (ui.available_width() - 3.0 * button_spacing) / 4.0;
            let button_width_3x = button_width * 3.0 + 2.0 * button_spacing;
            let button_width_4x = button_width * 4.0 + 3.0 * button_spacing;
            let button_height = (ui.available_height() - 4.0 * button_spacing) / 5.0;
            let button_height_3x = button_height * 3.0 + 2.0 * button_spacing;
            let button_height_4x = button_height * 4.0 + 3.0 * button_spacing;

            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing = (button_spacing, button_spacing).into();
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = (button_spacing, button_spacing).into();
                    ui.vertical(|ui| {
                        ui.spacing_mut().item_spacing = (button_spacing, button_spacing).into();
                        ui.add_sized((button_width_3x, button_height), |ui: &mut Ui| {
                            self.control(ui)
                        });
                        ui.add_sized((button_width_3x, button_height_3x), |ui: &mut Ui| {
                            self.numpad(ui)
                        });
                    });
                    ui.add_sized((button_width, button_height_4x), |ui: &mut Ui| {
                        self.operands(ui)
                    });
                });
                ui.add_sized((button_width_4x, button_height), |ui: &mut Ui| {
                    self.zero_point_eq(ui)
                });
            });
        });
    }
}

fn rich_text(text: &str) -> impl Into<WidgetText> {
    let font_id = FontId::new(28.0, FontFamily::Proportional);
    RichText::new(text).font(font_id)
}
