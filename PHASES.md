# Fases de Implementação

## Fase 1 — Shell TUI (fundação) ✅

**Objetivo:** App roda no terminal, exibe layout, responde a teclas.

- [x] Setup do terminal (alternate screen, raw mode)
- [x] Event loop com crossterm (teclas, resize)
- [x] Layout base com ratatui (3 painéis: sidebar, request, response)
- [x] Navegação entre painéis com Tab
- [x] Quit com `q` / `Ctrl+C`

**Entrega:** App abre, mostra layout vazio, navega entre painéis e fecha.

---

## Fase 2 — Request Builder ✅

**Objetivo:** Usuário monta uma request completa pela TUI.

- [x] Seletor de método HTTP (GET/POST/PUT/DELETE/PATCH) — `Ctrl+M`
- [x] Campo de input para URL — `Ctrl+U`
- [x] Editor de headers (key: value, adicionar/remover) — `Ctrl+H`
- [x] Editor de body (textarea) — `Ctrl+B`
- [x] Editor de query params — `Ctrl+P`
- [x] Footer global com keybindings contextuais

**Entrega:** Usuário preenche todos os campos de uma request via atalhos diretos.

---

## Fase 3 — HTTP Client ✅

**Objetivo:** Enviar requests e exibir responses.

- [x] Integração com reqwest (async via tokio)
- [x] Envio com `Ctrl+Enter`
- [x] Exibição de status code + tempo de resposta
- [x] Exibição de response headers
- [x] Exibição de response body
- [x] Syntax highlighting do body (JSON/XML) com syntect
- [x] Indicador de loading durante request

**Entrega:** Request completa ida e volta, com resposta formatada.

---

## Fase 4 — Coleções ✅

**Objetivo:** Salvar e organizar requests.

- [x] Modelo de dados para coleções (JSON em disco)
- [x] Sidebar com lista de coleções/pastas
- [x] Salvar request atual (`Ctrl+S`)
- [x] Carregar request da coleção (Enter na sidebar)
- [x] Criar/deletar coleções (`Ctrl+N` / `d`)
- [x] Diretório padrão: `~/.config/posterm/collections/`

**Entrega:** Requests persistem entre sessões, organizadas em pastas.

---

## Fase 5 — Ambientes e Variáveis ✅

**Objetivo:** Suporte a variáveis de ambiente com interpolação.

- [x] Modelo de dados para environments (JSON)
- [x] Seletor de ambiente ativo (`Ctrl+E`)
- [x] Interpolação `{{variable}}` em URL, headers e body
- [x] Editor de variáveis por ambiente (`Ctrl+W`)
- [x] Indicador do ambiente ativo no header

**Entrega:** Usuário troca entre dev/staging/prod e variáveis são substituídas.

---

## Fase 6 — Histórico ✅

**Objetivo:** Log de requests recentes.

- [x] Salvar cada request+response enviada
- [x] Visualizar histórico na sidebar (tab toggle ←/→)
- [x] Re-executar request do histórico (Enter)
- [x] Limite configurável (últimas 50 requests)
- [x] Deletar entries (d)

**Entrega:** Usuário consulta e repete requests anteriores.

---

## Fase 7 — Polimento ✅

**Objetivo:** UX refinada e features de qualidade de vida.

- [ ] Menu de configurações (keybindings customizáveis)
- [x] Editor de body avançado (multilinhas com Enter, cursor)
- [ ] Temas de cores (pelo menos dark/light)
- [x] Copy response body para clipboard (`Ctrl+Y`)
- [x] Export de request como cURL (`Ctrl+X`)
- [x] Help popup com keybindings (`F1`)
- [x] Tratamento de erros amigável (timeouts, DNS, etc.)
- [x] Scroll no response body (`Up`/`Down`)

**Entrega:** App polida, pronta para uso diário.

**Entrega:** App polida, pronta para uso diário.
