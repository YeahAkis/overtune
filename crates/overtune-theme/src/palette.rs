use std::path::PathBuf;

use serde_json::Value;
use thiserror::Error;

use crate::color::Rgba;
use crate::theme::{Role, Theme};

#[derive(Debug, Error)]
pub enum PaletteError {
    #[error("palette is not valid json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("palette is not a json object")]
    NotObject,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Palette {
    pub name: Option<String>,
    pub colors: Vec<(Role, Rgba)>,
    pub font: Option<String>,
    pub wallpaper: Option<PathBuf>,
}

impl Palette {
    pub fn from_json(text: &str) -> Result<Palette, PaletteError> {
        let value: Value = serde_json::from_str(text)?;
        let object = value.as_object().ok_or(PaletteError::NotObject)?;
        let string = |key: &str| {
            object
                .get(key)
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_owned)
        };
        let colors = Role::ALL
            .into_iter()
            .filter_map(|role| {
                object
                    .get(role.key())
                    .and_then(Value::as_str)
                    .and_then(Rgba::parse)
                    .map(|c| (role, c))
            })
            .collect();
        Ok(Palette {
            name: string("name"),
            colors,
            font: string("font"),
            wallpaper: string("wallpaper").map(PathBuf::from),
        })
    }

    pub fn apply(&self, base: &Theme) -> Theme {
        let mut theme = base.clone();
        for (role, color) in &self.colors {
            theme.set(*role, *color);
        }
        if let Some(name) = &self.name {
            theme.name.clone_from(name);
        }
        if self.font.is_some() {
            theme.font.clone_from(&self.font);
        }
        if self.wallpaper.is_some() {
            theme.wallpaper.clone_from(&self.wallpaper);
        }
        theme
    }
}
