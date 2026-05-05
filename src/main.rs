#![allow(dead_code)]
#![no_std]

use core::num::NonZeroU32;

extern crate alloc;
use alloc::rc::Rc;

use log::info;
use softbuffer::{Context, Surface};
use vello_cpu::{
    PaintType, RenderContext, RenderMode, RenderSettings,
    color::{AlphaColor, Srgb, palette::css},
    kurbo::{Affine, BezPath, Circle, Line, Point, Rect, RoundedRect, Shape, Stroke, Vec2},
};
use web_time::Instant;
use winit::{
    application::ApplicationHandler,
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{Key, NamedKey},
    window::{Window, WindowAttributes, WindowId},
};

const TOLERANCE: f64 = 5.;
const STROKE_WIDTH: f64 = 7.;
const STRIPE_WIDTH: f64 = 3.;

const BEZEL: f64 = 30.;
const RADIUS: f64 = 60.;
const CARD_WIDTH: f64 = 400.;
const CARD_HEIGHT: f64 = 600.;

const WINDOW_MARGIN: f64 = 20.;
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

const COLORS: [AlphaColor<Srgb>; 6] = [
    map_col(css::RED),
    map_col(css::ORANGE),
    map_col(css::YELLOW),
    map_col(css::GREEN),
    map_col(css::BLUE),
    map_col(css::PURPLE),
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
    hovered_card: Option<usize>,
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
                500,
                270,
                RenderSettings {
                    num_threads: 0,
                    ..Default::default()
                },
            ),
            circle: Circle::new(Point::ZERO, RADIUS).to_path(TOLERANCE),
            card: RoundedRect::new(0., 0., CARD_WIDTH, CARD_HEIGHT, BEZEL).to_path(TOLERANCE),
            diamond,
            squiggle,
            oval,
            scale: 1.,
            cards: *all_cards[..7].as_array().unwrap(),
            all_cards,
            selection: 0,
            card_head: 7,
            hovered_card: None,
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

        let attributes =
            WindowAttributes::default().with_inner_size(winit::dpi::LogicalSize::new(800., 600.));
        #[cfg(target_family = "wasm")]
        let attributes =
            winit::platform::web::WindowAttributesExtWebSys::with_append(attributes, true);

        let window = Rc::new(event_loop.create_window(attributes).unwrap());
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
                if str == "q" {
                    print_solution(&self.cards);
                }
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
                        info!("You got a set!");
                        let mut sels = self.selection;
                        while sels != 0 {
                            self.cards[sels.trailing_zeros() as usize] =
                                self.all_cards[self.card_head];
                            self.card_head += 1;
                            if self.card_head >= 63 {
                                info!("You win!");
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
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: winit::event::MouseButton::Left,
                ..
            } => {
                if let Some(card) = self.hovered_card
                    && (0..7).contains(&card)
                {
                    self.selection ^= 1 << card;

                    // TODO: Absolutely horrid code duplication, please fix the borrowing issue at some point
                    let mut res = 0;
                    let mut sels = self.selection;
                    while sels != 0 {
                        res ^= self.cards[sels.trailing_zeros() as usize];
                        sels &= sels - 1;
                    }
                    if res == 0 && self.selection != 0 {
                        info!("You got a set!");
                        let mut sels = self.selection;
                        while sels != 0 {
                            self.cards[sels.trailing_zeros() as usize] =
                                self.all_cards[self.card_head];
                            self.card_head += 1;
                            if self.card_head >= 63 {
                                info!("You win!");
                                event_loop.exit();
                                return;
                            }

                            sels &= sels - 1;
                        }
                        self.selection = 0;
                    }

                    window.request_redraw();
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                // TODO: code duplication, and math can be simplified
                let size = window.inner_size();
                let x_cardscale = size.width as f64
                    / ((CARD_WIDTH + SPACING) * 4. - SPACING + WINDOW_MARGIN * 2.);
                let y_cardscale = size.height as f64
                    / ((CARD_HEIGHT + SPACING) * 2. - SPACING + WINDOW_MARGIN * 2.);
                let cardscale = f64::min(x_cardscale, y_cardscale);

                let horizontal = ((position.x / cardscale - WINDOW_MARGIN) / (CARD_WIDTH + SPACING))
                    .floor() as i32;
                let vertical = ((position.y / cardscale - WINDOW_MARGIN) / (CARD_HEIGHT + SPACING))
                    .floor() as i32;
                let res_idx = vertical * 4 + horizontal;
                self.hovered_card = if !(0..4).contains(&horizontal) || !(0..7).contains(&res_idx) {
                    None
                } else {
                    Some(res_idx as usize)
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

                self.renderer =
                    RenderContext::new_with(width as u16, height as u16, RenderSettings::default());

                window.request_redraw();
            }
            WindowEvent::RedrawRequested => {
                self.renderer.reset();

                let size = window.inner_size();
                let x_cardscale = size.width as f64
                    / ((CARD_WIDTH + SPACING) * 4. - SPACING + WINDOW_MARGIN * 2.);
                let y_cardscale = size.height as f64
                    / ((CARD_HEIGHT + SPACING) * 2. - SPACING + WINDOW_MARGIN * 2.);

                self.renderer.set_paint(css::WHITE);
                self.renderer
                    .fill_rect(&Rect::new(0., 0., size.width as f64, size.height as f64));

                for (i, &card) in self.cards.iter().enumerate() {
                    // use bithacks instead?
                    self.renderer.set_transform(
                        Affine::translate(Vec2::new(
                            WINDOW_MARGIN + (CARD_WIDTH + SPACING) * (i % 4) as f64,
                            WINDOW_MARGIN + (CARD_HEIGHT + SPACING) * (i / 4) as f64,
                        ))
                        .then_scale(f64::min(x_cardscale, y_cardscale)),
                    );
                    let selected = self.selection & (1 << i) != 0;
                    draw_projcard(&mut self.renderer, &self.card, &self.circle, card, selected);
                }

                self.renderer.flush();

                let mut buffer = surface.buffer_mut().unwrap();

                let bufslice = {
                    let len = buffer.len() * 4;
                    let ptr = buffer.as_mut_ptr() as *mut u8;
                    unsafe { core::slice::from_raw_parts_mut(ptr, len) }
                };

                self.renderer.render_to_buffer(
                    bufslice,
                    size.width as u16,
                    size.height as u16,
                    RenderMode::OptimizeSpeed,
                );

                buffer.present().unwrap();
            }
            _ => {}
        }
    }
}

fn print_solution(cards: &[u8; 7]) {
    for i in 1..0b10000000_usize {
        if i.count_ones() < 3 {
            continue;
        }

        let mut sels = i;
        let mut res = 0;
        while sels != 0 {
            res ^= cards[sels.trailing_zeros() as usize];
            sels &= sels - 1;
        }
        if res == 0 {
            info!("{:b}", i);
            return;
        }
    }
}

fn draw_projcard(ctx: &mut RenderContext, card: &BezPath, dot: &BezPath, mask: u8, selected: bool) {
    let trans = *ctx.transform();
    if selected {
        ctx.set_paint(css::GRAY);
        ctx.fill_path(card);
    }

    ctx.set_paint(css::BLACK);
    ctx.set_stroke(Stroke::new(3.));
    ctx.stroke_path(card);

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

const fn map_col(input: AlphaColor<Srgb>) -> AlphaColor<Srgb> {
    let mut comps = input.components;
    comps.swap(0, 2);
    AlphaColor::new(comps)
}

#[allow(unused_mut)]
fn main() {
    #[cfg(not(target_family = "wasm"))]
    simple_logger::init_with_level(log::Level::Info).unwrap();
    #[cfg(target_family = "wasm")]
    console_log::init_with_level(log::Level::Info).unwrap();

    let now = Instant::now();

    let event_loop = EventLoop::new().unwrap();
    let mut app = SetApp::new();

    #[cfg(not(target_family = "wasm"))]
    event_loop.run_app(&mut app).unwrap();
    #[cfg(target_family = "wasm")]
    winit::platform::web::EventLoopExtWebSys::spawn_app(event_loop, app);

    info!("Finished in {} seconds.", now.elapsed().as_secs());
}
