project(coolttviz)
cmake_minimum_required(VERSION 3.5)

set(COOLTTVIZ_LIBRARIES)
set(IMGUI_LIBRARIES)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

# OpenGL
if(APPLE)
  list(APPEND IMGUI_LIBRARIES "-framework OpenGL" "-framework Carbon")
endif(APPLE)

# SDL
find_package(SDL2 REQUIRED)
include_directories(${SDL2_INCLUDE_DIRS})

# CGLM
list(APPEND COOLTTVIZ_LIBRARIES cglm_headers)

# FreeType
find_package(Freetype REQUIRED)

# ImGUI
add_definitions("-DIMGUI_IMPL_API=extern \"C\" ")
set(CIMGUI_VENDOR ./vendor/cimgui)
set(IMGUI_STATIC_LIBRARY ON)

include_directories(${CIMGUI_VENDOR})
include_directories(${CIMGUI_VENDOR}/imgui/)

set(IMGUI_STATIC ON)

set(IMGUI_SOURCES
${CIMGUI_VENDOR}/cimgui.cpp
${CIMGUI_VENDOR}/imgui/imgui.cpp
${CIMGUI_VENDOR}/imgui/imgui_draw.cpp
${CIMGUI_VENDOR}/imgui/imgui_demo.cpp
${CIMGUI_VENDOR}/imgui/imgui_tables.cpp
${CIMGUI_VENDOR}/imgui/imgui_widgets.cpp
# Some helpers for the GL3W loader
./cimgui_extras.cpp)

set(IMGUI_BACKENDS ${CIMGUI_VENDOR}/imgui/backends)

# Set up the OpenGL Loader
add_compile_definitions("IMGUI_IMPL_OPENGL_LOADER_GL3W")
list(APPEND IMGUI_SOURCES ${CIMGUI_VENDOR}/imgui/examples/libs/gl3w/GL/gl3w.c)
include_directories(${CIMGUI_VENDOR}/imgui/examples/libs/gl3w)

list(APPEND IMGUI_SOURCES ${IMGUI_BACKENDS}/imgui_impl_sdl.cpp ${IMGUI_BACKENDS}/imgui_impl_opengl3.cpp)

# Coolttviz
set(COOLTTVIZ_SOURCES
cube.c
shader.c
render.c)

add_library(coolttviz STATIC ${IMGUI_SOURCES} ${COOLTTVIZ_SOURCES})

target_link_libraries(coolttviz ${COOLTTVIZ_LIBRARIES} ${IMGUI_LIBRARIES} ${SDL2_LIBRARIES})
include_directories(${CIMGUI_VENDOR}/generator/output/)

include_directories(./)

add_subdirectory(./vendor/cglm ./build)
