use std::path::PathBuf;

use crate::color::Rgba;
use crate::config::Config;
use crate::palette::Palette;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Role {
    Bg,
    BgAlt,
    Surface,
    Fg,
    Dim,
    Accent,
    Accent2,
    Accent3,
    Warn,
    Crit,
}

impl Role {
    pub const ALL: [Role; 10] = [
        Role::Bg,
        Role::BgAlt,
        Role::Surface,
        Role::Fg,
        Role::Dim,
        Role::Accent,
        Role::Accent2,
        Role::Accent3,
        Role::Warn,
        Role::Crit,
    ];

    pub fn key(self) -> &'static str {
        match self {
            Role::Bg => "bg",
            Role::BgAlt => "bg_alt",
            Role::Surface => "surface",
            Role::Fg => "fg",
            Role::Dim => "dim",
            Role::Accent => "accent",
            Role::Accent2 => "accent2",
            Role::Accent3 => "accent3",
            Role::Warn => "warn",
            Role::Crit => "crit",
        }
    }

    pub fn from_key(key: &str) -> Option<Role> {
        Role::ALL.into_iter().find(|r| r.key() == key)
    }

    fn index(self) -> usize {
        self as usize
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Theme {
    pub name: String,
    colors: [Rgba; 10],
    pub font: Option<String>,
    pub wallpaper: Option<PathBuf>,
}

impl Theme {
    pub fn dark() -> Theme {
        Theme {
            name: "dark".into(),
            colors: [
                Rgba::rgb(0x12, 0x12, 0x12),
                Rgba::rgb(0x1a, 0x1a, 0x1a),
                Rgba::rgb(0x2b, 0x2b, 0x2b),
                Rgba::rgb(0xe8, 0xe8, 0xe8),
                Rgba::rgb(0x8a, 0x8a, 0x8a),
                Rgba::rgb(0xe0, 0xb2, 0x5c),
                Rgba::rgb(0xf0, 0xcf, 0x8e),
                Rgba::rgb(0x9c, 0x7a, 0x3c),
                Rgba::rgb(0xe5, 0xa5, 0x4b),
                Rgba::rgb(0xe5, 0x53, 0x4b),
            ],
            font: None,
            wallpaper: None,
        }
    }

    pub fn light() -> Theme {
        Theme {
            name: "light".into(),
            colors: [
                Rgba::rgb(0xf6, 0xf5, 0xf2),
                Rgba::rgb(0xec, 0xea, 0xe5),
                Rgba::rgb(0xdc, 0xd9, 0xd2),
                Rgba::rgb(0x1c, 0x1b, 0x19),
                Rgba::rgb(0x6f, 0x6c, 0x66),
                Rgba::rgb(0xa8, 0x76, 0x1c),
                Rgba::rgb(0xc4, 0x93, 0x3c),
                Rgba::rgb(0x7a, 0x55, 0x12),
                Rgba::rgb(0xb8, 0x6e, 0x10),
                Rgba::rgb(0xc0, 0x36, 0x2c),
            ],
            font: None,
            wallpaper: None,
        }
    }

    pub fn builtin(name: &str) -> Theme {
        match name {
            "light" => Theme::light(),
            _ => Theme::dark(),
        }
    }

    pub fn color(&self, role: Role) -> Rgba {
        self.colors[role.index()]
    }

    pub fn set(&mut self, role: Role, color: Rgba) {
        self.colors[role.index()] = color;
    }
}

pub fn resolve(config: &Config, palette_text: Option<&str>) -> Theme {
    let mut theme = Theme::builtin(&config.theme);
    for (key, value) in &config.colors {
        if let (Some(role), Some(color)) = (Role::from_key(key), Rgba::parse(value)) {
            theme.set(role, color);
        }
    }
    if config.font.is_some() {
        theme.font.clone_from(&config.font);
    }
    if config.follow_palette
        && let Some(palette) = palette_text.and_then(|t| Palette::from_json(t).ok())
    {
        theme = palette.apply(&theme);
    }
    theme.wallpaper = theme.wallpaper.filter(|p| p.exists());
    theme
}
