FROM ubuntu:18.04

# Install stuff to allow installing stuff
RUN apt-get -q update \
    && apt-get install --fix-missing -qy \
        apt-transport-https \
        apt-utils \
        curl \
        software-properties-common \
        sudo

# Clang stuff
RUN curl -sS https://apt.llvm.org/llvm-snapshot.gpg.key|sudo apt-key add -
RUN apt-add-repository "deb [arch=amd64] http://apt.llvm.org/bionic/ llvm-toolchain-bionic-8 main"
RUN apt-get -q update \
    && apt-get install --fix-missing -qy \
        clang-8 \
        clang-tools-8 \
        clang-8-doc \
        libclang-common-8-dev \
        libclang-8-dev \
        libclang1-8 \
        clang-format-8 \
        python-clang-8 \
        lldb-8 \
        lld-8 \
        libc++-8-dev \
        libc++abi-8-dev;

# Mesa software rendering stuff
RUN apt-get -q update \
    && apt-get install --fix-missing -qy \
        mesa-common-dev \
        mesa-utils \
        freeglut3-dev \
        libglu1-mesa-dev;

# Bazel
RUN apt-get -q update \
    && apt-get install --fix-missing -qy \
        openjdk-8-jdk \
        openjdk-8-jre \
        bash-completion

RUN curl -sS https://bazel.build/bazel-release.pub.gpg | apt-key add -
RUN apt-add-repository "deb [arch=amd64] http://storage.googleapis.com/bazel-apt stable jdk1.8"

RUN apt-get -q update \
    && apt-get install --fix-missing -qy \
      bazel

# Build deps
RUN apt-get -q update \
    && apt-get install --fix-missing -qy \
        git \
        vim \
        libsdl2-dev \
        libgoogle-perftools-dev;
