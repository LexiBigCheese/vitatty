We'll need to recompile the shader any time the terminal width changes, fine.

To store all of the chars and their colors, Firstly:

We store the Lower and Upper bytes of each char separately, as to render a char with a different Upper byte, we need to change textures

Secondly, to store the different color types, fused with the lower byte, and assuming a 68 column x 26 row screen:


|Color Mode|Bytes used|Storage Type|Bytes per Row|Bytes per Screen|
|----------|----------|------------|-------------|----------------|
|16 color  |1         |u16         |136          |3536            |
|256 color |4         |u32         |272          |7072            |
|True color|8         |u64         |544          |14144           |

Honestly, 14KB isn't that big
