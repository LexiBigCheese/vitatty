Given that all we're rendering is a grid of tiles,
passing the full float vertex position to each vertex seems like a waste of bandwidth,
as well as the uv.

instead, we can pass X and Y as single bytes, UV as a single byte (allowing for 256 different chars on a single texture), and Color as... 6 bytes.

that's 9 bytes total hang on

okay so like, most of the time the color is going to be limited to a 256 color palette, so...

u8 x, u8 y, u8 uv, u8 fg, u8 bg

five bytes, and we can switch to a different vertex shader to draw full spectrum colors, and in fact for normal 16 color operations we can shrink our vertex data down to
x,y,uv,col (col is ffffbbbb)

we can use the vertex index's least significant two bits to modify the calculated UV values

wait hang on if we just use an index buffer and normal triangle mode then we don't need x or y, those can be calculated too


In any case:

a 40ish column terminal is going to need characters 13 pixels wide (which actually gives us enough space for 41 columns)

looks like we'll need to recompile the shaders any time we want to change the width of the terminal, that's fine!
