cmake_minimum_required(VERSION 3.0.0)
project(examples VERSION 0.1.0 LANGUAGES C CXX)

include(CTest)
enable_testing()

string(TOLOWER "${CMAKE_BUILD_TYPE}" BUILD_TYPE_PATH)

set(CMAKE_RUNTIME_OUTPUT_DIRECTORY_DEBUG ../../../target/debug/examples)
set(CMAKE_RUNTIME_OUTPUT_DIRECTORY_RELEASE ../../../target/release/examples)

include_directories(../include)
link_directories(../../target/${BUILD_TYPE_PATH})

link_libraries(lemurs_8080.lib)

add_executable(diag diag.cpp)


set(CPACK_PROJECT_NAME ${PROJECT_NAME})
set(CPACK_PROJECT_VERSION ${PROJECT_VERSION})
include(CPack)
