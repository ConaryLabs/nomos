export class CdpFailure extends Error {}

export const connect = async (url, timeout = 20_000) => {
  const socket = new WebSocket(url);
  const pending = new Map();
  const handlers = new Map();
  let sequence = 1;
  await new Promise((resolve, reject) => {
    const timer = setTimeout(() => reject(new CdpFailure("DevTools connection timed out")), timeout);
    socket.addEventListener("open", () => { clearTimeout(timer); resolve(); }, { once: true });
    socket.addEventListener("error", () => { clearTimeout(timer); reject(new CdpFailure("DevTools connection failed")); }, { once: true });
  });
  socket.addEventListener("message", ({ data }) => {
    const message = JSON.parse(data);
    if (message.id !== undefined) {
      const waiter = pending.get(message.id);
      if (!waiter) return;
      pending.delete(message.id);
      clearTimeout(waiter.timer);
      if (message.error) waiter.reject(new CdpFailure(`${waiter.method}: ${message.error.message}`));
      else waiter.resolve(message.result);
      return;
    }
    for (const handler of handlers.get(message.method) ?? []) handler(message.params ?? {});
  });
  socket.addEventListener("close", () => {
    for (const waiter of pending.values()) waiter.reject(new CdpFailure("DevTools socket closed"));
    pending.clear();
  });
  const send = (method, params = {}) => new Promise((resolve, reject) => {
    const id = sequence;
    sequence += 1;
    const timer = setTimeout(() => {
      pending.delete(id);
      reject(new CdpFailure(`${method} timed out`));
    }, timeout);
    pending.set(id, { method, reject, resolve, timer });
    socket.send(JSON.stringify({ id, method, params }));
  });
  const on = (method, handler) => {
    if (!handlers.has(method)) handlers.set(method, []);
    handlers.get(method).push(handler);
  };
  const once = (method, wait = timeout) => new Promise((resolve, reject) => {
    const timer = setTimeout(() => reject(new CdpFailure(`${method} event timed out`)), wait);
    const handler = (params) => {
      clearTimeout(timer);
      const list = handlers.get(method) ?? [];
      handlers.set(method, list.filter((item) => item !== handler));
      resolve(params);
    };
    on(method, handler);
  });
  const evaluate = async (expression) => {
    const result = await send("Runtime.evaluate", { awaitPromise: true, expression, returnByValue: true });
    if (result.exceptionDetails) throw new CdpFailure(result.exceptionDetails.text);
    return result.result.value;
  };
  const close = () => new Promise((resolve, reject) => {
    if (socket.readyState === WebSocket.CLOSED) return resolve();
    const timer = setTimeout(() => reject(new CdpFailure("DevTools socket did not close")), 1_000);
    socket.addEventListener("close", () => { clearTimeout(timer); resolve(); }, { once: true });
    socket.close();
  });
  return Object.freeze({ close, evaluate, on, once, send });
};
