include_directories(${CMAKE_BINARY_DIR}/src/runtime)

add_executable(fix "fix.cc" "tester-utils.cc")
target_link_libraries(fix runtime storage)
target_include_directories (fix PUBLIC "${PROJECT_SOURCE_DIR}/src/tests")

add_executable(fixpoint-server "fixpoint-server.cc" "main.cc" "tester-utils.cc")
target_link_libraries(fixpoint-server runtime)

add_executable(fixpoint-client "fixpoint-client.cc" "main.cc" "tester-utils.cc")
target_link_libraries(fixpoint-client runtime)

add_executable(http-tester "http-tester.cc" "tester-utils.cc")
target_link_libraries(http-tester runtime storage http)
