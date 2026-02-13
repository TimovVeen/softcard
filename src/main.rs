use std::{num::NonZeroU32, rc::Rc};

use softbuffer::{Context, Surface};
use vello_cpu::{
    RenderContext, RenderMode,
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

                    let setrect = RoundedRect::new(0., 0., 200., 100., 50.).to_path(TOLERANCE);
                    renderctx.set_transform(Affine::translate(Vec2::new(500., 500.)));
                    renderctx.set_paint(css::RED);
                    renderctx.set_stroke(Stroke::new(7.));
                    renderctx.stroke_path(&setrect);

                    renderctx.set_stroke(Stroke::new(3.));
                    renderctx.push_clip_path(&setrect);
                    for i in 0..20 {
                        renderctx.stroke_path(
                            &Line::new((10. * i as f32, 0.), (10. * i as f32, 100.))
                                .to_path(TOLERANCE),
                        );
                    }
                    renderctx.pop_clip_path();

                    let (dia_width, dia_height) = (200., 100.);
                    let mut diamond = BezPath::with_capacity(5);
                    diamond.move_to((0., dia_height / 2.));
                    diamond.line_to((dia_width / 2., 0.));
                    diamond.line_to((dia_width, dia_height / 2.));
                    diamond.line_to((dia_width / 2., dia_height));
                    diamond.close_path();

                    renderctx.set_transform(Affine::translate(Vec2::new(500., 700.)));
                    renderctx.set_paint(css::GREEN);
                    renderctx.set_stroke(Stroke::new(7.));
                    renderctx.stroke_path(&diamond);

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
