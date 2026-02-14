use std::{num::NonZeroU32, rc::Rc};

use softbuffer::{Context, Surface};
use vello_cpu::{
    PaintType, RenderContext, RenderMode, RenderSettings,
    color::palette::css,
    kurbo::{Affine, BezPath, Circle, Line, Point, RoundedRect, Shape, Stroke, Vec2},
};
use winit::{
    application::ApplicationHandler,
    event::{KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{Key, NamedKey},
    window::{Window, WindowId},
};

const TOLERANCE: f64 = 0.1;
const STROKE_WIDTH: f64 = 7.;
const STRIPE_WIDTH: f64 = 3.;

const BEZEL: f64 = 30.;
const RADIUS: f64 = 60.;

const MARGIN: f64 = 50.;
const PADDING: f64 = 60.;
const OFFSET: f64 = 2. * RADIUS + PADDING;
const MARGIN_OFFSET: f64 = MARGIN + RADIUS;

enum SetShape {
    Diamond,
    Squiggle,
    Oval,
}

enum SetFilling {
    Solid,
    Striped,
    Open,
}

enum RenderState {
    Active {
        window: Rc<Window>,
        surface: Surface<Rc<Window>, Rc<Window>>,
    },
    Suspended,
}

struct SetApp {
    render_state: RenderState,
    renderer: RenderContext,
    circle: BezPath,
    rect: BezPath,
    scale: f64,
}

impl SetApp {
    fn new() -> Self {
        Self {
            render_state: RenderState::Suspended,
            renderer: RenderContext::new_with(
                1980,
                1080,
                RenderSettings {
                    num_threads: 0,
                    ..Default::default()
                },
            ),
            circle: Circle::new(Point::ZERO, RADIUS).to_path(TOLERANCE),
            rect: RoundedRect::new(0., 0., 400., 600., BEZEL).to_path(TOLERANCE),
            scale: 1.,
        }
    }
}

impl ApplicationHandler for SetApp {
    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        self.render_state = RenderState::Suspended;
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if matches!(self.render_state, RenderState::Active { .. }) {
            return;
        }

        let window = Rc::new(
            event_loop
                .create_window(Window::default_attributes())
                .unwrap(),
        );
        let context = Context::new(window.clone()).unwrap();
        let surface = Surface::new(&context, window.clone()).unwrap();

        self.render_state = RenderState::Active { window, surface };
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let RenderState::Active { window, surface } = &mut self.render_state else {
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
            WindowEvent::ScaleFactorChanged {
                scale_factor,
                inner_size_writer: _,
            } => {
                self.scale = scale_factor;
                println!("{scale_factor}");
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

                let size = window.inner_size();

                let mut buffer = surface.buffer_mut().unwrap();

                let bufslice = {
                    let len = buffer.len() * 4;
                    let ptr = buffer.as_mut_ptr() as *mut u8;
                    unsafe { std::slice::from_raw_parts_mut(ptr, len) }
                };

                self.renderer.set_paint(css::WHITE);
                self.renderer.fill_path(&self.rect);

                self.renderer
                    .set_transform(Affine::translate(Vec2::new(MARGIN_OFFSET, MARGIN_OFFSET)));
                self.renderer.set_paint(css::RED);
                self.renderer.fill_path(&self.circle);

                self.renderer.set_transform(Affine::translate(Vec2::new(
                    MARGIN_OFFSET,
                    MARGIN_OFFSET + OFFSET,
                )));
                self.renderer.set_paint(css::YELLOW);
                self.renderer.fill_path(&self.circle);

                self.renderer.set_transform(Affine::translate(Vec2::new(
                    MARGIN_OFFSET,
                    MARGIN_OFFSET + OFFSET * 2.,
                )));
                self.renderer.set_paint(css::BLUE);
                self.renderer.fill_path(&self.circle);

                self.renderer.set_transform(Affine::translate(Vec2::new(
                    MARGIN_OFFSET + OFFSET,
                    MARGIN_OFFSET,
                )));
                self.renderer.set_paint(css::ORANGE);
                self.renderer.fill_path(&self.circle);

                self.renderer.set_transform(Affine::translate(Vec2::new(
                    MARGIN_OFFSET + OFFSET,
                    MARGIN_OFFSET + OFFSET,
                )));
                self.renderer.set_paint(css::GREEN);
                self.renderer.fill_path(&self.circle);

                self.renderer.set_transform(Affine::translate(Vec2::new(
                    MARGIN_OFFSET + OFFSET,
                    MARGIN_OFFSET + OFFSET * 2.,
                )));
                self.renderer.set_paint(css::PURPLE);
                self.renderer.fill_path(&self.circle);

                self.renderer
                    .set_transform(Affine::scale(0.8).with_translation(Vec2::new(440., 460.)));
                self.renderer.set_paint(css::WHITE);
                self.renderer.fill_path(&self.rect);

                self.renderer
                    .set_transform(Affine::translate(Vec2::new(500., 500.)));
                draw_shape(
                    &mut self.renderer,
                    SetShape::Oval,
                    SetFilling::Striped,
                    css::RED,
                );

                self.renderer
                    .set_transform(Affine::translate(Vec2::new(500., 650.)));
                draw_shape(
                    &mut self.renderer,
                    SetShape::Diamond,
                    SetFilling::Open,
                    css::GREEN,
                );

                self.renderer
                    .set_transform(Affine::translate(Vec2::new(500., 800.)));
                draw_shape(
                    &mut self.renderer,
                    SetShape::Squiggle,
                    SetFilling::Solid,
                    css::REBECCA_PURPLE,
                );

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

fn draw_shape(
    ctx: &mut RenderContext,
    shape: SetShape,
    filling: SetFilling,
    color: impl Into<PaintType>,
) {
    ctx.set_paint(color);
    ctx.set_stroke(Stroke::new(STROKE_WIDTH));

    let path = match shape {
        SetShape::Diamond => {
            let (dia_width, dia_height) = (200., 100.);
            let mut diamond = BezPath::with_capacity(5);
            diamond.move_to((0., dia_height / 2.));
            diamond.line_to((dia_width / 2., 0.));
            diamond.line_to((dia_width, dia_height / 2.));
            diamond.line_to((dia_width / 2., dia_height));
            diamond.close_path();
            diamond
        }
        SetShape::Squiggle => {
            let mut squiggle = BezPath::with_capacity(7);
            squiggle.move_to((198.0, 11.0));
            squiggle.curve_to((214.8, 54.8), (169.4, 102.6), (116.0, 89.0));
            squiggle.curve_to((94.6, 83.6), (74.4, 65.0), (44.0, 87.0));
            squiggle.curve_to((9.2, 112.2), (0.8, 97.6), (0.0, 61.0));
            squiggle.curve_to((-0.8, 25.0), (28.2, 0.4), (62.0, 5.0));
            squiggle.curve_to((108.4, 11.4), (113.8, 44.0), (168.0, 9.0));
            squiggle.curve_to((180.6, 1.0), (191.8, -5.2), (198.0, 11.0));
            squiggle
        }
        SetShape::Oval => RoundedRect::new(0., 0., 200., 100., 50.).to_path(TOLERANCE),
    };

    ctx.stroke_path(&path);

    match filling {
        SetFilling::Solid => ctx.fill_path(&path),
        SetFilling::Striped => {
            ctx.set_stroke(Stroke::new(STRIPE_WIDTH));
            ctx.push_clip_path(&path);
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
