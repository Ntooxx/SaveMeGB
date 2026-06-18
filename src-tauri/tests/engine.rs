use savemegb_lib::scanner;
use savemegb_lib::model::{OrphanedFile, Whitelist};

fn make_orphan(path: &str, name: &str) -> OrphanedFile {
    OrphanedFile {
        id: "x".into(),
        game_hint: name.into(),
        category: savemegb_lib::model::FileCategory::Other,
        path: std::path::PathBuf::from(path),
        size_bytes: 1024,
        file_count: 1,
        reason: "test".into(),
        last_modified: None,
        confidence: 80,
        source: None,
    }
}

#[test]
fn is_denied_publisher_blocks_dev_tools() {
    assert!(scanner::test_helpers::is_denied_publisher_for_test("microsoft"));
    assert!(scanner::test_helpers::is_denied_publisher_for_test("google"));
    assert!(scanner::test_helpers::is_denied_publisher_for_test("JetBrains"));
    assert!(scanner::test_helpers::is_denied_publisher_for_test("LM Studio"));
    assert!(scanner::test_helpers::is_denied_publisher_for_test("OBS Studio"));
    assert!(!scanner::test_helpers::is_denied_publisher_for_test("Larian Studios"));
    assert!(!scanner::test_helpers::is_denied_publisher_for_test("Ubisoft"));
    assert!(!scanner::test_helpers::is_denied_publisher_for_test("Paradox Interactive"));
    assert!(!scanner::test_helpers::is_denied_publisher_for_test("EA"));
    assert!(!scanner::test_helpers::is_denied_publisher_for_test("Steam"));
    assert!(!scanner::test_helpers::is_denied_publisher_for_test("Epic Games"));
    assert!(scanner::test_helpers::is_denied_publisher_for_test("Reallusion"));
    assert!(scanner::test_helpers::is_denied_publisher_for_test("Reallusion/iClone"));
    assert!(scanner::test_helpers::is_denied_publisher_for_test("SideFX"));
}

#[test]
fn is_likely_publisher_accepts_studios() {
    assert!(scanner::test_helpers::is_likely_publisher_for_test("Larian Studios"));
    assert!(scanner::test_helpers::is_likely_publisher_for_test("Ubisoft Entertainment"));
    assert!(scanner::test_helpers::is_likely_publisher_for_test("Paradox Interactive"));
    assert!(scanner::test_helpers::is_likely_publisher_for_test("CD Projekt Red"));
    assert!(scanner::test_helpers::is_likely_publisher_for_test("EA"));
    assert!(!scanner::test_helpers::is_likely_publisher_for_test("Microsoft"));
    assert!(!scanner::test_helpers::is_likely_publisher_for_test("NVIDIA Corporation"));
}

#[test]
fn is_denied_game_name_blocks_launchers_and_caches() {
    assert!(scanner::test_helpers::is_denied_game_name_for_test("launcher"));
    assert!(scanner::test_helpers::is_denied_game_name_for_test("launcher-v2"));
    assert!(scanner::test_helpers::is_denied_game_name_for_test("Launcher"));
    assert!(scanner::test_helpers::is_denied_game_name_for_test("cache"));
    assert!(scanner::test_helpers::is_denied_game_name_for_test("GPUCache"));
    assert!(scanner::test_helpers::is_denied_game_name_for_test("962c6b6db976409683df28e36e1e82de"));
    assert!(scanner::test_helpers::is_denied_game_name_for_test("ab"));
    assert!(scanner::test_helpers::is_denied_game_name_for_test("EasyAntiCheat"));
    assert!(!scanner::test_helpers::is_denied_game_name_for_test("Baldur's Gate 3"));
    assert!(!scanner::test_helpers::is_denied_game_name_for_test("Saves"));
    assert!(!scanner::test_helpers::is_denied_game_name_for_test("scu"));
    assert!(!scanner::test_helpers::is_denied_game_name_for_test("twinkle"));
}

#[test]
fn whitelist_excludes_by_path() {
    let wl = Whitelist {
        paths: vec![r"C:\Users\anton\AppData\Local\Paradox Interactive".into()],
        publishers: vec![],
        names: vec![],
    };
    let o = make_orphan(r"C:\Users\anton\AppData\Local\Paradox Interactive\launcher-v2", "launcher-v2");
    assert!(scanner::test_helpers::is_whitelisted_path_for_test(&o.path, &wl));
}

#[test]
fn whitelist_excludes_by_name() {
    let wl = Whitelist {
        paths: vec![],
        publishers: vec![],
        names: vec!["Baldur's Gate 3".into()],
    };
    let o = make_orphan(r"C:\Larian\bg3", "Baldur's Gate 3");
    assert!(wl.names.iter().any(|n| n.to_lowercase() == o.game_hint.to_lowercase()));
}

#[test]
fn whitelist_does_not_match_other_paths() {
    let wl = Whitelist {
        paths: vec![r"C:\ShouldNotMatch".into()],
        publishers: vec![],
        names: vec![],
    };
    let o = make_orphan(r"C:\Other\Path", "Other");
    assert!(!scanner::test_helpers::is_whitelisted_path_for_test(&o.path, &wl));
}

#[test]
fn scan_mode_parses() {
    use savemegb_lib::scanner::ScanMode;
    assert_eq!(ScanMode::parse("quick"), ScanMode::Quick);
    assert_eq!(ScanMode::parse("deep"), ScanMode::Deep);
    assert_eq!(ScanMode::parse("standard"), ScanMode::Standard);
    assert_eq!(ScanMode::parse("xyz"), ScanMode::Standard);
    assert_eq!(ScanMode::parse(""), ScanMode::Standard);
}

#[test]
fn category_safer_to_delete() {
    use savemegb_lib::model::FileCategory;
    assert!(FileCategory::Cache.is_safer_to_delete());
    assert!(FileCategory::Shaders.is_safer_to_delete());
    assert!(FileCategory::Crashes.is_safer_to_delete());
    assert!(FileCategory::Logs.is_safer_to_delete());
    assert!(!FileCategory::Saves.is_safer_to_delete());
    assert!(!FileCategory::Settings.is_safer_to_delete());
    assert!(!FileCategory::Other.is_safer_to_delete());
}

#[test]
fn human_size_smoke() {
    let s = scanner::test_helpers::human_size(2048);
    assert!(s.contains("KB") || s.contains("2"));
    let s2 = scanner::test_helpers::human_size(0);
    assert!(s2.contains("B"));
}

#[test]
fn empty_whitelist_works() {
    let wl = Whitelist::default();
    assert!(wl.paths.is_empty());
    assert!(wl.names.is_empty());
    let o = make_orphan(r"C:\Anywhere", "Anything");
    assert!(!scanner::test_helpers::is_whitelisted_path_for_test(&o.path, &wl));
}

#[test]
fn dir_tree_walks_directory() {
    let tmp = std::env::temp_dir().join("savelock_test_tree");
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).unwrap();
    std::fs::write(tmp.join("a.txt"), b"hello").unwrap();
    std::fs::create_dir_all(tmp.join("sub")).unwrap();
    std::fs::write(tmp.join("sub").join("b.txt"), b"world!").unwrap();
    let result = savemegb_lib::run_test::build_tree(&tmp, 2, 50);
    assert_eq!(result.name, "savelock_test_tree");
    assert!(!result.children.is_empty());
    let _ = std::fs::remove_dir_all(&tmp);
}
