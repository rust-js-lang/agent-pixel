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
        <a key={project.id.clone()} className="project-card" href={project.href()}>
            <div className={if poster { "project-tile orbit-tile" } else { "project-tile" }} aria-hidden="true">
                <div className={if poster { "project-preview orbit-preview" } else { "project-preview" }}>
                    {preview}
                    {poster.then(|| jsx! { <b className="preview-caption">"Orbit study."</b> })}
                    {(count == 0).then(|| jsx! { <span className="empty-preview">"＋"</span> })}
                </div>
            </div>
            <div className="project-meta">
                <div><h2>{project.title.clone()}</h2><p>{label}</p></div>
                <span className="project-arrow" aria-hidden="true">"↗"</span>
            </div>
        </a>
    }
}

pub fn Dashboard() -> JSX::Element {
    let (error, set_error) = use_state(|| None::<String>);
    let (projects, warning) = projects::list();
    let cards = projects.iter().map(project_card).collect::<Vec<_>>();
    jsx! {
        <>
            <header className="dashboard-header">
                <b className="brand"><i>"▦"</i>" Agent Pixel"</b>
                <a href="https://github.com/rust-js-lang/agent-pixel/blob/main/DESIGN.md">"Design doc ↗"</a>
            </header>
            <main className="project-dashboard">
                <div className="project-heading">
                    <div><h1>"Design projects"</h1><p>"Open an artboard or start a new design."</p></div>
                    <form className="create-project" action={move |data: &webapi::FormData| {
                        if let Some(FormDataEntryValue::Str(title)) = data.get("name") {
                            match projects::create(&title) {
                                Ok(project) => window_global::location().assign(&project.href()),
                                Err(error) => set_error.set(Some(error)),
                            }
                        }
                    }}>
                        <label htmlFor="project-name">"Project name"</label>
                        <div className="project-controls">
                            <input id="project-name" name="name" placeholder="Untitled design" required maxLength={80} />
                            <button type="submit">"＋ New project"</button>
                        </div>
                    </form>
                </div>
                <div role="status" className="project-status">{error.clone().or(warning)}</div>
                <section className="project-grid" aria-label="Projects">{cards}</section>
            </main>
        </>
    }
}

pub fn home() -> JSX::Element {
    jsx! { <Dashboard /> }
}

pub fn missing() -> JSX::Element {
    jsx! {
        <main className="project-dashboard">
            <h1>"Project unavailable"</h1>
            <p>"This project could not be found in this browser."</p>
            <a href="./">"Back to projects"</a>
        </main>
    }
}
