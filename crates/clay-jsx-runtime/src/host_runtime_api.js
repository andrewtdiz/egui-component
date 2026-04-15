function normalizeDelay(value, minimum = 0) {
  const numeric = Number(value);
  if (!Number.isFinite(numeric)) {
    return minimum;
  }
  return Math.max(minimum, Math.trunc(numeric));
}

const MIN_REPEAT_TIMER_DELAY_MS = 1;
// v1 scheduling model: one host drain may invoke at most this many callbacks.
// If more timer/RAF callbacks remain after a batch, JS must request another
// host wake before returning so the embedder can schedule another frame/drain.
const MAX_HOST_CALLBACKS_PER_DRAIN_CALL = 128;
const hostTimerCallbacks = new Map();
let pendingDueHandles = [];
let pendingDueHandleOffset = 0;

function hasPendingDueHandles() {
  return pendingDueHandleOffset < pendingDueHandles.length;
}

function queueDueHandles(dueHandles) {
  if (!Array.isArray(dueHandles) || dueHandles.length === 0) {
    return;
  }
  pendingDueHandles.push(...dueHandles);
}

function nextPendingDueHandle() {
  const handle = pendingDueHandles[pendingDueHandleOffset];
  pendingDueHandleOffset += 1;
  if (pendingDueHandleOffset >= pendingDueHandles.length) {
    pendingDueHandles = [];
    pendingDueHandleOffset = 0;
  }
  return handle;
}

function loadPendingDueHandles() {
  if (hasPendingDueHandles()) {
    return;
  }
  queueDueHandles(Deno.core.ops.op_host_take_due_timers());
}

function scheduleHostTimer(callback, delay, interval, args, animationFrame = false) {
  if (typeof callback !== "function") {
    throw new TypeError("Timer callback must be a function.");
  }

  const waitMs = normalizeDelay(delay);
  // Repeating timers must always make forward progress. Clamp to at least 1ms
  // so setInterval(..., 0) cannot busy-loop the timer worker.
  const repeatMs =
    interval == null ? -1 : normalizeDelay(interval, MIN_REPEAT_TIMER_DELAY_MS);
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
  // requestRepaint is an invalidation-only wake. It may produce zero callback
  // invocations in the subsequent drain, but it still requests one host frame.
  Deno.core.ops.op_host_request_wake();
};

// Exposed for runtime tests that assert bounded per-drain callback behavior.
globalThis.__clayHostCallbackDrainLimit = MAX_HOST_CALLBACKS_PER_DRAIN_CALL;
globalThis.__clayDescribeHostSchedulingForTest = function describeHostSchedulingForTest() {
  return {
    activeTimerCallbackCount: hostTimerCallbacks.size,
    drainLimit: MAX_HOST_CALLBACKS_PER_DRAIN_CALL,
    pendingDueHandleCount: pendingDueHandles.length - pendingDueHandleOffset,
  };
};

globalThis.__clayDrainHostCallbacks = function drainHostCallbacks() {
  let invoked = 0;
  let processed = 0;
  while (processed < MAX_HOST_CALLBACKS_PER_DRAIN_CALL) {
    loadPendingDueHandles();
    if (!hasPendingDueHandles()) {
      break;
    }

    const rawHandle = nextPendingDueHandle();
    processed += 1;
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

  if (hasPendingDueHandles()) {
    Deno.core.ops.op_host_request_wake();
  }

  return invoked;
};
