//! Tools for repetitive import generation and externref binding

use walrus::{
    ir::{Block, Call, Instr, InstrSeqId, LoadKind, Loop, MemArg},
    FunctionBuilder, FunctionId, ImportKind, LocalFunction, Module, TableId, ValType,
};

fn process_instruction(
    instr: &mut Instr,
    original_function_id: FunctionId,
    to_process: &mut Vec<InstrSeqId>,
    new_id: FunctionId,
) {
    if let Instr::Call(Call { ref mut func }) = instr {
        if *func == original_function_id {
            *func = new_id;
        }
    } else if let Instr::Block(Block { seq }) | Instr::Loop(Loop { seq }) = instr {
        to_process.push(*seq);
    }
}
fn process(
    to_process: &mut Vec<InstrSeqId>,
    function: &mut LocalFunction,
    instruction_sequence_id: InstrSeqId,
    original_function_id: FunctionId,
    new_id: FunctionId,
) {
    function
        .block_mut(instruction_sequence_id)
        .iter_mut()
        .for_each(|(ref mut instr, _)| {
            process_instruction(instr, original_function_id, to_process, new_id);
        });
}

/// Locates the import and removes it.
/// If it does exist and is removed, calls on_removal
/// Otherwise, nothing
/// Returns error if import exists but is not a function
pub fn replace_import_function<F>(
    module: &mut Module,
    module_name: &str,
    name: &str,
    add_replacement: F,
) -> Result<(), String>
where
    F: Fn(&mut Module, String) -> FunctionId,
{
    if let Some(id) = module.imports.find(module_name, name) {
        if let ImportKind::Function(original_function_id) = module.imports.get(id).kind {
            let name = module.funcs.get(original_function_id).name.clone().unwrap();
            // rename original function
            module.funcs.get_mut(original_function_id).name = Some(name.clone() + ".original");
            // add the replacement to the module
            let new_id = add_replacement(module, name.clone());
            // change all occurences of the old id to be the new id
            // unfortunately, unable to directly change the contents of the function / change
            // the function type with walrus
            module.funcs.iter_local_mut().for_each(|i| {
                let mut to_process: Vec<InstrSeqId> = vec![];
                i.1.builder_mut()
                    .func_body()
                    .instrs_mut()
                    .iter_mut()
                    .for_each(|(ref mut instr, _)| {
                        process_instruction(instr, original_function_id, &mut to_process, new_id);
                    });
                while let Some(instruction_sequence_id) = to_process.pop() {
                    process(
                        &mut to_process,
                        i.1,
                        instruction_sequence_id,
                        original_function_id,
                        new_id,
                    );
                }
            });

            // delete old id's
            module
                .funcs
                .delete(module.funcs.by_name(&(name + ".original")).unwrap());
            module.imports.delete(id);
        } else {
            Err(format!("Import '{module_name}' '{name}' is not a function",))?
        }
    }
    Ok(())
}

pub struct ImportModule {
    pub name: String,
    pub categories: Vec<FunctionCategory>,
}

pub struct FunctionCategory {
    pub inputs: Vec<Type>,
    pub output: Option<Type>,
    pub functions: Vec<String>,
}

#[derive(PartialEq, Clone, Copy)]
pub enum Type {
    I32,
    ExternRef,
}

pub fn generate_replacements(
    module: &mut Module,
    imports: Vec<ImportModule>,
    externref_table: TableId,
    take_slab: FunctionId,
) -> Result<(), String> {
    for import in imports {
        for category in import.categories {
            let inputs: Vec<_> = category
                .inputs
                .iter()
                .map(|t| match t {
                    Type::I32 => ValType::I32,
                    Type::ExternRef => ValType::Externref,
                })
                .collect();
            let output = match category.output {
                Some(Type::ExternRef) => vec![ValType::Externref],
                Some(Type::I32) => vec![ValType::I32],
                None => vec![],
            };
            for function in category.functions {
                let import_type = module.types.add(&inputs, &output);
                let imported_function = module
                    .add_import_func(&import.name, &function, import_type)
                    .0;
                replace_import_function(module, "fix_api_shim", &function, |module, name| {
                    let mut wrapper = FunctionBuilder::new(
                        &mut module.types,
                        &vec![ValType::I32; inputs.len()],
                        &vec![ValType::I32; output.len()],
                    );

                    let mut builder = wrapper.func_body();
                    let mut args = vec![];

                    // load arguments
                    for input in &category.inputs {
                        match input {
                            Type::I32 => {
                                let local = module.locals.add(ValType::I32);
                                args.push(local);
                                builder.local_get(local);
                            }
                            Type::ExternRef => {
                                let local = module.locals.add(ValType::Externref);
                                args.push(local);
                                builder
                                    .local_get(local)
                                    .load(
                                        module.memories.iter().next().unwrap().id(),
                                        LoadKind::I32 { atomic: false },
                                        MemArg {
                                            align: 4,
                                            offset: 0,
                                        },
                                    )
                                    .table_get(externref_table);
                            }
                        }
                    }
                    // call the real imported function while
                    // transforming an externref output to i32
                    if category.output == Some(Type::ExternRef) {
                        let extern_ref = module.locals.add(ValType::Externref);
                        let index = module.locals.add(ValType::I32);
                        builder.call(imported_function);
                        builder.local_set(extern_ref);
                        builder.call(take_slab);
                        builder.local_tee(index);
                        builder.local_get(extern_ref);
                        builder.table_set(externref_table).local_get(index);
                    } else {
                        builder.call(imported_function);
                    }
                    // transform externref to i32 if the output value is externref

                    builder.name(name);
                    wrapper.finish(args, &mut module.funcs)
                })?;
            }
        }
    }
    Ok(())
}

pub fn get_default_imports() -> Vec<ImportModule> {
    let gen_names = |count: usize, name: String| {
        (0..count)
            .map(|i| name.clone() + &i.to_string())
            .collect::<Vec<_>>()
    };
    let mut imports = vec![ImportModule {
        name: "fixpoint".to_string(),
        categories: vec![
            FunctionCategory {
                inputs: vec![Type::ExternRef],
                output: Some(Type::ExternRef),
                functions: vec!["create_thunk".to_string()],
            },
            FunctionCategory {
                inputs: vec![Type::I32],
                output: Some(Type::ExternRef),
                functions: vec!["create_blob_i32".to_string()],
            },
            FunctionCategory {
                inputs: vec![Type::ExternRef],
                output: Some(Type::I32),
                functions: vec!["value_type".to_string()],
            },
        ],
    }];

    // the enumerated fixpoint functions are as follows
    // ro_table_[0-7]
    // - get_ = i32->ref STORAGE
    // - size_ = ()->i32 STORAGE
    // - get_attached_tree_ = ()->ref
    // - attach_tree_ = ref=>()
    // rw_table_[0-2]
    // - get_ = i32->ref STORAGE
    // - size_ = ()->i32
    // - grow_ = i32,ref->i32 STORAGE
    // - set_ = i32,ref->() STORAGE
    // - create_tree_ = i32->ref
    // ro_mem_[0-3]
    // - get_i32_ = i32->i32 STORAGE 
    // - byte_size_ = ()->i32
    // - get_attached_blob_ = ()->ref
    // - attach_blob_ = ref->()
    // rw_mem_[0-2]
    // - get_i32_ = i32->i32 STORAGE
    // - page_size_ = ()->i32 STORAGE
    // - grow_ = i32->i32 STORAGE
    // - set_i32_ = i32,i32->() STORAGE
    // - create_blob_ = i32->ref
    // copy_ro_mem_[0-3]_to_rw_[0-2] = i32,i32,i32->() STORAGE
    // copy_ro_table_[0-7]_to_rw_[0-2] = i32,i32,i32->() STORAGE
    let data = {
        // shorthand
        // return ref, return i32, return none, input ref, input i32
        let rref = Some(Type::ExternRef);
        let ri32 = Some(Type::I32);
        let rnone = None;
        let iref = Type::ExternRef;
        let ii32 = Type::I32;
        // module storage
        let mnone = "";
        let mstorage = "storage";
        [
            (
                "ro_table_",
                7,
                vec![
                    ("get", vec![ii32], rref, mstorage),
                    ("size", vec![], ri32, mstorage),
                    ("get_attached_tree", vec![], rref, mnone),
                    ("attach_tree", vec![iref], rnone, mnone),
                ],
            ),
            (
                "rw_table_",
                2,
                vec![
                    ("get", vec![ii32], rref, mstorage),
                    ("size", vec![], ri32, mnone),
                    ("grow", vec![ii32, iref], ri32, mstorage),
                    ("set", vec![ii32, iref], rnone, mstorage),
                    ("create_tree", vec![ii32], rref, mnone),
                ],
            ),
            (
                "ro_mem_",
                3,
                vec![
                    ("get_i32", vec![ii32], ri32, mstorage),
                    ("byte_size", vec![], ri32, mnone),
                    ("get_attached_blob", vec![], rref, mnone),
                    ("attach_blob", vec![iref], rnone, mnone),
                ],
            ),
            (
                "rw_mem_",
                2,
                vec![
                    ("get_i32", vec![ii32], ri32, mstorage),
                    ("size", vec![], ri32, mstorage),
                    ("grow", vec![ii32], ri32, mstorage),
                    ("set_i32", vec![ii32, ii32], rnone, mstorage),
                    ("create_blob", vec![ii32], rref, mnone),
                ],
            ),
            (
                "",
                2,
                vec![
                    ("copy_ro_mem_0_to_rw", vec![ii32, ii32, ii32], rnone, mstorage),
                    ("copy_ro_mem_1_to_rw", vec![ii32, ii32, ii32], rnone, mstorage),
                    ("copy_ro_mem_2_to_rw", vec![ii32, ii32, ii32], rnone, mstorage),
                    ("copy_ro_mem_3_to_rw", vec![ii32, ii32, ii32], rnone, mstorage),
                    ("copy_ro_table_0_to_rw", vec![ii32, ii32, ii32], rnone, mstorage),
                    ("copy_ro_table_1_to_rw", vec![ii32, ii32, ii32], rnone, mstorage),
                    ("copy_ro_table_2_to_rw", vec![ii32, ii32, ii32], rnone, mstorage),
                    ("copy_ro_table_3_to_rw", vec![ii32, ii32, ii32], rnone, mstorage),
                    ("copy_ro_table_4_to_rw", vec![ii32, ii32, ii32], rnone, mstorage),
                    ("copy_ro_table_5_to_rw", vec![ii32, ii32, ii32], rnone, mstorage),
                    ("copy_ro_table_6_to_rw", vec![ii32, ii32, ii32], rnone, mstorage),
                    ("copy_ro_table_7_to_rw", vec![ii32, ii32, ii32], rnone, mstorage),
                ],
            ),
        ]
    };

    let mut fixpoint = vec![];
    let mut fixpoint_storage = vec![];

    // [(&str, i32, Vec<(&str, Vec<Type>, Option<Type>)>); 6]
    for (base, count, variations) in data {
        for (prefix, inputs, output, module_ending) in variations {
            let category = FunctionCategory {
                inputs,
                output,
                functions: gen_names(count + 1, format!("{prefix}_{base}")),
            };
            match module_ending {
                "" => fixpoint.push(category),
                "storage" => fixpoint_storage.push(category),
                _ => unreachable!()
            }
        }
    }

    imports.push(ImportModule {
        name: "fixpoint_storage".to_string(),
        categories: fixpoint_storage,
    });
    imports.push(ImportModule {
        name: "fixpoint".to_string(),
        categories: fixpoint,
    });

    imports
}
