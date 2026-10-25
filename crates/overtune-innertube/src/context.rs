use serde_json::{Value, json};

pub const DEFAULT_VISITOR_DATA: &str = "CgtsZG1ySnZiQWtSbyiMjuGSBg%3D%3D";

const DESKTOP_UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/74.0.3729.157 Safari/537.36";
const VISIONOS_UA: &str =
    "com.google.visionos.youtube/1.02 (RealityDevice14,1; U; CPU visionOS 25_6_0 like Mac OS X)";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientContext {
    pub id: u32,
    pub name: &'static str,
    pub version: &'static str,
    pub platform: &'static str,
    pub client_screen: Option<&'static str>,
    pub user_agent: &'static str,
    pub referer: Option<&'static str>,
    pub device_make: Option<&'static str>,
    pub device_model: Option<&'static str>,
    pub os_name: Option<&'static str>,
    pub os_version: Option<&'static str>,
    pub music: bool,
}

impl ClientContext {
    pub const WEB_REMIX: ClientContext = ClientContext {
        id: 67,
        name: "WEB_REMIX",
        version: "1.20220606.03.00",
        platform: "DESKTOP",
        client_screen: None,
        user_agent: DESKTOP_UA,
        referer: Some("https://music.youtube.com/"),
        device_make: None,
        device_model: None,
        os_name: None,
        os_version: None,
        music: true,
    };

    pub const VISIONOS: ClientContext = ClientContext {
        id: 101,
        name: "VISIONOS",
        version: "1.02",
        platform: "MOBILE",
        client_screen: Some("WATCH"),
        user_agent: VISIONOS_UA,
        referer: None,
        device_make: Some("Apple"),
        device_model: Some("RealityDevice14,1"),
        os_name: Some("visionOS"),
        os_version: Some("25.6.0.23O471"),
        music: false,
    };

    pub fn to_json(&self, hl: &str, gl: &str, visitor_data: &str) -> Value {
        let mut client = json!({
            "clientName": self.name,
            "clientVersion": self.version,
            "platform": self.platform,
            "hl": hl,
            "gl": gl,
            "visitorData": visitor_data,
            "userAgent": self.user_agent,
            "timeZone": "UTC",
            "utcOffsetMinutes": 0,
        });
        let extra = [
            ("clientScreen", self.client_screen),
            ("referer", self.referer),
            ("deviceMake", self.device_make),
            ("deviceModel", self.device_model),
            ("osName", self.os_name),
            ("osVersion", self.os_version),
        ];
        for (key, value) in extra {
            if let Some(value) = value {
                client[key] = Value::from(value);
            }
        }
        json!({ "client": client, "user": { "lockedSafetyMode": false } })
    }
}
