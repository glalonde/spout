use spout::game_params::GameParams;
use spout::input::{InputFrame, PointerPress};
use spout::text::TextRenderer;
use spout::ui::{self, RectStyle, UiButton, UiRect, UiRenderer};

const BUTTON_W: f32 = 96.0;
const BUTTON_H: f32 = 44.0;
const BUTTON_GAP: f32 = 12.0;
const BUTTON_BOTTOM_MARGIN: f32 = 10.0;
const BUTTON_LABEL_H: f32 = 12.0;

#[derive(Debug)]
pub struct TitleScreen {
    instructions_open: bool,
    focused_button: ButtonAction,
    pressed_button: Option<ButtonAction>,
    focus_visible: bool,
}

impl Default for TitleScreen {
    fn default() -> Self {
        Self {
            instructions_open: false,
            focused_button: ButtonAction::Play,
            pressed_button: None,
            focus_visible: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TitleAction {
    StartGame,
    ToggleMusic,
}

#[derive(Debug, Clone, Copy)]
pub struct TitleRenderFlags {
    pub music_playing: bool,
    pub using_touch: bool,
}

pub struct TitleUiRenderContext<'a> {
    pub device: &'a wgpu::Device,
    pub encoder: &'a mut wgpu::CommandEncoder,
    pub title_ui_view: &'a wgpu::TextureView,
    pub ui: &'a UiRenderer,
    pub params: &'a GameParams,
    pub text: &'a TextRenderer,
    pub flags: TitleRenderFlags,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ButtonAction {
    Play,
    Menu,
    Music,
}

impl TitleScreen {
    pub fn update(
        &mut self,
        input: InputFrame,
        params: &GameParams,
        text: &TextRenderer,
        music_playing: bool,
        surface_size: (u32, u32),
    ) -> Option<TitleAction> {
        if input.help_pressed() {
            self.focus_visible = true;
            self.pressed_button = None;
            self.toggle_menu();
            return None;
        }

        let pointer_pressed = input.pointer_pressed();
        if let Some(point) = pointer_pressed {
            self.focus_visible = false;
            self.pressed_button = self.button_at(
                point,
                params,
                text,
                music_playing,
                surface_size.0,
                surface_size.1,
            );
        }

        if let Some(point) = input.pointer_released() {
            self.focus_visible = false;
            let pressed_button = self.pressed_button.take();
            let released_button = self.button_at(
                point,
                params,
                text,
                music_playing,
                surface_size.0,
                surface_size.1,
            );
            if let (Some(pressed), Some(released)) = (pressed_button, released_button) {
                if pressed == released {
                    return self.activate_button(released);
                }
            }
            return None;
        }

        if pointer_pressed.is_some() {
            return None;
        }

        let buttons = self.buttons(params, text, music_playing);
        self.ensure_focus_visible(&buttons);

        if input.menu_cancel_pressed() && self.instructions_open {
            self.focus_visible = true;
            self.close_menu();
            return None;
        }

        if self.move_focus_from_input(input, &buttons) {
            return None;
        }

        if input.menu_confirm_pressed() {
            self.focus_visible = true;
            return self.activate_button(self.focused_button);
        }

        None
    }

    pub fn prepare_ui(&self, ctx: TitleUiRenderContext<'_>) {
        let clear_color = if self.instructions_open {
            wgpu::Color {
                r: 0.02,
                g: 0.05,
                b: 0.07,
                a: 0.76,
            }
        } else {
            wgpu::Color::TRANSPARENT
        };
        {
            let _pass = ctx.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("title_ui_clear"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: ctx.title_ui_view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(clear_color),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                ..Default::default()
            });
        }

        let button_color = [0.7, 0.78, 0.78, 1.0];
        let text_color = [0.82, 0.86, 0.82, 1.0];
        let accent_color = [0.9, 0.72, 0.48, 1.0];
        let buttons = self.buttons(ctx.params, ctx.text, ctx.flags.music_playing);
        let rects: Vec<(UiRect, RectStyle)> = buttons
            .iter()
            .map(|button| (button.rect, self.button_style(button.action)))
            .collect();
        ctx.ui
            .draw_rects(ctx.device, ctx.encoder, ctx.title_ui_view, &rects);

        let mut texts: Vec<(&str, f32, f32, f32, [f32; 4])> = Vec::new();
        for button in &buttons {
            let scale = 1.0;
            let label_x =
                button.rect.x + (button.rect.w - ctx.text.text_width(button.label, scale)) / 2.0;
            let label_y = button.rect.y + (button.rect.h - BUTTON_LABEL_H) / 2.0;
            let color = if self.button_highlighted(button.action) {
                accent_color
            } else {
                button_color
            };
            texts.push((button.label, label_x, label_y, scale, color));
        }

        if self.instructions_open {
            let w = ctx.text.surface_width;
            let scale = 1.0;
            let lines: Vec<(&str, f32, [f32; 4])> = if ctx.flags.using_touch {
                vec![
                    ("OBJECTIVE", 24.0, accent_color),
                    ("BLAST AND CLIMB", 42.0, text_color),
                    ("LEFT SIDE -> GAS", 64.0, text_color),
                    ("RIGHT SIDE -> STEER", 82.0, text_color),
                ]
            } else {
                vec![
                    ("OBJECTIVE", 24.0, accent_color),
                    ("BLAST AND CLIMB", 42.0, text_color),
                    ("W -> GAS", 64.0, text_color),
                    ("A/D -> STEER", 82.0, text_color),
                ]
            };
            texts.extend(lines.into_iter().map(|(line, y, color)| {
                let x = (w - ctx.text.text_width(line, scale)) / 2.0;
                (line, x, y, scale, color)
            }));
        }

        ctx.text
            .draw(ctx.device, ctx.encoder, ctx.title_ui_view, &texts);
    }

    fn buttons(
        &self,
        params: &GameParams,
        text: &TextRenderer,
        music_playing: bool,
    ) -> Vec<UiButton<ButtonAction>> {
        let y = params.viewport_height as f32 - BUTTON_H - BUTTON_BOTTOM_MARGIN;
        if self.instructions_open {
            let music_label = if music_playing {
                "MUSIC ON"
            } else {
                "MUSIC OFF"
            };
            let music_w = (text.text_width(music_label, 1.0) + 22.0).max(112.0);
            return vec![
                UiButton {
                    action: ButtonAction::Music,
                    label: music_label,
                    rect: UiRect {
                        x: 18.0,
                        y,
                        w: music_w,
                        h: BUTTON_H,
                    },
                },
                UiButton {
                    action: ButtonAction::Menu,
                    label: "X",
                    rect: UiRect {
                        x: params.viewport_width as f32 - 44.0 - BUTTON_BOTTOM_MARGIN,
                        y,
                        w: 44.0,
                        h: BUTTON_H,
                    },
                },
            ];
        }

        let row_w = BUTTON_W * 2.0 + BUTTON_GAP;
        let start_x = (params.viewport_width as f32 - row_w) / 2.0;
        vec![
            UiButton {
                action: ButtonAction::Play,
                label: "PLAY",
                rect: UiRect {
                    x: start_x,
                    y,
                    w: BUTTON_W,
                    h: BUTTON_H,
                },
            },
            UiButton {
                action: ButtonAction::Menu,
                label: "MENU",
                rect: UiRect {
                    x: start_x + BUTTON_W + BUTTON_GAP,
                    y,
                    w: BUTTON_W,
                    h: BUTTON_H,
                },
            },
        ]
    }

    fn button_at(
        &self,
        point: PointerPress,
        params: &GameParams,
        text: &TextRenderer,
        music_playing: bool,
        surface_width: u32,
        surface_height: u32,
    ) -> Option<ButtonAction> {
        let (game_x, game_y) = ui::surface_to_game_point(
            point,
            params.viewport_width,
            params.viewport_height,
            surface_width,
            surface_height,
        )?;
        self.buttons(params, text, music_playing)
            .iter()
            .find(|button| button.rect.contains(game_x, game_y))
            .map(|button| button.action)
    }

    fn activate_button(&mut self, action: ButtonAction) -> Option<TitleAction> {
        match action {
            ButtonAction::Play => Some(TitleAction::StartGame),
            ButtonAction::Menu => {
                self.toggle_menu();
                None
            }
            ButtonAction::Music => Some(TitleAction::ToggleMusic),
        }
    }

    fn toggle_menu(&mut self) {
        self.pressed_button = None;
        if self.instructions_open {
            self.close_menu();
        } else {
            self.instructions_open = true;
            self.focused_button = ButtonAction::Menu;
        }
    }

    fn close_menu(&mut self) {
        self.pressed_button = None;
        self.instructions_open = false;
        self.focused_button = ButtonAction::Menu;
    }

    fn ensure_focus_visible(&mut self, buttons: &[UiButton<ButtonAction>]) {
        if buttons
            .iter()
            .any(|button| button.action == self.focused_button)
        {
            return;
        }

        if let Some(button) = buttons.first() {
            self.focused_button = button.action;
        }
    }

    fn move_focus_from_input(
        &mut self,
        input: InputFrame,
        buttons: &[UiButton<ButtonAction>],
    ) -> bool {
        let delta = if input.menu_left_pressed() || input.menu_up_pressed() {
            -1
        } else if input.menu_right_pressed() || input.menu_down_pressed() {
            1
        } else {
            0
        };

        if delta == 0 || buttons.is_empty() {
            return false;
        }

        let current = buttons
            .iter()
            .position(|button| button.action == self.focused_button)
            .unwrap_or(0);
        let next = if delta < 0 {
            current.checked_sub(1).unwrap_or(buttons.len() - 1)
        } else {
            (current + 1) % buttons.len()
        };
        self.focus_visible = true;
        self.pressed_button = None;
        self.focused_button = buttons[next].action;
        true
    }

    fn button_style(&self, action: ButtonAction) -> RectStyle {
        let pressed = self.pressed_button == Some(action);
        let focused = self.focus_visible && action == self.focused_button;
        RectStyle {
            fill_color: if pressed {
                [0.12, 0.16, 0.16, 0.9]
            } else if focused {
                [0.08, 0.12, 0.13, 0.78]
            } else {
                [0.02, 0.05, 0.07, 0.68]
            },
            outline_color: if pressed || focused {
                [0.9, 0.72, 0.48, 1.0]
            } else {
                [0.45, 0.57, 0.58, 0.92]
            },
            outline_px: if pressed || focused { 2.0 } else { 1.0 },
        }
    }

    fn button_highlighted(&self, action: ButtonAction) -> bool {
        self.pressed_button == Some(action) || (self.focus_visible && action == self.focused_button)
    }
}
