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

# Only required for glog, everything else uses absl flags
http_archive(
    name = "com_github_gflags_gflags",
    strip_prefix = "gflags-2.2.2",
    urls = [
        "https://mirror.bazel.build/github.com/gflags/gflags/archive/v2.2.2.tar.gz",
        "https://github.com/gflags/gflags/archive/v2.2.2.tar.gz",
    ],
)

http_archive(
    name = "com_google_glog",
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
