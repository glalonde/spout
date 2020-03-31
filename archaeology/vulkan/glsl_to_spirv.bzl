def glsl_to_spirv(name, main, srcs = []):
    """ Convert a shader file to spir-v
    """
    native.genrule(
        name = name,
        srcs = [main] + srcs,
        outs = [main + ".spv"],
        cmd = "glslc -c $(location %s) -I . -o $@" % main,
    )
