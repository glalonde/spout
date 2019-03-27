load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")

# Bazel toolchains
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

http_archive(
    name = "com_google_absl",
    strip_prefix = "abseil-cpp-master",
    urls = ["https://github.com/abseil/abseil-cpp/archive/master.tar.gz"],
)

http_archive(
     name = "com_google_googletest",
     urls = ["https://github.com/google/googletest/archive/master.zip"],
     strip_prefix = "googletest-master",
)

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
     urls = ["https://github.com/google/glog/archive/master.zip"],
     strip_prefix = "glog-master",
)

http_archive(
    name = "eigen",
    build_file = "@//third_party:eigen.BUILD",
    strip_prefix = "eigen-git-mirror-master",
    urls = ["https://github.com/eigenteam/eigen-git-mirror/archive/master.tar.gz"]
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
