const pointer = Deno.core.opSync('into_rust_obj', -5);
console.log(pointer);
const output = Deno.core.opSync('op_unwrap_rust_pointer', pointer);
console.log('unwrapped:', output);

export function draw() {
  return "hot reloading is cool";
}
