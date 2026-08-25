// A localhost static server for the staged artifact.
//
// `node:http` and nothing else. It serves one directory, refuses anything that
// climbs out of it, and records every request so the receipt can say exactly
// what the page asked for.

import { createServer } from "node:http";
import { readFileSync, statSync } from "node:fs";
import { extname, resolve, sep } from "node:path";

const TYPES = {
  ".html": "text/html; charset=utf-8",
  ".mjs": "text/javascript; charset=utf-8",
  ".js": "text/javascript; charset=utf-8",
  ".json": "application/json",
  ".png": "image/png",
  ".css": "text/css; charset=utf-8",
};

/// Serves `root` on a loopback port the operating system chooses.
export async function serve(root) {
  const base = `${resolve(root)}${sep}`;
  const requests = [];
  const server = createServer((request, response) => {
    const urlPath = decodeURIComponent(request.url.split("?")[0]);
    const path = resolve(root, urlPath === "/" ? "index.html" : urlPath.replace(/^\/+/, ""));
    const inside = path.startsWith(base) || path === resolve(root);
    const file = inside ? statSync(path, { throwIfNoEntry: false }) : null;
    if (!file?.isFile()) {
      requests.push({ path: urlPath, status: 404 });
      response.writeHead(404);
      response.end("not found\n");
      return;
    }
    requests.push({ path: urlPath, status: 200 });
    response.writeHead(200, {
      "content-type": TYPES[extname(path)] ?? "application/octet-stream",
      "cache-control": "no-store",
    });
    response.end(readFileSync(path));
  });

  await new Promise((done) => server.listen(0, "127.0.0.1", done));
  const { port } = server.address();
  // Bound to the loopback address, addressed by the one name the lane's
  // host-resolver rule excludes. Chrome maps every other name to NOTFOUND, and
  // it applies that to an address literal too, so `localhost` is what the page
  // is served as.
  return {
    origin: `http://localhost:${port}`,
    address: "127.0.0.1",
    port,
    requests,
    close: () => new Promise((done) => server.close(done)),
  };
}
