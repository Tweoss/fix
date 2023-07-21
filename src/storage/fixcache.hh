#pragma once

#include <condition_variable>
#include <mutex>
#include <optional>
#include <queue>
#include <shared_mutex>
#include <stdexcept>

#include "absl/container/flat_hash_map.h"
#include "absl/hash/hash.h"
#include "entry.hh"
#include "task.hh"

class fixcache
{
public:
  /// Any function which takes a Task as its argument and returns void.
  /// Used to schedule tasks to be run. 
  /// Required for liveness to call `cache()` when a task is finished. 
  using QueueFunction = std::function<void( Task )>;

private:
  absl::flat_hash_map<Task, std::optional<Handle>, absl::Hash<Task>> fixcache_;

  absl::flat_hash_map<std::pair<Task, std::size_t>, Task, absl::Hash<std::pair<Task, std::size_t>>>
    dependency_cache_;
  absl::flat_hash_map<Task, std::shared_ptr<std::atomic<std::size_t>>, absl::Hash<Task>> blocked_count_;
  std::shared_mutex fixcache_mutex_;
  /// Condition variable that is notified every time a Task finishes.
  std::condition_variable_any fixcache_cv_;

  /// Mark `depender` as depending on `dependee`. 

  /// Uses multidependency index if there are multiple dependers on a single 
  /// dependee. A unique index is selected by linear scan.
  /// @throws runtime_error if dependee equals depender.
  void insert_dependency( Task dependee, Task depender )
  {
    if ( dependee == depender ) {
      throw std::runtime_error( "attempted to insert self-dependency" );
    }
    for ( size_t i = 1;; ++i ) {
      // we deal with duplicates by adding a dependency index; we linear scan until we find a free index
      auto pair = std::make_pair( dependee, i );
      auto ins = dependency_cache_.insert( { pair, depender } );
      // If insertion was sucessful
      if ( ins.second ) {
        break;
      }
    }
  }

  /// Requeue tasks that depend on `finished` using provided `QueueFunction`.

  /// Uses multidependency index to find all depending tasks efficiently.
  /// Reduces the count for each depending task (requestor) by 1 and if the 
  /// count reaches 0, will queue that task.
  void unblock_jobs( Task finished, QueueFunction queue )
  {
    for ( int i = 1;; ++i ) {
      auto pair = std::make_pair( finished, i );
      if ( dependency_cache_.contains( pair ) ) {
        Task requestor = dependency_cache_.at( pair );
        size_t blocked = blocked_count_.at( requestor )->fetch_sub( 1 ) - 1;
        if ( blocked == 0 ) {
          queue( requestor );
        }
      } else {
        break;
      }
    }
  }

  /// If `task` does not exist in cache, inserts and queues the task.

  /// Initializes the fixcache entry to nullopt to signal that the task is 
  /// queued and sets the blocking count to be 0. 
  /// @returns True if the task is queued and inserted. Otherwise false.
  bool add_task( Task task, QueueFunction queue )
  {
    if ( not fixcache_.contains( task ) ) {
      fixcache_.insert( { task, std::nullopt } );
      blocked_count_.insert( { task, std::make_shared<std::atomic<std::size_t>>( 0 ) } );
      queue( task );
      return true;
    }
    return false;
  }

public:
  fixcache()
    : fixcache_()
    , dependency_cache_()
    , blocked_count_()
    , fixcache_mutex_()
    , fixcache_cv_()
  {}

  std::optional<Handle> get( Task task )
  {
    std::shared_lock lock( fixcache_mutex_ );
    if ( fixcache_.contains( task ) ) {
      return fixcache_.at( task );
    } else {
      return {};
    }
  }

  /// Stores the result of a computation in fixcache and may unblock dependers.

  /// Decreases the count and possibly requeues depending tasks. 
  /// @throws runtime_error if the cache already contains this value.
  /// @throws runtime_error if the task is still marked as blocked.
  void cache( Task task, Handle result, QueueFunction queue )
  {
    std::unique_lock lock( fixcache_mutex_ );
    if ( fixcache_.at( task ).has_value() ) {
      throw std::runtime_error( "double-cache" );
    }
    if ( blocked_count_.at( task )->load() != 0 ) {
      throw std::runtime_error( "caching result of task which is still blocked" );
    }
    fixcache_.at( task ) = result;
    fixcache_cv_.notify_all();
    unblock_jobs( task, queue );
  }

  /// Starts a task if not already in cache. 
  void start( Task task, QueueFunction queue )
  {
    std::unique_lock lock( fixcache_mutex_ );
    add_task( task, queue );
  }

  /// Queues dependee if not queued and marks dependency if not finished.

  /// Adds the dependee to queue if it is not already queued.
  /// Inserts dependency of depender on dependee and increments blocked count 
  /// of depender if dependee is not already cached. 
  /// @returns { result } of computing dependee if already cached.
  /// @returns {} otherwise.
  std::optional<Handle> get_or_add_dependency( Task dependee, Task depender, QueueFunction queue )
  {
    std::unique_lock lock( fixcache_mutex_ );
    add_task( dependee, queue );
    if ( fixcache_.at( dependee ).has_value() ) {
      return fixcache_.at( dependee );
    }
    insert_dependency( dependee, depender );
    blocked_count_.at( depender )->fetch_add( 1 );
    return {};
  }

  /// Increasing blocking_count of task.
  void increment_blocking_count( Task task, std::size_t count = 1 )
  {
    std::shared_lock lock( fixcache_mutex_ );
    blocked_count_.at( task )->fetch_add( count );
  }

  /// Queues dependee if not already queued. Insert dependency if not finished. 

  /// Decreases the blocking count if dependee if already finished.
  /// Otherwise, inserts the dependency.
  /// @returns the new blocked_count value
  size_t add_dependency_or_decrement_blocking_count( Task dependee, Task depender, QueueFunction queue )
  {
    std::unique_lock lock( fixcache_mutex_ );
    add_task( dependee, queue );
    if ( fixcache_.at( dependee ).has_value() ) {
      return blocked_count_.at( depender )->fetch_sub( 1 ) - 1;
    }
    insert_dependency( dependee, depender );
    return *blocked_count_.at( depender );
  }

  /// Blockingly waits for the target task to finish.

  /// @returns the resulting Handle
  Handle get_blocking( Task target )
  {
    using namespace std;
    unique_lock lock( fixcache_mutex_ );
    fixcache_cv_.wait(
      lock, [this, target] { return fixcache_.contains( target ) and fixcache_.at( target ).has_value(); } );
    return fixcache_.at( target ).value();
  }
};
