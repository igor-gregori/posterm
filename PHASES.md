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

## Fase 4 — Coleções

**Objetivo:** Salvar e organizar requests.

- [ ] Modelo de dados para coleções (JSON em disco)
- [ ] Sidebar com lista de coleções/pastas
- [ ] Salvar request atual (`Ctrl+S`)
- [ ] Carregar request da coleção (Enter na sidebar)
- [ ] Criar/renomear/deletar coleções e pastas
- [ ] Diretório padrão: `~/.config/posterm/collections/`

**Entrega:** Requests persistem entre sessões, organizadas em pastas.

---

## Fase 5 — Ambientes e Variáveis

**Objetivo:** Suporte a variáveis de ambiente com interpolação.

- [ ] Modelo de dados para environments (JSON)
- [ ] Seletor de ambiente ativo (`Ctrl+E`)
- [ ] Interpolação `{{variable}}` em URL, headers e body
- [ ] Editor de variáveis por ambiente
- [ ] Indicador do ambiente ativo no header

**Entrega:** Usuário troca entre dev/staging/prod e variáveis são substituídas.

---

## Fase 6 — Histórico

**Objetivo:** Log de requests recentes.

- [ ] Salvar cada request+response enviada
- [ ] Visualizar histórico na sidebar (tab ou toggle)
- [ ] Re-executar request do histórico
- [ ] Limite configurável (últimas N requests)

**Entrega:** Usuário consulta e repete requests anteriores.

---

## Fase 7 — Polimento

**Objetivo:** UX refinada e features de qualidade de vida.

- [ ] Menu de configurações (keybindings customizáveis)
- [ ] Temas de cores (pelo menos dark/light)
- [ ] Resize responsivo
- [ ] Copy response body para clipboard
- [ ] Export de request como cURL
- [ ] Help popup com keybindings
- [ ] Tratamento de erros amigável (timeouts, DNS, etc.)

**Entrega:** App polida, pronta para uso diário.
