#!/bin/bash -e

if [ $# -lt 3 ]; then
    echo "Not enough command line arguments:"
    echo "Usage:"
    echo "     ./profile_target.sh [path_to_target] [target_name] [target_arguments]" 
    echo "Example:" 
    echo "     ./profile_target.sh src/image_viewer color_map_vis --texture_scale=.1" 
else
    echo "Your command line contains no arguments"
fi

BAZEL_PATH=$1
BAZEL_TARGET=$2
shift 2
BAZEL_PATH_COMPLETE="//$BAZEL_PATH:$BAZEL_TARGET"
PROFILER_OUTPUT=$(mktemp /tmp/$BAZEL_TARGET.XXXXXX)

bazel build $BAZEL_PATH_COMPLETE

EXECUTION_ROOT="$(bazel info bazel-bin)/$BAZEL_PATH/$BAZEL_TARGET.runfiles/__main__"
WORKSPACE_ROOT="$(bazel info workspace)"
RELATIVE_BINARY_PATH="$BAZEL_PATH/$BAZEL_TARGET"
cd $EXECUTION_ROOT
$RELATIVE_BINARY_PATH --profile_output=$PROFILER_OUTPUT $@

pprof -http=:8080 -source_path=$WORKSPACE_ROOT $RELATIVE_BINARY_PATH $PROFILER_OUTPUT
