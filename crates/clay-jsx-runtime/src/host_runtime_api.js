function normalizeDelay(value) {
  const numeric = Number(value);
  if (!Number.isFinite(numeric) || numeric <= 0) {
    return 0;
  }
  return Math.max(0, Math.trunc(numeric));
}

const hostTimerCallbacks = new Map();

function scheduleHostTimer(callback, delay, interval, args, animationFrame = false) {
  if (typeof callback !== "function") {
    throw new TypeError("Timer callback must be a function.");
  }

  const waitMs = normalizeDelay(delay);
  const repeatMs = interval == null ? -1 : normalizeDelay(interval);
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
      entry.callback(performance.now());
      continue;
    }
    entry.callback(...entry.args);
  }
};
