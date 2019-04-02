/**
 * 8x8 monochrome bitmap fonts for rendering
 * Author: Daniel Hepper <daniel@hepper.net>
 *
 * License: Public Domain
 *
 * Based on:
 * // Summary: font8x8.h
 * // 8x8 monochrome bitmap fonts for rendering
 * //
 * // Author:
 * //     Marcel Sondaar
 * //     International Business Machines (public domain VGA fonts)
 * //
 * // License:
 * //     Public Domain
 *
 * Fetched from:
 *http://dimensionalrift.homelinux.net/combuster/mos3/?p=viewsource&file=/modules/gfx/font8_8.asm
 **/

// Contains an 8x8 font map for unicode points U+3040 - U+309F (Hiragana)
// Constant: font8x8_3040
uint8_t font8x8_hiragana[96][8] = {
    {0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00},  // U+3040
    {0x04, 0x3F, 0x04, 0x3C, 0x56, 0x4D, 0x26, 0x00},  // U+3041 (Hiragana a)
    {0x04, 0x3F, 0x04, 0x3C, 0x56, 0x4D, 0x26, 0x00},  // U+3042 (Hiragana A)
    {0x00, 0x00, 0x00, 0x11, 0x21, 0x25, 0x02, 0x00},  // U+3043 (Hiragana i)
    {0x00, 0x01, 0x11, 0x21, 0x21, 0x25, 0x02, 0x00},  // U+3044 (Hiragana I)
    {0x00, 0x1C, 0x00, 0x1C, 0x22, 0x20, 0x18, 0x00},  // U+3045 (Hiragana u)
    {0x3C, 0x00, 0x3C, 0x42, 0x40, 0x20, 0x18, 0x00},  // U+3046 (Hiragana U)
    {0x1C, 0x00, 0x3E, 0x10, 0x38, 0x24, 0x62, 0x00},  // U+3047 (Hiragana e)
    {0x1C, 0x00, 0x3E, 0x10, 0x38, 0x24, 0x62, 0x00},  // U+3048 (Hiragana E)
    {0x24, 0x4F, 0x04, 0x3C, 0x46, 0x45, 0x22, 0x00},  // U+3049 (Hiragana o)
    {0x24, 0x4F, 0x04, 0x3C, 0x46, 0x45, 0x22, 0x00},  // U+304A (Hiragana O)
    {0x04, 0x24, 0x4F, 0x54, 0x52, 0x12, 0x09, 0x00},  // U+304B (Hiragana KA)
    {0x44, 0x24, 0x0F, 0x54, 0x52, 0x52, 0x09, 0x00},  // U+304C (Hiragana GA)
    {0x08, 0x1F, 0x08, 0x3F, 0x1C, 0x02, 0x3C, 0x00},  // U+304D (Hiragana KI)
    {0x44, 0x2F, 0x04, 0x1F, 0x0E, 0x01, 0x1E, 0x00},  // U+304E (Hiragana GI)
    {0x10, 0x08, 0x04, 0x02, 0x04, 0x08, 0x10, 0x00},  // U+304F (Hiragana KU)
    {0x28, 0x44, 0x12, 0x21, 0x02, 0x04, 0x08, 0x00},  // U+3050 (Hiragana GU)
    {0x00, 0x22, 0x79, 0x21, 0x21, 0x22, 0x10, 0x00},  // U+3051 (Hiragana KE)
    {0x40, 0x22, 0x11, 0x3D, 0x11, 0x12, 0x08, 0x00},  // U+3052 (Hiragana GE)
    {0x00, 0x00, 0x3C, 0x00, 0x02, 0x02, 0x3C, 0x00},  // U+3053 (Hiragana KO)
    {0x20, 0x40, 0x16, 0x20, 0x01, 0x01, 0x0E, 0x00},  // U+3054 (Hiragana GO)
    {0x10, 0x7E, 0x10, 0x3C, 0x02, 0x02, 0x1C, 0x00},  // U+3055 (Hiragana SA)
    {0x24, 0x4F, 0x14, 0x2E, 0x01, 0x01, 0x0E, 0x00},  // U+3056 (Hiragana ZA)
    {0x00, 0x02, 0x02, 0x02, 0x42, 0x22, 0x1C, 0x00},  // U+3057 (Hiragana SI)
    {0x20, 0x42, 0x12, 0x22, 0x02, 0x22, 0x1C, 0x00},  // U+3058 (Hiragana ZI)
    {0x10, 0x7E, 0x18, 0x14, 0x18, 0x10, 0x0C, 0x00},  // U+3059 (Hiragana SU)
    {0x44, 0x2F, 0x06, 0x05, 0x06, 0x04, 0x03, 0x00},  // U+305A (Hiragana ZU)
    {0x20, 0x72, 0x2F, 0x22, 0x1A, 0x02, 0x1C, 0x00},  // U+305B (Hiragana SE)
    {0x80, 0x50, 0x3A, 0x17, 0x1A, 0x02, 0x1C, 0x00},  // U+305C (Hiragana ZE)
    {0x1E, 0x08, 0x04, 0x7F, 0x08, 0x04, 0x38, 0x00},  // U+305D (Hiragana SO)
    {0x4F, 0x24, 0x02, 0x7F, 0x08, 0x04, 0x38, 0x00},  // U+305E (Hiragana ZO)
    {0x02, 0x0F, 0x02, 0x72, 0x02, 0x09, 0x71, 0x00},  // U+305F (Hiragana TA)
    {0x42, 0x2F, 0x02, 0x72, 0x02, 0x09, 0x71, 0x00},  // U+3060 (Hiragana DA)
    {0x08, 0x7E, 0x08, 0x3C, 0x40, 0x40, 0x38, 0x00},  // U+3061 (Hiragana TI)
    {0x44, 0x2F, 0x04, 0x1E, 0x20, 0x20, 0x1C, 0x00},  // U+3062 (Hiragana DI)
    {0x00, 0x00, 0x00, 0x1C, 0x22, 0x20, 0x1C, 0x00},  // U+3063 (Hiragana tu)
    {0x00, 0x1C, 0x22, 0x41, 0x40, 0x20, 0x1C, 0x00},  // U+3064 (Hiragana TU)
    {0x40, 0x20, 0x1E, 0x21, 0x20, 0x20, 0x1C, 0x00},  // U+3065 (Hiragana DU)
    {0x00, 0x3E, 0x08, 0x04, 0x04, 0x04, 0x38, 0x00},  // U+3066 (Hiragana TE)
    {0x00, 0x3E, 0x48, 0x24, 0x04, 0x04, 0x38, 0x00},  // U+3067 (Hiragana DE)
    {0x04, 0x04, 0x08, 0x3C, 0x02, 0x02, 0x3C, 0x00},  // U+3068 (Hiragana TO)
    {0x44, 0x24, 0x08, 0x3C, 0x02, 0x02, 0x3C, 0x00},  // U+3069 (Hiragana DO)
    {0x32, 0x02, 0x27, 0x22, 0x72, 0x29, 0x11, 0x00},  // U+306A (Hiragana NA)
    {0x00, 0x02, 0x7A, 0x02, 0x0A, 0x72, 0x02, 0x00},  // U+306B (Hiragana NI)
    {0x08, 0x09, 0x3E, 0x4B, 0x65, 0x55, 0x22, 0x00},  // U+306C (Hiragana NU)
    {0x04, 0x07, 0x34, 0x4C, 0x66, 0x54, 0x24, 0x00},  // U+306D (Hiragana NE)
    {0x00, 0x00, 0x3C, 0x4A, 0x49, 0x45, 0x22, 0x00},  // U+306E (Hiragana NO)
    {0x00, 0x22, 0x7A, 0x22, 0x72, 0x2A, 0x12, 0x00},  // U+306F (Hiragana HA)
    {0x80, 0x51, 0x1D, 0x11, 0x39, 0x15, 0x09, 0x00},  // U+3070 (Hiragana BA)
    {0x40, 0xB1, 0x5D, 0x11, 0x39, 0x15, 0x09, 0x00},  // U+3071 (Hiragana PA)
    {0x00, 0x00, 0x13, 0x32, 0x51, 0x11, 0x0E, 0x00},  // U+3072 (Hiragana HI)
    {0x40, 0x20, 0x03, 0x32, 0x51, 0x11, 0x0E, 0x00},  // U+3073 (Hiragana BI)
    {0x40, 0xA0, 0x43, 0x32, 0x51, 0x11, 0x0E, 0x00},  // U+3074 (Hiragana PI)
    {0x1C, 0x00, 0x08, 0x2A, 0x49, 0x10, 0x0C, 0x00},  // U+3075 (Hiragana HU)
    {0x4C, 0x20, 0x08, 0x2A, 0x49, 0x10, 0x0C, 0x00},  // U+3076 (Hiragana BU)
    {0x4C, 0xA0, 0x48, 0x0A, 0x29, 0x48, 0x0C, 0x00},  // U+3077 (Hiragana PU)
    {0x00, 0x00, 0x04, 0x0A, 0x11, 0x20, 0x40, 0x00},  // U+3078 (Hiragana HE)
    {0x20, 0x40, 0x14, 0x2A, 0x11, 0x20, 0x40, 0x00},  // U+3079 (Hiragana BE)
    {0x20, 0x50, 0x24, 0x0A, 0x11, 0x20, 0x40, 0x00},  // U+307A (Hiragana PE)
    {0x7D, 0x11, 0x7D, 0x11, 0x39, 0x55, 0x09, 0x00},  // U+307B (Hiragana HO)
    {0x9D, 0x51, 0x1D, 0x11, 0x39, 0x55, 0x09, 0x00},  // U+307C (Hiragana BO)
    {0x5D, 0xB1, 0x5D, 0x11, 0x39, 0x55, 0x09, 0x00},  // U+307D (Hiragana PO)
    {0x7E, 0x08, 0x3E, 0x08, 0x1C, 0x2A, 0x04, 0x00},  // U+307E (Hiragana MA)
    {0x00, 0x07, 0x24, 0x24, 0x7E, 0x25, 0x12, 0x00},  // U+307F (Hiragana MI)
    {0x04, 0x0F, 0x64, 0x06, 0x05, 0x26, 0x3C, 0x00},  // U+3080 (Hiragana MU)
    {0x00, 0x09, 0x3D, 0x4A, 0x4B, 0x45, 0x2A, 0x00},  // U+3081 (Hiragana ME)
    {0x02, 0x0F, 0x02, 0x0F, 0x62, 0x42, 0x3C, 0x00},  // U+3082 (Hiragana MO)
    {0x00, 0x00, 0x12, 0x1F, 0x22, 0x12, 0x04, 0x00},  // U+3083 (Hiragana ya)
    {0x00, 0x12, 0x3F, 0x42, 0x42, 0x34, 0x04, 0x00},  // U+3084 (Hiragana YA)
    {0x00, 0x00, 0x11, 0x3D, 0x53, 0x39, 0x11, 0x00},  // U+3085 (Hiragana yu)
    {0x00, 0x11, 0x3D, 0x53, 0x51, 0x39, 0x11, 0x00},  // U+3086 (Hiragana YU)
    {0x00, 0x08, 0x38, 0x08, 0x1C, 0x2A, 0x04, 0x00},  // U+3087 (Hiragana yo)
    {0x08, 0x08, 0x38, 0x08, 0x1C, 0x2A, 0x04, 0x00},  // U+3088 (Hiragana YO)
    {0x1E, 0x00, 0x02, 0x3A, 0x46, 0x42, 0x30, 0x00},  // U+3089 (Hiragana RA)
    {0x00, 0x20, 0x22, 0x22, 0x2A, 0x24, 0x10, 0x00},  // U+308A (Hiragana RI)
    {0x1F, 0x08, 0x3C, 0x42, 0x49, 0x54, 0x38, 0x00},  // U+308B (Hiragana RU)
    {0x04, 0x07, 0x04, 0x0C, 0x16, 0x55, 0x24, 0x00},  // U+308C (Hiragana RE)
    {0x3F, 0x10, 0x08, 0x3C, 0x42, 0x41, 0x30, 0x00},  // U+308D (Hiragana RO)
    {0x00, 0x00, 0x08, 0x0E, 0x38, 0x4C, 0x2A, 0x00},  // U+308E (Hiragana wa)
    {0x04, 0x07, 0x04, 0x3C, 0x46, 0x45, 0x24, 0x00},  // U+308F (Hiragana WA)
    {0x0E, 0x08, 0x3C, 0x4A, 0x69, 0x55, 0x32, 0x00},  // U+3090 (Hiragana WI)
    {0x06, 0x3C, 0x42, 0x39, 0x04, 0x36, 0x49, 0x00},  // U+3091 (Hiragana WE)
    {0x04, 0x0F, 0x04, 0x6E, 0x11, 0x08, 0x70, 0x00},  // U+3092 (Hiragana WO)
    {0x08, 0x08, 0x04, 0x0C, 0x56, 0x52, 0x21, 0x00},  // U+3093 (Hiragana N)
    {0x40, 0x2E, 0x00, 0x3C, 0x42, 0x40, 0x38, 0x00},  // U+3094 (Hiragana VU)
    {0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00},  // U+3095
    {0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00},  // U+3096
    {0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00},  // U+3097
    {0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00},  // U+3098
    {0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
     0x00},  // U+3099 (voiced combinator mark)
    {0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
     0x00},  // U+309A (semivoiced combinator mark)
    {0x40, 0x80, 0x20, 0x40, 0x00, 0x00, 0x00,
     0x00},  // U+309B (Hiragana voiced mark)
    {0x40, 0xA0, 0x40, 0x00, 0x00, 0x00, 0x00,
     0x00},  // U+309C (Hiragana semivoiced mark)
    {0x00, 0x00, 0x08, 0x08, 0x10, 0x30, 0x0C,
     0x00},  // U+309D (Hiragana iteration mark)
    {0x20, 0x40, 0x14, 0x24, 0x08, 0x18, 0x06,
     0x00},  // U+309E (Hiragana voiced iteration mark)
    {0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00}  // U+309F
};
