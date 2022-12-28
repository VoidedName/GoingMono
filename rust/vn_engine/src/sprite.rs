// inspiration taken from olc::PixelEngine

use crate::color::RGBA;
use crate::opengl::{create_program, create_shader};
use crate::render::{VNEFullRenderer, VNERenderer, VNERendererCommit, VNERendererWindow};
use gl::types::{GLint, GLsizei, GLsizeiptr, GLuint};
use glutin::config::ConfigTemplateBuilder;
use glutin::context::{ContextApi, ContextAttributesBuilder, PossiblyCurrentContext};
use glutin::display::GetGlDisplay;
use glutin::prelude::*;
use glutin::prelude::{GlDisplay, GlSurface, NotCurrentGlContextSurfaceAccessor};
use glutin::surface::{Surface, SurfaceAttributesBuilder, SwapInterval, WindowSurface};
use glutin_winit::DisplayBuilder;
use raw_window_handle::HasRawWindowHandle;
use std::ffi::CString;
use std::mem::size_of;
use std::num::NonZeroU32;
use std::ops;
use winit::dpi::PhysicalSize;
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowBuilder};

#[derive(Copy, Clone)]
pub struct Pixel {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl Default for Pixel {
    fn default() -> Self {
        Self {
            r: 255,
            g: 0,
            b: 0,
            a: 255,
        }
    }
}

pub type PixelPosition = (u32, u32);

impl Pixel {
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Pixel {
        Pixel { r, g, b, a }
    }

    pub fn from_floats(r: f32, g: f32, b: f32, a: f32) -> Pixel {
        Pixel {
            r: (255.0 * r).min(255.0).max(0.0) as u8,
            g: (255.0 * g).min(255.0).max(0.0) as u8,
            b: (255.0 * b).min(255.0).max(0.0) as u8,
            a: (255.0 * a).min(255.0).max(0.0) as u8,
        }
    }
}

macro_rules! pixel {
    ($r: expr, $g: expr, $b: expr, $a: expr) => {
        Pixel {
            r: $r,
            g: $g,
            b: $b,
            a: $a,
        }
    };
    ($r: expr, $g: expr, $b: expr) => {
        Pixel {
            r: $r,
            g: $g,
            b: $b,
            ..Default::default()
        }
    };
    ($r: expr, $g: expr) => {
        Pixel {
            r: $r,
            g: $g,
            ..Default::default()
        }
    };
    ($r: expr) => {
        Pixel {
            r: $r,
            ..Default::default()
        }
    };
    () => {
        Pixel {
            ..Default::default()
        }
    };
}

pub struct Sprite {
    width: usize,
    height: usize,
    data: Vec<Pixel>,
    tex_id: Option<GLuint>,
}

fn create_texture() -> Result<GLuint, String> {
    let mut id = 0;
    unsafe {
        gl::GenTextures(1, &mut id);
        if id == 0 {
            return Err("Failed generate Texture!".to_string());
        }
        gl::BindTexture(gl::TEXTURE_2D, id);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as GLint);
    }
    Ok(id)
}

fn make_active_texture(id: GLuint) {
    unsafe {
        gl::BindTexture(gl::TEXTURE_2D, id);
    }
}

fn update_texture(width: usize, height: usize, data: &Vec<Pixel>) {
    unsafe {
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGBA as GLint,
            width as GLsizei,
            height as GLsizei,
            0,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            data.as_ptr().cast(),
        );
    }
}

fn delete_texture(id: GLuint) {
    unsafe {
        gl::DeleteTextures(1, &id);
    }
}

impl Sprite {
    pub fn new(width: usize, height: usize) -> Sprite {
        Sprite {
            width,
            height,
            data: vec![Default::default(); width * height],
            tex_id: None,
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    fn xy_idx(&self, x: u32, y: u32) -> usize {
        self.width * y as usize + x as usize
    }

    /// make this sprite the current texture loaded in tex_2d
    /// this will dirty the opengl config
    pub fn make_current(&self) {
        let id = self
            .tex_id
            .unwrap_or_else(|| create_texture().expect("Failed to create texture"));
        make_active_texture(id);
        update_texture(self.width, self.height, &self.data);
    }
}

impl ops::Index<PixelPosition> for Sprite {
    type Output = Pixel;

    fn index(&self, (x, y): PixelPosition) -> &Self::Output {
        if x as usize >= self.width || y as usize >= self.height {
            panic!(
                "Index {:?} out of range of {:?}",
                (x, y),
                (self.width, self.height)
            )
        }
        &self.data[self.xy_idx(x, y)]
    }
}

impl ops::IndexMut<PixelPosition> for Sprite {
    fn index_mut(&mut self, (x, y): PixelPosition) -> &mut Self::Output {
        if x as usize >= self.width || y as usize >= self.height {
            panic!(
                "Index {:?} out of range of {:?}",
                (x, y),
                (self.width, self.height)
            )
        }
        let idx = self.xy_idx(x, y);
        &mut self.data[idx]
    }
}

impl Drop for Sprite {
    fn drop(&mut self) {
        if let Some(id) = self.tex_id {
            delete_texture(id);
        }
    }
}

type TexturedPixelPixelPos = [f32; 3];
type TexturedPixelTexturePos = [f32; 2];
type TexturedPixel = [f32; 5]; // x y z | t s

fn setup() {
    // create layer buffer
    let mut vb_layer = 0;
    let mut va_layer = 0;
    unsafe {
        gl::GenVertexArrays(1, &mut va_layer);
        gl::BindVertexArray(va_layer);

        gl::GenBuffers(1, &mut vb_layer);
        gl::BindBuffer(gl::ARRAY_BUFFER, vb_layer);
    };

    unsafe {
        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            size_of::<TexturedPixel>() as i32,
            0 as *const _,
        );

        gl::VertexAttribPointer(
            1,
            2,
            gl::FLOAT,
            gl::FALSE,
            size_of::<TexturedPixel>() as i32,
            (size_of::<f32>() * 3) as *const _,
        );

        gl::EnableVertexAttribArray(0);
        gl::EnableVertexAttribArray(1);
    };

    // shaders & program
    let v_shader =
        create_shader(VERTEX_SHADER, gl::VERTEX_SHADER).expect("Unable to create vertex shader!");
    let f_shader = create_shader(FRAGMENT_SHADER, gl::FRAGMENT_SHADER)
        .expect("Unable to create fragment shader!");
    let program = create_program(vec![v_shader, f_shader]).expect("Unable to create program!");
    unsafe {
        gl::UseProgram(program);
    };
}

const VERTEX_SHADER: &str = r"
#version 330 core
layout(location = 0) in vec3 in_pos;
layout(location = 1) in vec2 in_tex;
out vec2 out_tex;

void main(){
    float p = 1.0 / in_pos.z;
    gl_Position = p * vec4(in_pos.x, in_pos.y, 0.0, 1.0);
    out_tex = p * in_tex;
}
";

const FRAGMENT_SHADER: &str = r"
#version 330 core
out vec4 pixel;
in vec2 out_tex;
uniform sampler2D sprite_texture;

void main(){
    pixel = texture(sprite_texture, out_tex);
}
";

pub struct SpriteBased {
    window: Window,
    context: PossiblyCurrentContext,
    surface: Surface<WindowSurface>,
    sprite: Sprite,
    scale: f32,
}

impl SpriteBased {
    pub fn new(width: usize, height: usize, scale: u32, event_loop: &EventLoop<()>) -> SpriteBased {
        let window_builder = Some(WindowBuilder::new().with_resizable(false).with_inner_size(
            PhysicalSize {
                width: width as u32 * scale,
                height: height as u32 * scale,
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
            NonZeroU32::new((width as u32 * scale) as u32).unwrap(),
            NonZeroU32::new((height as u32 * scale) as u32).unwrap(),
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

        setup();

        surface
            .set_swap_interval(&context, SwapInterval::DontWait)
            .expect("Failed to set vsync to off!");

        unsafe {
            gl::Enable(gl::TEXTURE_2D);
            gl::PointSize(scale as f32);
        }

        SpriteBased {
            sprite: Sprite::new(width, height),
            window,
            surface,
            context,
            scale: scale as f32,
        }
    }
}

impl VNERenderer for SpriteBased {
    fn clear_screen(&mut self, color: RGBA) {
        let color = Pixel::from_floats(color.r, color.g, color.b, color.a);
        for idx in 0..self.sprite.width as usize * self.sprite.height as usize {
            self.sprite.data[idx] = color;
        }
    }

    fn draw_pixel(&mut self, position: crate::render::PixelPosition, color: RGBA) {
        let x = position.x;
        let y = position.y;
        if x >= self.sprite.width() as u32 || y >= self.sprite.height() as u32 {
            return;
        }

        self.sprite[(x, y)] = Pixel::from_floats(color.r, color.g, color.b, color.a);
    }

    fn set_title(&mut self, title: &str) {
        self.window.set_title(title);
    }
}

impl VNERendererWindow for SpriteBased {
    fn window(&mut self) -> &Window {
        &self.window
    }
}

impl VNERendererCommit for SpriteBased {
    fn commit(&mut self) {
        self.sprite.make_current();

        #[rustfmt::skip]
        let vec: Vec<TexturedPixel> = vec![
            //  x     y    z    t    s
            [-1.0, -1.0, 1.0, 0.0, 1.0],
            [ 1.0, -1.0, 1.0, 1.0, 1.0],
            [-1.0,  1.0, 1.0, 0.0, 0.0],
            [ 1.0,  1.0, 1.0, 1.0, 0.0],
        ];

        unsafe {
            gl::BufferData(
                gl::ARRAY_BUFFER,
                size_of::<TexturedPixel>() as isize * vec.len() as isize,
                vec.as_ptr().cast(),
                gl::STREAM_DRAW,
            );
            gl::DrawArrays(gl::TRIANGLE_STRIP, 0, vec.len() as GLint);
        }

        self.surface
            .swap_buffers(&self.context)
            .expect("Failed to commit!");
    }
}

impl VNEFullRenderer for SpriteBased {}
