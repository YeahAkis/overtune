use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use overtune_theme::{Config, Rgba, Role, Theme, css, palette_path, resolve};

const MONO: &str = r##"{"name":"mono","bg":"#0c0c0c","bg_alt":"#161616","surface":"#2a2a2a","fg":"#ececec","dim":"#7c7c7c","accent":"#ffffff","accent2":"#c8c8c8","accent3":"#6e6e6e","warn":"#d6d6d6","crit":"#ffffff","font":"JetBrains Mono"}"##;

#[test]
fn parses_short_and_long_hex() {
    assert_eq!(Rgba::parse("#fff").unwrap().hex(), "#ffffff");
    assert_eq!(
        Rgba::parse("#0c0c0c").unwrap(),
        Rgba {
            r: 12,
            g: 12,
            b: 12,
            a: 255
        }
    );
    assert_eq!(Rgba::parse("#0c0c0c80").unwrap().a, 128);
    assert!(Rgba::parse("red").is_none());
    assert!(Rgba::parse("#12345").is_none());
}

#[test]
fn full_palette_replaces_builtin() {
    let t = resolve(&Config::default(), Some(MONO));
    assert_eq!(t.color(Role::Bg).hex(), "#0c0c0c");
    assert_eq!(t.color(Role::BgAlt).hex(), "#161616");
    assert_eq!(t.color(Role::Accent3).hex(), "#6e6e6e");
    assert_eq!(t.font.as_deref(), Some("JetBrains Mono"));
    assert_eq!(t.name, "mono");
}

#[test]
fn missing_key_falls_back_per_role() {
    let t = resolve(&Config::default(), Some(r##"{"bg":"#000000"}"##));
    assert_eq!(t.color(Role::Bg).hex(), "#000000");
    assert_eq!(t.color(Role::Fg), Theme::dark().color(Role::Fg));
}

#[test]
fn invalid_values_are_ignored_individually() {
    let t = resolve(
        &Config::default(),
        Some(r##"{"bg":"red","fg":"#eeeeee","accent":5}"##),
    );
    assert_eq!(t.color(Role::Bg), Theme::dark().color(Role::Bg));
    assert_eq!(t.color(Role::Fg).hex(), "#eeeeee");
    assert_eq!(t.color(Role::Accent), Theme::dark().color(Role::Accent));
}

#[test]
fn truncated_json_gives_builtin() {
    assert_eq!(
        resolve(&Config::default(), Some(r##"{"bg":"#0c0c"##)),
        Theme::dark()
    );
}

#[test]
fn non_object_json_gives_builtin() {
    assert_eq!(resolve(&Config::default(), Some("[1,2]")), Theme::dark());
}

#[test]
fn no_palette_gives_builtin() {
    assert_eq!(resolve(&Config::default(), None), Theme::dark());
}

#[test]
fn follow_palette_off_ignores_file() {
    let c = Config {
        follow_palette: false,
        ..Config::default()
    };
    assert_eq!(resolve(&c, Some(MONO)), Theme::dark());
}

#[test]
fn light_theme_selected_by_config() {
    let c = Config {
        theme: "light".into(),
        ..Config::default()
    };
    assert_eq!(resolve(&c, None), Theme::light());
}

#[test]
fn config_colors_override_builtin_and_palette_overrides_config() {
    let mut colors = BTreeMap::new();
    colors.insert("accent".to_string(), "#ff0000".to_string());
    let c = Config {
        colors,
        ..Config::default()
    };
    assert_eq!(
        resolve(&c, Some(r##"{"accent":"#00ff00"}"##))
            .color(Role::Accent)
            .hex(),
        "#00ff00"
    );
    assert_eq!(
        resolve(&c, Some(r##"{"bg":"#000000"}"##))
            .color(Role::Accent)
            .hex(),
        "#ff0000"
    );
}

#[test]
fn config_loads_from_toml() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");
    std::fs::write(
        &path,
        "theme = \"light\"\nfollow_palette = false\npalette_file = \"/x/p.json\"\n\n[colors]\naccent = \"#123456\"\n",
    )
    .unwrap();
    let c = Config::load(&path);
    assert_eq!(c.theme, "light");
    assert!(!c.follow_palette);
    assert_eq!(c.palette_file, Some(PathBuf::from("/x/p.json")));
    assert_eq!(c.colors.get("accent").map(String::as_str), Some("#123456"));
}

#[test]
fn broken_or_missing_config_gives_default() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");
    assert_eq!(Config::load(&path), Config::default());
    std::fs::write(&path, "theme = = =").unwrap();
    assert_eq!(Config::load(&path), Config::default());
    let d = Config::default();
    assert_eq!(d.theme, "dark");
    assert!(d.follow_palette);
}

#[test]
fn palette_path_priority() {
    let home = Path::new("/home/someone");
    let mut c = Config::default();
    assert_eq!(
        palette_path(&c, None, home),
        PathBuf::from("/home/someone/.config/rice/palette.json")
    );
    c.palette_file = Some(PathBuf::from("/cfg/p.json"));
    assert_eq!(palette_path(&c, None, home), PathBuf::from("/cfg/p.json"));
    assert_eq!(
        palette_path(&c, Some(OsStr::new("/env/p.json")), home),
        PathBuf::from("/env/p.json")
    );
    assert_eq!(
        palette_path(&c, Some(OsStr::new("")), home),
        PathBuf::from("/cfg/p.json")
    );
}

#[test]
fn nonexistent_wallpaper_is_dropped() {
    let t = resolve(
        &Config::default(),
        Some(r##"{"wallpaper":"/definitely/not/here.png"}"##),
    );
    assert_eq!(t.wallpaper, None);
}

#[test]
fn existing_wallpaper_is_kept() {
    let dir = tempfile::tempdir().unwrap();
    let img = dir.path().join("w.png");
    std::fs::write(&img, b"x").unwrap();
    let json = format!(r#"{{"wallpaper":"{}"}}"#, img.display());
    assert_eq!(
        resolve(&Config::default(), Some(&json)).wallpaper,
        Some(img)
    );
}

#[test]
fn css_defines_every_role() {
    let s = css(&Theme::dark());
    for r in Role::ALL {
        assert!(
            s.contains(&format!("@define-color ot_{} ", r.key())),
            "{}",
            r.key()
        );
    }
}

#[test]
fn css_sets_font_only_when_present() {
    assert!(!css(&Theme::dark()).contains("font-family"));
    let t = resolve(&Config::default(), Some(MONO));
    assert!(css(&t).contains("font-family: \"JetBrains Mono\""));
}
