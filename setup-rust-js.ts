import { mkdir, symlink, readlink } from "node:fs/promises";
import { resolve } from "node:path";

const checkout = resolve(process.env.RUST_JS_DIR ?? "../rust-js");
if (!await Bun.file(resolve(checkout, "std/package.json")).exists()) {
  throw new Error("Set RUST_JS_DIR to the rust-js checkout documented in the README.");
}
await mkdir("node_modules/@rust-js", { recursive: true });
for (const [target, link] of [[checkout, ".rust-js"], [resolve(checkout, "std"), "node_modules/@rust-js/std"]]) {
  if (await readlink(link).catch(() => undefined) !== target) {
    await symlink(target, link, "dir");
  }
}
