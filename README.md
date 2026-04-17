# flowtrace (MVP)

CLI em Rust para capturar e correlacionar interações frontend com requests/backend, usando um collector Node + Playwright.

## Pré-requisitos

- Rust stable
- Node.js 20+
- Dependências do collector:

```bash
cd collector
npm install
npx playwright install chromium
cd ..
```

## Comandos

```bash
# captura completa: abre browser, navegue e pressione Enter no terminal
cargo run -- record --url http://localhost:3000

# correlação offline a partir de um raw_trace.jsonl
cargo run -- analyze --input reports/<session_id>/raw_trace.jsonl

# gerar HTML a partir de correlated_trace.json
cargo run -- report --input reports/<session_id>/correlated_trace.json

# abrir relatório de sessão
cargo run -- open --session <session_id>
```

## Saídas

- `reports/<session_id>/raw_trace.jsonl`
- `reports/<session_id>/correlated_trace.json`
- `reports/<session_id>/index.html`
