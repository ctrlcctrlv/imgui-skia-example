# Dear ImGui, Skia, and Rust, all playing nice together

![](https://raw.githubusercontent.com/ctrlcctrlv/imgui-skia-example/master/screenshot.png)

**Turns out it's probably better to use Skulpin. Cf. [aclysma/skulpin#62](https://github.com/aclysma/skulpin/issues/62), [mfeq/Qglif#2](https://github.com/mfeq/Qglif/issues/2) and [jazzfool/reclutch#26](https://github.com/jazzfool/reclutch/issues/26). This repo will remain intact, but archived, if you're too stubborn to use Vulkan. :-)**

Much of the code here comes from the Reclutch project:

https://github.com/jazzfool/reclutch/blob/master/reclutch/examples/opengl/main.rs

Reclutch is however _not a dependency_, and Skia is _directly accessible_ via `skia-safe`!

(c) jazzfool <saveuselon@gmail.com> - Dual Apache 2 / MIT licensed.

I replaced the OpenGL cube example with Skia! :-)
