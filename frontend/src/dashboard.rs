use crate::{dashboard_styles::STYLES as DASHBOARD, shell_styles::STYLES as SHELL};
use crate::{
    projects::{self, Project},
    view,
};
use react::{JSX, jsx, use_state};
use webapi::{FormDataEntryValue, window_global};

fn project_card(project: &Project) -> JSX::Element {
    let poster = project.is_poster();
    let scene = project
        .load(false)
        .ok()
        .flatten()
        .and_then(|document| document.scene)
        .filter(|scene| scene.validate().is_ok())
        .or_else(|| {
            crate::backup::saved(project)
                .ok()
                .flatten()
                .and_then(|bundle| bundle.screens.into_iter().next())
                .map(|screen| screen.scene)
        })
        .unwrap_or_else(|| project.initial_scene());
    let count = scene.layers.len();
    let label = match count {
        0 => "Empty artboard".into(),
        1 => "1 layer".into(),
        _ => format!("{count} layers"),
    };
    let preview = scene
        .layers
        .iter()
        .map(view::preview_layer)
        .collect::<Vec<_>>();
    jsx! {
        <a key={project.id.clone()} className={DASHBOARD.project_card()} href={project.href()}>
            <div className={if poster { format!("{} {}", DASHBOARD.project_tile(), DASHBOARD.orbit_tile()) } else { DASHBOARD.project_tile() }} aria-hidden="true">
                <div className={if poster { format!("{} {}", DASHBOARD.project_preview(), DASHBOARD.orbit_preview()) } else { DASHBOARD.project_preview() }}>
                    {preview}
                    {poster.then(|| jsx! { <b className={DASHBOARD.preview_caption()}>"Orbit study."</b> })}
                    {(count == 0).then(|| jsx! { <span className={DASHBOARD.empty_preview()}>"＋"</span> })}
                </div>
            </div>
            <div className={DASHBOARD.project_meta()}>
                <div><h2>{project.title.clone()}</h2><p>{label}</p></div>
                <span className={DASHBOARD.project_arrow()} aria-hidden="true">"↗"</span>
            </div>
        </a>
    }
}

pub struct BackupProps {
    pub projects: Vec<Project>,
}

pub fn BackupPanel(BackupProps { projects }: BackupProps) -> JSX::Element {
    let (download, set_download) = use_state(|| None::<String>);
    let (error, set_error) = use_state(|| None::<String>);
    let options = projects.iter().map(|project| jsx! {
        <option key={project.id.clone()} value={project.id.clone()}>{project.title.clone()}</option>
    }).collect::<Vec<_>>();
    jsx! {
        <details className={DASHBOARD.backups()}>
            <summary>"Project backup"</summary>
            <form className={DASHBOARD.create_project()} action={move |data: &webapi::FormData| {
                if let Some(FormDataEntryValue::Str(id)) = data.get("project") {
                    match crate::backup::export(&id) {
                        Ok(json) => {
                            set_error.set(None);
                            set_download.set(Some(crate::backup::download_url(&json)));
                        }
                        Err(error) => { set_download.set(None); set_error.set(Some(error)); }
                    }
                }
            }}>
                <label htmlFor="backup-project">"Project to export"</label>
                <div className={DASHBOARD.project_controls()}>
                    <select id="backup-project" name="project">{options}</select>
                    <button type="submit">"Prepare backup"</button>
                </div>
            </form>
            {download.as_ref().map(|url| jsx! { <a href={url.clone()} download="agent-pixel-project.json">"Download project bundle"</a> })}
            <form className={DASHBOARD.create_project()} action={move |data: &webapi::FormData| {
                if let Some(FormDataEntryValue::File(file)) = data.get("bundle") {
                    js::spawn(Box::new(async move {
                        if file.size() > pixel_geometry::bundle::MAX_BYTES as f64 {
                            set_error.set(Some("Project bundle exceeds 2 MB.".into()));
                            return;
                        }
                        let result = match js::settle(file.text()).await {
                            Ok(json) => crate::backup::import(&json).await,
                            Err(error) => Err(crate::message(error)),
                        };
                        match result {
                            Ok(project) => window_global::location().assign(&project.href()),
                            Err(error) => set_error.set(Some(error)),
                        }
                    }));
                }
            }}>
                <label htmlFor="bundle-file">"Project bundle to restore"</label>
                <div className={DASHBOARD.project_controls()}>
                    <input id="bundle-file" name="bundle" type="file" accept=".json,application/json" required />
                    <button type="submit">"Restore project"</button>
                </div>
            </form>
            <div role="status" className={DASHBOARD.project_status()}>{error.clone()}</div>
        </details>
    }
}

pub fn Dashboard() -> JSX::Element {
    let (error, set_error) = use_state(|| None::<String>);
    let (projects, warning) = projects::list();
    let cards = projects.iter().map(project_card).collect::<Vec<_>>();
    jsx! {
        <>
            <header className={SHELL.header()}>
                <b className={SHELL.brand()}><i>"▦"</i>" Agent Pixel"</b>
                <a href="https://github.com/rust-js-lang/agent-pixel/blob/main/DESIGN.md">"Design doc ↗"</a>
            </header>
            <main className={DASHBOARD.project_dashboard()}>
                <div className={DASHBOARD.project_heading()}>
                    <div><h1>"Design projects"</h1><p>"Open an artboard or start a new design."</p></div>
                    <form className={DASHBOARD.create_project()} action={move |data: &webapi::FormData| {
                        if let Some(FormDataEntryValue::Str(title)) = data.get("name") {
                            js::spawn(Box::new(async move {
                                match projects::create(&title).await {
                                    Ok(project) => window_global::location().assign(&project.href()),
                                    Err(error) => set_error.set(Some(error)),
                                }
                            }));
                        }
                    }}>
                        <label htmlFor="project-name">"Project name"</label>
                        <div className={DASHBOARD.project_controls()}>
                            <input id="project-name" name="name" placeholder="Untitled design" required maxLength={80} />
                            <button type="submit">"＋ New project"</button>
                        </div>
                    </form>
                </div>
                <div role="status" className={DASHBOARD.project_status()}>{error.clone().or(warning)}</div>
                <BackupPanel projects={projects} />
                <section className={DASHBOARD.project_grid()} aria-label="Projects">{cards}</section>
            </main>
        </>
    }
}

pub fn home() -> JSX::Element {
    jsx! { <Dashboard /> }
}

pub fn missing() -> JSX::Element {
    unavailable("This project could not be found in this browser.")
}

pub fn unavailable(message: &str) -> JSX::Element {
    jsx! {
        <main className={DASHBOARD.project_dashboard()}>
            <h1>"Project unavailable"</h1>
            <p>{message}</p>
            <a href="./">"Back to projects"</a>
        </main>
    }
}
