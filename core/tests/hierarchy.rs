use pixel_geometry::scene::{Author, Design, Scene, Session};
use serde_json::{Value, json};

#[test]
fn grouping_brings_members_together_in_paint_order_and_rejects_cycles() {
    let mut session = Session::new(Scene::poster());
    let mut apply = |revision: &str, next: &str, edits: Value| -> Result<Design, String> {
        let response = session.command(
            "batch",
            &json!({"revision": revision, "edits": edits}).to_string(),
            next.into(),
            Author::Agent,
        )?;
        serde_json::from_str(&response).map_err(|error| error.to_string())
    };
    let outer = apply("", "a", json!([{"op":"create_group","name":"Artwork"}]))
        .unwrap()
        .edited
        .unwrap();
    let inner = apply(
        "a",
        "b",
        json!([{"op":"create_group","name":"Shapes","parentId":outer}]),
    )
    .unwrap()
    .edited
    .unwrap();
    let grouped = apply(
        "b",
        "c",
        json!([
            {"op":"set_layer_parent","id":"satellite","parentId":inner},
            {"op":"set_layer_parent","id":"outer-disc","parentId":inner}
        ]),
    )
    .unwrap();
    assert_eq!(
        grouped.scene.layers[grouped.scene.layers.len() - 2].id,
        "outer-disc"
    );
    assert_eq!(grouped.scene.layers.last().unwrap().id, "satellite");
    assert!(
        apply(
            "c",
            "d",
            json!([{"op":"set_layer_parent","id":outer,"parentId":inner}])
        )
        .is_err()
    );
    let restored = apply("c", "e", json!([{"op":"ungroup","id":inner}])).unwrap();
    assert_eq!(
        restored.scene.groups[0].children,
        ["satellite", "outer-disc"]
    );
}
