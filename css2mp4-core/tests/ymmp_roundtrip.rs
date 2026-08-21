use css2mp4_core::ymmp::{MotionSamples, YmmpProject};

const FIXTURE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/sample.ymmp");

#[test]
fn load_real_sample_ymmp() {
    let project = YmmpProject::load(FIXTURE).expect("実サンプルの読み込みに失敗した");

    assert_eq!(project.timelines.len(), 1);
    let timeline = &project.timelines[0];
    assert_eq!(timeline.video_info.fps, 24);
    assert_eq!(timeline.video_info.width, 1920);
    assert_eq!(timeline.video_info.height, 1080);
    assert_eq!(timeline.items.len(), 1);

    let item = &timeline.items[0];
    assert_eq!(
        item.type_name,
        "YukkuriMovieMaker.Project.Items.VideoItem, YukkuriMovieMaker"
    );

    let opacity = item.get_animatable("Opacity").expect("Opacityが読めない");
    assert_eq!(opacity.values.len(), 1);
    assert_eq!(opacity.values[0].value, 100.0);
    assert_eq!(opacity.animation_type, "なし");
}

#[test]
fn round_trip_is_lossless_at_json_value_level() {
    let raw = std::fs::read_to_string(FIXTURE).unwrap();
    let stripped = raw.strip_prefix('\u{feff}').unwrap_or(&raw);
    let original: serde_json::Value = serde_json::from_str(stripped).unwrap();

    let project = YmmpProject::load(FIXTURE).unwrap();
    let roundtrip_json = serde_json::to_string(&project).unwrap();
    let roundtrip: serde_json::Value = serde_json::from_str(&roundtrip_json).unwrap();

    assert_eq!(
        original, roundtrip,
        "読み込んで書き出したJSONが元データと意味的に一致しない"
    );
}

#[test]
fn overwrite_motion_updates_expected_fields() {
    let mut project = YmmpProject::load(FIXTURE).unwrap();
    let item = project.item_mut(0, 0).unwrap();

    let samples = MotionSamples {
        translate_x: vec![0.0, 10.0, 20.0],
        translate_y: vec![-907.0, -900.0, -890.0],
        zoom_percent: vec![100.0],
        rotation_deg: vec![0.0, 5.0, 10.0],
        opacity_percent: vec![0.0, 50.0, 100.0],
    };
    samples.overwrite_item(item, 2.0 / 24.0).unwrap();

    let x = item.get_animatable("X").unwrap();
    assert_eq!(x.values.len(), 3);
    assert_eq!(x.values[2].value, 20.0);

    // Zoomは1要素だったので静的値として扱われる。
    let zoom = item.get_animatable("Zoom").unwrap();
    assert_eq!(zoom.values.len(), 1);
    assert_eq!(zoom.animation_type, "なし");

    let opacity = item.get_animatable("Opacity").unwrap();
    assert_eq!(opacity.values.len(), 3);
}
