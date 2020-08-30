/* 
 * Much of the code here comes from the Reclutch project:
 *
 * https://github.com/jazzfool/reclutch/blob/master/reclutch/examples/opengl/main.rs
 *
 * (c) jazzfool <saveuselon@gmail.com> - MIT licensed.
 *
 * I replaced the OpenGL cube example with Skia! :-)
 *
 */

extern crate thiserror;
#[macro_use] extern crate glium;
extern crate skia_safe as skia;

use glium::glutin;
use glutin::event::{Event, WindowEvent, KeyboardInput, VirtualKeyCode};
use glutin::event_loop::{ControlFlow, EventLoop};
use glium::{GlObject, Surface};

use std::time::Instant;

#[macro_use] extern crate imgui; // for the macros, can't use one in imgui_glium_renderer
#[macro_use] extern crate imgui_glium_renderer;
extern crate imgui_winit_support;
use imgui_winit_support::WinitPlatform;
use imgui_glium_renderer::Renderer as ImguiRenderer;
use imgui::{Context as ImguiContext};

mod reclutch_skia;

#[derive(Copy, Clone)]
struct TextureVertex {
    position: [f32; 3],
    tex_coord: [f32; 2],
}

implement_vertex!(TextureVertex, position, tex_coord);

const fn texture_vertex(pos: [i8; 2], tex: [i8; 2]) -> TextureVertex {
    TextureVertex {
        position: [pos[0] as _, pos[1] as _, 0.0],
        tex_coord: [tex[0] as _, tex[1] as _],
    }
}

const QUAD_VERTICES: [TextureVertex; 4] = [
    texture_vertex([-1, -1], [0, 0]),
    texture_vertex([-1, 1], [0, 1]),
    texture_vertex([1, 1], [1, 1]),
    texture_vertex([1, -1], [1, 0]),
];

const QUAD_INDICES: [u32; 6] = [0, 1, 2, 0, 2, 3];

fn run_ui(ui: &mut imgui::Ui) {
    imgui::Window::new(im_str!("Hello world"))
        .size([300.0, 100.0], imgui::Condition::FirstUseEver)
        .build(ui, || {
            ui.text(im_str!("Hello world!"));
            ui.text(im_str!("This...is...imgui-rs!"));
            ui.separator();
            let mouse_pos = ui.io().mouse_pos;
            ui.text(format!(
                "Mouse Position: ({:.1},{:.1})",
                mouse_pos[0], mouse_pos[1]
            ));
        });
}

const HEIGHT: u32 = 500;
const WIDTH: u32 = HEIGHT;

fn main() {
    let window_size = (WIDTH, HEIGHT);

    let event_loop = EventLoop::new();

    let wb = glutin::window::WindowBuilder::new()
        .with_title("OpenGL 3D with Reclutch")
        .with_inner_size(glutin::dpi::PhysicalSize::new(window_size.0 as f64, window_size.1 as f64))
        .with_resizable(false);

    let cb = glutin::ContextBuilder::new().with_vsync(true).with_srgb(true);

    let gl_display = glium::Display::new(wb, cb, &event_loop).unwrap();

    let quad_vertex_buffer = glium::VertexBuffer::new(&gl_display, &QUAD_VERTICES).unwrap();
    let quad_indices = glium::IndexBuffer::new(
        &gl_display,
        glium::index::PrimitiveType::TrianglesList,
        &QUAD_INDICES,
    )
    .unwrap();

    let quad_vertex_shader_src = r#"
        #version 140

        in vec3 position;
        in vec2 tex_coord;

        out vec2 frag_tex_coord;

        void main() {
            frag_tex_coord = tex_coord;
            gl_Position = vec4(position, 1.0);
        }
    "#;

    let quad_fragment_shader_src = r#"
        #version 150

        in vec2 frag_tex_coord;
        out vec4 color;

        uniform sampler2D tex;

        void main() {
            color = texture(tex, frag_tex_coord);
        }
    "#;

    let quad_program = glium::Program::from_source(
        &gl_display,
        quad_vertex_shader_src,
        quad_fragment_shader_src,
        None,
    )
    .unwrap();

    let out_texture = glium::texture::SrgbTexture2d::empty_with_format(
        &gl_display,
        glium::texture::SrgbFormat::U8U8U8U8,
        glium::texture::MipmapsOption::NoMipmap,
        window_size.0,
        window_size.1,
    )
    .unwrap();
    let out_texture_depth =
        glium::texture::DepthTexture2d::empty(&gl_display, window_size.0, window_size.1).unwrap();

    let mut skia_context = Some(unsafe {
        glutin::ContextBuilder::new()
            .with_gl(glutin::GlRequest::Specific(glutin::Api::OpenGl, (3, 3)))
            .with_shared_lists(&gl_display.gl_window())
            .with_srgb(true)
            .build_headless(
                &event_loop,
                glutin::dpi::PhysicalSize::new(window_size.0 as _, window_size.1 as _),
            )
            .unwrap()
            .make_current()
            .unwrap()
    });


    let mut display =
        reclutch_skia::SkiaGraphicsDisplay::new_gl_texture(&reclutch_skia::SkiaOpenGlTexture {
            size: (window_size.0 as _, window_size.1 as _),
            texture_id: out_texture.get_id(),
            mip_mapped: false,
        })
        .unwrap();

    let mut last_frame = Instant::now();

    let mut imgui = ImguiContext::create();
    let mut platform = WinitPlatform::init(&mut imgui);
    imgui.set_ini_filename(None);
    imgui.io_mut().display_size = [window_size.0 as f32, window_size.1 as f32];
    let mut renderer = ImguiRenderer::init(&mut imgui, &gl_display).expect("Failed to initialize renderer");

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::WaitUntil(
            std::time::Instant::now() + std::time::Duration::from_nanos(16_666_667),
        );

        platform.handle_event(imgui.io_mut(), &gl_display.gl_window().window(), &event);

        match event {
            Event::RedrawRequested { .. } => {
                let mut out_texture_fb = glium::framebuffer::SimpleFrameBuffer::with_depth_buffer(
                    &gl_display,
                    &out_texture,
                    &out_texture_depth,
                )
                .unwrap();

                let mut frame_target = gl_display.draw();
                let target = &mut out_texture_fb;

                target.clear_color_and_depth((1.0, 1.0, 1.0, 1.0), 1.0);

                skia_context =
                    Some(unsafe { skia_context.take().unwrap().make_current().unwrap() });

                render_skia(&mut display);
                render_imgui_frame(target, &mut imgui, &mut last_frame, &mut renderer);
                frame_target
                    .draw(
                        &quad_vertex_buffer,
                        &quad_indices,
                        &quad_program,
                        &uniform! { tex: &out_texture },
                        &Default::default(),
                    )
                    .unwrap();
                frame_target.finish().unwrap();
            },
            Event::MainEventsCleared => {
                gl_display.gl_window().window().request_redraw();
            },
            Event::WindowEvent { event: WindowEvent::KeyboardInput { input: KeyboardInput { virtual_keycode: Some(VirtualKeyCode::Escape), .. }, .. }, ..} | Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                *control_flow = ControlFlow::Exit;
            },
            _ => return,
        }
    });
}

fn render_imgui_frame(target: &mut glium::framebuffer::SimpleFrameBuffer, imgui: &mut imgui::Context, last_frame: &mut Instant, renderer: &mut ImguiRenderer) {
    let io = imgui.io_mut();

    *last_frame = io.update_delta_time(*last_frame);
    let mut ui = imgui.frame();
    run_ui(&mut ui);

    let draw_data = ui.render();
    renderer.render(target, draw_data).expect("Rendering failed");
}

fn render_skia(display: &mut reclutch_skia::SkiaGraphicsDisplay) {
    let mut surface = &mut display.surface;
    let canvas = surface.canvas();
    let center = (HEIGHT as f32 / 4., WIDTH as f32 / 4.);

    let mut path = skia::Path::new();
    let mut paint = skia::Paint::default();
    paint.set_anti_alias(true);
    paint.set_style(skia::PaintStyle::StrokeAndFill);
    // Face
    paint.set_color(0x55_ffff00);
    path.add_circle(center, center.0, None);
    canvas.draw_path(&path, &paint);
    path = skia::Path::new();
    // Eyes
    paint.set_color(0x55_000000);
    let left_eye = (center.0 - (center.0 / 2.), center.1 - (center.1 / 3.));
    path.add_circle(left_eye, center.0 / 10., None);
    let right_eye = (center.0 + (center.0 / 2.), center.1 - (center.1 / 3.));
    path.add_circle(right_eye, center.0 / 10., None);
    canvas.draw_path(&path, &paint);

    let blur = skia::image_filters::blur(
        (4., 4.),
        skia::TileMode::Clamp,
        None,
        None
    ).unwrap();
    let count = canvas.save();
    canvas.save_layer(&skia::canvas::SaveLayerRec::default().backdrop(&blur));

    path = skia::Path::new();
    path.move_to((0. + (center.0 / 10.), center.1));
    path.cubic_to((0. + (center.0 / 10.), center.1 + (center.1 / 2.)), (WIDTH as f32 / 2. - (center.0 / 10.), center.1 + (center.1 / 2.)), (WIDTH as f32 / 2. - (center.0 / 10.), center.1));
    paint.set_color(0xff_000000);
    paint.set_style(skia::PaintStyle::Stroke);
    canvas.draw_path(&path, &paint);

    canvas.restore_to_count(count);

    display.surface.flush_and_submit();
}
