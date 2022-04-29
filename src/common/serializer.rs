use image::RgbaImage;
use serde::de::{self, MapAccess, SeqAccess, Visitor};
use serde::ser::SerializeStruct;
use serde::{Deserialize, Deserializer, Serializer};

pub(crate) fn rgba_ser<S: Serializer>(img: &RgbaImage, serializer: S) -> Result<S::Ok, S::Error> {
  let mut rgba = serializer.serialize_struct("RgbaImage", 2)?;
  rgba.serialize_field("width", &img.width())?;
  rgba.serialize_field("height", &img.height())?;
  rgba.end()
}

#[derive(Deserialize)]
#[serde(field_identifier, rename_all = "lowercase")]
enum Field {
  Width,
  Height,
}

const FIELDS: &[&str] = &["width", "height"];
struct RgbaImageVisitor;

impl<'de> Visitor<'de> for RgbaImageVisitor {
  type Value = RgbaImage;

  fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
    formatter.write_str("struct RgbaImage")
  }

  fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
  where
    A: SeqAccess<'de>,
  {
    let width = seq
      .next_element()?
      .ok_or_else(|| de::Error::invalid_length(0, &self))?;
    let height = seq
      .next_element()?
      .ok_or_else(|| de::Error::invalid_length(1, &self))?;
    Ok(RgbaImage::new(width, height))
  }

  fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
  where
    V: MapAccess<'de>,
  {
    let mut width = None;
    let mut height = None;
    while let Some(key) = map.next_key()? {
      match key {
        Field::Width => {
          if width.is_some() {
            return Err(de::Error::duplicate_field("width"));
          }
          width = Some(map.next_value()?);
        }

        Field::Height => {
          if height.is_some() {
            return Err(de::Error::duplicate_field("height"));
          }
          height = Some(map.next_value()?);
        }
      }
    }
    let width = width.ok_or_else(|| de::Error::missing_field("width"))?;
    let height = height.ok_or_else(|| de::Error::missing_field("height"))?;
    Ok(RgbaImage::new(width, height))
  }
}

pub(crate) fn rgba_deser<'a, D: Deserializer<'a>>(deserializer: D) -> Result<RgbaImage, D::Error> {
  deserializer.deserialize_struct("RgbaImage", FIELDS, RgbaImageVisitor)
}
