use deno_core::error::AnyError;
use deno_core::op;
use deno_core::Extension;
use std::rc::Rc;

#[op]
async fn op_read_file(path: String) -> Result<String, AnyError> {
    let contents = tokio::fs::read_to_string(path).await?;
    Ok(contents)
}

#[op]
async fn op_write_file(path: String, contents: String) -> Result<(), AnyError> {
    tokio::fs::write(path, contents).await?;
    Ok(())
}

#[op]
fn op_remove_file(path: String) -> Result<(), AnyError> {
    std::fs::remove_file(path)?;
    Ok(())
}

async fn run_js(file_path: &str) -> Result<(), AnyError> {
    // 文件路径 
    let main_module = deno_core::resolve_path(file_path)?;
    // 注册扩展 Extension，传递操作文件方法给 JS 调用
    let runjs_extension = Extension::builder("runjs")
        .ops(vec![
            op_read_file::decl(),
            op_write_file::decl(),
            op_remove_file::decl(),
        ])
        .build();

    //  new 一个 JsRuntime
    let mut js_runtime = deno_core::JsRuntime::new(deno_core::RuntimeOptions {
        module_loader: Some(Rc::new(deno_core::FsModuleLoader)),
        extensions: vec![runjs_extension],
        ..Default::default()
    });

    // 执行 runtime.js
    js_runtime
        .execute_script("[runjs:runtime.js]", include_str!("./runtime.js"))
        .unwrap();

    let mod_id = js_runtime.load_main_module(&main_module, None).await?;
    let result = js_runtime.mod_evaluate(mod_id);
    // 执行一个事件循环
    js_runtime.run_event_loop(false).await?;
    result.await?
}

fn main() {
    // 开启一个 tokio 运行时线程
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    if let Err(error) = runtime.block_on(run_js("./example.js")) {
        eprintln!("error: {}", error);
    }
}
