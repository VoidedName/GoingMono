use crate::color;
use crate::color::RGBA;
use gl::types::{GLenum, GLuint};
use glutin::config::ConfigTemplateBuilder;
use glutin::context::{ContextApi, ContextAttributesBuilder, PossiblyCurrentContext};
use glutin::surface::{Surface, SurfaceAttributesBuilder, SwapInterval, WindowSurface};
use glutin_winit::DisplayBuilder;
use raw_window_handle::HasRawWindowHandle;
use std::ffi::CString;
use std::mem::size_of;
use std::num::NonZeroU32;
use winit::dpi::PhysicalSize;
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowBuilder};

use crate::render::{VNEFullRenderer, VNERenderer, VNERendererCommit, VNERendererWindow};
use glutin::display::GetGlDisplay;
use glutin::prelude::*;
use glutin_winit::{self};

const VERTEX_SHADER: &str = r#"#version 330 core
#extension GL_ARB_explicit_uniform_location : require
  layout (location = 0) in vec3 pos;
  layout (location = 1) in vec4 in_col;
  layout (location = 2) uniform int scale;
  out vec4 pixel_color;

  void main() {
    gl_Position = vec4(pos.x, pos.y, 0.0, 1.0);

    pixel_color = in_col;
  }
"#;

const FRAGMENT_SHADER: &str = r#"#version 330 core
  in vec4 pixel_color;
  out vec4 final_color;

  void main() {
    final_color = pixel_color;
  }
"#;

#[derive(Copy, Clone)]
struct Vertex {
    pub x: f32,
    pub y: f32,
}

fn setup_buffers(width: u32, height: u32) -> Result<Vec<RGBA>, String> {
    let mut vertices: Vec<Vertex> = Vec::with_capacity(width as usize * height as usize);
    let offset_x = 0.5 / width as f32;
    let offset_y = -0.5 / height as f32;
    for y in 0..height {
        for x in 0..width {
            vertices.push(Vertex {
                x: ((x as f32 / width as f32) * 2.0) - 1.0 + offset_x,
                y: -(((y as f32 / height as f32) * 2.0) - 1.0) + offset_y,
            });
        }
    }

    let mut colors: Vec<RGBA> = vec![color::BLACK; width as usize * height as usize];

    unsafe {
        let mut vao = 0;
        let mut vbo = [0, 0];

        gl::GenVertexArrays(1, &mut vao);
        if vao == 0 {
            return Err("Failed generate Vertex Array!".to_string());
        }
        gl::BindVertexArray(vao);

        gl::GenBuffers(1, &mut vbo[0]);
        gl::GenBuffers(1, &mut vbo[1]);
        if vbo[0] == 0 {
            return Err("Failed generate Vertex Buffer!".to_string());
        }
        if vbo[1] == 0 {
            return Err("Failed generate Color Buffer!".to_string());
        }

        gl::BindBuffer(gl::ARRAY_BUFFER, vbo[0]);

        gl::BufferData(
            gl::ARRAY_BUFFER,
            size_of::<Vertex>() as isize * vertices.len() as isize,
            vertices.as_ptr().cast(),
            gl::STATIC_DRAW,
        );

        gl::VertexAttribPointer(
            0,
            2,
            gl::FLOAT,
            gl::FALSE,
            size_of::<Vertex>().try_into().unwrap(),
            0 as *const _,
        );

        gl::EnableVertexAttribArray(0);

        gl::BindBuffer(gl::ARRAY_BUFFER, vbo[1]);

        gl::BufferData(
            gl::ARRAY_BUFFER,
            size_of::<RGBA>() as isize * colors.len() as isize,
            colors.as_ptr().cast(),
            gl::DYNAMIC_DRAW,
        );

        gl::VertexAttribPointer(
            1,
            4,
            gl::FLOAT,
            gl::FALSE,
            size_of::<RGBA>().try_into().unwrap(),
            0 as *const _,
        );

        gl::EnableVertexAttribArray(1);

        let vertex_shader = create_shader(VERTEX_SHADER, gl::VERTEX_SHADER);
        if let Err(err) = vertex_shader {
            return Err(err);
        }

        let fragment_shader = create_shader(FRAGMENT_SHADER, gl::FRAGMENT_SHADER);
        if let Err(err) = fragment_shader {
            return Err(err);
        }

        let shader_program = create_program(vec![vertex_shader.unwrap(), fragment_shader.unwrap()]);
        if let Err(err) = shader_program {
            return Err(err);
        }

        gl::UseProgram(shader_program.unwrap());
    }

    Ok(colors)
}

pub fn create_shader(src: &str, kind: GLenum) -> Result<GLuint, String> {
    unsafe {
        let shader_handler = gl::CreateShader(kind);
        if shader_handler == 0 {
            return Err("Error creating shader handler!".to_string());
        }

        gl::ShaderSource(
            shader_handler,
            1,
            &(src.as_bytes().as_ptr().cast()),
            &(src.len().try_into().unwrap()),
        );
        gl::CompileShader(shader_handler);

        let mut success = 0;
        gl::GetShaderiv(shader_handler, gl::COMPILE_STATUS, &mut success);
        if success == 0 {
            let mut v: Vec<u8> = Vec::with_capacity(1024);
            let mut log_len = 0_i32;
            gl::GetShaderInfoLog(shader_handler, 1024, &mut log_len, v.as_mut_ptr().cast());
            v.set_len(log_len.try_into().unwrap());
            return Err(format!(
                "Shader Compile Error: {}\nsrc\n----\n{}\n----",
                String::from_utf8_lossy(&v),
                src
            )
            .to_string());
        }
        Ok(shader_handler)
    }
}

pub fn create_program(shaders: Vec<GLuint>) -> Result<GLuint, String> {
    unsafe {
        let shader_program = gl::CreateProgram();
        if shader_program == 0 {
            return Err("Error creating shader program!".to_string());
        }

        for shader in shaders.iter() {
            gl::AttachShader(shader_program, *shader);
        }

        gl::LinkProgram(shader_program);
        let mut success = 0;
        gl::GetProgramiv(shader_program, gl::LINK_STATUS, &mut success);
        if success == 0 {
            let mut v: Vec<u8> = Vec::with_capacity(1024);
            let mut log_len = 0_i32;
            gl::GetProgramInfoLog(shader_program, 1024, &mut log_len, v.as_mut_ptr().cast());
            v.set_len(log_len.try_into().unwrap());
            return Err(format!("Program Link Error: {}", String::from_utf8_lossy(&v)).to_string());
        }

        for shader in shaders.iter() {
            gl::DeleteShader(*shader);
        }

        Ok(shader_program)
    }
}

pub struct OpenGLRenderer {
    width: u32,
    height: u32,
    pub window: Window,
    surface: Surface<WindowSurface>,
    context: PossiblyCurrentContext,
    color_buffer: Vec<RGBA>,
}

impl VNERenderer for OpenGLRenderer {
    fn clear_screen(&mut self, color: RGBA) {
        for idx in 0..self.width as usize * self.height as usize {
            self.color_buffer[idx] = color;
        }
    }

    fn draw_pixel(&mut self, position: crate::render::PixelPosition, color: RGBA) {
        let x = position.x;
        let y = position.y;
        if x < 0 || x >= self.width || y < 0 || y >= self.height {
            return;
        }
        let idx = self.xy_index(x, y);
        self.color_buffer[idx] = color;
    }

    fn set_title(&mut self, title: &str) {
        self.window.set_title(title);
    }
}

impl VNERendererCommit for OpenGLRenderer {
    fn commit(&mut self) {
        unsafe {
            gl::BufferSubData(
                gl::ARRAY_BUFFER,
                0,
                size_of::<RGBA>() as isize * self.color_buffer.len() as isize,
                self.color_buffer.as_ptr().cast(),
            );
            gl::DrawArrays(gl::POINTS, 0, self.color_buffer.len() as i32);
        }
        self.surface
            .swap_buffers(&self.context)
            .expect("Failed to commit!");
    }
}

impl OpenGLRenderer {
    fn xy_index(&self, x: u32, y: u32) -> usize {
        (y * self.width + x) as usize
    }
}

impl VNERendererWindow for OpenGLRenderer {
    fn window(&mut self) -> &Window {
        &self.window
    }
}

impl VNEFullRenderer for OpenGLRenderer {}

impl OpenGLRenderer {
    pub fn new<'a>(
        width: u32,
        height: u32,
        scale: u32,
        event_loop: &EventLoop<()>,
    ) -> OpenGLRenderer {
        let window_builder = Some(WindowBuilder::new().with_resizable(false).with_inner_size(
            PhysicalSize {
                width: width * scale,
                height: height * scale,
            },
        ));
        let template = ConfigTemplateBuilder::new().with_alpha_size(8);

        let display_builder = DisplayBuilder::new().with_window_builder(window_builder);

        let (window, gl_config) = display_builder
            .build(event_loop, template, |configs| {
                configs
                    .reduce(|accum, config| {
                        let transparency_check = config.supports_transparency().unwrap_or(false)
                            & !accum.supports_transparency().unwrap_or(false);

                        if transparency_check || config.num_samples() > accum.num_samples() {
                            config
                        } else {
                            accum
                        }
                    })
                    .unwrap()
            })
            .unwrap();

        println!("Picked a config with {} samples", gl_config.num_samples());
        let window = window.unwrap();

        let raw_window_handle = window.raw_window_handle();

        // XXX The display could be obtained from the any object created by it, so we
        // can query it from the config.
        let gl_display = gl_config.display();

        // The context creation part. It can be created before surface and that's how
        // it's expected in multithreaded + multiwindow operation mode, since you
        // can send NotCurrentContext, but not Surface.
        let context_attributes = ContextAttributesBuilder::new().build(Some(raw_window_handle));

        // Since glutin by default tries to create OpenGL core context, which may not be
        // present we should try gles.
        let fallback_context_attributes = ContextAttributesBuilder::new()
            .with_context_api(ContextApi::Gles(None))
            .build(Some(raw_window_handle));

        let mut not_current_gl_context = Some(unsafe {
            gl_display
                .create_context(&gl_config, &context_attributes)
                .unwrap_or_else(|_| {
                    gl_display
                        .create_context(&gl_config, &fallback_context_attributes)
                        .expect("failed to create context")
                })
        });

        let attrs = SurfaceAttributesBuilder::<WindowSurface>::new().build(
            raw_window_handle,
            NonZeroU32::new(width * scale).unwrap(),
            NonZeroU32::new(height * scale).unwrap(),
        );

        let surface = unsafe {
            gl_display
                .create_window_surface(&gl_config, &attrs)
                .unwrap()
        };

        let not_current_context = not_current_gl_context.take().unwrap();
        let context = not_current_context.make_current(&surface).unwrap();

        gl::load_with(|symbol| {
            let symbol = CString::new(symbol).unwrap();
            gl_display.get_proc_address(symbol.as_c_str()).cast()
        });

        let mut color_buffer: Vec<RGBA> =
            setup_buffers(width, height).expect("Failed to setup buffers");

        unsafe {
            gl::PointSize(scale as f32);
        }

        surface
            .set_swap_interval(&context, SwapInterval::DontWait)
            .expect("Failed to set vsync to off!");

        let opengl = OpenGLRenderer {
            width,
            height,
            window,
            surface,
            context,
            color_buffer,
        };

        opengl
    }
}
