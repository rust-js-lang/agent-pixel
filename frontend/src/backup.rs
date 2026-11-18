use crate::{
    browser, components, journey,
    projects::{self, Project},
};
use pixel_geometry::{
    bundle::{BundleScreen, ProjectBundle, ScreenInfo, VERSION},
    scene::{ComponentVariant, Document, Scene},
    screens::ScreenCatalog,
};

pub fn saved(project: &Project) -> Result<Option<ProjectBundle>, String> {
    let raw = projects::saved_bundle(&project.id);
    raw.map(|raw| ProjectBundle::parse(&raw).map_err(str::to_owned))
        .transpose()
}

pub fn export(id: &str) -> Result<String, String> {
    let project = projects::find(id).ok_or("Project not found in this browser.")?;
    serialize(capture(&project, None, None)?)
}

pub fn download_url(json: &str) -> String {
    format!(
        "data:application/json;charset=utf-8,{}",
        js::encode_uri_component(json)
    )
}

pub fn download_unsaved() -> Result<(), String> {
    let json = crate::export_unsaved()?;
    browser::download(&download_url(&json), "agent-pixel-unsaved-project.json");
    Ok(())
}

pub struct ProjectSource {
    screens: Vec<BundleScreen>,
    pub poster: bool,
    pub handoff: Option<String>,
    pub library: Option<Vec<ComponentVariant>>,
}

impl ProjectSource {
    pub fn screen_info(&self) -> Vec<ScreenInfo> {
        self.screens
            .iter()
            .map(|screen| screen.info.clone())
            .collect()
    }

    pub fn seed(&self, id: &str) -> Scene {
        self.screens
            .iter()
            .find(|screen| screen.info.id == id)
            .map(|screen| screen.scene.clone())
            .unwrap_or_else(Scene::blank)
    }

    pub fn scene(
        &self,
        project: &Project,
        catalog: &ScreenCatalog,
        id: &str,
    ) -> Result<Scene, String> {
        let key = project.screen_key(false, &catalog.legacy_base_id, id);
        Ok(projects::load_document(&key)?
            .and_then(|document| document.scene)
            .unwrap_or_else(|| self.seed(id)))
    }
}

pub fn source(project: &Project) -> Result<ProjectSource, String> {
    if let Some(bundle) = saved(project)? {
        return Ok(ProjectSource {
            screens: bundle.screens,
            poster: bundle.poster,
            handoff: bundle.handoff,
            library: Some(bundle.library),
        });
    }
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
    Ok(ProjectSource {
        screens,
        poster: project.is_poster(),
        handoff: (project.id == journey::PROJECT).then(|| journey::HANDOFF.to_owned()),
        library: crate::drafts::recovery_library(&project.storage_key(false)),
    })
}

fn capture(
    project: &Project,
    replacement: Option<(&str, Scene)>,
    library: Option<Vec<ComponentVariant>>,
) -> Result<ProjectBundle, String> {
    let source = source(project)?;
    let catalog = crate::screens::catalog(project, source.screen_info());
    let mut replacement = replacement;
    let mut screens = Vec::new();
    for info in &catalog.screens {
        let scene = if replacement.as_ref().is_some_and(|(id, _)| *id == info.id) {
            replacement.take().expect("Replacement scene").1
        } else {
            source.scene(project, &catalog, &info.id)?
        };
        screens.push(BundleScreen {
            info: info.clone(),
            scene,
        });
    }
    if replacement.is_some() {
        return Err("Screen not found in this project.".into());
    }
    Ok(ProjectBundle {
        version: VERSION,
        title: project.title.clone(),
        poster: source.poster,
        handoff: source.handoff,
        screens,
        library: match library.or(source.library) {
            Some(library) => library,
            None => components::live_library()?,
        },
    })
}

pub fn export_unsaved(
    project: &Project,
    screen_id: &str,
    scene: Scene,
    library: Vec<ComponentVariant>,
) -> Result<String, String> {
    let mut bundle = capture(project, Some((screen_id, scene)), Some(library))?;
    bundle.title = copied_title("Unsaved", &project.title);
    serialize(bundle)
}

pub async fn restore_previous(
    project: &Project,
    screen_id: &str,
    document: Document,
    library: Option<Vec<ComponentVariant>>,
) -> Result<Project, String> {
    let scene = document.scene.ok_or("Previous snapshot has no artboard.")?;
    let mut bundle = capture(project, Some((screen_id, scene)), library)?;
    bundle.title = copied_title("Checkpoint", &project.title);
    import(&serialize(bundle)?).await
}

fn copied_title(prefix: &str, title: &str) -> String {
    format!("{prefix} — {title}").chars().take(80).collect()
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
    for screen in &bundle.screens {
        for asset in &screen.scene.assets {
            browser::image_dimensions(&asset.image).await?;
        }
    }
    for variant in &bundle.library {
        for asset in &variant.assets {
            browser::image_dimensions(&asset.image).await?;
        }
    }
    projects::restore_bundle(&bundle, json).await
}
