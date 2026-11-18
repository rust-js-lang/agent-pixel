//! Stable screen identity, order, and project-level screen commands.
use crate::bundle::{ScreenInfo, valid_title};
use serde::{Deserialize, Serialize};

pub const INITIAL_REVISION: &str = "initial";
const MAX_SCREENS: usize = 64;

#[derive(Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScreenCatalog {
    pub revision: String,
    pub legacy_base_id: String,
    pub screens: Vec<ScreenInfo>,
}

#[derive(Deserialize, Serialize)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
pub enum ScreenEdit {
    Create { title: String },
    Rename { id: String, title: String },
    Duplicate { id: String, title: String },
    Reorder { id: String, index: usize },
    Delete { id: String },
}

pub struct EditedScreen {
    pub id: String,
    pub copy_from: Option<String>,
}

fn title(title: String) -> Result<String, &'static str> {
    let title = title.trim();
    if !valid_title(title) {
        return Err("Enter a screen name between 1 and 80 characters.");
    }
    Ok(title.to_owned())
}

impl ScreenCatalog {
    pub fn from_screens(screens: Vec<ScreenInfo>) -> Self {
        let legacy_base_id = screens.first().expect("Initial screen").id.clone();
        Self {
            revision: INITIAL_REVISION.into(),
            legacy_base_id,
            screens,
        }
    }

    pub fn apply(
        &mut self,
        edit: ScreenEdit,
        expected: &str,
        next: String,
    ) -> Result<EditedScreen, &'static str> {
        if self.revision != expected {
            return Err("The screen list changed. Read its latest revision and retry.");
        }
        let (id, copy_from) = match edit {
            ScreenEdit::Create { title: name } => {
                self.add(
                    ScreenInfo {
                        id: next.clone(),
                        title: title(name)?,
                        action: None,
                    },
                    self.screens.len(),
                )?;
                (next.clone(), None)
            }
            ScreenEdit::Duplicate { id, title: name } => {
                let index = self.index(&id)?;
                let mut screen = self.screens[index].clone();
                screen.id = next.clone();
                screen.title = title(name)?;
                if let Some(action) = &mut screen.action {
                    if action.next == id {
                        action.next = next.clone();
                    }
                    if action.failure.as_deref() == Some(id.as_str()) {
                        action.failure = Some(next.clone());
                    }
                }
                self.add(screen, index + 1)?;
                (next.clone(), Some(id))
            }
            ScreenEdit::Rename { id, title: name } => {
                let index = self.index(&id)?;
                self.screens[index].title = title(name)?;
                (id, None)
            }
            ScreenEdit::Reorder { id, index } => {
                let source = self.index(&id)?;
                if index >= self.screens.len() {
                    return Err("Screen position is outside the screen list.");
                }
                let screen = self.screens.remove(source);
                self.screens.insert(index, screen);
                (id, None)
            }
            ScreenEdit::Delete { id } => {
                let index = self.index(&id)?;
                if self.screens.len() == 1 {
                    return Err("Keep at least one screen in the project.");
                }
                self.screens.remove(index);
                for screen in &mut self.screens {
                    if let Some(action) = &mut screen.action {
                        if action.next == id {
                            screen.action = None;
                        } else if action.failure.as_deref() == Some(id.as_str()) {
                            action.failure = None;
                        }
                    }
                }
                let successor = index.min(self.screens.len() - 1);
                (self.screens[successor].id.clone(), None)
            }
        };
        self.revision = next;
        Ok(EditedScreen { id, copy_from })
    }

    fn index(&self, id: &str) -> Result<usize, &'static str> {
        self.screens
            .iter()
            .position(|screen| screen.id == id)
            .ok_or("Screen not found.")
    }

    fn add(&mut self, screen: ScreenInfo, index: usize) -> Result<(), &'static str> {
        if self.screens.len() >= MAX_SCREENS {
            return Err("This project already has 64 screens.");
        }
        if !crate::scene::valid_id(&screen.id)
            || self.screens.iter().any(|existing| existing.id == screen.id)
        {
            return Err("Invalid or duplicate screen ID.");
        }
        self.screens.insert(index, screen);
        Ok(())
    }
}
