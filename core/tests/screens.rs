use pixel_geometry::{
    bundle::{Action, ScreenInfo},
    screens::{INITIAL_REVISION, ScreenCatalog, ScreenEdit},
};

#[test]
fn stable_screen_ids_survive_reorder_and_deletion_cleans_up_prototype_links() {
    let mut catalog = ScreenCatalog::from_screens(vec![
        ScreenInfo {
            id: "first".into(),
            title: "First".into(),
            action: Some(Action {
                layer_id: "button".into(),
                label: "Continue".into(),
                next: "second".into(),
                failure: Some("first".into()),
            }),
        },
        ScreenInfo {
            id: "second".into(),
            title: "Second".into(),
            action: None,
        },
    ]);
    let copy = catalog
        .apply(
            ScreenEdit::Duplicate {
                id: "first".into(),
                title: " First copy ".into(),
            },
            INITIAL_REVISION,
            "copy".into(),
        )
        .unwrap();
    assert_eq!(copy.id, "copy");
    assert_eq!(copy.copy_from.as_deref(), Some("first"));
    assert_eq!(catalog.screens[1].title, "First copy");
    assert_eq!(
        catalog.screens[1]
            .action
            .as_ref()
            .unwrap()
            .failure
            .as_deref(),
        Some("copy")
    );
    assert!(
        catalog
            .apply(
                ScreenEdit::Reorder {
                    id: "copy".into(),
                    index: 0
                },
                INITIAL_REVISION,
                "bad".into()
            )
            .is_err()
    );
    catalog
        .apply(
            ScreenEdit::Reorder {
                id: "copy".into(),
                index: 0,
            },
            "copy",
            "moved".into(),
        )
        .unwrap();
    assert_eq!(catalog.legacy_base_id, "first");
    assert_eq!(catalog.screens[0].id, "copy");
    catalog
        .apply(
            ScreenEdit::Delete {
                id: "second".into(),
            },
            "moved",
            "removed".into(),
        )
        .unwrap();
    assert!(catalog.screens.iter().all(|screen| screen.action.is_none()));
}
