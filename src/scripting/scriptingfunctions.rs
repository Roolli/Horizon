use rusty_v8 as v8;
use std::io::stdout;
use std::io::Write;
pub struct ScriptingFunctions;

impl ScriptingFunctions {
    pub fn print(
        scope: &mut v8::HandleScope,
        args: v8::FunctionCallbackArguments,
        _rv: v8::ReturnValue,
    ) {
        let obj = args.get(0);
        let try_catch_scope = &mut v8::TryCatch::new(scope);
        let string = obj.to_string(try_catch_scope).unwrap();

        print!("{}", string.to_rust_string_lossy(try_catch_scope));
        stdout().flush().unwrap();
    }
}
