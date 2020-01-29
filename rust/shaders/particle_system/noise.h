
float noise1d(float x) {
    float i = floor(x);
    float f = fract(x);
    return mix(hash11(i), hash11(i + 1.0), smoothstep(0.,1.,f));
}