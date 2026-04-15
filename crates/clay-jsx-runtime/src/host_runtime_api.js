function normalizeDelay(value, minimum = 0) {
  const numeric = Number(value);
  if (!Number.isFinite(numeric)) {
    return minimum;
  }
  return Math.max(minimum, Math.trunc(numeric));
}

const hostTimerCallbacks = new Map();

function scheduleHostTimer(callback, delay, interval, args, animationFrame = false) {
  if (typeof callback !== "function") {
    throw new TypeError("Timer callback must be a function.");
  }

  const waitMs = normalizeDelay(delay);
  // Repeating timers must always make forward progress. Clamp to at least 1ms
  // so setInterval(..., 0) cannot busy-loop the timer worker.
  const repeatMs = interval == null ? -1 : normalizeDelay(interval, 1);
  const handle = Deno.core.ops.op_host_schedule_timer(waitMs, repeatMs);
  hostTimerCallbacks.set(handle, {
    animationFrame,
    args,
    callback,
    repeat: repeatMs >= 0,
  });
  return handle;
}

function clearHostTimer(handle) {
  const numericHandle = Number(handle);
  if (!Number.isFinite(numericHandle)) {
    return;
  }
  hostTimerCallbacks.delete(numericHandle);
  Deno.core.ops.op_host_cancel_timer(numericHandle);
}

globalThis.setTimeout = function setTimeout(callback, delay = 0, ...args) {
  return scheduleHostTimer(callback, delay, null, args);
};

globalThis.clearTimeout = function clearTimeout(handle) {
  clearHostTimer(handle);
};

globalThis.setInterval = function setInterval(callback, delay = 0, ...args) {
  return scheduleHostTimer(callback, delay, delay, args);
};

globalThis.clearInterval = function clearInterval(handle) {
  clearHostTimer(handle);
};

globalThis.requestAnimationFrame = function requestAnimationFrame(callback) {
  return scheduleHostTimer(callback, 16, null, [], true);
};

globalThis.cancelAnimationFrame = function cancelAnimationFrame(handle) {
  clearHostTimer(handle);
};

globalThis.requestRepaint = function requestRepaint() {
  Deno.core.ops.op_host_request_wake();
};

globalThis.__clayDrainHostCallbacks = function drainHostCallbacks() {
  const dueHandles = JSON.parse(Deno.core.ops.op_host_take_due_timers());
  let invoked = 0;
  for (const rawHandle of dueHandles) {
    const handle = Number(rawHandle);
    const entry = hostTimerCallbacks.get(handle);
    if (entry == null) {
      continue;
    }
    if (!entry.repeat) {
      hostTimerCallbacks.delete(handle);
    }
    if (entry.animationFrame) {
      entry.callback(Date.now());
      invoked += 1;
      continue;
    }
    entry.callback(...entry.args);
    invoked += 1;
  }
  return invoked;
};
