use std::fs;
use std::process::Command;

fn run_zak(arguments: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_zak"))
        .args(arguments)
        .output()
        .expect("could not start the zak CLI")
}

#[test]
fn new_check_build_and_run_work_together() {
    let project = std::env::temp_dir().join(format!("zak-cli-smoke-{}", std::process::id()));
    if project.exists() {
        fs::remove_dir_all(&project).expect("could not clean previous smoke-test project");
    }
    let project_arg = project.to_string_lossy().into_owned();

    let created = run_zak(&["new", "smoke", &project_arg]);
    assert!(created.status.success(), "{}", String::from_utf8_lossy(&created.stderr));

    let checked = run_zak(&["check", "--project", &project_arg]);
    assert!(checked.status.success(), "{}", String::from_utf8_lossy(&checked.stderr));

    let built = run_zak(&["build", "--target", "host", "--project", &project_arg]);
    assert!(built.status.success(), "{}", String::from_utf8_lossy(&built.stderr));

    let executed = run_zak(&["run", "--project", &project_arg]);
    assert!(executed.status.success(), "{}", String::from_utf8_lossy(&executed.stderr));
    assert!(
        String::from_utf8_lossy(&executed.stdout).contains("Hello from ZAK"),
        "program output did not contain the greeting: {}",
        String::from_utf8_lossy(&executed.stdout)
    );

    fs::remove_dir_all(&project).expect("could not remove smoke-test project");
}
