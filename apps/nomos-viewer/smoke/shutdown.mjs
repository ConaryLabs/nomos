// Process-lifecycle helpers for the browser smoke lane.
//
// Kept separate from `smoke.mjs` so the backstop can be proved with a child
// process that deliberately leaves an event-loop handle open.

export const HARD_DEADLINE_DIAGNOSTIC = "NOMOS_VIEWER_SMOKE FAIL HARD_DEADLINE";

const errorText = (error) => error?.message ?? String(error);

/// Records one cleanup action without preventing later actions from running.
/// The returned error lets the caller fail the lane after every handle has had
/// its chance to close and the receipt has been written.
export async function recordShutdownStep(shutdown, name, action) {
  if (shutdown.started_unix_ms === undefined) shutdown.started_unix_ms = Date.now();
  const started = performance.now();
  const step = { name, outcome: "fail", duration_ms: 0 };
  let failure = null;
  try {
    const detail = await action();
    step.outcome = "pass";
    if (detail !== undefined) step.detail = detail;
  } catch (error) {
    failure = error;
    step.error = errorText(error);
  } finally {
    step.duration_ms = Math.ceil(performance.now() - started);
    shutdown.steps.push(step);
  }
  return failure;
}

export function finishShutdown(shutdown) {
  const completed = Date.now();
  shutdown.completed_unix_ms = completed;
  shutdown.duration_ms = shutdown.steps.reduce((total, step) => total + step.duration_ms, 0);
  shutdown.wall_duration_ms = Math.max(
    0,
    completed - (shutdown.started_unix_ms ?? completed),
  );
  shutdown.outcome = shutdown.steps.every((step) => step.outcome === "pass") ? "pass" : "fail";
}

/// Arms the lane-wide deadline. It is deliberately unreferenced: the timer is
/// a failure backstop, never a reason for an otherwise finished lane to remain
/// alive.
export function armHardDeadline(milliseconds) {
  const timer = setTimeout(() => {
    process.stderr.write(`${HARD_DEADLINE_DIAGNOSTIC} exceeded ${milliseconds}ms\n`);
    process.exit(1);
  }, milliseconds);
  timer.unref();
  return () => clearTimeout(timer);
}

const flush = (stream) =>
  new Promise((done) => {
    stream.write("", done);
  });

/// Flushes the result diagnostic, then exits even if a future regression leaves
/// a server, socket, child process, timer, or other event-loop handle open.
export async function exitAfterFlush(code) {
  try {
    await Promise.all([flush(process.stdout), flush(process.stderr)]);
  } finally {
    process.exit(code);
  }
}
