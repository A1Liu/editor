# Ted
Text editor using WebGL and Rust/Wasm. Very much WIP.

## Feature Stuffs
- [ ] https://rustwasm.github.io/wasm-bindgen/examples/request-animation-frame.html
- [ ] https://github.com/cmuratori/refterm
- [ ] https://webgl2fundamentals.org/webgl/lessons/webgl-qna-how-to-use-the-stencil-buffer.html
- [ ] https://medium.com/@evanwallace/easy-scalable-text-rendering-on-the-gpu-c3f4d782c5ac
- [ ] https://docs.rs/winit/latest/winit/
- [ ] https://developer.mozilla.org/en-US/docs/Web/API/WebGL_API/WebGL_best_practices
- [x] https://docs.rs/ttf-parser/latest/ttf_parser/
- [x] https://github.com/raphlinus/font-rs
- [x] https://webgl2fundamentals.org/webgl/lessons/webgl-how-it-works.html

## Size Opt Stuffs
- [ ] https://github.com/rustwasm/wasm-pack/issues/737
- [ ] https://github.com/johnthagen/min-sized-rust
- [ ] https://www.skypack.dev/view/esbuild-plugin-wasm-pack

## Next Up
- Getting input and inserting into the editor
- Deleting text (BTree ig)
- Cursor stuff, probably some more architectural changes in the process
- Make text less ugly
- Background color stuff
- Line numbers
- Select text
- Window sizing and whatnot

## Mid-term
- Find, Find and replace
- Multiple canvases on the web
- Syntax highlighting; probably just make something super duper simple
- Graphics/shader stuff
- Persist data and whatnot

## Far in the Future
- Customization?
- Cursor ref-counting or whatever works to get the behavior that Google docs has
- Client-server architecture so that we can have nice things
- Cross platform stuffs
- Abstract away graphics stuff with cute macros and stuff


## Architecture
- Windows for rendering, tightly connected to graphics for each system
  - Windows have views into files that store stuff like where you cursor is
  - Views do word wrapping by creating a string and inserting newlines, windows
    render as if there is no word wrapping, and cut off stuff when necessary
- Files are managed globally, you call functions to modify the files and those
  functions might end up being IPC calls or whatever, to support multiple windows
- Editor uses a command system for editor mutations. Like, literally, just use
  an enum to dispatch events. That way its easy to swap out an implementation
  with another
- Window doesn't do anything by default. It only dispatches events, and when certain
  events come in, it runs those events
