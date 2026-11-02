#!/bin/sh
# rustc wrapper (build.rustc-wrapper): [env] in .cargo/config.toml does not reach rustc, this does.
# jemalloc inside rustc: never return memory to the OS.
export _RJEM_MALLOC_CONF=dirty_decay_ms:-1,muzzy_decay_ms:-1
exec "$@"
