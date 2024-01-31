//! Steps required to initialize the OpenGL environment.
//!
//! 1. Initialize the GL context source(glfw window for glfw, canvas for webGL).
//! 2. Compile and link the shader program(vertex, fragment).
//!    The shader program is used to draw a single texture.
//!    Vertex shader take position, texture coordinates as input.
//!    Fragment shader take texture coordinates and texture, and output color of the texture.
//! 3. Create a VAO, VBO, EBO.
//!    VAO is used to store the vertex attributes.
//!    VBO is used to store the vertex data. (position, texture coordinates, layer (not using))
//!    EBO is used to store the indices of the vertex data, so that we can reuse the vertex data.
//!    And then we bind attributes(position, texture coordinates) to vertex shader.
//! 4. Create a texture.
//!    The texture is used to store the image data, so that we can draw the rgba data into texture.
//!    And then we bind the texture to fragment shader.
//!    
//! And then we can draw the texture to the screen.
#[rustfmt::skip]
pub(crate) const VERTICES: [f32; 20] =
[
// positions * 2, texture coords * 2, layer * 1 
   1.0,  1.0, 1.0, 0.0, 0.0,
   1.0, -1.0, 1.0, 1.0, 0.0,
  -1.0, -1.0, 0.0, 1.0, 0.0,
  -1.0,  1.0, 0.0, 0.0, 0.0,
  // 0.5,  0.5, 1.0, 0.0, 0.0,
  // 0.5, -0.5, 1.0, 1.0, 0.0,
  // -0.5, -0.5, 0.0, 1.0, 0.0,
  // -0.5,  0.5, 0.0, 0.0, 0.0,
];
pub(crate) const INDICES: [i32; 6] = [0, 1, 3, 1, 2, 3];

#[cfg(feature = "use_gl")]
pub(crate) const VERTEX_SHADER_SOURCE: &str = r#"
#version 330 core
layout(location = 0) in vec2 position;
layout(location = 1) in vec2 texCoord;
layout(location = 2) in float layer;
out vec2 uv;
out float layer_get;
void main()
{
    gl_Position = vec4(position.x, position.y, 0.0, 1.0);
    uv = texCoord;
    layer_get = layer;
}
"#;

#[cfg(feature = "use_gl")]
pub(crate) const FRAGMENT_SHADER_SOURCE: &str = r#"
#version 330 core
out vec4 color;

in vec2 uv;
in float layer_get;

//layout (binding=0) uniform sampler2DArray textureArray;
// uniform sampler2DArray textureArray;
uniform sampler2D texture1;

void main()
{
    // color = texture(textureArray, vec3(uv.x,uv.y, layer_get));
    color = texture(texture1, uv);
}
"#;

#[cfg(feature = "wasm")]
pub(crate) const VERTEX_SHADER_SOURCE: &str = r#"
attribute vec2 position;
attribute vec2 texCoord;
attribute float layer;
varying vec2 uv;
void main() {
    gl_Position = vec4(position.x, position.y, 0.0, 1.0);
    uv = texCoord;
}
"#;

#[cfg(feature = "wasm")]
pub(crate) const FRAGMENT_SHADER_SOURCE: &str = r#"
varying highp vec2 uv;
uniform sampler2D texture;
void main() {
    gl_FragColor = texture2D(texture, uv);
}
"#;
