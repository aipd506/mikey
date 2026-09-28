cmake_minimum_required(VERSION 3.16)
project(mikey C)

# libopus from the submodule (git submodule update --init), statically linked. Encoder only.
set(OPUS_BUILD_SHARED_LIBRARY OFF CACHE BOOL "" FORCE)
set(OPUS_BUILD_PROGRAMS OFF CACHE BOOL "" FORCE)
set(OPUS_BUILD_TESTING OFF CACHE BOOL "" FORCE)
set(OPUS_INSTALL_PKG_CONFIG_MODULE OFF CACHE BOOL "" FORCE)
set(OPUS_INSTALL_CMAKE_CONFIG_MODULE OFF CACHE BOOL "" FORCE)
add_subdirectory(opus EXCLUDE_FROM_ALL)

add_library(mikey SHARED opus_jni.c aaudio_jni.c)
# Lets aaudio_jni.c call APIs newer than our minimum Android behind a runtime check.
target_compile_definitions(mikey PRIVATE __ANDROID_UNAVAILABLE_SYMBOLS_ARE_WEAK__)
target_link_libraries(mikey PRIVATE opus aaudio)
