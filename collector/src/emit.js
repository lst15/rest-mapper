const fs = require('node:fs');
const crypto = require('node:crypto');

function nowUnixMs() {
  return Date.now();
}

function createEmitter({ sessionId, outputPath, allowedEventTypes }) {
  fs.mkdirSync(require('node:path').dirname(outputPath), { recursive: true });
  const stream = fs.createWriteStream(outputPath, { flags: 'a' });
  const allowedTypes = Array.isArray(allowedEventTypes) && allowedEventTypes.length > 0
    ? new Set(allowedEventTypes)
    : null;

  function emitTrace(type, data) {
    if (allowedTypes && !allowedTypes.has(type)) {
      return;
    }

    const envelope = {
      id: crypto.randomUUID(),
      ts_unix_ms: nowUnixMs(),
      session_id: sessionId,
      event: {
        type,
        data,
      },
    };
    stream.write(`${JSON.stringify(envelope)}\n`);
  }

  function status(message, detail) {
    const payload = {
      type: 'status',
      message,
    };

    if (detail !== undefined) {
      payload.detail = detail;
    }

    process.stdout.write(`${JSON.stringify(payload)}\n`);
  }

  async function close() {
    await new Promise((resolve) => stream.end(resolve));
  }

  return {
    emitTrace,
    status,
    close,
  };
}

module.exports = {
  createEmitter,
  nowUnixMs,
};
