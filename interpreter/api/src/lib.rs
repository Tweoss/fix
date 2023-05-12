pub mod fix_api;

pub use fix_api::*;

fn print(str: &str) {
    unsafe { unsafe_io(str.as_ptr(), str.len()) }
}

#[export_name = "_fixpoint_apply_base"]
pub extern "C" fn _fixpoint_apply_base(encode: ExternRef<Thunk>) -> ExternRef<Thunk> {
    // the encode argument is a tree
    // with entries: resource limits, this program, number, add_file
    unsafe {
        let encode: ExternRef<()> = encode.cast();
        attach_tree_ro_table_0(&encode);
        let resource_limits = get_ro_table_0(0);
        let main_blob = get_ro_table_0(1);
        attach_blob_ro_mem_0(&get_ro_table_0(2));
        let argument = get_i32_ro_mem_0(0);
        let add_file = get_ro_table_0(3);

        unsafe fn next_num(
            n: i32,
            resource_limits: &ExternRef<()>,
            main_blob: &ExternRef<()>,
            add_file: &ExternRef<()>,
        ) -> Result<ExternRef<()>, ()> {
            if n == 0 || n == 1 {
                Ok(create_blob_i32(1).cast())
            } else {
                if grow_rw_table_0(4, &resource_limits) == -1 {
                    Err(())?
                }
                set_rw_table_0(0, resource_limits);
                set_rw_table_0(1, main_blob);
                set_rw_table_0(2, &(create_blob_i32(n).cast()));
                set_rw_table_0(3, add_file);
                Ok(create_thunk(&create_tree_rw_table_0(4).cast()).cast())
            }
        }
        if grow_rw_table_1(4, &resource_limits) == -1 {
            return resource_limits.cast();
        }

        set_rw_table_1(0, &resource_limits);
        set_rw_table_1(1, &add_file);
        // add file takes resource, add_file, arg1, arg2
        if let Ok(result) = next_num(argument - 1, &resource_limits, &main_blob, &add_file) {
            set_rw_table_1(2, &result);
        } else {
            return resource_limits.cast();
        }
        if let Ok(result) = next_num(argument - 2, &resource_limits, &main_blob, &add_file) {
            set_rw_table_1(3, &result);
        } else {
            return resource_limits.cast();
        }
        let sum = create_thunk(&create_tree_rw_table_1(4).cast());
        sum
    }
}
