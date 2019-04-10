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
RUN add-apt-repository "ppa:oibaf/graphics-drivers"
RUN apt-get -q update \
    && apt-get install --fix-missing -qy \
        xvfb \
        x11-utils \
        mesa-common-dev \
        mesa-utils \
        freeglut3-dev \
        libglu1-mesa-dev;

# Copy our entrypoint into the container.
COPY ./entrypoint.sh /usr/local/bin/entrypoint.sh

# Setup our environment variables.
ENV XVFB_WHD="1920x1080x24"\
    DISPLAY=":99" \
    LIBGL_ALWAYS_SOFTWARE="1" \
    GALLIUM_DRIVER="llvmpipe" \
    LP_NO_RAST="false" \
    LP_DEBUG="" \
    LP_PERF="" \
    LP_NUM_THREADS="0"

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

# Set the default command.
ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]
