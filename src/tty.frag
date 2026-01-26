float4 main(
    sampler2D uniform the_texture,
    float4 in fg_color : COLOR0,
float4 in bg_color : COLOR1,
float4 in tex_coord : TEXCOORD0
) {
// return lerp(bg_color, fg_color, tex2D(the_texture, tex_coord.xy).x);
return fg_color;
// return float4(0.0, 1.0, 0.7, 1.0);
}
