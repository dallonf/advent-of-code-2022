import { rand } from "./util.js";

const pointer = Deno.core.opSync("into_rust_obj", -5);
console.log(pointer);
const output = Deno.core.opSync("op_unwrap_rust_pointer", pointer);
console.log("unwrapped:", output);

export function draw(ctx) {
  for (let index = 0; index < 100; index++) {
    ctx.drawRectangle(100, 100, 150, 150);
  }

  return rand().toString();
}
