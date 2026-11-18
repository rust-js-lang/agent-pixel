use pixel_geometry::scene::{Author, Design, Scene, Session};
use serde_json::Value;

#[test]
fn failed_edits_preserve_the_checkpoint_and_a_batch_undoes_as_one() {
    let examples: Value = serde_json::from_str(include_str!("fixtures/history.json")).unwrap();
    let mut session = Session::new(Scene::blank());
    let mut run = |command, input: &str, revision: &str| {
        session
            .command(command, input, revision.into(), Author::Agent)
            .map(|result| serde_json::from_str::<Design>(&result).unwrap())
    };

    let initial = run("restore", &examples["initial"].to_string(), "r1").unwrap();
    assert!(run("batch", &examples["invalid"].to_string(), "r2").is_err());
    let after_failure = run("read", "null", "ignored").unwrap();
    assert_eq!(after_failure.scene, initial.scene);
    assert_eq!(after_failure.revision, "r1");
    assert!(after_failure.history.undo.is_empty());

    let edited = run("batch", &examples["valid"].to_string(), "r3").unwrap();
    assert_eq!(edited.scene.layers.len(), 2);
    assert_eq!(edited.history.undo.len(), 1);
    assert!(matches!(edited.history.undo[0].author, Author::Agent));
    let undone = run("undo", "\"r3\"", "r4").unwrap();
    assert_eq!(undone.scene, initial.scene);
    assert_eq!(undone.history.redo.len(), 1);
    assert!(run("redo", "\"r3\"", "stale").is_err());
    assert_eq!(run("redo", "\"r4\"", "r5").unwrap().scene, edited.scene);
}
