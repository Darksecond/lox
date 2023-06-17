use lox_vm::VirtualMachine;

/// Add the lox standard library to a VirtualMachine instance.
/// Right now the stdlib consists of 'clock'.
pub fn set_stdlib(vm: &mut VirtualMachine) {
    let mut native = vm.native();

    native.set_global_fn("clock", |_native, _this, _args| {
        use std::time::{SystemTime, UNIX_EPOCH};

        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs_f64();
        time.into()
    });

    native.set_method(native.list_class(), "append", |_native, this, args| {
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

    native.set_global_fn("printx", |mut native, _this, args| {
        let symbol_to_string = native.intern("toString");
        let func = native.get_global(symbol_to_string).expect("Could not find toString");
        let func = func.as_object();

        for arg in args {
            let string = native.call(func, &[*arg]);
            println!("{}", string);
        }

        lox_vm::value::Value::NIL
    });

    native.set_global_fn("toString", |native, _this, _args| {
        lox_vm::value::Value::from_object(native.manage(lox_vm::string::LoxString::from("Hello, World!")))
    });
}

