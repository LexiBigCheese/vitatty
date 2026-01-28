Okay so looks like how we were previously handling rendering wasn't the best option considering bold, italics, underline, flashing, etc are a thing.

Instead, we should be rendering multiple layers, making use of blending and such.

It should look like:

```rust
fn create_render(parser: &vt100::Parser) -> Render {..} //or perhaps reuse the same render?
impl Render {
    fn draw(&self) {
        self.background();
        self.foreground();
    }
    fn background(&self) {..}
    fn foreground(&self) {
        
    }
}
```

then, `background` should be rendering just the bg tile grid,
then, each style... uhh...

i'm just gonna write some shaders and figure it out from there actually
