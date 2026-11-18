use crate::{
    components, journey,
    projects::{self, Project},
};
use pixel_geometry::{
    bundle::{BundleScreen, ProjectBundle, ScreenInfo, VERSION},
    scene::{ComponentVariant, Document},
};

pub fn saved(project: &Project) -> Result<Option<ProjectBundle>, String> {
    let raw = projects::saved_bundle(&project.id);
    raw.map(|raw| ProjectBundle::parse(&raw).map_err(str::to_owned))
        .transpose()
}

pub fn export(id: &str) -> Result<String, String> {
    let project = projects::find(id).ok_or("Project not found in this browser.")?;
    serialize(capture(&project)?)
}

fn capture(project: &Project) -> Result<ProjectBundle, String> {
    let mut bundle = match saved(project)? {
        Some(bundle) => bundle,
        None => {
            let screens = if project.id == journey::PROJECT {
                journey::SCREENS
                    .iter()
                    .map(|screen| BundleScreen {
                        info: screen.info(),
                        scene: screen.initial_scene(),
                    })
                    .collect()
            } else {
                vec![BundleScreen {
                    info: ScreenInfo {
                        id: "main".into(),
                        title: project.title.clone(),
                        action: None,
                    },
                    scene: project.initial_scene(),
                }]
            };
            ProjectBundle {
                version: VERSION,
                title: project.title.clone(),
                poster: project.is_poster(),
                handoff: (project.id == journey::PROJECT).then(|| journey::HANDOFF.to_owned()),
                screens,
                library: match crate::drafts::recovery_library(&project.storage_key(false)) {
                    Some(library) => library,
                    None => components::live_library()?,
                },
            }
        }
    };
    for (index, screen) in bundle.screens.iter_mut().enumerate() {
        let key = project.screen_key(false, (index > 0).then_some(screen.info.id.as_str()));
        if let Some(scene) = projects::load_document(&key)?.and_then(|document| document.scene) {
            screen.scene = scene;
        }
    }
    Ok(bundle)
}

pub async fn restore_previous(
    project: &Project,
    screen_id: &str,
    document: Document,
    library: Option<Vec<ComponentVariant>>,
) -> Result<Project, String> {
    let mut bundle = capture(project)?;
    let screen = bundle
        .screens
        .iter_mut()
        .find(|screen| screen.info.id == screen_id)
        .ok_or("Screen not found in this project.")?;
    screen.scene = document.scene.ok_or("Previous snapshot has no artboard.")?;
    if let Some(library) = library {
        bundle.library = library;
    }
    bundle.title = format!("Checkpoint — {}", project.title)
        .chars()
        .take(80)
        .collect();
    import(&serialize(bundle)?).await
}

fn serialize(mut bundle: ProjectBundle) -> Result<String, String> {
    for screen in &mut bundle.screens {
        if screen.info.action.as_ref().is_some_and(|action| {
            !screen
                .scene
                .layers
                .iter()
                .any(|layer| layer.id == action.layer_id)
        }) {
            screen.info.action = None;
        }
        if !screen.scene.instances.is_empty() {
            screen.scene.refresh_components(&bundle.library)?;
        }
    }
    bundle.validate()?;
    let json = serde_json::to_string(&bundle).map_err(|error| error.to_string())?;
    if json.len() > pixel_geometry::bundle::MAX_BYTES {
        return Err("Project bundle exceeds 2 MB.".into());
    }
    Ok(json)
}

pub async fn import(json: &str) -> Result<Project, String> {
    let bundle = ProjectBundle::parse(json)?;
    projects::restore_bundle(&bundle, json).await
}
