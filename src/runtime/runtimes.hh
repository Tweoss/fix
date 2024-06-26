#pragma once

#include "handle.hh"
#include "network.hh"
#include "relater.hh"
#include "repository.hh"

#include <memory>

class FrontendRT
{
  virtual Handle<Value> execute( Handle<Relation> ) = 0;

public:
  virtual ~FrontendRT() {}
};

class ReadOnlyRT : public FrontendRT
{
protected:
  Relater relater_ {};

public:
  ReadOnlyRT() {}
  static std::shared_ptr<ReadOnlyRT> init();
  virtual Handle<Value> execute( Handle<Relation> x ) override;
  IRuntime& get_rt() { return relater_; }
  std::unordered_set<Handle<Relation>> lookup( Handle<Object> handle );
};

class ReadWriteRT : public ReadOnlyRT
{
public:
  ReadWriteRT() {}

  static std::shared_ptr<ReadWriteRT> init();
  virtual Handle<Value> execute( Handle<Relation> x ) override;
};

class Client : public FrontendRT
{
protected:
  Relater relater_ { 0 };
  std::optional<NetworkWorker> network_worker_ {};

public:
  Client() {}
  ~Client();

  static std::shared_ptr<Client> init( const Address& address );
  virtual Handle<Value> execute( Handle<Relation> x ) override;
  IRuntime& get_rt() { return relater_; }
};

class Server
{
protected:
  Relater relater_ {};
  std::optional<NetworkWorker> network_worker_ {};

public:
  Server() {}

  static std::shared_ptr<Server> init( const Address& address );
  ~Server();
};
