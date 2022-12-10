use std::{
  ffi::{c_void, CStr, CString},
  mem, ptr,
};

use gl::types::*;

use super::shader::{FRAGMENT_SHADER_SOURCE, INDICES, VERTEX_SHADER_SOURCE, VERTICES};

const BUF_SIZE: usize = 512;

pub(crate) unsafe fn compile_shader() -> u32 {
  let mut success = gl::FALSE as GLint;
  let mut info_log = Vec::with_capacity(BUF_SIZE);
  info_log.set_len(BUF_SIZE - 1); // space for trailing null character

  // build and compile our shader program
  let vertex_shader = gl::CreateShader(gl::VERTEX_SHADER);
  let c_str_vert = CString::new(VERTEX_SHADER_SOURCE.as_bytes()).unwrap();
  gl::ShaderSource(vertex_shader, 1, &c_str_vert.as_ptr(), ptr::null());
  gl::CompileShader(vertex_shader);

  gl::GetShaderiv(vertex_shader, gl::COMPILE_STATUS, &mut success);
  if success != gl::TRUE as GLint {
    gl::GetShaderInfoLog(
      vertex_shader,
      BUF_SIZE as i32,
      ptr::null_mut(),
      info_log.as_mut_ptr() as *mut GLchar,
    );
    let c_str = CStr::from_ptr(info_log.as_mut_ptr() as *mut GLchar);
    log::error!("Vertex shader compile failed {}", c_str.to_str().unwrap());
  }

  let fragment_shader = gl::CreateShader(gl::FRAGMENT_SHADER);
  let c_str_frag = CString::new(FRAGMENT_SHADER_SOURCE.as_bytes()).unwrap();
  gl::ShaderSource(fragment_shader, 1, &c_str_frag.as_ptr(), ptr::null());
  gl::CompileShader(fragment_shader);

  gl::GetShaderiv(fragment_shader, gl::COMPILE_STATUS, &mut success);
  if success != gl::TRUE as GLint {
    gl::GetShaderInfoLog(
      vertex_shader,
      BUF_SIZE as i32,
      ptr::null_mut(),
      info_log.as_mut_ptr() as *mut GLchar,
    );
    let c_str = CStr::from_ptr(info_log.as_mut_ptr() as *mut GLchar);
    log::error!("Fragment shader compile failed {}", c_str.to_str().unwrap());
  }

  // link shaders
  let shader_program = gl::CreateProgram();
  gl::AttachShader(shader_program, vertex_shader);
  gl::AttachShader(shader_program, fragment_shader);
  gl::LinkProgram(shader_program);

  gl::GetProgramiv(shader_program, gl::LINK_STATUS, &mut success);
  if success != gl::TRUE as GLint {
    gl::GetProgramInfoLog(
      shader_program,
      BUF_SIZE as i32,
      ptr::null_mut(),
      info_log.as_mut_ptr() as *mut GLchar,
    );
    let c_str = CStr::from_ptr(info_log.as_mut_ptr() as *mut GLchar);
    log::error!("Shader program link failed {}", c_str.to_str().unwrap());
  }

  gl::DeleteShader(vertex_shader);
  gl::DeleteShader(fragment_shader);

  shader_program
}

pub(crate) unsafe fn create_vao() -> u32 {
  #[allow(non_snake_case)]
  let (mut VBO, mut VAO, mut EBO) = (0, 0, 0);
  gl::GenVertexArrays(1, &mut VAO);
  gl::GenBuffers(1, &mut VBO);
  gl::GenBuffers(1, &mut EBO);
  gl::BindVertexArray(VAO);

  gl::BindBuffer(gl::ARRAY_BUFFER, VBO);
  gl::BufferData(
    gl::ARRAY_BUFFER,
    (VERTICES.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
    &VERTICES[0] as *const f32 as *const c_void,
    gl::STATIC_DRAW,
  );

  gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, EBO);
  gl::BufferData(
    gl::ELEMENT_ARRAY_BUFFER,
    (INDICES.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
    &INDICES[0] as *const i32 as *const c_void,
    gl::STATIC_DRAW,
  );

  let stripe = 5 * mem::size_of::<GLfloat>() as GLsizei;
  gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, stripe, ptr::null());
  gl::EnableVertexAttribArray(0);

  gl::VertexAttribPointer(
    1,
    2,
    gl::FLOAT,
    gl::FALSE,
    stripe,
    (2 * mem::size_of::<GLfloat>()) as *const c_void,
  );
  gl::EnableVertexAttribArray(1);

  gl::BindBuffer(gl::ARRAY_BUFFER, 0);
  gl::BindVertexArray(0);
  VAO
}

pub(crate) unsafe fn create_texture(shader: u32) -> u32 {
  let mut texture: u32 = 1;
  gl::GenTextures(1, &mut texture);
  gl::BindTexture(gl::TEXTURE_2D, texture);
  gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
  gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
  gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
  gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);

  gl::Uniform1i(
    gl::GetUniformLocation(shader, "texture".as_ptr() as *const i8),
    0,
  );
  gl::BindTexture(gl::TEXTURE_2D, 0);
  texture
}

pub(crate) unsafe fn set_texture(buf: image::RgbaImage) {
  let (width, height) = (buf.width(), buf.height());
  let data = buf.into_vec();
  gl::TexImage2D(
    gl::TEXTURE_2D,
    0,
    gl::RGBA as i32,
    width as i32,
    height as i32,
    0,
    gl::RGBA,
    gl::UNSIGNED_BYTE,
    &data[0] as *const u8 as *const c_void,
  )
}

pub(crate) unsafe fn draw_frame(shader: u32, VAO: u32, texture: u32) {
  gl::Clear(gl::COLOR_BUFFER_BIT);

  gl::ActiveTexture(gl::TEXTURE0);
  gl::BindTexture(gl::TEXTURE_2D, texture);
  // draw our first triangle
  gl::UseProgram(shader);
  gl::BindVertexArray(VAO);
  gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, ptr::null());
}

#[cfg(test)]
mod test {
  use std::path::Path;

  use glfw::Context;

  use super::*;
  use crate::render::glfw_window::*;

  // settings
  #[test]
  pub fn launch_window() {
    const SCR_WIDTH: u32 = 800;
    const SCR_HEIGHT: u32 = 600;
    // glfw: initialize and configure
    // ------------------------------
    let mut glfw = init_glfw();

    // glfw window creation
    // --------------------
    let (mut window, _) = init_window_and_gl(&glfw, SCR_WIDTH, SCR_HEIGHT);

    #[allow(non_snake_case)]
    let (shader, VAO) = unsafe { (compile_shader(), create_vao()) };
    let texture = unsafe {
      let texture = create_texture(shader);

      gl::BindTexture(gl::TEXTURE_2D, texture);
      let img = image::open(&Path::new("assets/logo.png")).expect("Failed to open image.");
      let rgba = img.to_rgba8();
      set_texture(rgba);
      gl::GenerateMipmap(gl::TEXTURE_2D);
      gl::BindTexture(gl::TEXTURE_2D, 0);

      texture
    };
    // render loop
    // -----------
    while !window.should_close() {
      // render
      // ------
      unsafe {
        gl::ClearColor(0.2, 0.3, 0.3, 1.0);
        draw_frame(shader, VAO, texture);
      }

      window.swap_buffers();
      glfw.poll_events();
    }
  }
}
