use std::ffi::{CString, NulError};
use std::num::NonZeroU32;
use std::path::Path;
use std::ptr;
use std::string::FromUtf8Error;

use crate::opengl::image_format::ImageFormatBase;
use gl::types::{GLenum, GLint, GLsizei, GLsizeiptr, GLuint};
use glutin::config::{Config, ConfigTemplateBuilder};
use glutin::context::ContextApi::OpenGl;
use glutin::context::{ContextAttributesBuilder, PossiblyCurrentContext, Version};
use glutin::display::GetGlDisplay;
use glutin::prelude::*;
use glutin::surface::{Surface, SurfaceAttributesBuilder, SwapInterval, WindowSurface};
use glutin_winit::DisplayBuilder;
use image::{EncodableLayout, ImageError};
use raw_window_handle::HasRawWindowHandle;
use thiserror::Error;
use winit::dpi::PhysicalSize;
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowBuilder};

pub mod quad {
    use crate::opengl::{Buffer, Shader, ShaderError, ShaderProgram, VertexArray};
    use crate::set_attribute;
    use gl::types::GLsizei;
    use std::ptr;

    type Pos = [f32; 2];
    type TextureCoords = [f32; 2];

    #[repr(C, packed)]
    struct Vertex(Pos, TextureCoords);

    #[rustfmt::skip]
    const VERTICES: [Vertex; 4] = [
        Vertex([-1.0, -1.0], [0.0, 1.0]),
        Vertex([1.0, -1.0], [1.0, 1.0]),
        Vertex([1.0, 1.0], [1.0, 0.0]),
        Vertex([-1.0, 1.0], [0.0, 0.0]),
    ];

    #[rustfmt::skip]
    const INDICES: [i32; 6] = [
        0, 1, 2,
        2, 3, 0
    ];
    pub const QUAD_VERTEX_POSITION: &str = "position";
    pub const QUAD_RESOLUTION_TEXTURE_COORDINATES: &str = "vertexTexCoord";

    pub const QUAD_MOUSE_UNIFORM: &str = "u_mouse";
    pub const QUAD_RESOLUTION_UNIFORM: &str = "u_resolution";

    pub struct Quad {
        pub program: ShaderProgram,
        pub vertex_array: VertexArray,
        vertex_buffer: Buffer,
        index_buffer: Buffer,
    }

    impl Quad {
        pub unsafe fn new(
            vertex_shader: Shader,
            fragment_shader: Shader,
        ) -> Result<Self, ShaderError> {
            let program = ShaderProgram::new(&[vertex_shader, fragment_shader])?;

            let vertex_array = VertexArray::new();
            vertex_array.bind();

            let vertex_buffer = Buffer::new(gl::ARRAY_BUFFER);
            vertex_buffer.set_data(&VERTICES, gl::STATIC_DRAW);

            let index_buffer = Buffer::new(gl::ELEMENT_ARRAY_BUFFER);
            index_buffer.set_data(&INDICES, gl::STATIC_DRAW);

            let pos_attrib = program.get_attrib_location(QUAD_VERTEX_POSITION)?;
            set_attribute!(vertex_array, pos_attrib, Vertex::0);

            Ok(Self {
                program,
                vertex_array,
                vertex_buffer,
                index_buffer,
            })
        }

        pub unsafe fn draw(&self, resolution: (f32, f32), mouse_pos: (f32, f32)) {
            self.program.apply();
            self.program
                .set_float32_2_uniform(QUAD_RESOLUTION_UNIFORM, resolution.0, resolution.1)
                .unwrap();
            self.program
                .set_float32_2_uniform(QUAD_MOUSE_UNIFORM, mouse_pos.0, mouse_pos.1)
                .unwrap();
            gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, ptr::null());
        }

        pub unsafe fn draw_instanced(
            &self,
            resolution: (f32, f32),
            mouse_pos: (f32, f32),
            instances: usize,
        ) {
            self.program.apply();
            self.program
                .set_float32_2_uniform(QUAD_RESOLUTION_UNIFORM, resolution.0, resolution.1)
                .unwrap();
            self.program
                .set_float32_2_uniform(QUAD_MOUSE_UNIFORM, mouse_pos.0, mouse_pos.1)
                .unwrap();
            gl::DrawElementsInstanced(
                gl::TRIANGLES,
                6,
                gl::UNSIGNED_INT,
                ptr::null(),
                instances as GLsizei,
            );
        }
    }
}

#[derive(Debug, Error)]
pub enum ShaderError {
    #[error("Error while compiling shader: {0}")]
    CompilationError(String),
    #[error("Error while linking shaders: {0}")]
    LinkingError(String),
    #[error{"{0}"}]
    Utf8Error(#[from] FromUtf8Error),
    #[error{"{0}"}]
    NulError(#[from] NulError),
}

pub struct Graphics {
    pub window: Window,
    surface: Surface<WindowSurface>,
    context: PossiblyCurrentContext,
}

impl Graphics {
    pub unsafe fn new(width: u32, height: u32, scale: f32) -> (Self, EventLoop<()>) {
        let event_loop = EventLoop::new();

        let window_builder = Some(WindowBuilder::new().with_resizable(false).with_inner_size(
            PhysicalSize {
                width: width as f32 * scale,
                height: height as f32 * scale,
            },
        ));

        let template = ConfigTemplateBuilder::new().with_alpha_size(8);

        let display_builder = DisplayBuilder::new().with_window_builder(window_builder);

        let (window, gl_config) = Self::build_window(&event_loop, template, display_builder);

        println!("Picked a config with {} samples", gl_config.num_samples());

        let raw_window_handle = window.raw_window_handle();
        let gl_display = gl_config.display();
        let context_attributes = ContextAttributesBuilder::new()
            .with_context_api(OpenGl(Some(Version::new(4, 6))))
            .build(Some(raw_window_handle));

        let not_current_context = gl_display
            .create_context(&gl_config, &context_attributes)
            .unwrap();

        let attrs = SurfaceAttributesBuilder::<WindowSurface>::new().build(
            raw_window_handle,
            NonZeroU32::new((width as f32 * scale) as u32).unwrap(),
            NonZeroU32::new((height as f32 * scale) as u32).unwrap(),
        );

        let surface = gl_display
            .create_window_surface(&gl_config, &attrs)
            .unwrap();

        let context = not_current_context.make_current(&surface).unwrap();

        gl::load_with(|symbol| {
            let symbol = CString::new(symbol).unwrap();
            gl_display.get_proc_address(symbol.as_c_str()).cast()
        });

        gl::Enable(gl::TEXTURE_2D);
        gl::PointSize(scale as f32);

        surface
            .set_swap_interval(&context, SwapInterval::DontWait)
            .expect("Failed to set vsync to off!");

        (
            Self {
                window,
                surface,
                context,
            },
            event_loop,
        )
    }

    unsafe fn build_window(
        event_loop: &EventLoop<()>,
        template: ConfigTemplateBuilder,
        display_builder: DisplayBuilder,
    ) -> (Window, Config) {
        let (w, c) = display_builder
            .build(&event_loop, template, |configs| {
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
        (w.unwrap(), c)
    }

    pub unsafe fn swap_buffers(&self) {
        self.surface.swap_buffers(&self.context).unwrap();
    }
}

pub struct Shader {
    pub id: GLuint,
}

impl Shader {
    pub unsafe fn new(source_code: &str, shader_type: GLenum) -> Result<Shader, ShaderError> {
        let source_code = CString::new(source_code)?;
        let shader = Self {
            id: gl::CreateShader(shader_type),
        };

        gl::ShaderSource(shader.id, 1, &source_code.as_ptr(), ptr::null());
        gl::CompileShader(shader.id);

        let mut success: GLint = 0;
        gl::GetShaderiv(shader.id, gl::COMPILE_STATUS, &mut success);

        if success == 1 {
            Ok(shader)
        } else {
            let mut error_log_size: GLint = 0;
            gl::GetShaderiv(shader.id, gl::INFO_LOG_LENGTH, &mut error_log_size);
            let mut error_log: Vec<u8> = Vec::with_capacity(error_log_size as usize);
            gl::GetShaderInfoLog(
                shader.id,
                error_log_size,
                &mut error_log_size,
                error_log.as_mut_ptr() as *mut _,
            );

            error_log.set_len(error_log_size as usize);
            let log = String::from_utf8(error_log)?;
            Err(ShaderError::CompilationError(log))
        }
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteShader(self.id);
        }
    }
}

pub struct ShaderProgram {
    pub id: GLuint,
}

impl ShaderProgram {
    pub unsafe fn new(shaders: &[Shader]) -> Result<ShaderProgram, ShaderError> {
        let program = Self {
            id: gl::CreateProgram(),
        };

        for shader in shaders {
            gl::AttachShader(program.id, shader.id);
        }

        gl::LinkProgram(program.id);

        let mut success: GLint = 0;
        gl::GetProgramiv(program.id, gl::LINK_STATUS, &mut success);

        if success == 1 {
            Ok(program)
        } else {
            let mut error_log_size: GLint = 0;
            gl::GetProgramiv(program.id, gl::INFO_LOG_LENGTH, &mut error_log_size);
            let mut error_log: Vec<u8> = Vec::with_capacity(error_log_size as usize);
            gl::GetProgramInfoLog(
                program.id,
                error_log_size,
                &mut error_log_size,
                error_log.as_mut_ptr() as *mut _,
            );

            error_log.set_len(error_log_size as usize);
            let log = String::from_utf8(error_log)?;
            Err(ShaderError::LinkingError(log))
        }
    }

    pub unsafe fn apply(&self) {
        gl::UseProgram(self.id);
    }

    pub unsafe fn set_int_uniform(&self, name: &str, value: i32) -> Result<(), ShaderError> {
        self.apply();
        let uniform = CString::new(name)?;
        gl::Uniform1i(gl::GetUniformLocation(self.id, uniform.as_ptr()), value);
        Ok(())
    }

    pub unsafe fn set_float32_uniform(&self, name: &str, value: f32) -> Result<(), ShaderError> {
        self.apply();
        let uniform = CString::new(name)?;
        gl::Uniform1f(gl::GetUniformLocation(self.id, uniform.as_ptr()), value);
        Ok(())
    }

    pub unsafe fn set_float32_2_uniform(
        &self,
        name: &str,
        value1: f32,
        value2: f32,
    ) -> Result<(), ShaderError> {
        self.apply();
        let uniform = CString::new(name)?;
        gl::Uniform2f(
            gl::GetUniformLocation(self.id, uniform.as_ptr()),
            value1,
            value2,
        );
        Ok(())
    }

    pub unsafe fn set_float32_2_uniform_vector(
        &self,
        name: &str,
        data: &[(f32, f32)],
    ) -> Result<(), ShaderError> {
        self.apply();
        let uniform = CString::new(name)?;
        gl::Uniform2fv(
            gl::GetUniformLocation(self.id, uniform.as_ptr()),
            data.len() as GLsizei,
            data.as_ptr().cast(),
        );
        Ok(())
    }
}

impl Drop for ShaderProgram {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.id);
        }
    }
}

pub struct Buffer {
    pub id: GLuint,
    target: GLuint,
}

impl Buffer {
    pub unsafe fn new(target: GLuint) -> Self {
        let mut id: GLuint = 0;
        gl::GenBuffers(1, &mut id);
        Self { id, target }
    }

    pub unsafe fn bind(&self) {
        gl::BindBuffer(self.target, self.id);
    }

    pub unsafe fn bind_base(&self, unit: GLuint) {
        gl::BindBufferBase(self.target, unit, self.id);
    }

    pub unsafe fn set_data<D>(&self, data: &[D], usage: GLuint) {
        self.bind();
        let (_, data_bytes, _) = data.align_to::<u8>();
        gl::BufferData(
            self.target,
            data_bytes.len() as GLsizeiptr,
            data_bytes.as_ptr() as *const _,
            usage,
        );
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, [self.id].as_ptr());
        }
    }
}

pub struct VertexArray {
    pub id: GLuint,
}

impl VertexArray {
    pub unsafe fn new() -> Self {
        let mut id: GLuint = 0;
        gl::GenVertexArrays(1, &mut id);
        Self { id }
    }

    pub unsafe fn bind(&self) {
        gl::BindVertexArray(self.id);
    }

    pub unsafe fn set_attribute<V: Sized>(
        &self,
        attrib_pos: GLuint,
        components: GLint,
        offset: GLint,
    ) {
        self.bind();
        gl::VertexAttribPointer(
            attrib_pos,
            components,
            gl::FLOAT,
            gl::FALSE,
            std::mem::size_of::<V>() as GLint,
            offset as *const _,
        );
        gl::EnableVertexAttribArray(attrib_pos)
    }
}

impl Drop for VertexArray {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, [self.id].as_ptr());
        }
    }
}

impl ShaderProgram {
    pub unsafe fn get_attrib_location(&self, attrib: &str) -> Result<GLuint, NulError> {
        let attrib = CString::new(attrib)?;
        Ok(gl::GetAttribLocation(self.id, attrib.as_ptr()) as GLuint)
    }
}

#[macro_export]
macro_rules! set_attribute {
    ($vbo:ident, $pos:tt, $t:ident :: $field:tt) => {{
        let dummy = core::mem::MaybeUninit::<$t>::uninit();
        let dummy_ptr = dummy.as_ptr();
        let member_ptr = core::ptr::addr_of!((*dummy_ptr).$field);
        const fn size_of_raw<T>(_: *const T) -> usize {
            core::mem::size_of::<T>()
        }
        let member_offset = member_ptr as i32 - dummy_ptr as i32;
        $vbo.set_attribute::<$t>(
            $pos,
            (size_of_raw(member_ptr) / core::mem::size_of::<f32>()) as i32,
            member_offset,
        )
    }};
}

pub struct Texture<T: ImageFormatBase> {
    pub id: GLuint,
    format: T,
}

impl<T: ImageFormatBase> Texture<T> {
    pub unsafe fn new(format: T) -> Self {
        let mut id: GLuint = 0;
        gl::GenTextures(1, &mut id);

        let _self = Self { id, format };
        _self.bind();

        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint);

        _self
    }
}

impl<T: ImageFormatBase> Drop for Texture<T> {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, [self.id].as_ptr());
        }
    }
}

pub enum ReadWriteAccess {
    Read,
    Write,
    ReadWrite,
}

impl ReadWriteAccess {
    fn value(&self) -> GLenum {
        match self {
            Self::Read => gl::READ_ONLY,
            Self::Write => gl::WRITE_ONLY,
            Self::ReadWrite => gl::READ_WRITE,
        }
    }
}

pub mod image_format {
    use gl::types::GLenum;

    pub trait ImageFormatBase {
        type RustDataType: Sized;
        fn internal_format(&self) -> GLenum;
        fn format(&self) -> GLenum;
        fn data_type(&self) -> GLenum;
    }

    pub struct RGBAFloat {}

    impl ImageFormatBase for RGBAFloat {
        type RustDataType = [f32; 4];

        fn internal_format(&self) -> GLenum {
            gl::RGBA32F
        }

        fn format(&self) -> GLenum {
            gl::RGBA
        }

        fn data_type(&self) -> GLenum {
            gl::FLOAT
        }
    }
}

impl<T: ImageFormatBase> Texture<T> {
    pub unsafe fn bind(&self) {
        gl::BindTexture(gl::TEXTURE_2D, self.id);
    }

    pub unsafe fn bind_image(&self, unit: GLuint, access: ReadWriteAccess) {
        gl::BindImageTexture(
            unit,
            self.id,
            0,
            gl::FALSE,
            0,
            access.value(),
            self.format.internal_format(),
        );
    }

    // might need a different setter for other types of textures...
    pub unsafe fn set_data(&self, width: i32, height: i32, data: &[T::RustDataType]) {
        self.bind();
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            self.format.internal_format() as GLint,
            width as i32,
            height as i32,
            0,
            self.format.format(),
            self.format.data_type(),
            data.as_ptr().cast(),
        );
    }

    pub unsafe fn activate(&self, unit: GLuint) {
        gl::ActiveTexture(unit);
        self.bind();
    }
}
