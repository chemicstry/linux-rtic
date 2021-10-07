# linux-rtic

[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](https://github.com/chemicstry/linux-rtic)
[![Cargo](https://img.shields.io/crates/v/linux-rtic.svg)](https://crates.io/crates/linux-rtic)
[![Documentation](https://docs.rs/linux-rtic/badge.svg)](https://docs.rs/linux-rtic)

An [RTIC](https://rtic.rs/) implementation for real-time Linux.

## How it Works

This implementation of RTIC is based on `std::thread` by spawning a thread for each task priority group. Threads are initialized with `SCHED_FIFO` (requires PRREMPT-RT kernel patch) real-time policy. Task priorities correspond 1:1 to Linux priorities and usually have a range of 1-99.

### Scheduling

Scheduling of tasks is done by [futex-queue](https://crates.io/crates/futex-queue), which cleverly utilizes futex syscall to wait on both immediate and scheduled (timed) tasks on a single syscall. No timer thread (and additional context switching) is required.

### Resource Locking

Original [cortex-m-rtic](https://github.com/rtic-rs/cortex-m-rtic) uses Stack Resource Policy (SRP), but it is difficult to emulate in userspace Linux. Firstly, setting thread priority for each lock/unlock involves an expensive syscall (~10us on Raspberry Pi 4). Secondly, setting thread priority does not guarantee that lower priority thread will not run. Lower priority thread can be executed on a different core, or when higher priority thread is suspended (i.e. I/O syscall). While it is possible to fix memory safety issues by a backup synchronisation mechanism (mutex), the syscall overhead is too high for real-time applications.

To solve the issue, a [pcp-mutex](https://crates.io/crates/pcp-mutex) library was written, which implements Original Priority Ceiling Protocol (OPCP). This allows preserving two important properties of SRP: bounding priority inversion and statically preventing deadlocks. This mutex is lock-free in the fast path. Technical details are in the pcp-mutex README.

### Other Notes

Scheduling tasks in userspace threads is slow due to context switching overhead (~10us on Raspberry Pi 4) and other approaches were explored:
- POSIX signals are used in the older [linux-rtfm](https://github.com/japaric/linux-rtfm) implementation. They are faster than thread context switching, however, tasks are limited to reentrant (signal safe) functions only, which forces to use `no_std`. Resource locking is also slower, because of signal masking syscalls.
- Kernel threads are only marginally faster as most of the overhead seems to be in the scheduler itself. So losing userspace safety and std library didn't seem worth it.
- Hard interrupt context would be closest to what cortex-m-rtic does, but Linux does not support interrupt prioritization (only IRQ threads have priorities) and would require major kernel modifications.

## Examples

Running examples requires Linux with PREEMPT-RT patched kernel for `SCHED_FIFO` and root privileges. This requirement can be lifted by compiling with `--no-default-features`, but then all tasks will share the same priority.

Build:
> cargo build --release --example priority_inversion

Run (requires sudo for `sched_setscheduler` syscall):
> sudo target/release/examples/priority_inversion

Single core:
> sudo taskset -c 1 target/release/examples/priority_inversion

No real-time priorities:
> cargo run --release --example priority_inversion --no-default-features

## Credits

This work was done as a part of my Thesis at University of Twente.
