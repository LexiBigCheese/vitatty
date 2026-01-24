void main(
    float3 vPosition,
    float4 out gl_Position : POSITION
) {
gl_Position = float4(vPosition, 1.0);
}
