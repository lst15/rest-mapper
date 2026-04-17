const fs = require('node:fs');
const crypto = require('node:crypto');

function nowUnixMs() {
  return Date.now();
}

function createEmitter({ sessionId, outputPath }) {
  fs.mkdirSync(require('node:path').dirname(outputPath), { recursive: true });
  const stream = fs.createWriteStream(outputPath, { flags: 'a' });

  function emitTrace(type, data) {
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
