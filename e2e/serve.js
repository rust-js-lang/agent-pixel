// Keep the test server's data/document.json outside the developer's workspace.
import { mkdtemp, rm, symlink } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";

const dir = await mkdtemp(join(tmpdir(), "agent-pixel-e2e-"));
try {
  await symlink(resolve("dist"), join(dir, "dist"), "dir");
  const server = Bun.spawn([resolve("server/target/debug/agent-pixel-server")], {
    cwd: dir, stdout: "inherit", stderr: "inherit"
  });
  for (const signal of ["SIGTERM", "SIGINT"]) process.on(signal, () => server.kill());
  process.exitCode = await server.exited;
} finally {
  await rm(dir, { recursive: true, force: true });
}
