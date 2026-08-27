import { createReadStream, lstatSync, realpathSync } from "node:fs";
import { createServer } from "node:http";
import { extname, join, resolve, sep } from "node:path";

const types = new Map([
  [".css", "text/css; charset=utf-8"],
  [".html", "text/html; charset=utf-8"],
  [".js", "text/javascript; charset=utf-8"],
  [".json", "application/json"],
  [".mjs", "text/javascript; charset=utf-8"],
]);

export const serve = async (directory) => {
  const root = realpathSync(resolve(directory));
  const requests = [];
  const sockets = new Set();
  const server = createServer((request, response) => {
    const url = new URL(request.url, "http://localhost");
    const relative = url.pathname === "/" ? "index.html" : decodeURIComponent(url.pathname.slice(1));
    const path = resolve(join(root, relative));
    if (!path.startsWith(`${root}${sep}`)) {
      response.writeHead(404).end();
      return;
    }
    try {
      const info = lstatSync(path);
      if (!info.isFile() || info.isSymbolicLink()) throw new Error("unsafe file");
      requests.push({ path: url.pathname, status: 200 });
      response.writeHead(200, { "cache-control": "no-store", "content-type": types.get(extname(path)) ?? "application/octet-stream" });
      createReadStream(path).pipe(response);
    } catch {
      requests.push({ path: url.pathname, status: 404 });
      response.writeHead(404).end();
    }
  });
  server.on("connection", (socket) => {
    sockets.add(socket);
    socket.on("close", () => sockets.delete(socket));
  });
  await new Promise((resolveListen, reject) => {
    server.once("error", reject);
    server.listen(0, "127.0.0.1", resolveListen);
  });
  const port = server.address().port;
  const close = async () => {
    const started = process.hrtime.bigint();
    for (const socket of sockets) socket.destroy();
    await new Promise((resolveClose) => server.close(resolveClose));
    return { duration_ms: Number(process.hrtime.bigint() - started) / 1_000_000, sockets_destroyed: sockets.size };
  };
  return Object.freeze({ close, port, requests });
};
