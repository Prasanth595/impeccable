//! ZCode detection coverage: a `~/.zcode` home dir must register as a
//! user-scope harness detection (so `impeccable install` auto-detect and
//! `check` pick it up), and a project `.zcode` dir as a project-scope one.

use std::collections::HashMap;

use impeccable_skills::providers::{Scope, Sys};
use impeccable_common::jsp;

fn temp_root(name: &str) -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("impeccable-{name}-{}-{nanos}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp root");
    let real = dir.canonicalize().unwrap().to_string_lossy().into_owned();
    real.strip_prefix(r"\\?\").map(str::to_string).unwrap_or(real)
}

fn sys_with(home: &str) -> Sys {
    let mut env: HashMap<String, String> = HashMap::new();
    env.insert("HOME".into(), home.to_string());
    env.insert("USERPROFILE".into(), home.to_string());
    Sys::new(env, home.to_string())
}

fn zcode_detections(sys: &Sys, root: &str) -> Vec<(Scope, String)> {
    sys.collect_install_detections(root)
        .into_iter()
        .filter(|d| d.provider == ".zcode")
        .map(|d| (d.scope, d.found_path))
        .collect()
}

#[test]
fn zcode_home_dir_is_detected_as_user_scope() {
    let home = temp_root("zcode-home-detect");
    let project = temp_root("zcode-home-project");
    std::fs::create_dir_all(jsp::join(&[&home, ".zcode"])).unwrap();
    let sys = sys_with(&home);

    let found = zcode_detections(&sys, &project);
    assert_eq!(found, vec![(Scope::User, jsp::join(&[&home, ".zcode"]))]);
}

#[test]
fn zcode_project_dir_is_detected_as_project_scope() {
    let home = temp_root("zcode-proj-home");
    let project = temp_root("zcode-proj-project");
    std::fs::create_dir_all(jsp::join(&[&project, ".zcode"])).unwrap();
    let sys = sys_with(&home);

    let found = zcode_detections(&sys, &project);
    assert_eq!(found, vec![(Scope::Project, jsp::join(&[&project, ".zcode"]))]);
}

#[test]
fn no_zcode_dir_means_no_detection() {
    let home = temp_root("zcode-absent-home");
    let project = temp_root("zcode-absent-project");
    let sys = sys_with(&home);

    assert!(zcode_detections(&sys, &project).is_empty());
}
