import { mkdir, cp } from "node:fs/promises";
await mkdir("dist", { recursive: true });
const result = await Bun.build({ entrypoints: ["site/app.js", "site/worker.js"], outdir: "dist", target: "browser" });
if (!result.success) throw new AggregateError(result.logs, "Browser build failed");
await Promise.all(["index.html", "style.css"].map(file => cp("site/" + file, "dist/" + file)));
