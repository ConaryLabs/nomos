import { createServer } from "node:http";
import { readFileSync, statSync } from "node:fs";
import { extname, resolve, sep } from "node:path";

const [siteDir] = process.argv.slice(2);
if (!siteDir) throw new Error("usage: serve.mjs <site-dir>");
const root = `${resolve(siteDir)}${sep}`;
const types = { ".html": "text/html; charset=utf-8", ".mjs": "text/javascript; charset=utf-8", ".json": "application/json", ".png": "image/png" };
const server = createServer((request, response) => {
  const urlPath = decodeURIComponent(request.url.split("?")[0]);
  const path = resolve(siteDir, urlPath === "/" ? "index.html" : urlPath.slice(1));
  if (!path.startsWith(root) || !statSync(path, { throwIfNoEntry: false })?.isFile()) {
    response.writeHead(404); response.end("not found\n"); return;
  }
  response.writeHead(200, { "content-type": types[extname(path)] ?? "application/octet-stream", "cache-control": "no-store" });
  response.end(readFileSync(path));
});
server.listen(4173, "127.0.0.1", () => console.log("Executable gaol: http://127.0.0.1:4173"));
