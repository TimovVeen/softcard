use std::{num::NonZeroU32, rc::Rc};

use softbuffer::{Context, Surface};
use vello_cpu::{
    PaintType, RenderContext, RenderMode,
    color::palette::css,
    kurbo::{Affine, BezPath, Circle, Line, Point, RoundedRect, Shape, Stroke, Vec2},
};
use winit::{
    event::{KeyEvent, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    keyboard::{Key, NamedKey},
    window::Window,
};

mod app;

const TOLERANCE: f64 = 0.1;
const STROKE_WIDTH: f64 = 7.;
const STRIPE_WIDTH: f64 = 3.;

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
    let context = Context::new(event_loop.owned_display_handle()).unwrap();

    let bezel = 30.;
    let radius = 60.;

    let margin = 50.;
    let padding = 60.;
    let offset = 2. * radius + padding;
    let margin_offset = margin + radius;

    let rect = RoundedRect::new(0., 0., 400., 600., bezel).to_path(TOLERANCE);
    let circle = Circle::new(Point::ZERO, radius).to_path(TOLERANCE);

    let mut app = app::WinitAppBuilder::with_init(
        |elwt| {
            let window = elwt.create_window(Window::default_attributes());
            Rc::new(window.unwrap())
        },
        |_elwt, window| Surface::new(&context, window.clone()).unwrap(),
    )
    .with_event_handler(|window, surface, window_id, event, elwt| {
        elwt.set_control_flow(ControlFlow::Wait);

        if window_id != window.id() {
            return;
        }

        match event {
            WindowEvent::RedrawRequested => {
                let Some(surface) = surface else {
                    eprintln!("RedrawRequested fired before Resumed or after Suspended");
                    return;
                };
                // TODO: resizes surface every frame, use Resized window event instead
                let size = window.inner_size();
                surface
                    .resize(
                        NonZeroU32::new(size.width).unwrap(),
                        NonZeroU32::new(size.height).unwrap(),
                    )
                    .unwrap();

                let mut buffer = surface.buffer_mut().unwrap();

                let bufslice = {
                    let len = buffer.len() * 4;
                    let ptr = buffer.as_mut_ptr() as *mut u8;
                    unsafe { std::slice::from_raw_parts_mut(ptr, len) }
                };

                {
                    let mut renderctx = RenderContext::new(size.width as u16, size.height as u16);

                    renderctx.set_paint(css::WHITE);
                    renderctx.fill_path(&rect);

                    renderctx
                        .set_transform(Affine::translate(Vec2::new(margin_offset, margin_offset)));
                    renderctx.set_paint(css::RED);
                    renderctx.fill_path(&circle);

                    renderctx.set_transform(Affine::translate(Vec2::new(
                        margin_offset,
                        margin_offset + offset,
                    )));
                    renderctx.set_paint(css::YELLOW);
                    renderctx.fill_path(&circle);

                    renderctx.set_transform(Affine::translate(Vec2::new(
                        margin_offset,
                        margin_offset + offset * 2.,
                    )));
                    renderctx.set_paint(css::BLUE);
                    renderctx.fill_path(&circle);

                    renderctx.set_transform(Affine::translate(Vec2::new(
                        margin_offset + offset,
                        margin_offset,
                    )));
                    renderctx.set_paint(css::ORANGE);
                    renderctx.fill_path(&circle);

                    renderctx.set_transform(Affine::translate(Vec2::new(
                        margin_offset + offset,
                        margin_offset + offset,
                    )));
                    renderctx.set_paint(css::GREEN);
                    renderctx.fill_path(&circle);

                    renderctx.set_transform(Affine::translate(Vec2::new(
                        margin_offset + offset,
                        margin_offset + offset * 2.,
                    )));
                    renderctx.set_paint(css::PURPLE);
                    renderctx.fill_path(&circle);

                    renderctx
                        .set_transform(Affine::scale(0.8).with_translation(Vec2::new(440., 460.)));
                    renderctx.set_paint(css::WHITE);
                    renderctx.fill_path(&rect);

                    renderctx.set_transform(Affine::translate(Vec2::new(500., 500.)));
                    draw_shape(
                        &mut renderctx,
                        SetShape::Oval,
                        SetFilling::Striped,
                        css::RED,
                    );

                    renderctx.set_transform(Affine::translate(Vec2::new(500., 650.)));
                    draw_shape(
                        &mut renderctx,
                        SetShape::Diamond,
                        SetFilling::Open,
                        css::GREEN,
                    );

                    renderctx.set_transform(Affine::translate(Vec2::new(500., 800.)));
                    draw_shape(
                        &mut renderctx,
                        SetShape::Squiggle,
                        SetFilling::Solid,
                        css::REBECCA_PURPLE,
                    );

                    renderctx.render_to_buffer(
                        bufslice,
                        size.width as u16,
                        size.height as u16,
                        RenderMode::OptimizeSpeed,
                    );
                }
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
            WindowEvent::CloseRequested
            | WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        logical_key: Key::Named(NamedKey::Escape),
                        ..
                    },
                ..
            } => {
                elwt.exit();
            }
            _ => {}
        }
    });

    event_loop.run_app(&mut app).unwrap();
}
