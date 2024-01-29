use wasm_bindgen::prelude::*;
use web_sys::WebGlVertexArrayObject;
use web_sys::{WebGl2RenderingContext, WebGlProgram, WebGlShader, WebGlTexture};

type GL = WebGl2RenderingContext;

use crate::instance::FrameBuffer;

use super::shader::{FRAGMENT_SHADER_SOURCE, INDICES, VERTEX_SHADER_SOURCE, VERTICES};

#[derive(Clone, Debug)]
pub struct GlWrapper {
  context: GL,
  program: WebGlProgram,
  vao: WebGlVertexArrayObject,
  texture: WebGlTexture,
}

impl GlWrapper {
  pub fn init_webgl(context: &GL) -> Result<Self, JsValue> {
    let vert_shader = compile_shader(&context, GL::VERTEX_SHADER, VERTEX_SHADER_SOURCE)?;
    let frag_shader = compile_shader(&context, GL::FRAGMENT_SHADER, FRAGMENT_SHADER_SOURCE)?;
    let program = link_program(&context, &vert_shader, &frag_shader)?;
    context.use_program(Some(&program));

    let vao = create_vao(&context, &program, &VERTICES, &INDICES)?;
    let texture = create_texture(&context, &program)?;
    check_error(context);
    Ok(Self {
      context: context.clone(),
      program,
      vao,
      texture,
    })
  }

  pub fn render(&self, frame_data: FrameBuffer) -> Result<(), JsValue> {
    set_texture(&self.context, &self.texture, frame_data)?;
    self.draw_frame();
    Ok(())
  }

  pub(crate) fn draw_frame(&self) {
    let context = &self.context;
    let program = &self.program;
    let vao = &self.vao;
    let texture = &self.texture;

    context.use_program(Some(program));
    context.clear(GL::COLOR_BUFFER_BIT);
    context.active_texture(GL::TEXTURE0);
    context.bind_texture(GL::TEXTURE_2D, Some(texture));
    context.bind_vertex_array(Some(vao));
    // context.bind_buffer(GL::ELEMENT_ARRAY_BUFFER, Some(vbo));
    context.draw_elements_with_i32(GL::TRIANGLES, 6, GL::UNSIGNED_INT, 0);
    context.bind_vertex_array(None);
    check_error(context);
  }
}

pub(crate) fn compile_shader(
  context: &GL,
  shader_type: u32,
  source: &str,
) -> Result<WebGlShader, String> {
  let shader = context
    .create_shader(shader_type)
    .ok_or_else(|| String::from("Unable to create shader object"))?;
  context.shader_source(&shader, source);
  context.compile_shader(&shader);

  if context
    .get_shader_parameter(&shader, GL::COMPILE_STATUS)
    .as_bool()
    .unwrap_or(false)
  {
    Ok(shader)
  } else {
    Err(
      context
        .get_shader_info_log(&shader)
        .unwrap_or_else(|| String::from("Unknown error creating shader")),
    )
  }
}

pub(crate) fn link_program(
  context: &GL,
  vert_shader: &WebGlShader,
  frag_shader: &WebGlShader,
) -> Result<WebGlProgram, String> {
  let program = context
    .create_program()
    .ok_or_else(|| String::from("Unable to create shader object"))?;

  context.attach_shader(&program, vert_shader);
  context.attach_shader(&program, frag_shader);
  context.link_program(&program);

  if context
    .get_program_parameter(&program, GL::LINK_STATUS)
    .as_bool()
    .unwrap_or(false)
  {
    Ok(program)
  } else {
    Err(
      context
        .get_program_info_log(&program)
        .unwrap_or_else(|| String::from("Unknown error creating program object")),
    )
  }
}

pub(crate) fn create_vao(
  context: &GL,
  program: &WebGlProgram,
  vertices: &[f32],
  indices: &[i32],
) -> Result<WebGlVertexArrayObject, String> {
  let vao = context
    .create_vertex_array()
    .ok_or_else(|| String::from("Unable to create vertex array object"))?;

  context.bind_vertex_array(Some(&vao));

  let vbo = context.create_buffer().ok_or("failed to create buffer")?;
  context.bind_buffer(GL::ARRAY_BUFFER, Some(&vbo));
  unsafe {
    let array = js_sys::Float32Array::view(vertices);
    context.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &array, GL::STATIC_DRAW);
  }

  let ebo = context.create_buffer().ok_or("failed to create buffer")?;
  context.bind_buffer(GL::ELEMENT_ARRAY_BUFFER, Some(&ebo));
  unsafe {
    let array = js_sys::Int32Array::view(indices);
    context.buffer_data_with_array_buffer_view(GL::ELEMENT_ARRAY_BUFFER, &array, GL::STATIC_DRAW);
  }

  let stride = 5 * 4;
  let location_p = context.get_attrib_location(program, "position") as u32;
  context.vertex_attrib_pointer_with_i32(location_p, 2, GL::FLOAT, false, stride, 0);
  context.enable_vertex_attrib_array(location_p);

  let location_t = context.get_attrib_location(program, "texCoord") as u32;
  context.vertex_attrib_pointer_with_i32(location_t, 2, GL::FLOAT, false, stride, 2 * 4);
  context.enable_vertex_attrib_array(location_t);

  check_error(context);
  Ok(vao)
}

pub(crate) fn create_texture(context: &GL, program: &WebGlProgram) -> Result<WebGlTexture, String> {
  let texture = context
    .create_texture()
    .ok_or_else(|| String::from("Unable to create texture"))?;

  context.active_texture(GL::TEXTURE0);
  context.bind_texture(GL::TEXTURE_2D, Some(&texture));

  context.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_WRAP_S, GL::CLAMP_TO_EDGE as i32);
  context.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_WRAP_T, GL::CLAMP_TO_EDGE as i32);
  context.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_MIN_FILTER, GL::LINEAR as i32);
  context.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_MAG_FILTER, GL::LINEAR as i32);

  context.uniform1i(context.get_uniform_location(program, "texture").as_ref(), 0);
  // context.generate_mipmap(GL::TEXTURE_2D);
  context.bind_texture(GL::TEXTURE_2D, None);
  check_error(context);
  Ok(texture)
}

pub(crate) fn set_texture(
  context: &GL,
  texture: &WebGlTexture,
  buf: image::RgbaImage,
) -> Result<(), JsValue> {
  let (width, height) = (buf.width() as i32, buf.height() as i32);
  context.bind_texture(GL::TEXTURE_2D, Some(texture));
  context.active_texture(GL::TEXTURE0);
  unsafe {
    let array = js_sys::Uint8Array::view(buf.into_vec().as_slice());

    context.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_array_buffer_view_and_src_offset(
      GL::TEXTURE_2D,
      0, GL::RGBA as i32,
      width, height, 0, GL::RGBA,
      GL::UNSIGNED_BYTE, &array, 0)?;
  }
  context.bind_texture(GL::TEXTURE_2D, None);
  check_error(context);
  Ok(())
}

pub(crate) fn check_error(context: &GL) {
  let e = context.get_error();
  if e != GL::NO_ERROR {
    log::error!("Error: {}", e);
  }
}
