const readline = require('node:readline');
const crypto = require('node:crypto');
const { chromium, firefox, webkit } = require('playwright');

const { createEmitter, nowUnixMs } = require('./emit');
const {
  createCollectorState,
  installRuntimeInstrumentation,
  matchInitiatorHint,
} = require('./instrumentation');

const KNOWN_EVENT_TYPES = new Set([
  'BrowserOpened',
  'PageNavigated',
  'RouteChanged',
  'UserAction',
  'NetworkRequest',
  'NetworkResponse',
  'ConsoleLog',
  'DomSnapshotMarker',
  'BrowserClosed',
]);

async function main() {
  const args = parseArgs(process.argv.slice(2));

  if (!args.sessionId || !args.output || !args.url) {
    throw new Error(
      'uso: node index.js --session-id <id> --output <path> --url <url> [--browser chromium] [--headless] [--event-type NetworkRequest]'
    );
  }

  const emitter = createEmitter({
    sessionId: args.sessionId,
    outputPath: args.output,
    allowedEventTypes: args.eventTypes,
  });

  const state = createCollectorState();
  const browserType = resolveBrowser(args.browser);
  const browser = await browserType.launch({ headless: !!args.headless });
  const context = await browser.newContext();
  const page = await context.newPage();

  const requestIdByRequest = new WeakMap();
  let lastNavigatedUrl = null;
  let shuttingDown = false;
  let browserClosed = false;
  let stopResolve;

  const stopPromise = new Promise((resolve) => {
    stopResolve = resolve;
  });

  const stop = (reason) => {
    if (shuttingDown) {
      return;
    }
    shuttingDown = true;
    stopResolve(reason || 'shutdown');
  };

  browser.on('disconnected', () => {
    browserClosed = true;
    emitter.status('browser_disconnected');
    stop('browser_disconnected');
  });

  process.on('SIGINT', () => stop('sigint'));
  process.on('SIGTERM', () => stop('sigterm'));

  const rl = readline.createInterface({ input: process.stdin });
  rl.on('line', (line) => {
    if (String(line || '').trim().toLowerCase() === 'shutdown') {
      stop('stdin_shutdown');
    }
  });

  await installRuntimeInstrumentation({ context, emitter, state });

  page.on('console', (msg) => {
    emitter.emitTrace('ConsoleLog', {
      level: msg.type(),
      text: msg.text(),
    });
  });

  page.on('framenavigated', (frame) => {
    if (frame !== page.mainFrame()) {
      return;
    }

    const toUrl = frame.url();
    const fromUrl = lastNavigatedUrl;

    state.currentRoute = safeRouteFromUrl(toUrl);
    state.currentPageUrl = toUrl;
    lastNavigatedUrl = toUrl;

    emitter.emitTrace('PageNavigated', {
      from_url: fromUrl,
      to_url: toUrl,
    });

    emitter.emitTrace('RouteChanged', {
      from_url: fromUrl,
      to_url: toUrl,
      navigation_type: fromUrl ? 'FullNavigation' : 'InitialLoad',
    });
  });

  page.on('request', (request) => {
    const ts = nowUnixMs();
    const requestId = crypto.randomUUID();
    requestIdByRequest.set(request, requestId);

    const hint = matchInitiatorHint({
      state,
      requestUrl: request.url(),
      requestMethod: request.method(),
      requestTsUnixMs: ts,
    });

    emitter.emitTrace('NetworkRequest', {
      request_id: requestId,
      page_url: state.currentPageUrl || page.url() || '',
      route: state.currentRoute,
      method: request.method(),
      url: request.url(),
      resource_type: request.resourceType(),
      headers: Object.entries(request.headers() || {}),
      post_data: request.postData() || null,
      initiator_hint: hint
        ? {
            source_type: hint.sourceType,
            related_action_id: hint.actionId,
            js_stack: hint.jsStack || [],
            trigger_ts_unix_ms: hint.triggerTsUnixMs || null,
          }
        : null,
    });
  });

  page.on('response', async (response) => {
    const request = response.request();
    const requestId = requestIdByRequest.get(request) || crypto.randomUUID();

    emitter.emitTrace('NetworkResponse', {
      request_id: requestId,
      status: response.status(),
      url: response.url(),
      headers: Object.entries(response.headers() || {}),
    });
  });

  emitter.emitTrace('BrowserOpened', {
    browser: args.browser || 'chromium',
    url: args.url,
  });
  if (args.eventTypes.length > 0) {
    emitter.status('event_type_filter_enabled', {
      event_types: args.eventTypes,
    });
  }
  emitter.status('browser_opened');

  try {
    await page.goto(args.url, { waitUntil: 'domcontentloaded' });
    state.currentPageUrl = page.url();
    state.currentRoute = safeRouteFromUrl(page.url());
    emitter.status('page_loaded');
  } catch (error) {
    emitter.status('page_load_error', { message: String(error && error.message ? error.message : error) });
  }

  await stopPromise;

  emitter.status('shutdown_requested');

  if (!browserClosed) {
    try {
      await browser.close();
    } catch (error) {
      emitter.status('browser_close_error', { message: String(error && error.message ? error.message : error) });
    }
  }

  emitter.emitTrace('BrowserClosed', {
    reason: browserClosed ? 'browser_disconnected' : 'graceful_shutdown',
  });

  rl.close();
  await emitter.close();
  emitter.status('shutdown_complete');
}

function parseArgs(argv) {
  const args = {
    browser: 'chromium',
    headless: false,
    eventTypes: [],
  };

  for (let i = 0; i < argv.length; i += 1) {
    const token = argv[i];
    if (token === '--session-id') {
      args.sessionId = argv[i + 1];
      i += 1;
    } else if (token === '--output') {
      args.output = argv[i + 1];
      i += 1;
    } else if (token === '--url') {
      args.url = argv[i + 1];
      i += 1;
    } else if (token === '--browser') {
      args.browser = argv[i + 1] || 'chromium';
      i += 1;
    } else if (token === '--headless') {
      args.headless = true;
    } else if (token === '--event-type') {
      args.eventTypes.push(argv[i + 1]);
      i += 1;
    } else if (token === '--event-types') {
      const raw = String(argv[i + 1] || '');
      raw
        .split(',')
        .map((entry) => entry.trim())
        .filter(Boolean)
        .forEach((eventType) => args.eventTypes.push(eventType));
      i += 1;
    }
  }

  args.eventTypes = normalizeRequestedEventTypes(args.eventTypes);

  return args;
}

function normalizeRequestedEventTypes(rawEventTypes) {
  if (!Array.isArray(rawEventTypes) || rawEventTypes.length === 0) {
    return [];
  }

  const unique = [];
  const seen = new Set();
  for (const rawEventType of rawEventTypes) {
    const eventType = String(rawEventType || '').trim();
    if (!eventType) {
      continue;
    }

    if (!KNOWN_EVENT_TYPES.has(eventType)) {
      throw new Error(
        `event-type inválido: ${eventType}. Valores válidos: ${Array.from(KNOWN_EVENT_TYPES).join(', ')}`
      );
    }

    if (!seen.has(eventType)) {
      seen.add(eventType);
      unique.push(eventType);
    }
  }

  return unique;
}

function resolveBrowser(browser) {
  switch (String(browser || 'chromium').toLowerCase()) {
    case 'firefox':
      return firefox;
    case 'webkit':
      return webkit;
    case 'chromium':
    default:
      return chromium;
  }
}

function safeRouteFromUrl(url) {
  try {
    const parsed = new URL(url);
    return `${parsed.pathname}${parsed.search}${parsed.hash}`;
  } catch {
    return url || null;
  }
}

main().catch((error) => {
  const message = String(error && error.stack ? error.stack : error);
  process.stderr.write(`[collector] ${message}\n`);
  process.exitCode = 1;
});
