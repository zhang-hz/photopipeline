cmake_minimum_required(VERSION 3.16)
project(photopipeline_halide LANGUAGES CXX)
set(CMAKE_CXX_STANDARD 17)

find_package(Halide REQUIRED)

add_library(colorspace_generator SHARED colorspace_generator.cpp)
target_link_libraries(colorspace_generator Halide::Halide)

add_library(resize_generator SHARED resize_generator.cpp)
target_link_libraries(resize_generator Halide::Halide)

add_library(tonemap_generator SHARED tonemap_generator.cpp)
target_link_libraries(tonemap_generator Halide::Halide)
