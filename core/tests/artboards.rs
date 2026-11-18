use pixel_geometry::{
    Dimensions,
    scene::{Author, Design, Session},
};
use serde_json::json;

#[test]
fn legacy_dimensions_and_resizing_share_bounds_with_edits_and_history() {
    let mut session = Session::new(pixel_geometry::scene::Scene::blank());
    let legacy = json!({"version": 1, "layers": []});
    let read = |body: String| serde_json::from_str::<Design>(&body).unwrap();
    let restored = read(
        session
            .command(
                "restore",
                &legacy.to_string(),
                "start".into(),
                Author::Agent,
            )
            .unwrap(),
    );
    assert_eq!(restored.board, Dimensions::default());

    let batch = json!({"revision":"start","edits":[
        {"op":"resize_artboard","width":1440,"height":900},
        {"op":"patch","layer":{"kind":"rectangle","x":1250,"y":750,"width":180,"height":120}}
    ]});
    let changed = read(
        session
            .command("batch", &batch.to_string(), "desktop".into(), Author::Agent)
            .unwrap(),
    );
    assert_eq!(
        changed.board,
        Dimensions {
            width: 1440,
            height: 900
        }
    );
    assert_eq!(changed.scene.board, changed.board);
    assert_eq!(changed.scene.layers[0].x, 1250);

    for [width, height] in [[480, 600], [63, 900], [1440, 8193]] {
        let invalid = json!({"revision":"desktop","edits":[{"op":"resize_artboard","width":width,"height":height}]});
        assert!(
            session
                .command(
                    "batch",
                    &invalid.to_string(),
                    "failed".into(),
                    Author::Agent
                )
                .is_err()
        );
    }

    let undone = read(
        session
            .command("undo", "\"desktop\"", "undo".into(), Author::Agent)
            .unwrap(),
    );
    assert_eq!(undone.scene, restored.scene);
    let redone = read(
        session
            .command("redo", "\"undo\"", "redo".into(), Author::Agent)
            .unwrap(),
    );
    assert_eq!(redone.scene, changed.scene);
}
