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

# ImGUI
add_definitions("-DIMGUI_IMPL_API=extern \"C\" ")
set(CIMGUI_VENDOR ../vendor/cimgui)

include_directories(${CIMGUI_VENDOR})
include_directories(${CIMGUI_VENDOR}/imgui/)

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

add_library(cimgui_sdl SHARED ${IMGUI_SOURCES})
target_link_libraries(cimgui_sdl ${IMGUI_LIBRARIES} ${SDL2_LIBRARIES})

# Compiling the actual executable.
include_directories(${CIMGUI_VENDOR}/generator/output/)

add_executable(coolttviz main.c)
target_link_libraries(coolttviz ${SDL2_LIBRARIES} cimgui_sdl)
