(() => {
  if (window.__flowtraceInstalled) {
    return;
  }
  window.__flowtraceInstalled = true;

  const emit = (payload) => {
    if (typeof window.__flowtraceEmit === 'function') {
      window.__flowtraceEmit(payload).catch(() => {});
    }
  };

  const getRoute = () => window.location.pathname + window.location.search + window.location.hash;
  const now = () => Date.now();

  const randomId = () => {
    if (window.crypto && typeof window.crypto.randomUUID === 'function') {
      return window.crypto.randomUUID();
    }
    return `action-${Math.random().toString(36).slice(2)}-${Date.now()}`;
  };

  window.__flowtrace = {
    lastAction: null,
  };

  function actionTarget(target) {
    if (!target || !(target instanceof Element)) {
      return {
        tagName: null,
        id: null,
        classes: [],
        text: null,
        testId: null,
        name: null,
        role: null,
        cssSelector: null,
      };
    }

    return {
      tagName: target.tagName ? target.tagName.toLowerCase() : null,
      id: target.id || null,
      classes: Array.from(target.classList || []),
      text: (target.textContent || '').trim().slice(0, 120) || null,
      testId: target.getAttribute('data-testid') || null,
      name: target.getAttribute('name') || null,
      role: target.getAttribute('role') || null,
      cssSelector: buildCssSelector(target),
    };
  }

  function buildCssSelector(el) {
    if (!el || !(el instanceof Element)) {
      return null;
    }
    const parts = [];
    let current = el;
    while (current && current.nodeType === 1 && parts.length < 5) {
      let selector = current.tagName.toLowerCase();
      if (current.id) {
        selector += `#${current.id}`;
        parts.unshift(selector);
        break;
      }
      const classes = Array.from(current.classList || []).slice(0, 2);
      if (classes.length) {
        selector += `.${classes.join('.')}`;
      }
      parts.unshift(selector);
      current = current.parentElement;
    }
    return parts.join(' > ');
  }

  function registerAction(actionType, target, metadata = {}) {
    const actionId = randomId();
    const payload = {
      actionId,
      actionType,
      pageUrl: window.location.href,
      route: getRoute(),
      tsUnixMs: now(),
      target: actionTarget(target),
      metadata,
    };

    window.__flowtrace.lastAction = payload;
    emit({ kind: 'UserAction', data: payload });
    return payload;
  }

  window.addEventListener('click', (evt) => {
    registerAction('click', evt.target, { x: evt.clientX, y: evt.clientY });
  }, true);

  window.addEventListener('submit', (evt) => {
    registerAction('submit', evt.target, {});
  }, true);

  window.addEventListener('change', (evt) => {
    registerAction('change', evt.target, {});
  }, true);

  window.addEventListener('input', (evt) => {
    registerAction('input', evt.target, {});
  }, true);

  window.addEventListener('keydown', (evt) => {
    if (evt.key === 'Enter' || evt.key === 'Tab') {
      registerAction('keypress', evt.target, { key: evt.key });
    }
  }, true);

  const originalPushState = history.pushState;
  history.pushState = function pushStateWrapper(state, title, url) {
    const fromUrl = window.location.href;
    const result = originalPushState.apply(this, arguments);
    emit({
      kind: 'RouteChanged',
      data: {
        fromUrl,
        toUrl: window.location.href,
        navigationType: 'PushState',
      },
    });
    return result;
  };

  const originalReplaceState = history.replaceState;
  history.replaceState = function replaceStateWrapper(state, title, url) {
    const fromUrl = window.location.href;
    const result = originalReplaceState.apply(this, arguments);
    emit({
      kind: 'RouteChanged',
      data: {
        fromUrl,
        toUrl: window.location.href,
        navigationType: 'ReplaceState',
      },
    });
    return result;
  };

  window.addEventListener('popstate', () => {
    emit({
      kind: 'RouteChanged',
      data: {
        fromUrl: null,
        toUrl: window.location.href,
        navigationType: 'PopState',
      },
    });
  });

  const originalFetch = window.fetch;
  window.fetch = function flowtraceFetch(input, init) {
    const url = typeof input === 'string' ? input : input && input.url ? input.url : '';
    const method = (init && init.method) || (input && input.method) || 'GET';
    const stack = (new Error().stack || '')
      .split('\n')
      .slice(1, 6)
      .map((line) => line.trim())
      .filter(Boolean);

    emit({
      kind: 'NetworkInitiator',
      data: {
        sourceType: 'Fetch',
        actionId: window.__flowtrace.lastAction ? window.__flowtrace.lastAction.actionId : null,
        url,
        method,
        route: getRoute(),
        jsStack: stack,
        triggerTsUnixMs: now(),
      },
    });

    return originalFetch.apply(this, arguments);
  };

  const originalXhrOpen = XMLHttpRequest.prototype.open;
  const originalXhrSend = XMLHttpRequest.prototype.send;

  XMLHttpRequest.prototype.open = function flowtraceXhrOpen(method, url) {
    this.__flowtraceMethod = method;
    this.__flowtraceUrl = url;
    return originalXhrOpen.apply(this, arguments);
  };

  XMLHttpRequest.prototype.send = function flowtraceXhrSend(body) {
    const stack = (new Error().stack || '')
      .split('\n')
      .slice(1, 6)
      .map((line) => line.trim())
      .filter(Boolean);

    emit({
      kind: 'NetworkInitiator',
      data: {
        sourceType: 'Xhr',
        actionId: window.__flowtrace.lastAction ? window.__flowtrace.lastAction.actionId : null,
        url: this.__flowtraceUrl || '',
        method: this.__flowtraceMethod || 'GET',
        route: getRoute(),
        jsStack: stack,
        triggerTsUnixMs: now(),
      },
    });

    return originalXhrSend.apply(this, arguments);
  };

  emit({
    kind: 'RouteChanged',
    data: {
      fromUrl: null,
      toUrl: window.location.href,
      navigationType: 'InitialLoad',
    },
  });
})();
