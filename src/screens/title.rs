use spout::game_params::GameParams;
use spout::input::{InputFrame, PointerPress};
use spout::text::TextRenderer;

#[derive(Debug, Default)]
pub struct TitleScreen {
    instructions_open: bool,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ButtonAction {
    Music,
    Help,
}

#[derive(Debug, Clone, Copy)]
struct UiRect {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}

impl UiRect {
    fn contains(&self, x: f32, y: f32) -> bool {
        x >= self.x && x <= self.x + self.w && y >= self.y && y <= self.y + self.h
    }
}

struct Button {
    action: ButtonAction,
    label: &'static str,
    rect: UiRect,
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
        let clicked_button = input.pointer_pressed().and_then(|point| {
            self.button_at(
                point,
                params,
                text,
                music_playing,
                surface_size.0,
                surface_size.1,
            )
        });
        let mut title_action_handled = false;

        if input.help_pressed() {
            self.instructions_open = !self.instructions_open;
            title_action_handled = true;
        }

        if let Some(action) = clicked_button {
            match action {
                ButtonAction::Music => {
                    return Some(TitleAction::ToggleMusic);
                }
                ButtonAction::Help => {
                    self.instructions_open = !self.instructions_open;
                }
            }
            title_action_handled = true;
        } else if self.instructions_open && input.pointer_pressed().is_some() {
            self.instructions_open = false;
            title_action_handled = true;
        }

        if !title_action_handled
            && !self.instructions_open
            && (input.thrust_started() || input.rotate_started())
        {
            return Some(TitleAction::StartGame);
        }

        None
    }

    pub fn draw_help_button(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        game_view_texture: &wgpu::TextureView,
        params: &GameParams,
        text: &TextRenderer,
    ) {
        if self.instructions_open {
            return;
        }

        let Some(button) = self
            .buttons(params, text, false)
            .into_iter()
            .find(|button| button.action == ButtonAction::Help)
        else {
            return;
        };

        let color = [0.55, 0.55, 0.55, 1.0];
        let label_x = button.rect.x + (button.rect.w - text.text_width(button.label, 1.0)) / 2.0;
        let label_y = button.rect.y + (button.rect.h - 12.0) / 2.0;

        text.draw(
            device,
            encoder,
            game_view_texture,
            &[(button.label, label_x, label_y, 1.0, color)],
        );
    }

    pub fn prepare_ui(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        title_ui_view: &wgpu::TextureView,
        params: &GameParams,
        text: &TextRenderer,
        flags: TitleRenderFlags,
    ) {
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
            let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("title_ui_clear"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: title_ui_view,
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
        let buttons = self.buttons(params, text, flags.music_playing);

        let mut texts: Vec<(&str, f32, f32, f32, [f32; 4])> = Vec::new();
        for button in buttons
            .iter()
            .filter(|button| button.action != ButtonAction::Help || self.instructions_open)
        {
            let scale = 1.0;
            let label_x =
                button.rect.x + (button.rect.w - text.text_width(button.label, scale)) / 2.0;
            let label_y = button.rect.y + (button.rect.h - 12.0) / 2.0;
            texts.push((button.label, label_x, label_y, scale, button_color));
        }

        if self.instructions_open {
            let w = text.surface_width;
            let scale = 1.0;
            let lines: Vec<(&str, f32, [f32; 4])> = if flags.using_touch {
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
                let x = (w - text.text_width(line, scale)) / 2.0;
                (line, x, y, scale, color)
            }));
        }

        text.draw(device, encoder, title_ui_view, &texts);
    }

    fn buttons(
        &self,
        params: &GameParams,
        text: &TextRenderer,
        music_playing: bool,
    ) -> Vec<Button> {
        let pad_x = 4.0;
        let button_h = 32.0;
        let help_y = params.viewport_height as f32 - button_h - 6.0;
        let help_label = if self.instructions_open { "X" } else { "?" };
        let mut buttons = vec![Button {
            action: ButtonAction::Help,
            label: help_label,
            rect: UiRect {
                x: params.viewport_width as f32 - 56.0 - 6.0,
                y: help_y,
                w: 56.0,
                h: button_h,
            },
        }];

        if self.instructions_open {
            let music_label = if music_playing {
                "[MUSIC ON]"
            } else {
                "[MUSIC OFF]"
            };
            let music_w = text.text_width(music_label, 1.0) + pad_x * 2.0;
            buttons.push(Button {
                action: ButtonAction::Music,
                label: music_label,
                rect: UiRect {
                    x: 18.0,
                    y: 100.0,
                    w: music_w,
                    h: button_h,
                },
            });
        }

        buttons
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
        if title_help_surface_hit(point, surface_width, surface_height) {
            return Some(ButtonAction::Help);
        }

        let (game_x, game_y) = surface_to_game_point(point, params, surface_width, surface_height)?;
        self.buttons(params, text, music_playing)
            .iter()
            .find(|button| button.rect.contains(game_x, game_y))
            .map(|button| button.action)
    }
}

fn surface_to_game_point(
    point: PointerPress,
    params: &GameParams,
    surface_width: u32,
    surface_height: u32,
) -> Option<(f32, f32)> {
    if surface_width == 0 || surface_height == 0 {
        return None;
    }

    let game_w = params.viewport_width as f32;
    let game_h = params.viewport_height as f32;
    let surface_w = surface_width as f32;
    let surface_h = surface_height as f32;
    let scale = (surface_w / game_w)
        .min(surface_h / game_h)
        .floor()
        .max(1.0);
    let draw_w = game_w * scale;
    let draw_h = game_h * scale;
    let offset_x = ((surface_w - draw_w) * 0.5).floor();
    let offset_y = ((surface_h - draw_h) * 0.5).floor();

    if point.x < offset_x
        || point.x > offset_x + draw_w
        || point.y < offset_y
        || point.y > offset_y + draw_h
    {
        return None;
    }

    let game_x = (point.x - offset_x) / draw_w * game_w;
    let game_y = (point.y - offset_y) / draw_h * game_h;
    Some((game_x, game_y))
}

fn title_help_surface_hit(point: PointerPress, surface_width: u32, surface_height: u32) -> bool {
    let w = surface_width as f32;
    let h = surface_height as f32;
    point.x >= w * 0.72 && point.y >= h * 0.62
}
