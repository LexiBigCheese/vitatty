sampler2D uniform the_texture;

float4 in v_color : COLOR0;
float2 in v_uv : TEXCOORD0;

float4 main() : COLOR {
    float4 t = tex2D(the_texture, v_uv);
return float4(v_color.xyz*t.xyz, t.r);
}
