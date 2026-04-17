const fs = require('node:fs');
const path = require('node:path');

function createCollectorState() {
  return {
    lastAction: null,
    currentRoute: null,
    currentPageUrl: null,
    initiatorHints: [],
  };
}

function installRuntimeInstrumentation({ context, emitter, state }) {
  const preloadPath = path.join(__dirname, 'preload.js');
  const preloadSource = fs.readFileSync(preloadPath, 'utf-8');

  return Promise.all([
    context.exposeBinding('__flowtraceEmit', async (_source, payload) => {
      handleInjectedEvent(payload, emitter, state);
    }),
    context.addInitScript({ content: preloadSource }),
  ]);
}

function handleInjectedEvent(payload, emitter, state) {
  if (!payload || typeof payload !== 'object' || !payload.kind) {
    return;
  }

  if (payload.kind === 'UserAction') {
    const data = payload.data || {};
    state.lastAction = {
      actionId: data.actionId,
      tsUnixMs: data.tsUnixMs,
      route: data.route || null,
      target: data.target || {},
    };

    emitter.emitTrace('UserAction', {
      action_id: data.actionId,
      action_type: mapUserActionType(data.actionType),
      page_url: data.pageUrl || data.route || '',
      route: data.route || null,
      target: {
        tag_name: data.target?.tagName || null,
        id: data.target?.id || null,
        classes: data.target?.classes || [],
        text: data.target?.text || null,
        test_id: data.target?.testId || null,
        name: data.target?.name || null,
        role: data.target?.role || null,
        css_selector: data.target?.cssSelector || null,
        xpath: null,
      },
      metadata: data.metadata || {},
    });
    return;
  }

  if (payload.kind === 'RouteChanged') {
    const data = payload.data || {};
    state.currentRoute = data.toUrl || state.currentRoute;
    state.currentPageUrl = data.toUrl || state.currentPageUrl;

    emitter.emitTrace('RouteChanged', {
      from_url: data.fromUrl || null,
      to_url: data.toUrl || '',
      navigation_type: mapNavigationType(data.navigationType),
    });
    return;
  }

  if (payload.kind === 'NetworkInitiator') {
    const data = payload.data || {};
    state.initiatorHints.push({
      sourceType: mapSourceType(data.sourceType),
      actionId: data.actionId || null,
      url: data.url || '',
      method: (data.method || 'GET').toUpperCase(),
      route: data.route || null,
      jsStack: Array.isArray(data.jsStack) ? data.jsStack : [],
      triggerTsUnixMs: Number(data.triggerTsUnixMs || Date.now()),
    });

    if (state.initiatorHints.length > 5000) {
      state.initiatorHints.splice(0, state.initiatorHints.length - 5000);
    }
  }
}

function matchInitiatorHint({ state, requestUrl, requestMethod, requestTsUnixMs }) {
  const normalizedMethod = (requestMethod || 'GET').toUpperCase();
  const hints = state.initiatorHints;

  for (let i = hints.length - 1; i >= 0; i -= 1) {
    const hint = hints[i];
    if (hint.method !== normalizedMethod) {
      continue;
    }

    if (requestUrl && hint.url && !sameRequestTarget(requestUrl, hint.url)) {
      continue;
    }

    const delta = Math.abs(requestTsUnixMs - hint.triggerTsUnixMs);
    if (delta > 2500) {
      continue;
    }

    hints.splice(i, 1);
    return hint;
  }

  return null;
}

function sameRequestTarget(left, right) {
  try {
    const l = new URL(left, 'http://dummy.local');
    const r = new URL(right, 'http://dummy.local');
    return l.pathname === r.pathname;
  } catch {
    return left === right;
  }
}

function mapUserActionType(actionType) {
  switch ((actionType || '').toLowerCase()) {
    case 'submit':
      return 'Submit';
    case 'input':
      return 'Input';
    case 'change':
      return 'Change';
    case 'keypress':
      return 'KeyPress';
    case 'navigation':
      return 'Navigation';
    case 'click':
    default:
      return 'Click';
  }
}

function mapNavigationType(navigationType) {
  switch ((navigationType || '').toLowerCase()) {
    case 'pushstate':
      return 'PushState';
    case 'replacestate':
      return 'ReplaceState';
    case 'popstate':
      return 'PopState';
    case 'fullnavigation':
      return 'FullNavigation';
    case 'initialload':
    default:
      return 'InitialLoad';
  }
}

function mapSourceType(sourceType) {
  switch ((sourceType || '').toLowerCase()) {
    case 'fetch':
      return 'Fetch';
    case 'xhr':
      return 'Xhr';
    case 'document':
      return 'Document';
    case 'script':
      return 'Script';
    case 'router':
      return 'Router';
    default:
      return 'Unknown';
  }
}

module.exports = {
  createCollectorState,
  installRuntimeInstrumentation,
  matchInitiatorHint,
};
