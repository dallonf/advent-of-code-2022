(() => {
  const core = Deno.core;
  function createDrawCtx(pointer) {
    return {
      drawRectangle(x, y, width, height) {
        core.opSync("op_draw", pointer, "rectangle", [x, y, width, height]);
      },
    };
  }

  return {
    createDrawCtx,
  };
})();
