use std::error::Error;

use walrus::{
    ir::{MemArg, StoreKind},
    FunctionBuilder, FunctionId, Module, ModuleConfig, ValType,
};

use crate::generator::{generate_replacements, get_default_imports, replace_import_function};

mod generator;

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args();
    let file = args
        .nth(1)
        .ok_or("Pass wasm file to be modified as argument")?;

    let output_file = if let Some(arg) = args.next() {
        (arg == "-o")
            .then(|| args.next())
            .flatten()
            .ok_or("Usage: '-o output_file'")?
    } else {
        "out.wasm".to_string()
    };

    println!("Parsing {}", file);

    let mut module =
        Module::from_file_with_config(file, &ModuleConfig::new().preserve_code_transform(true))?;

    let get_func_id = |module: &Module, name: &str| -> Result<FunctionId, String> {
        module
            .funcs
            .by_name(name)
            .ok_or_else(|| format!("Could not find function {}", name))
    };

    let delete_export = |module: &mut Module, id: FunctionId| -> Result<(), String> {
        module.exports.delete(
            module
                .exports
                .get_exported_func(id)
                .ok_or_else(|| format!("Could not find export"))?
                .id(),
        );
        Ok(())
    };

    println!("Removing return and take slab exports");

    // remove the return and take slab definitions
    let return_slab = get_func_id(&module, "_fixpoint_return_slab.command_export")?;
    let take_slab = get_func_id(&module, "_fixpoint_take_slab.command_export")?;
    delete_export(&mut module, return_slab)?;
    delete_export(&mut module, take_slab)?;

    println!("Adding externref table");

    // add a custom externref table
    let externref_table = module.tables.add_local(0, None, ValType::Externref);

    println!("Replacing table_grow, environ_get, environ_sizes_get, proc_exit");

    replace_import_function(&mut module, "fix_api_shim", "table_grow", |module, name| {
        let mut table_grow = FunctionBuilder::new(&mut module.types, &[ValType::I32], &[]);
        let offset = module.locals.add(ValType::Externref);
        table_grow
            .func_body()
            .ref_null(ValType::Externref)
            .local_get(offset)
            .table_grow(externref_table)
            .drop()
            .name(name);
        table_grow.finish(vec![offset], &mut module.funcs)
    })?;

    println!("Generating replacement imports");

    // create_blob_i32
    generate_replacements(
        &mut module,
        get_default_imports(),
        externref_table,
        take_slab,
    )?;

    replace_import_function(
        &mut module,
        "wasi_snapshot_preview1",
        "environ_get",
        |module, name| {
            let mut environ_get = FunctionBuilder::new(
                &mut module.types,
                &[ValType::I32, ValType::I32],
                &[ValType::I32],
            );
            environ_get.func_body().i32_const(0).name(name);
            environ_get.finish(vec![], &mut module.funcs)
        },
    )?;

    replace_import_function(
        &mut module,
        "wasi_snapshot_preview1",
        "environ_sizes_get",
        |module, name| {
            let mut environ_get_sizes = FunctionBuilder::new(
                &mut module.types,
                &[ValType::I32, ValType::I32],
                &[ValType::I32],
            );
            let i0 = module.locals.add(ValType::I32);
            let i1 = module.locals.add(ValType::I32);
            environ_get_sizes
                .func_body()
                .local_get(i0)
                .i32_const(0)
                .store(
                    module.memories.iter().next().unwrap().id(),
                    StoreKind::I32 { atomic: false },
                    MemArg {
                        align: 4,
                        offset: 0,
                    },
                )
                .local_get(i1)
                .i32_const(0)
                .store(
                    module.memories.iter().next().unwrap().id(),
                    StoreKind::I32 { atomic: false },
                    MemArg {
                        align: 4,
                        offset: 0,
                    },
                )
                .i32_const(0)
                .name(name);
            environ_get_sizes.finish(vec![], &mut module.funcs)
        },
    )?;

    replace_import_function(
        &mut module,
        "wasi_snapshot_preview1",
        "proc_exit",
        |module, name| {
            let mut proc_exit = FunctionBuilder::new(&mut module.types, &[ValType::I32], &[]);
            proc_exit.func_body().loop_(None, |_loop_| ()).name(name);
            proc_exit.finish(vec![], &mut module.funcs)
        },
    )?;

    println!("Adding _fixpoint_apply wrapper");
    {
        let mut fixpoint_apply = FunctionBuilder::new(
            &mut module.types,
            &[ValType::Externref],
            &[ValType::Externref],
        );
        let input = module.locals.add(ValType::Externref);
        let index = module.locals.add(ValType::I32);
        fixpoint_apply
            .func_body()
            .call(take_slab)
            .local_set(index)
            .local_get(index)
            .local_get(input)
            .table_set(externref_table)
            .local_get(index)
            .call(get_func_id(&module, "_fixpoint_apply_base.command_export")?)
            .table_get(externref_table)
            .name("_fixpoint_apply".to_string());
        let id = fixpoint_apply.finish(vec![input], &mut module.funcs);
        module.exports.add("_fixpoint_apply", id);
    }

    println!("Writing to {output_file}");
    module.emit_wasm_file(output_file)?;
    Ok(())
}
