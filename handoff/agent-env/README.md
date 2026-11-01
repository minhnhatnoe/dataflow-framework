# Agent environment (cloud container)

- `cap.sh`: memory cap (cgroup v1 child; systemd-run is unavailable in the container) plus a machine-wide
  lock: `build` takes it shared (CARGO_BUILD_JOBS=2), `measure` exclusive (CARGO_BUILD_JOBS=4), so timings never overlap
  another agent's build. Usage: `cap.sh <build|measure> <cap> <seconds> <cmd...>`.
- `rules.md`: the rules block pasted into every subagent brief.

Both hard-code the scratchpad path of the session that wrote them
(`/tmp/claude-0/-home-user-repo/8e981b07-5287-54fa-a02f-4121b77bfce6/scratchpad`, used for the lock file and agents'
scratch dirs). A new session: copy `cap.sh` to its scratchpad and replace that path in both files.
