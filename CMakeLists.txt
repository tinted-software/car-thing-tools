cmake_minimum_required(VERSION 3.30 FATAL_ERROR FATAL_ERROR FATAL_ERROR FATAL_ERROR FATAL_ERROR FATAL_ERROR FATAL_ERROR FATAL_ERROR FATAL_ERROR)

set(CMAKE_DISABLE_IN_SOURCE_BUILDS ON)
set(CMAKE_DISABLE_SOURCE_CHANGES ON)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)
set(CMAKE_EXPERIMENTAL_CXX_IMPORT_STD "0e5b6991-d74f-4b3d-a41c-cf096e0b2508")

project(car-thing-tools)

set(CMAKE_CXX_CLANG_TIDY clang-tidy -p ${CMAKE_BINARY_DIR})
set(CMAKE_CXX_MODULE_STD ON)

find_package(PkgConfig REQUIRED)
pkg_check_modules(LIBUSB REQUIRED libusb-1.0)

find_package(glog REQUIRED)

add_library(
	usbplusplus
	STATIC
)
target_sources(
	usbplusplus
	PUBLIC
	FILE_SET
	CXX_MODULES
	FILES
	usb.cxx
)
target_link_libraries(usbplusplus PUBLIC ${LIBUSB_LIBRARIES})
target_include_directories(usbplusplus
	PRIVATE
	${LIBUSB_INCLUDE_DIRS}
)
target_compile_features(
	usbplusplus
	PUBLIC
	cxx_std_26
)

add_executable(
	amlogic-update
	update.cpp
)
target_link_libraries(amlogic-update PUBLIC usbplusplus glog::glog)

install(
	TARGETS usbplusplus 
	EXPORT usbplusplusTargets 
	FILE_SET CXX_MODULES
	DESTINATION include
)
install(
	DIRECTORY "${CMAKE_CURRENT_SOURCE_DIR}/include/usbplusplus/"
	DESTINATION "include"
	FILES_MATCHING
	PATTERN "*.h"
)

install(TARGETS amlogic-update)
