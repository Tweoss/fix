#pragma once

#include "wasm-rt.h"

namespace fixpoint {
void attach_tree( __m256i ro_handle, wasm_rt_externref_table_t* target_memory );

/* Traps if handle is inaccessible, if handle does not refer to a Blob */
void attach_blob( __m256i ro_handle, wasm_rt_memory_t* target_memory );

__m256i create_tree( wasm_rt_externref_table_t* table, size_t size );

__m256i create_blob( wasm_rt_memory_t* memory, size_t size );

}
