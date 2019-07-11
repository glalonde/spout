load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")

http_archive(
    name = "com_google_absl",
    strip_prefix = "abseil-cpp-master",
    urls = ["https://github.com/abseil/abseil-cpp/archive/master.tar.gz"],
)

http_archive(
    name = "com_google_googletest",
    strip_prefix = "googletest-master",
    urls = ["https://github.com/google/googletest/archive/master.zip"],
)

http_archive(
    name = "com_google_glog",
    build_file_content =
        """
licenses(['notice'])
load(':bazel/glog.bzl', 'glog_library')
glog_library(with_gflags=0)
""",
    strip_prefix = "glog-master",
    urls = ["https://github.com/google/glog/archive/master.zip"],
)

http_archive(
    name = "com_github_google_benchmark",
    strip_prefix = "benchmark-master",
    urls = ["https://github.com/google/benchmark/archive/master.zip"],
)

http_archive(
    name = "eigen",
    build_file = "@//third_party:eigen.BUILD",
    strip_prefix = "eigen-git-mirror-master",
    urls = ["https://github.com/eigenteam/eigen-git-mirror/archive/master.tar.gz"],
)

http_archive(
    name = "com_github_c42f_tinyformat",
    build_file = "@//third_party:tinyformat.BUILD",
    strip_prefix = "tinyformat-master",
    urls = ["https://github.com/c42f/tinyformat/archive/master.zip"],
)

http_archive(
    name = "com_github_nothings_stb",
    build_file = "@//third_party:stb.BUILD",
    strip_prefix = "stb-master",
    urls = ["https://github.com/nothings/stb/archive/master.zip"],
)

http_archive(
    name = "com_github_gtruc_glm",
    build_file = "@//third_party:glm.BUILD",
    strip_prefix = "glm-master",
    urls = ["https://github.com/g-truc/glm/archive/master.zip"],
)

http_archive(
    name = "com_github_vulkan_memory_allocator",
    build_file = "@//third_party:vulkan_memory_allocator.BUILD",
    strip_prefix = "VulkanMemoryAllocator-master",
    urls = ["https://github.com/GPUOpen-LibrariesAndSDKs/VulkanMemoryAllocator/archive/master.zip"],
)

# Change master to the git tag you want.
http_archive(
    name = "com_grail_bazel_toolchain",
    strip_prefix = "bazel-toolchain-master",
    urls = ["https://github.com/grailbio/bazel-toolchain/archive/master.tar.gz"],
)

load("@com_grail_bazel_toolchain//toolchain:rules.bzl", "llvm_toolchain")

llvm_toolchain(
    name = "llvm_toolchain",
    llvm_version = "8.0.0",
)

load("@llvm_toolchain//:toolchains.bzl", "llvm_register_toolchains")

llvm_register_toolchains()
