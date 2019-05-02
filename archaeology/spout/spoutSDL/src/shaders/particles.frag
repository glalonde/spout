sampler1D Palette;                     // A palette of 256 colors
uniform sampler2D IndexedColorTexture; // A texture using indexed color
varying vec2 TexCoord0;                // UVs

void main() {
    // Pick up a color index
    vec4 index = texture2D(IndexedColorTexture, TexCoord0);
    // Retrieve the actual color from the palette
    vec4 texel = texture1D(Palette, myindex.x);
    gl_FragColor = texel;   //Output the color
}
