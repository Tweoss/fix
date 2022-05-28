(module
  (import "flatware" "memory" (memory $tmem 0))
  (import "sloth" "memory" (memory $smem 0))
  (import "sloth" "_start" (func $start))
  ;; (import "fixpoint" "exit" (func $fixpoint_exit (param externref)))
  (memory $mymem0 (export "rw_mem_0") 1)
  (memory $mymem1 (export "rw_mem_1") 1)
  (table $return (export "rw_table_0") 2 externref)
  (func (export "memory_copy_rw_0") (param $ptr i32) (param $len i32)
  	(memory.copy $tmem $mymem0
		(i32.const 0)
		(local.get $ptr)
		(local.get $len))
  )
  (func (export "memory_copy_rw_1") (param $ptr i32) (param $len i32)
  	(memory.copy $tmem $mymem1
		(i32.const 0)
		(local.get $ptr)
		(local.get $len))
  )
  (func (export "get_i32") (param $offset i32) (result i32)
    (local.get $offset)
    (i32.load $smem)
  )
  (func (export "designate_output") (param $a externref)
	(table.set $return (i32.const 0) (local.get $a))
  )
  (func (export "write_stdout") (param $a externref)
	(table.set $return (i32.const 1) (local.get $a))
  )
  (func (export "get_from_return") (param $index i32) (result externref)
  (table.get $return (local.get $index))
  )
  (func (export "set_return") (param $index i32) (param $val externref)
  (table.set $return (local.get $index) (local.get $val))
  )
  ;; (func (export "_fixpoint_apply") (param externref) (result externref)
	;; (call $start)
  ;; (call $fixpoint_exit (table.get $return (i32.const 0)))
  ;; unreachable
  ;; )
  ;; (func (export "flatware_exit")
  ;; (call $fixpoint_exit (table.get $return (i32.const 0)))
  ;; unreachable)
)

