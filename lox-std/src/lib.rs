use lox_vm::VirtualMachine;

/// Add the lox standard library to a VirtualMachine instance.
/// Right now the stdlib consists of 'clock'.
pub fn set_stdlib(vm: &mut VirtualMachine) {
    let mut native = vm.native();

    native.set_global_fn("clock", |_this, _args| {
        use std::time::{SystemTime, UNIX_EPOCH};

        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs_f64();
        time.into()
    });

    native.set_method(native.list_class(), "append", |this, args| {
        use lox_vm::memory::List;

        if !this.is_object_of_type::<List>() {
            //TODO Not panic!?
            panic!("this is not a list");
        }

        let this_list = this.as_object().cast::<List>();
        for value in args {
            this_list.push(*value);
        }

        this
    });
}

