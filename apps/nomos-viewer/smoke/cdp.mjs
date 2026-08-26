// A Chrome DevTools Protocol client, over Node's global WebSocket.
//
// No dependency, and no `Target` domain: the lane attaches straight to the page
// target's own endpoint, so every message is a plain request or event on one
// socket. `docs/review/nomos-viewer.md` section 5.2 lists what it uses.

export class CdpError extends Error {
  constructor(message) {
    super(message);
    this.name = "CdpError";
  }
}

/// Connects to one DevTools endpoint.
export async function connect(url, { timeout = 20_000 } = {}) {
  if (typeof WebSocket !== "function") {
    throw new CdpError(
      "this Node has no global WebSocket; the smoke lane needs Node 22 or newer",
    );
  }
  const socket = new WebSocket(url);
  const pending = new Map();
  const handlers = new Map();
  let nextId = 1;
  let closed = null;

  await new Promise((resolve, reject) => {
    const timer = setTimeout(() => reject(new CdpError(`no DevTools connection within ${timeout}ms`)), timeout);
    socket.addEventListener("open", () => {
      clearTimeout(timer);
      resolve();
    });
    socket.addEventListener("error", () => {
      clearTimeout(timer);
      reject(new CdpError(`could not connect to ${url}`));
    });
  });

  socket.addEventListener("close", () => {
    closed = new CdpError("the DevTools connection closed");
    for (const { reject, timer } of pending.values()) {
      clearTimeout(timer);
      reject(closed);
    }
    pending.clear();
  });

  socket.addEventListener("message", (event) => {
    const message = JSON.parse(event.data);
    if (message.id !== undefined) {
      const waiter = pending.get(message.id);
      if (!waiter) return;
      pending.delete(message.id);
      clearTimeout(waiter.timer);
      if (message.error) waiter.reject(new CdpError(`${waiter.method}: ${message.error.message}`));
      else waiter.resolve(message.result);
      return;
    }
    for (const handler of handlers.get(message.method) ?? []) handler(message.params ?? {});
    for (const handler of handlers.get("*") ?? []) handler(message.params ?? {}, message.method);
  });

  const send = (method, params = {}) => {
    if (closed) return Promise.reject(closed);
    const id = nextId++;
    return new Promise((resolve, reject) => {
      const timer = setTimeout(() => {
        if (!pending.has(id)) return;
        pending.delete(id);
        reject(new CdpError(`${method} did not answer within ${timeout}ms`));
      }, timeout);
      pending.set(id, { resolve, reject, method, timer });
      try {
        socket.send(JSON.stringify({ id, method, params }));
      } catch (error) {
        clearTimeout(timer);
        pending.delete(id);
        reject(error);
      }
    });
  };

  const on = (event, handler) => {
    if (!handlers.has(event)) handlers.set(event, []);
    handlers.get(event).push(handler);
  };

  /// Resolves when `event` arrives, or rejects on timeout.
  const once = (event, wait = timeout) =>
    new Promise((resolve, reject) => {
      const timer = setTimeout(() => reject(new CdpError(`${event} did not arrive within ${wait}ms`)), wait);
      on(event, (params) => {
        clearTimeout(timer);
        resolve(params);
      });
    });

  /// Evaluates an expression in the page and returns its value.
  const evaluate = async (expression) => {
    const result = await send("Runtime.evaluate", {
      expression,
      returnByValue: true,
      awaitPromise: true,
    });
    if (result.exceptionDetails) {
      throw new CdpError(`evaluate failed: ${result.exceptionDetails.text}`);
    }
    return result.result.value;
  };

  /// Polls `read` until `accept` is happy, or gives up.
  const until = async (read, accept, { wait = timeout, every = 25 } = {}) => {
    const deadline = Date.now() + wait;
    let last;
    for (;;) {
      last = await read();
      if (accept(last)) return last;
      if (Date.now() > deadline) {
        throw new CdpError(`condition not met within ${wait}ms; last value ${JSON.stringify(last)}`);
      }
      await new Promise((resolve) => setTimeout(resolve, every));
    }
  };

  // `close()` starts a handshake, and the socket stays an open handle until it
  // finishes. Make a missing close event a recorded shutdown failure; the
  // caller will still kill Chrome and run the remaining cleanup steps.
  const close = () =>
    new Promise((resolve, reject) => {
      if (socket.readyState === WebSocket.CLOSED) {
        resolve({ ready_state: "closed" });
        return;
      }
      const onClose = () => {
        clearTimeout(timer);
        resolve({ ready_state: "closed" });
      };
      const timer = setTimeout(() => {
        socket.removeEventListener("close", onClose);
        reject(new CdpError("the DevTools connection did not close within 1000ms"));
      }, 1_000);
      socket.addEventListener("close", onClose, { once: true });
      if (socket.readyState === WebSocket.OPEN) socket.close();
    });

  return { send, on, once, evaluate, until, close };
}
