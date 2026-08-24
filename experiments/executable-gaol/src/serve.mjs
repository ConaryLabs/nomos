import { createServer } from "node:http";
import { readFileSync } from "node:fs";
import { extname, join, normalize } from "node:path";

const [experimentDir, outputDir] = process.argv.slice(2);
const roots = {
  "/": join(experimentDir, "viewer.html"),
  "/render-core.mjs": join(experimentDir, "src/render-core.mjs"),
  "/play-state.mjs": join(experimentDir, "src/play-state.mjs"),
  "/rendering-plan.json": join(outputDir, "rendering-plan.json"),
};
const types = { ".html": "text/html; charset=utf-8", ".mjs": "text/javascript; charset=utf-8", ".json": "application/json" };
const server = createServer((request, response) => {
  const path = roots[normalize(request.url.split("?")[0])];
  if (!path) { response.writeHead(404); response.end("not found\n"); return; }
  response.writeHead(200, { "content-type": types[extname(path)] ?? "application/octet-stream", "cache-control": "no-store" });
  response.end(readFileSync(path));
});
server.listen(4173, "127.0.0.1", () => console.log("Executable gaol: http://127.0.0.1:4173"));
