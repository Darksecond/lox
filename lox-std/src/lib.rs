use lox_vm::VirtualMachine;

/// Add the lox standard library to a VirtualMachine instance.
/// Right now the stdlib consists of 'clock'.
pub fn set_stdlib(vm: &mut VirtualMachine) {
    let mut native = vm.native();

    native.set_global_fn("clock", |_args| {
        use std::time::{SystemTime, UNIX_EPOCH};

        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs_f64();
        time.into()
    });
}
