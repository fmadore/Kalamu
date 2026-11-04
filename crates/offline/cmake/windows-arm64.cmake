# Read by CMake builds for Windows on ARM64 (see `.cargo/config.toml`). CMake names this processor
# "ARM64", which Tesseract's CMakeLists does not recognise: its NEON code is then left out while
# its processor check, which sees `__aarch64__` under clang-cl, still calls it, and linking
# fails. Naming the system and processor here gives the lower-case name Tesseract (and ggml,
# for Whisper) recognise.
set(CMAKE_SYSTEM_NAME Windows)
set(CMAKE_SYSTEM_PROCESSOR arm64)
