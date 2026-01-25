sampler2D uniform the_texture;
float4 in vUv : TEXCOORD0;

float4 main() : COLOR {
return float4(tex2D(the_texture, vUv.xy).xy, 0.5, 1.0);
}
