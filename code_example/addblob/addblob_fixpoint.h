#ifndef ADDBLOB_H_GENERATED_
#define ADDBLOB_H_GENERATED_
/* Automically generated by wasm2c */
#ifdef __cplusplus
extern "C" {
#endif

#include <stdint.h>

#include "wasm-rt.h"

/* TODO(binji): only use stdint.h types in header */
#ifndef WASM_RT_CORE_TYPES_DEFINED
#define WASM_RT_CORE_TYPES_DEFINED
typedef uint8_t u8;
typedef int8_t s8;
typedef uint16_t u16;
typedef int16_t s16;
typedef uint32_t u32;
typedef int32_t s32;
typedef uint64_t u64;
typedef int64_t s64;
typedef float f32;
typedef double f64;
#endif

struct Z_env_module_instance_t;

typedef struct Z_addblob_module_instance_t {
  struct Z_env_module_instance_t* Z_env_module_instance;
  wasm_rt_memory_t w2c_ro_mem_0;
  wasm_rt_memory_t w2c_ro_mem_1;
  wasm_rt_memory_t w2c_rw_mem_0;
  wasm_rt_table_t w2c_T0;
} Z_addblob_module_instance_t;

extern void Z_addblob_init_module();
extern void Z_addblob_init(Z_addblob_module_instance_t*, struct Z_env_module_instance_t*);
extern void Z_addblob_free(Z_addblob_module_instance_t*);

/* import: 'env' 'attach_blob' */
extern void Z_addblob_Z_env_Z_attach_blob(struct Z_env_module_instance_t*, u32, u32);
/* import: 'env' 'designate_output' */
extern void Z_addblob_Z_env_Z_designate_output(struct Z_env_module_instance_t*, u32);
/* import: 'env' 'detach_mem' */
extern void Z_addblob_Z_env_Z_detach_mem(struct Z_env_module_instance_t*, u32, u32);
/* import: 'env' 'freeze_blob' */
extern void Z_addblob_Z_env_Z_freeze_blob(struct Z_env_module_instance_t*, u32, u32, u32);
/* import: 'env' 'get_tree_entry' */
extern void Z_addblob_Z_env_Z_get_tree_entry(struct Z_env_module_instance_t*, u32, u32, u32);

/* export: '_fixpoint_apply' */
extern void Z_addblob_Z__fixpoint_apply(Z_addblob_module_instance_t*);
/* export: 'ro_mem_0' */
extern wasm_rt_memory_t* Z_addblob_Z_ro_mem_0(Z_addblob_module_instance_t* module_instance);
/* export: 'ro_mem_1' */
extern wasm_rt_memory_t* Z_addblob_Z_ro_mem_1(Z_addblob_module_instance_t* module_instance);
/* export: 'rw_mem_0' */
extern wasm_rt_memory_t* Z_addblob_Z_rw_mem_0(Z_addblob_module_instance_t* module_instance);
#ifdef __cplusplus
}
#endif

#endif  /* ADDBLOB_H_GENERATED_ */