use std::fmt::Write;

use crate::theme::{Role, Theme};

const STYLESHEET: &str = r#"
window, .main-window {
  background-color: @ot_bg;
  color: @ot_fg;
}

label { color: inherit; }
.dim { color: @ot_dim; }
.accent { color: @ot_accent; }
.bold { font-weight: 700; }
.small { font-size: 0.85em; }

.sidebar {
  background-color: @ot_bg_alt;
  padding: 14px 10px;
}

.sidebar .brand {
  font-weight: 700;
  padding: 0 8px 14px 8px;
}

.sidebar .section-label {
  color: @ot_dim;
  font-size: 0.75em;
  letter-spacing: 0.08em;
  padding: 12px 8px 4px 8px;
}

.nav-item {
  background: none;
  border: none;
  box-shadow: none;
  border-radius: 4px;
  padding: 6px 8px;
  color: @ot_accent2;
  min-height: 0;
}

.nav-item:hover { background-color: alpha(@ot_surface, 0.6); color: @ot_fg; }
.nav-item.active { background-color: @ot_surface; color: @ot_fg; }
.nav-item:disabled { color: @ot_dim; }

.content { padding: 16px 20px; }

.page-title { font-weight: 700; font-size: 1.35em; }

entry.search {
  background-color: @ot_surface;
  color: @ot_fg;
  border: none;
  border-radius: 4px;
  box-shadow: none;
  outline: none;
  padding: 4px 10px;
  min-height: 30px;
  caret-color: @ot_accent;
}

entry.search:focus-within { box-shadow: inset 0 0 0 1px @ot_accent3; }
entry.search > text > placeholder { color: @ot_dim; }

.key-hint {
  color: @ot_dim;
  font-size: 0.75em;
  border: 1px solid @ot_accent3;
  border-radius: 3px;
  padding: 0 4px;
}

.tabs { border-bottom: 1px solid @ot_surface; margin: 14px 0 12px 0; }

.tab {
  background: none;
  border: none;
  box-shadow: none;
  border-radius: 0;
  padding: 6px 0;
  margin-right: 18px;
  color: @ot_dim;
  border-bottom: 2px solid transparent;
  min-height: 0;
}

.tab:hover { color: @ot_fg; }
.tab.active { color: @ot_fg; border-bottom-color: @ot_accent; }

list, listview, .song-list { background: none; }

row.song-row, .song-row {
  border-radius: 3px;
  padding: 5px 6px;
  background: none;
}

row.song-row:hover, .song-row:hover { background-color: @ot_bg_alt; }
row.song-row:selected { background-color: @ot_surface; }
.song-row.playing { background-color: @ot_bg_alt; }
.song-row.playing .title { color: @ot_accent; }
.song-row.playing .subtitle { color: @ot_accent2; }
.song-row .subtitle, .song-row .duration, .song-row .index { color: @ot_dim; }

.art {
  background-color: @ot_surface;
  border-radius: 3px;
}

.art.round { border-radius: 9999px; }

.player-bar {
  background-color: @ot_bg_alt;
  border-top: 1px solid @ot_surface;
  min-height: 72px;
  padding: 0 16px;
}

.player-bar .title { font-weight: 600; }
.player-bar .subtitle { color: @ot_dim; }
.player-bar .time { color: @ot_dim; font-size: 0.8em; font-feature-settings: "tnum"; }

.icon-button {
  background: none;
  border: none;
  box-shadow: none;
  padding: 4px;
  min-height: 0;
  min-width: 0;
  border-radius: 4px;
}

.icon-button:hover { background-color: alpha(@ot_surface, 0.6); }
.icon-button.active { color: @ot_accent; }

.play-button {
  background-color: @ot_fg;
  border-radius: 9999px;
  min-width: 32px;
  min-height: 32px;
  padding: 0;
  border: none;
  box-shadow: none;
}

.play-button:hover { background-color: @ot_accent2; }

scale { padding: 6px 0; }
scale trough {
  background-color: @ot_surface;
  min-height: 3px;
  border-radius: 2px;
  border: none;
}
scale highlight {
  background-color: @ot_accent;
  border-radius: 2px;
  border: none;
}
scale slider {
  background-color: @ot_fg;
  min-width: 10px;
  min-height: 10px;
  margin: -4px;
  border-radius: 9999px;
  border: none;
  box-shadow: none;
  opacity: 0;
}
scale:hover slider { opacity: 1; }
scale.volume highlight { background-color: @ot_accent2; }

popover.menu > contents, popover > contents {
  background-color: @ot_bg_alt;
  color: @ot_fg;
  border: 1px solid @ot_surface;
  border-radius: 6px;
  padding: 4px;
  box-shadow: 0 12px 32px rgba(0, 0, 0, 0.45);
}

.suggestion {
  border-radius: 4px;
  padding: 6px 8px;
}

.suggestion:hover, .suggestion:selected, .suggestion.selected { background-color: @ot_surface; }

.inline-error { color: @ot_warn; }

separator { background-color: @ot_surface; min-height: 1px; min-width: 1px; }

scrollbar { background: none; border: none; }
scrollbar slider { background-color: @ot_surface; border-radius: 9999px; min-width: 6px; min-height: 6px; border: none; }
"#;

pub fn css(theme: &Theme) -> String {
    let mut out = String::new();
    for role in Role::ALL {
        let _ = writeln!(
            out,
            "@define-color ot_{} {};",
            role.key(),
            theme.color(role).css()
        );
    }
    if let Some(font) = &theme.font {
        let escaped = font.replace(['\\', '"'], "");
        let _ = writeln!(out, "window {{ font-family: \"{escaped}\"; }}");
    }
    out.push_str(STYLESHEET);
    out
}
