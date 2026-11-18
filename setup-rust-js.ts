import { mkdir, readlink, symlink, unlink } from "node:fs/promises";
import { resolve } from "node:path";

const checkout = resolve(process.env.RUST_JS_DIR ?? "../rust-js");
if (!await Bun.file(resolve(checkout, "std/package.json")).exists()) {
  throw new Error("Set RUST_JS_DIR to the rust-js checkout documented in the README.");
}

await mkdir("node_modules/@rust-js", { recursive: true });
for (const [target, link] of [[checkout, ".rust-js"], [resolve(checkout, "std"), "node_modules/@rust-js/std"]]) {
  const previous = await readlink(link).catch(error => {
    if (error.code === "ENOENT") return undefined;
    throw error;
  });
  if (previous === target) continue;
  if (previous !== undefined) await unlink(link);
  await symlink(target, link, "dir");
}
