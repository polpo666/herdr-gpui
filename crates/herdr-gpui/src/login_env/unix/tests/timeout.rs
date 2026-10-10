use super::*;

#[test]
fn timeout_kills_shell_and_descendant_group() {
    let (dir, path) = shell(
        "/bin/sleep 60 &\nprintf '%s %s' \"$$\" \"$!\" > \"$0.pids.tmp\"\n/bin/mv \"$0.pids.tmp\" \"$0.pids\"\nwait",
    );
    let (mut reader, writer) = UnixStream::pair().unwrap();
    reader.set_nonblocking(true).unwrap();
    let mut child = Command::new(&path)
        .args(["-l", "-i", "-c", "unused"])
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .stdout(Stdio::from(OwnedFd::from(writer)))
        .process_group(0)
        .spawn()
        .unwrap();

    // Wait for both processes before timing their cleanup. Starting a shell
    // under suite-wide load can take longer than the timeout being tested.
    let ready_deadline = Instant::now() + Duration::from_secs(10);
    let pids = loop {
        let pids = std::fs::read_to_string(dir.path().join("shell.pids"));
        if pids
            .as_ref()
            .is_ok_and(|pids| pids.split_whitespace().count() == 2)
            || Instant::now() >= ready_deadline
        {
            break pids;
        }
        std::thread::sleep(Duration::from_millis(10));
    };
    let start = Instant::now();
    let result = capture_started(
        &mut reader,
        &mut child,
        Duration::from_millis(100),
        LIMIT,
        b"start",
        b"end",
    );
    // Cleanup runs before checking readiness, even if the fixture never starts.
    assert!(matches!(result, Err(ProbeError::Timeout)), "{result:?}");
    assert!(start.elapsed() < Duration::from_secs(5));
    let pids = pids.unwrap();
    assert_eq!(pids.split_whitespace().count(), 2);
    for pid in pids.split_whitespace() {
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            let output = Command::new("/bin/ps")
                .args(["-o", "stat=", "-p", pid])
                .output()
                .unwrap();
            let state = String::from_utf8(output.stdout).unwrap();
            // Orphaned children may remain zombies until the system reaps them.
            if state.trim().is_empty() || state.trim().starts_with('Z') {
                break;
            }
            assert!(Instant::now() < deadline, "process {pid} survived: {state}");
            std::thread::sleep(Duration::from_millis(10));
        }
    }
}
