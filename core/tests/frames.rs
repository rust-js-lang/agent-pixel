use pixel_geometry::{
    frames::Rect,
    scene::{Author, Scene, Session},
};
use serde_json::{Value, json};

#[test]
fn clipped_nested_frames_move_as_a_unit_and_reject_out_of_bounds_moves() {
    let mut session = Session::new(Scene::blank());
    let mut batch = |revision: &str, next: &str, edits: Value| -> Result<Value, String> {
        session
            .command(
                "batch",
                &json!({"revision": revision, "edits": edits}).to_string(),
                next.into(),
                Author::Agent,
            )
            .map(|output| serde_json::from_str(&output).unwrap())
    };
    let frame = |x, y, width, height| json!({"bounds":{"x":x,"y":y,"width":width,"height":height},"fill":"#ffffff","clip":true});
    batch(
        "",
        "outer",
        json!([{"op":"create_frame","name":"Outer","frame":frame(20, 20, 200, 150)}]),
    )
    .unwrap();
    batch("outer", "inner", json!([{"op":"create_frame","name":"Inner","parentId":"outer-0","frame":frame(150, 30, 150, 110)}])).unwrap();
    let design = batch(
        "inner",
        "leaf",
        json!([
            {"op":"patch","layer":{"kind":"rectangle","x":200,"y":60,"width":100,"height":30}},
            {"op":"set_layer_parent","id":"leaf-0","parentId":"inner-0"}
        ]),
    )
    .unwrap();
    let scene: Scene = serde_json::from_value(design["scene"].clone()).unwrap();
    let nodes = scene.paint_nodes();
    assert_eq!(
        nodes
            .iter()
            .map(|node| node.id.as_str())
            .collect::<Vec<_>>(),
        ["outer-0", "inner-0", "leaf-0"]
    );
    assert_eq!(
        nodes[2].clip,
        Some(Rect {
            x: 150,
            y: 30,
            width: 70,
            height: 110
        })
    );
    let moved = batch(
        "leaf",
        "moved",
        json!([{"op":"set_frame","id":"outer-0","frame":frame(40, 50, 200, 150)}]),
    )
    .unwrap();
    assert_eq!(moved["scene"]["layers"][0]["x"], 220);
    assert_eq!(moved["scene"]["layers"][0]["y"], 90);
    assert_eq!(moved["scene"]["groups"][1]["frame"]["bounds"]["x"], 170);
    assert!(
        batch(
            "moved",
            "bad",
            json!([{"op":"set_frame","id":"outer-0","frame":frame(280, 50, 200, 150)}])
        )
        .is_err()
    );
    let undo = session
        .command("undo", "\"moved\"", "undo".into(), Author::Agent)
        .unwrap();
    let undo: Value = serde_json::from_str(&undo).unwrap();
    assert_eq!(undo["scene"], design["scene"]);
}
