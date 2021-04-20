#pragma once

#include <map>
#include <string>
#include <string_view>
#include <unordered_map>

#include "blob.hh"
#include "ccompiler.hh"
#include "name.hh"
#include "program.hh"
#include "storage.hh"
#include "thunk.hh"
#include "tree.hh"
#include "wasmcompiler.hh"

#include "absl/container/flat_hash_map.h"
#include "wasienv.hh"

class RuntimeStorage {
  private:
    // Map from name to Blob
    InMemoryStorage<Blob> name_to_blob_;
    // Map from name to Tree
    InMemoryStorage<Tree> name_to_tree_;
    // Map from name to Thunk
    InMemoryStorage<Thunk> name_to_thunk_;
    // Map from name to program
    InMemoryStorage<Program> name_to_program_;

    // Map from thunk name to blob name
    absl::flat_hash_map<std::string, std::string> thunk_to_blob_;

    RuntimeStorage ()
      : name_to_blob_(),
        name_to_tree_(),
        name_to_thunk_(),
        name_to_program_(),
        thunk_to_blob_()
    {}

  public:
    // Return reference to blob content
    std::string_view getBlob( const Name & name );

    // Return reference to static runtime storage
    static RuntimeStorage & getInstance () 
    {
      static RuntimeStorage runtime_instance;
      return runtime_instance;
    }

    // Return reference to Tree
    const Tree & getTree ( const Name & name );

    // add blob
    template<typename T>
    std::string addBlob( T&& content )
    {
      std::string blob_content ( reinterpret_cast<char *>( &content ), sizeof( T ) );
      Blob blob ( move( blob_content ) );
      std::string name = blob.name();
      name_to_blob_.put( name, std::move( blob ) );
      return name;
    }

    // add wasm module
    void addWasm( const std::string & name, const std::string & wasm_content );

    // add elf program
    void addProgram( const std::string & name, std::vector<std::string> && inputs, std::vector<std::string> && outputs, std::string & program_content );

    // force the object refered to by a name
    void force( const Name & name );

    // force a Tree
    void forceTree( const Name & tree_name );

    // force a Thunk
    void forceThunk( const Thunk & thunk ); 
   
    // Force all strict inputs, return the blob name of the wasm module
    void prepareEncode( const Tree & encode, Invocation & invocation );

    // Evaluate an encode
    void evaluateEncode( const Name & encode_name );

    // add encode
    // std::string addEncode( const std::string & program_name, const std::vector<std::string> & input_blobs );

    // execute encode
    // void executeEncode( const std::string & encode_name, int arg1, int arg2 );
};