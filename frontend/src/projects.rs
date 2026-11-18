use crate::browser;
use pixel_geometry::scene::{Document, Scene};
use serde::{Deserialize, Serialize};
use webapi::global::crypto;

const ORBIT: &str = "orbit";
const WEB_APP: &str = "web-app";
const CATALOG: &str = "agent-pixel:projects:v1";

#[derive(Clone, Default, Deserialize, Serialize)]
pub struct Project {
    pub id: String,
    pub title: String,
}

impl Project {
    pub fn is_poster(&self) -> bool {
        self.id == ORBIT
    }

    pub fn href(&self) -> String {
        format!("?project={}", self.id)
    }

    pub fn storage_key(&self, server: bool) -> String {
        match self.id.as_str() {
            ORBIT if server => "agent-pixel:poster:v1:server".into(),
            ORBIT => "agent-pixel:poster:v1".into(),
            WEB_APP => "agent-pixel:web-app:v1".into(),
            id => format!("agent-pixel:project:{id}:v1"),
        }
    }

    pub fn initial_scene(&self) -> Scene {
        if self.is_poster() {
            Scene::poster()
        } else {
            Scene::blank()
        }
    }

    pub fn load(&self, server: bool) -> Result<Option<Document>, String> {
        let json = browser::load_draft(&self.storage_key(server)).map_err(crate::message)?;
        json.map(|json| serde_json::from_str(&json).map_err(|error| error.to_string()))
            .transpose()
    }
}

fn saved_projects() -> Result<Vec<Project>, String> {
    match browser::load_draft(CATALOG).map_err(crate::message)? {
        Some(json) => serde_json::from_str(&json).map_err(|error| error.to_string()),
        None => Ok(Vec::new()),
    }
}

pub fn list() -> (Vec<Project>, Option<String>) {
    let mut projects = vec![
        Project {
            id: ORBIT.into(),
            title: "Orbit study".into(),
        },
        Project {
            id: WEB_APP.into(),
            title: "Web app mockup".into(),
        },
    ];
    let warning = match saved_projects() {
        Ok(saved) => {
            projects.extend(saved.into_iter().rev());
            None
        }
        Err(_) => Some("Could not read your saved project list.".into()),
    };
    (projects, warning)
}

pub fn route() -> Option<(String, bool)> {
    if let Some(project) = browser::query_value("project") {
        Some((project, false))
    } else if browser::workspace_query("design") {
        Some((WEB_APP.into(), false))
    } else if browser::workspace_query("server") {
        Some((ORBIT.into(), true))
    } else {
        None
    }
}

pub fn create(title: &str) -> Result<Project, String> {
    let title = title.trim();
    if title.is_empty() || title.chars().count() > 80 {
        return Err("Enter a name between 1 and 80 characters.".into());
    }
    let mut projects =
        saved_projects().map_err(|_| "Could not read your project list.".to_string())?;
    let project = Project {
        id: crypto().random_uuid(),
        title: title.into(),
    };
    projects.push(project.clone());
    let body = serde_json::to_string(&projects).map_err(|error| error.to_string())?;
    browser::save_draft(CATALOG, Some(&body))
        .map_err(|_| "Could not save the new project in this browser.".to_string())?;
    Ok(project)
}
