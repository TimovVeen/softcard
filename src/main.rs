#![allow(dead_code)]

use std::{num::NonZeroU32, rc::Rc};

use softbuffer::{Context, Surface};
use vello_cpu::{
    PaintType, RenderContext, RenderMode, RenderSettings,
    color::palette::css,
    kurbo::{Affine, BezPath, Circle, Line, Point, RoundedRect, Shape, Stroke, Vec2},
};
use winit::{
    application::ApplicationHandler,
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{Key, NamedKey},
    window::{Window, WindowId},
};

const TOLERANCE: f64 = 0.1;
const STROKE_WIDTH: f64 = 7.;
const STRIPE_WIDTH: f64 = 3.;

const BEZEL: f64 = 30.;
const RADIUS: f64 = 60.;

const SPACING: f64 = 20.;
const MARGIN: f64 = 50.;
const PADDING: f64 = 60.;
const OFFSET: f64 = 2. * RADIUS + PADDING;
const MARGIN_OFFSET: f64 = MARGIN + RADIUS;

const CARDS: [u8; 63] = {
    let mut res = [0_u8; _];
    let mut i = 0;
    while i < res.len() {
        res[i] = i as u8 + 1;
        i += 1;
    }
    res
};
const COLORS: [vello_cpu::color::AlphaColor<vello_cpu::color::Srgb>; 6] = [
    css::RED,
    css::ORANGE,
    css::YELLOW,
    css::GREEN,
    css::BLUE,
    css::PURPLE,
];

enum SetFilling {
    Solid,
    Striped,
    Open,
}

struct RenderState {
    window: Rc<Window>,
    surface: Surface<Rc<Window>, Rc<Window>>,
}

struct SetApp {
    render_state: Option<RenderState>,
    renderer: RenderContext,
    circle: BezPath,
    card: BezPath,
    diamond: BezPath,
    squiggle: BezPath,
    oval: BezPath,
    scale: f64,
    cards: [u8; 7],
    all_cards: [u8; 63],
    selection: u8,
    card_head: usize,
}

impl SetApp {
    fn new() -> Self {
        let mut all_cards = CARDS;
        fastrand::shuffle(&mut all_cards);
        let diamond = {
            let mut diamond = BezPath::with_capacity(5);
            diamond.move_to((0., 50.));
            diamond.line_to((100., 0.));
            diamond.line_to((200., 50.));
            diamond.line_to((100., 100.));
            diamond.close_path();
            diamond
        };
        let squiggle = {
            let mut squiggle = BezPath::with_capacity(7);
            squiggle.move_to((198.0, 11.0));
            squiggle.curve_to((214.8, 54.8), (169.4, 102.6), (116.0, 89.0));
            squiggle.curve_to((94.6, 83.6), (74.4, 65.0), (44.0, 87.0));
            squiggle.curve_to((9.2, 112.2), (0.8, 97.6), (0.0, 61.0));
            squiggle.curve_to((-0.8, 25.0), (28.2, 0.4), (62.0, 5.0));
            squiggle.curve_to((108.4, 11.4), (113.8, 44.0), (168.0, 9.0));
            squiggle.curve_to((180.6, 1.0), (191.8, -5.2), (198.0, 11.0));
            squiggle
        };
        let oval = RoundedRect::new(0., 0., 200., 100., 50.).to_path(TOLERANCE);
        Self {
            render_state: None,
            renderer: RenderContext::new_with(
                1980,
                1080,
                RenderSettings {
                    num_threads: 0,
                    ..Default::default()
                },
            ),
            circle: Circle::new(Point::ZERO, RADIUS).to_path(TOLERANCE),
            card: RoundedRect::new(0., 0., 400., 600., BEZEL).to_path(TOLERANCE),
            diamond,
            squiggle,
            oval,
            scale: 1.,
            cards: *all_cards[..7].as_array().unwrap(),
            all_cards,
            selection: 0,
            card_head: 7,
        }
    }
}

impl ApplicationHandler for SetApp {
    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        self.render_state = None;
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.render_state.is_some() {
            return;
        }

        let window = Rc::new(
            event_loop
                .create_window(Window::default_attributes())
                .unwrap(),
        );
        let context = Context::new(window.clone()).unwrap();
        let surface = Surface::new(&context, window.clone()).unwrap();

        self.render_state = Some(RenderState { window, surface });
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(RenderState { window, surface }) = &mut self.render_state else {
            return;
        };
        if window_id != window.id() {
            return;
        }
        match event {
            WindowEvent::CloseRequested
            | WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        logical_key: Key::Named(NamedKey::Escape),
                        ..
                    },
                ..
            } => {
                event_loop.exit();
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        logical_key: Key::Character(str),
                        state: ElementState::Pressed,
                        repeat: false,
                        ..
                    },
                ..
            } => {
                if let Ok(num) = str.parse::<u8>()
                    && num > 0
                    && num <= 7
                {
                    self.selection ^= 1 << (num - 1);
                    let mut res = 0;
                    let mut sels = self.selection;
                    while sels != 0 {
                        res ^= self.cards[sels.trailing_zeros() as usize];
                        sels &= sels - 1;
                    }
                    if res == 0 && self.selection != 0 {
                        println!("you got a set");
                        let mut sels = self.selection;
                        while sels != 0 {
                            self.cards[sels.trailing_zeros() as usize] =
                                self.all_cards[self.card_head];
                            self.card_head += 1;
                            if self.card_head >= 63 {
                                println!("you win");
                                event_loop.exit();
                                return;
                            }

                            sels &= sels - 1;
                        }
                        self.selection = 0;
                    }
                    window.request_redraw();
                };
            }
            WindowEvent::ScaleFactorChanged {
                scale_factor,
                inner_size_writer: _,
            } => {
                self.scale = scale_factor;
            }
            WindowEvent::Resized(size) => {
                let width = size.width.max(1);
                let height = size.height.max(1);

                surface
                    .resize(
                        NonZeroU32::new(width).unwrap(),
                        NonZeroU32::new(height).unwrap(),
                    )
                    .unwrap();

                self.renderer = RenderContext::new_with(
                    width as u16,
                    height as u16,
                    RenderSettings {
                        num_threads: 0,
                        ..Default::default()
                    },
                );

                window.request_redraw();
            }
            WindowEvent::RedrawRequested => {
                self.renderer.reset();
                self.renderer.set_transform(Affine::scale(self.scale));

                let size = window.inner_size();
                let cardscale = size.width as f64 / ((400. + SPACING) * 7. - SPACING);

                let mut buffer = surface.buffer_mut().unwrap();

                let bufslice = {
                    let len = buffer.len() * 4;
                    let ptr = buffer.as_mut_ptr() as *mut u8;
                    unsafe { std::slice::from_raw_parts_mut(ptr, len) }
                };

                for (i, &card) in self.cards.iter().enumerate() {
                    self.renderer.set_transform(
                        Affine::translate(Vec2::new((400. + SPACING) * i as f64, 0.))
                            .then_scale(cardscale),
                    );
                    let selected = self.selection & (1 << i) != 0;
                    draw_projcard(&mut self.renderer, &self.card, &self.circle, card, selected);
                }

                self.renderer.flush();
                self.renderer.render_to_buffer(
                    bufslice,
                    size.width as u16,
                    size.height as u16,
                    RenderMode::OptimizeSpeed,
                );

                // Convert BGRA to RGBA
                unsafe {
                    bufslice
                        .as_chunks_unchecked_mut::<4>()
                        .iter_mut()
                        .for_each(|chunk| {
                            chunk.swap(0, 2);
                        });
                }

                buffer.present().unwrap();
            }
            _ => {}
        }
    }
}

fn draw_projcard(ctx: &mut RenderContext, card: &BezPath, dot: &BezPath, mask: u8, selected: bool) {
    let trans = *ctx.transform();
    ctx.set_paint(if selected { css::GRAY } else { css::WHITE });
    ctx.fill_path(card);

    for i in 0..3 {
        for j in 0..2 {
            let idx = i * 2 + j;
            if (1 << idx) & mask != 0 {
                ctx.set_transform(
                    trans
                        * Affine::translate(Vec2::new(
                            MARGIN_OFFSET + OFFSET * j as f64,
                            MARGIN_OFFSET + OFFSET * i as f64,
                        )),
                );
                ctx.set_paint(COLORS[idx]);
                ctx.fill_path(dot);
            }
        }
    }

    ctx.set_transform(trans);
}

fn draw_shape(
    ctx: &mut RenderContext,
    path: &BezPath,
    filling: SetFilling,
    color: impl Into<PaintType>,
) {
    ctx.set_paint(color);
    ctx.set_stroke(Stroke::new(STROKE_WIDTH));
    ctx.stroke_path(path);

    match filling {
        SetFilling::Solid => ctx.fill_path(path),
        SetFilling::Striped => {
            ctx.set_stroke(Stroke::new(STRIPE_WIDTH));
            ctx.push_clip_path(path);
            for i in 0..20 {
                ctx.stroke_path(
                    &Line::new((10. * i as f32, 0.), (10. * i as f32, 100.)).to_path(TOLERANCE),
                );
            }
            ctx.pop_clip_path();
        }
        SetFilling::Open => (),
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let mut app = SetApp::new();
    event_loop.run_app(&mut app).unwrap();
}
