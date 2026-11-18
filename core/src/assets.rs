//! Raster assets referenced by layers and stored with each scene revision.
use serde::{Deserialize, Serialize};

pub const MAX_IMAGE_BYTES: usize = 512 * 1024;
pub const MAX_SCENE_IMAGE_BYTES: usize = 1_200_000;

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RasterImage {
    pub name: String,
    pub data_url: String,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct ImageAsset {
    pub id: String,
    pub image: RasterImage,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ImagePlacement {
    pub image: RasterImage,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl RasterImage {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.name.trim().is_empty() || self.name.len() > 240 {
            return Err("Enter an image name between 1 and 240 bytes.");
        }
        let (media_type, encoded) = self
            .data_url
            .split_once(";base64,")
            .ok_or("Provide a base64 PNG, JPEG or WebP data URL.")?;
        let payload = encoded.trim_end_matches('=');
        let padding = encoded.len() - payload.len();
        if payload.len() < 16 || padding > 2 || !encoded.len().is_multiple_of(4) {
            return Err("Invalid image base64.");
        }
        if encoded.len() / 4 * 3 - padding > MAX_IMAGE_BYTES {
            return Err("Each image must be 512 KiB or smaller.");
        }
        if !payload.bytes().all(|byte| digit(byte).is_some()) {
            return Err("Invalid image base64.");
        }
        let mut signature = [0; 12];
        for (index, group) in payload.as_bytes()[..16].chunks_exact(4).enumerate() {
            let a = digit(group[0]).expect("Base64 digit");
            let b = digit(group[1]).expect("Base64 digit");
            let c = digit(group[2]).expect("Base64 digit");
            let d = digit(group[3]).expect("Base64 digit");
            signature[index * 3] = (a << 2) | (b >> 4);
            signature[index * 3 + 1] = (b << 4) | (c >> 2);
            signature[index * 3 + 2] = (c << 6) | d;
        }
        let matched = match media_type {
            "data:image/png" => signature.starts_with(b"\x89PNG\r\n\x1a\n"),
            "data:image/jpeg" => signature.starts_with(b"\xff\xd8\xff"),
            "data:image/webp" => signature.starts_with(b"RIFF") && signature.ends_with(b"WEBP"),
            _ => false,
        };
        if !matched {
            return Err("The image bytes do not match a supported PNG, JPEG or WebP type.");
        }
        Ok(())
    }
}

fn digit(byte: u8) -> Option<u8> {
    match byte {
        b'A'..=b'Z' => Some(byte - b'A'),
        b'a'..=b'z' => Some(byte - b'a' + 26),
        b'0'..=b'9' => Some(byte - b'0' + 52),
        b'+' => Some(62),
        b'/' => Some(63),
        _ => None,
    }
}
