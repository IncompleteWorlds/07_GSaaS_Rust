# Install script for directory: /home/alberto/.cargo/registry/src/github.com-1ecc6299db9ec823/nng-sys-1.2.4-rc.1/nng/src

# Set the install prefix
if(NOT DEFINED CMAKE_INSTALL_PREFIX)
  set(CMAKE_INSTALL_PREFIX "/home/alberto/Projects/07_GSaaS_Rust/03_TM_Decoder/target/debug/build/nng-sys-43ddbed0e36856bc/out")
endif()
string(REGEX REPLACE "/$" "" CMAKE_INSTALL_PREFIX "${CMAKE_INSTALL_PREFIX}")

# Set the install configuration name.
if(NOT DEFINED CMAKE_INSTALL_CONFIG_NAME)
  if(BUILD_TYPE)
    string(REGEX REPLACE "^[^A-Za-z0-9_]+" ""
           CMAKE_INSTALL_CONFIG_NAME "${BUILD_TYPE}")
  else()
    set(CMAKE_INSTALL_CONFIG_NAME "Debug")
  endif()
  message(STATUS "Install configuration: \"${CMAKE_INSTALL_CONFIG_NAME}\"")
endif()

# Set the component getting installed.
if(NOT CMAKE_INSTALL_COMPONENT)
  if(COMPONENT)
    message(STATUS "Install component: \"${COMPONENT}\"")
    set(CMAKE_INSTALL_COMPONENT "${COMPONENT}")
  else()
    set(CMAKE_INSTALL_COMPONENT)
  endif()
endif()

# Install shared libraries without execute permission?
if(NOT DEFINED CMAKE_INSTALL_SO_NO_EXE)
  set(CMAKE_INSTALL_SO_NO_EXE "1")
endif()

# Is this installation the result of a crosscompile?
if(NOT DEFINED CMAKE_CROSSCOMPILING)
  set(CMAKE_CROSSCOMPILING "FALSE")
endif()

if("x${CMAKE_INSTALL_COMPONENT}x" STREQUAL "xLibraryx" OR NOT CMAKE_INSTALL_COMPONENT)
  file(INSTALL DESTINATION "${CMAKE_INSTALL_PREFIX}/lib" TYPE STATIC_LIBRARY FILES "/home/alberto/Projects/07_GSaaS_Rust/03_TM_Decoder/target/debug/build/nng-sys-43ddbed0e36856bc/out/build/libnng.a")
endif()

if("x${CMAKE_INSTALL_COMPONENT}x" STREQUAL "xLibraryx" OR NOT CMAKE_INSTALL_COMPONENT)
  if(EXISTS "$ENV{DESTDIR}${CMAKE_INSTALL_PREFIX}/lib/cmake/nng/nng-targets.cmake")
    file(DIFFERENT EXPORT_FILE_CHANGED FILES
         "$ENV{DESTDIR}${CMAKE_INSTALL_PREFIX}/lib/cmake/nng/nng-targets.cmake"
         "/home/alberto/Projects/07_GSaaS_Rust/03_TM_Decoder/target/debug/build/nng-sys-43ddbed0e36856bc/out/build/src/CMakeFiles/Export/lib/cmake/nng/nng-targets.cmake")
    if(EXPORT_FILE_CHANGED)
      file(GLOB OLD_CONFIG_FILES "$ENV{DESTDIR}${CMAKE_INSTALL_PREFIX}/lib/cmake/nng/nng-targets-*.cmake")
      if(OLD_CONFIG_FILES)
        message(STATUS "Old export file \"$ENV{DESTDIR}${CMAKE_INSTALL_PREFIX}/lib/cmake/nng/nng-targets.cmake\" will be replaced.  Removing files [${OLD_CONFIG_FILES}].")
        file(REMOVE ${OLD_CONFIG_FILES})
      endif()
    endif()
  endif()
  file(INSTALL DESTINATION "${CMAKE_INSTALL_PREFIX}/lib/cmake/nng" TYPE FILE FILES "/home/alberto/Projects/07_GSaaS_Rust/03_TM_Decoder/target/debug/build/nng-sys-43ddbed0e36856bc/out/build/src/CMakeFiles/Export/lib/cmake/nng/nng-targets.cmake")
  if("${CMAKE_INSTALL_CONFIG_NAME}" MATCHES "^([Dd][Ee][Bb][Uu][Gg])$")
    file(INSTALL DESTINATION "${CMAKE_INSTALL_PREFIX}/lib/cmake/nng" TYPE FILE FILES "/home/alberto/Projects/07_GSaaS_Rust/03_TM_Decoder/target/debug/build/nng-sys-43ddbed0e36856bc/out/build/src/CMakeFiles/Export/lib/cmake/nng/nng-targets-debug.cmake")
  endif()
endif()

if("x${CMAKE_INSTALL_COMPONENT}x" STREQUAL "xHeadersx" OR NOT CMAKE_INSTALL_COMPONENT)
  file(INSTALL DESTINATION "${CMAKE_INSTALL_PREFIX}/include" TYPE DIRECTORY FILES "/home/alberto/.cargo/registry/src/github.com-1ecc6299db9ec823/nng-sys-1.2.4-rc.1/nng/src/../include/nng")
endif()

if("x${CMAKE_INSTALL_COMPONENT}x" STREQUAL "xLibraryx" OR NOT CMAKE_INSTALL_COMPONENT)
  file(INSTALL DESTINATION "${CMAKE_INSTALL_PREFIX}/lib/cmake/nng" TYPE FILE FILES
    "/home/alberto/Projects/07_GSaaS_Rust/03_TM_Decoder/target/debug/build/nng-sys-43ddbed0e36856bc/out/build/src/nng-config.cmake"
    "/home/alberto/Projects/07_GSaaS_Rust/03_TM_Decoder/target/debug/build/nng-sys-43ddbed0e36856bc/out/build/src/nng-config-version.cmake"
    )
endif()

if(NOT CMAKE_INSTALL_LOCAL_ONLY)
  # Include the install script for each subdirectory.
  include("/home/alberto/Projects/07_GSaaS_Rust/03_TM_Decoder/target/debug/build/nng-sys-43ddbed0e36856bc/out/build/src/compat/nanomsg/cmake_install.cmake")
  include("/home/alberto/Projects/07_GSaaS_Rust/03_TM_Decoder/target/debug/build/nng-sys-43ddbed0e36856bc/out/build/src/protocol/bus0/cmake_install.cmake")
  include("/home/alberto/Projects/07_GSaaS_Rust/03_TM_Decoder/target/debug/build/nng-sys-43ddbed0e36856bc/out/build/src/protocol/pair0/cmake_install.cmake")
  include("/home/alberto/Projects/07_GSaaS_Rust/03_TM_Decoder/target/debug/build/nng-sys-43ddbed0e36856bc/out/build/src/protocol/pair1/cmake_install.cmake")
  include("/home/alberto/Projects/07_GSaaS_Rust/03_TM_Decoder/target/debug/build/nng-sys-43ddbed0e36856bc/out/build/src/protocol/pipeline0/cmake_install.cmake")
  include("/home/alberto/Projects/07_GSaaS_Rust/03_TM_Decoder/target/debug/build/nng-sys-43ddbed0e36856bc/out/build/src/protocol/pubsub0/cmake_install.cmake")
  include("/home/alberto/Projects/07_GSaaS_Rust/03_TM_Decoder/target/debug/build/nng-sys-43ddbed0e36856bc/out/build/src/protocol/reqrep0/cmake_install.cmake")
  include("/home/alberto/Projects/07_GSaaS_Rust/03_TM_Decoder/target/debug/build/nng-sys-43ddbed0e36856bc/out/build/src/protocol/survey0/cmake_install.cmake")
  include("/home/alberto/Projects/07_GSaaS_Rust/03_TM_Decoder/target/debug/build/nng-sys-43ddbed0e36856bc/out/build/src/transport/inproc/cmake_install.cmake")
  include("/home/alberto/Projects/07_GSaaS_Rust/03_TM_Decoder/target/debug/build/nng-sys-43ddbed0e36856bc/out/build/src/transport/ipc/cmake_install.cmake")
  include("/home/alberto/Projects/07_GSaaS_Rust/03_TM_Decoder/target/debug/build/nng-sys-43ddbed0e36856bc/out/build/src/transport/tcp/cmake_install.cmake")
  include("/home/alberto/Projects/07_GSaaS_Rust/03_TM_Decoder/target/debug/build/nng-sys-43ddbed0e36856bc/out/build/src/transport/tls/cmake_install.cmake")
  include("/home/alberto/Projects/07_GSaaS_Rust/03_TM_Decoder/target/debug/build/nng-sys-43ddbed0e36856bc/out/build/src/transport/ws/cmake_install.cmake")
  include("/home/alberto/Projects/07_GSaaS_Rust/03_TM_Decoder/target/debug/build/nng-sys-43ddbed0e36856bc/out/build/src/transport/zerotier/cmake_install.cmake")
  include("/home/alberto/Projects/07_GSaaS_Rust/03_TM_Decoder/target/debug/build/nng-sys-43ddbed0e36856bc/out/build/src/supplemental/base64/cmake_install.cmake")
  include("/home/alberto/Projects/07_GSaaS_Rust/03_TM_Decoder/target/debug/build/nng-sys-43ddbed0e36856bc/out/build/src/supplemental/http/cmake_install.cmake")
  include("/home/alberto/Projects/07_GSaaS_Rust/03_TM_Decoder/target/debug/build/nng-sys-43ddbed0e36856bc/out/build/src/supplemental/sha1/cmake_install.cmake")
  include("/home/alberto/Projects/07_GSaaS_Rust/03_TM_Decoder/target/debug/build/nng-sys-43ddbed0e36856bc/out/build/src/supplemental/tcp/cmake_install.cmake")
  include("/home/alberto/Projects/07_GSaaS_Rust/03_TM_Decoder/target/debug/build/nng-sys-43ddbed0e36856bc/out/build/src/supplemental/tls/cmake_install.cmake")
  include("/home/alberto/Projects/07_GSaaS_Rust/03_TM_Decoder/target/debug/build/nng-sys-43ddbed0e36856bc/out/build/src/supplemental/util/cmake_install.cmake")
  include("/home/alberto/Projects/07_GSaaS_Rust/03_TM_Decoder/target/debug/build/nng-sys-43ddbed0e36856bc/out/build/src/supplemental/websocket/cmake_install.cmake")

endif()

