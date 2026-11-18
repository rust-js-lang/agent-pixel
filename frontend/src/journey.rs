use pixel_geometry::{
    bundle::{Action, ScreenInfo},
    scene::Scene,
};

pub const PROJECT: &str = "current-reminders";
pub const HANDOFF: &str =
    "https://github.com/rust-js-lang/agent-pixel/blob/main/handoff/CURRENT-REMINDERS.md";

pub struct Screen {
    pub id: &'static str,
    pub title: &'static str,
    pub primary: &'static str,
    pub next: &'static str,
    seed: &'static str,
}

impl Screen {
    pub fn info(&self) -> ScreenInfo {
        ScreenInfo {
            id: self.id.into(),
            title: self.title.into(),
            action: Some(Action {
                layer_id: "action".into(),
                label: self.primary.into(),
                next: self.next.into(),
                failure: (self.id == "review").then(|| "error".into()),
            }),
        }
    }

    pub fn initial_scene(&self) -> Scene {
        serde_json::from_str(self.seed).expect("Valid journey artboard")
    }
}

pub const SCREENS: [Screen; 5] = [
    Screen {
        id: "overview",
        title: "Energy overview",
        primary: "Set off-peak reminder",
        next: "schedule",
        seed: include_str!("../demos/reminder-overview.json"),
    },
    Screen {
        id: "schedule",
        title: "Set schedule",
        primary: "Review reminder",
        next: "review",
        seed: include_str!("../demos/reminder-schedule.json"),
    },
    Screen {
        id: "review",
        title: "Review",
        primary: "Save reminder",
        next: "success",
        seed: include_str!("../demos/reminder-review.json"),
    },
    Screen {
        id: "success",
        title: "Confirmation",
        primary: "Back to Home energy",
        next: "overview",
        seed: include_str!("../demos/reminder-success.json"),
    },
    Screen {
        id: "error",
        title: "Save failure",
        primary: "Try again",
        next: "review",
        seed: include_str!("../demos/reminder-error.json"),
    },
];

pub fn find(id: &str) -> Option<&'static Screen> {
    SCREENS.iter().find(|screen| screen.id == id)
}

pub fn href(project: &str, id: &str, preview: bool) -> String {
    let mode = if preview { "&preview" } else { "" };
    format!("?project={project}&screen={id}{mode}")
}
