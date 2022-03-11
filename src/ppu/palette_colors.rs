use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sfml::graphics::Color;

pub static COLORS: [Color; 64] = [
  Color::rgb(0x66, 0x66, 0x66),
  Color::rgb(0x00, 0x2a, 0x88),
  Color::rgb(0x14, 0x12, 0xa7),
  Color::rgb(0x3b, 0x00, 0xa4),
  Color::rgb(0x5c, 0x00, 0x7e),
  Color::rgb(0x6e, 0x00, 0x40),
  Color::rgb(0x6c, 0x06, 0x00),
  Color::rgb(0x56, 0x1d, 0x00),
  Color::rgb(0x33, 0x35, 0x00),
  Color::rgb(0x0b, 0x48, 0x00),
  Color::rgb(0x00, 0x52, 0x00),
  Color::rgb(0x00, 0x4f, 0x08),
  Color::rgb(0x00, 0x40, 0x4d),
  Color::rgb(0x00, 0x00, 0x00),
  Color::rgb(0x00, 0x00, 0x00),
  Color::rgb(0x00, 0x00, 0x00),
  Color::rgb(0xad, 0xad, 0xad),
  Color::rgb(0x15, 0x5f, 0xd9),
  Color::rgb(0x42, 0x40, 0xff),
  Color::rgb(0x75, 0x27, 0xfe),
  Color::rgb(0xa0, 0x1a, 0xcc),
  Color::rgb(0xb7, 0x1e, 0x7b),
  Color::rgb(0xb5, 0x31, 0x20),
  Color::rgb(0x99, 0x4e, 0x00),
  Color::rgb(0x6b, 0x6d, 0x00),
  Color::rgb(0x38, 0x87, 0x00),
  Color::rgb(0x0c, 0x93, 0x00),
  Color::rgb(0x00, 0x8f, 0x32),
  Color::rgb(0x00, 0x7c, 0x8d),
  Color::rgb(0x00, 0x00, 0x00),
  Color::rgb(0x00, 0x00, 0x00),
  Color::rgb(0x00, 0x00, 0x00),
  Color::rgb(0xff, 0xfe, 0xff),
  Color::rgb(0x64, 0xb0, 0xff),
  Color::rgb(0x92, 0x90, 0xff),
  Color::rgb(0xc6, 0x76, 0xff),
  Color::rgb(0xf3, 0x6a, 0xff),
  Color::rgb(0xfe, 0x6e, 0xcc),
  Color::rgb(0xfe, 0x81, 0x70),
  Color::rgb(0xea, 0x9e, 0x22),
  Color::rgb(0xbc, 0xbe, 0x00),
  Color::rgb(0x88, 0xd8, 0x00),
  Color::rgb(0x5c, 0xe4, 0x30),
  Color::rgb(0x45, 0xe0, 0x82),
  Color::rgb(0x48, 0xcd, 0xde),
  Color::rgb(0x4f, 0x4f, 0x4f),
  Color::rgb(0x00, 0x00, 0x00),
  Color::rgb(0x00, 0x00, 0x00),
  Color::rgb(0xff, 0xfe, 0xff),
  Color::rgb(0xc0, 0xdf, 0xff),
  Color::rgb(0xd3, 0xd2, 0xff),
  Color::rgb(0xe8, 0xc8, 0xff),
  Color::rgb(0xfb, 0xc2, 0xff),
  Color::rgb(0xfe, 0xc4, 0xea),
  Color::rgb(0xfe, 0xcc, 0xc5),
  Color::rgb(0xf7, 0xd8, 0xa5),
  Color::rgb(0xe4, 0xe5, 0x94),
  Color::rgb(0xcf, 0xef, 0x96),
  Color::rgb(0xbd, 0xf4, 0xab),
  Color::rgb(0xb3, 0xf3, 0xcc),
  Color::rgb(0xb5, 0xeb, 0xf2),
  Color::rgb(0xb8, 0xb8, 0xb8),
  Color::rgb(0x00, 0x00, 0x00),
  Color::rgb(0x00, 0x00, 0x00),
];

#[derive(Serialize, Deserialize)]
#[serde(remote = "Color")]
pub(crate) struct ColorDef {
  #[serde(getter = "Color::red")]
  red: u8,
  #[serde(getter = "Color::green")]
  green: u8,
  #[serde(getter = "Color::blue")]
  blue: u8,
}

impl From<ColorDef> for Color {
  fn from(color: ColorDef) -> Color {
    Color::rgb(color.red, color.green, color.blue)
  }
}

impl From<Color> for ColorDef {
  fn from(color: Color) -> ColorDef {
    ColorDef {
      red: color.red(),
      green: color.green(),
      blue: color.blue(),
    }
  }
}

impl serde::Serialize for ColorDef {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    serializer.serialize_str(&format!(
      "0x{:02x}{:02x}{:02x}FF",
      self.red, self.green, self.blue
    ))
  }
}

impl<'d> serde::Deserialize<'d> for ColorDef {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'d>,
  {
    let s = String::deserialize(deserializer)?;
    let s = s.as_str();
    Ok(ColorDef {
      red: u8::from_str_radix(&s[2..4], 16).unwrap(),
      green: u8::from_str_radix(&s[4..6], 16).unwrap(),
      blue: u8::from_str_radix(&s[6..8], 16).unwrap(),
    })
  }
}

pub(super) fn color_vec_ser<S: Serializer>(
  vec: &Vec<Vec<Color>>,
  serializer: S,
) -> Result<S::Ok, S::Error> {
  // First convert the vector into a Vec<LocalColor>.
  let to_local = |color: &Color| -> ColorDef { ColorDef::from(*color) };
  let to_local_v1 = |vec: &Vec<Color>| -> Vec<ColorDef> { vec.iter().map(to_local).collect() };
  let vec2: Vec<Vec<ColorDef>> = vec.iter().map(to_local_v1).collect();

  // Instead of serializing Vec<ExternalCrateColor>, we serialize Vec<LocalColor>.
  vec2.serialize(serializer)
}

pub(super) fn color_vec_deser<'a, D: Deserializer<'a>>(
  deserializer: D,
) -> Result<Vec<Vec<Color>>, D::Error> {
  let to_external = |color: &ColorDef| -> Color { Color::rgb(color.red, color.green, color.blue) };

  let to_external_v1 =
    |vec: &Vec<ColorDef>| -> Vec<Color> { vec.iter().map(to_external).collect() };
  // Deserialize as if it was a Vec<LocalColor>.
  let vec: Vec<Vec<ColorDef>> = Vec::<Vec<ColorDef>>::deserialize(deserializer)?;
  // Convert it into an Vec<ExternalCrateColor>
  Ok(vec.iter().map(to_external_v1).collect())
}
