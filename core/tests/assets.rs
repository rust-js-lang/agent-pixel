use pixel_geometry::assets::RasterImage;
use pixel_geometry::scene::{Author, Design, Scene, Session};
use serde_json::{Value, json};

fn command(session: &mut Session, name: &str, value: Value, revision: &str) -> Design {
    let result = session
        .command(name, &value.to_string(), revision.into(), Author::Agent)
        .expect("Successful command");
    serde_json::from_str(&result).expect("Design")
}

#[test]
fn images_are_referenced_and_restored_with_their_revision() {
    let image: RasterImage = serde_json::from_str(include_str!("fixtures/raster.json")).unwrap();
    image.validate().unwrap();
    let mut session = Session::new(Scene::blank());
    let placement = json!({"op": "place_image", "placement": {"image": image, "x": 20, "y": 30, "width": 60, "height": 60}});
    let edited = command(
        &mut session,
        "batch",
        json!({"revision": "", "edits": [placement, placement]}),
        "place",
    );
    assert_eq!(edited.scene.assets.len(), 1);
    assert_eq!(edited.scene.layers.len(), 2);
    let id = &edited.scene.assets[0].id;
    assert!(
        edited
            .scene
            .layers
            .iter()
            .all(|layer| layer.image_id.as_ref() == Some(id))
    );

    let undone = command(&mut session, "undo", json!("place"), "undone");
    assert_eq!(undone.scene, Scene::blank());
    let redone = command(&mut session, "redo", json!("undone"), "redone");
    assert_eq!(redone.scene, edited.scene);

    let deletes: Vec<_> = edited
        .scene
        .layers
        .iter()
        .map(|layer| json!({"op": "delete", "id": layer.id}))
        .collect();
    let deleted = command(
        &mut session,
        "batch",
        json!({"revision": "redone", "edits": deletes}),
        "deleted",
    );
    assert_eq!(deleted.scene, Scene::blank());

    let mut corrupt = edited.scene;
    corrupt.assets.clear();
    assert_eq!(
        corrupt.validate(),
        Err("Image layer refers to a missing asset.")
    );
}

#[test]
fn native_validation_rejects_disguised_or_oversized_images() {
    let mut image: RasterImage =
        serde_json::from_str(include_str!("fixtures/raster.json")).unwrap();
    image.data_url = image.data_url.replace("image/png", "image/jpeg");
    assert!(image.validate().is_err());
    image.data_url = "data:image/png;base64,".to_owned() + &"A".repeat(700_000);
    assert_eq!(
        image.validate(),
        Err("Each image must be 512 KiB or smaller.")
    );
}
