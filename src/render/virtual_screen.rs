use image::RgbaImage;
use sfml::graphics::{Color, Drawable, PrimitiveType, RenderStates, RenderTarget, VertexArray};
use sfml::system::{Vector2f, Vector2u};

const TWO_TRIANGLE_POINTS: usize = 6;

pub struct VirtualScreen {
  screen_size: Vector2u,
  pixel_size: f32,
  vertices: VertexArray,
}

impl VirtualScreen {
  pub fn new() -> Self {
    Self {
      screen_size: Vector2u::new(0, 0),
      pixel_size: 0.,
      vertices: VertexArray::default(),
    }
  }

  pub fn create(&mut self, w: u32, h: u32, pixel_size: f32, color: Color) {
    self.vertices.resize((w * h) as usize * TWO_TRIANGLE_POINTS);
    self.screen_size = Vector2u::new(w, h);
    self.vertices.set_primitive_type(PrimitiveType::TRIANGLES);
    self.pixel_size = pixel_size;

    let vec_right = Vector2f::new(pixel_size, 0.);
    let vec_bottom = Vector2f::new(0., pixel_size);
    let mut index = 0;
    for x in 0..w {
      for y in 0..h {
        // let index = (x * h + y) as usize * TWO_TRIANGLE_POINTS;
        let coord_top_left = Vector2f::new(x as f32 * pixel_size, y as f32 * pixel_size);
        let coord_top_right = coord_top_left + vec_right;
        let coord_bottom_left = coord_top_left + vec_bottom;
        let coord_bottom_right = coord_top_right + vec_bottom;
        // 0/5-----1
        // |\ Tri1 |
        // | \     |
        // |  \    |
        // |   \   |
        // |Tri2 \ |
        // 4-----2/3

        // Triangle-1
        // top-left
        self.vertices[index].position = coord_top_left;
        self.vertices[index].color = color;
        // top-right
        self.vertices[index + 1].position = coord_top_right;
        self.vertices[index + 1].color = color;
        // bottom-right
        self.vertices[index + 2].position = coord_bottom_right;
        self.vertices[index + 2].color = color;
        // Triangle-2
        // bottom-right
        self.vertices[index + 3].position = coord_bottom_right;
        self.vertices[index + 3].color = color;
        // bottom-left
        self.vertices[index + 4].position = coord_bottom_left;
        self.vertices[index + 4].color = color;
        // top-right
        self.vertices[index + 5].position = coord_top_left;
        self.vertices[index + 5].color = color;

        index += TWO_TRIANGLE_POINTS;
      }
    }
  }

  pub fn set_picture(&mut self, picture_buffer: RgbaImage) {
    let mut index = 0;
    for x in 0..self.screen_size.x as usize {
      for y in 0..self.screen_size.y as usize {
        for i in 0..TWO_TRIANGLE_POINTS {
          let pix = picture_buffer.get_pixel(x as u32, y as u32);

          self.vertices[index + i].color =
            sfml::graphics::Color::rgba(pix.0[0], pix.0[1], pix.0[2], pix.0[3]);
        }
        index += TWO_TRIANGLE_POINTS;
      }
    }
  }
}

impl Drawable for VirtualScreen {
  fn draw<'a: 'shader, 'texture, 'shader, 'shader_texture>(
    &'a self,
    target: &mut dyn RenderTarget,
    states: &RenderStates<'texture, 'shader, 'shader_texture>,
  ) {
    target.draw_vertex_array(&self.vertices, states);
  }
}
