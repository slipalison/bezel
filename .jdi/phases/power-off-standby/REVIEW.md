# Phase 12: Review  (slug: power-off-standby)

**Verdict:** APPROVED_WITH_WARNINGS

> Revisão em modo `verify`, iteração 3 do loop (`/jdi-issue`). Branch `jdi/power-off-standby`, `HEAD` = `90b0d54`
> (44 commits desde `origin/main`); último commit de código `0e588e9`. A árvore ficou limpa durante toda a revisão.
>
> - **Escopo julgado:**
>   - o diff `5746fe9..0e588e9` (7 commits; 14 arquivos fora de `.jdi`, +551/−132), lido por inteiro;
>   - comparado com CONTEXT, PLAN, SUMMARY, PROJECT e as decisões D-2026-10-03-power-off-standby-1..9 (a D-9 é
>     nova);
>   - cada achado da iteração 2 (W1–W6 do revisor e a linha 2 do crítico de DoD, que deu BLOCKED) conferido no código
>     e por execução.
> - **Números:** todos vêm das minhas execuções nesta sessão; nenhum foi copiado do SUMMARY. Os `Verify:` do CONTEXT
>   rodaram exatamente como escritos: extraídos do arquivo e passados a `bash`, sem edição.
> - **CI (D-8):** o run `37157542418` (workflow_dispatch, `headSha` `0e588e9`) terminou `success`, e
>   `gh run watch --exit-status` deu exit 0.
>   - `rust-windows` (job `111304037196`): `success`, com o passo `cargo clippy --all-targets --all-features -D
>     warnings` verde.
>   - No log do Windows: 940 passed, 0 failed, 12 ignored, incluindo
>     `power::tests::a_session_end_applies_the_choice_and_a_quit_does_not ... ok`.
>   - `rust-linux`, `node-ui` e CodeQL (rust): `success`.
> - **Hardware e studio do usuário:** nada abriu `/dev/ttyACM*`; nenhum teste `#[ignore]`/`BEZEL_HW_TESTS` rodou.
>   - O studio instalado (PID 570720) seguiu vivo e intocado.
>   - O `studio-starts-silent.sh` isolou cada execução (compositor, barramento e pastas próprios) e passou 2 de 2: uma
>     dentro da linha 7 e outra avulsa. O KWin não caiu.
> - **Prova por mutação da linha 2:** numa cópia do `0e588e9` no scratchpad (`git archive`, `CARGO_TARGET_DIR`
>   próprio; o repositório não foi tocado), troquei a cláusula `choice_for(catalog, model) != Standby::Keep` de
>   `worth_opening` por `true`.
>   - `linux_keep_opens_no_awake_screen_that_is_not_live` **falhou** com `the shutdown opened the 5": ["connect"]`
>     (`power/tests.rs:1128`).
>   - O teste antigo `linux_keep_releases_the_lock_at_once_and_sends_nothing` continuou passando, como o crítico
>     tinha dito.

## Gates
| Gate | Status | Details |
|---|---|---|
| Build | PASS | `cargo build --workspace --locked`: exit 0 |
| Tests | PASS | **1041 passed, 0 failed, 12 ignored** (hardware/ffmpeg real/KLIPY real). Na iteração 2 eram 1036: +5, nenhum removido (core +1, studio `power` +3, doctest `compile_fail` +1) |
| Coverage | PASS | **94.94%** linhas (TOTAL, sem `main.rs`/`build.rs`), exit 0. Pelo comando do PROJECT, sem filtro: **94.90%**. Arquivos tocados: `power.rs` (studio) 85.08%, `standby.rs` (studio) 97.66%, core `app/standby.rs` 99.38% / `domain/standby.rs` 100%, CLI `standby.rs` 98.92%, driver rev C 97.35% |
| Lint | PASS | `cargo fmt --all --check` e `cargo clippy --workspace --all-targets --locked -- -D warnings`: exit 0. Clippy `x86_64-pc-windows-msvc` dos 8 crates sem studio (`bezel-core`, `-devices`, `-sensors`, `-render`, `bezel`, `-themes`, `-media`, `-power`): exit 0. Clippy do studio no Windows: verde no CI. Nenhum `allow` novo no diff |
| Hexagonal/Safety/Protocol/Hygiene | PASS | 5.1–5.11 limpos (detalhe abaixo). `cargo audit`: nenhuma vulnerabilidade (1290 advisories, 621 crates) |
| Consistency | PASS | Escopos: 42 commits `power-off-standby` + 2 `chore(jdi)`. Arquivos do diff dentro do `files_modified` do PLAN. O contrato novo `at_shutdown(link, store, key)` está registrado no SUMMARY (Deviations). O 0x7B do `album` e a escrita do catálogo no desligamento estão cobertos pela D-9. Nenhuma D-XX quebrada |
| UI Validation | PASS (com aviso) | `npm test`: **265/265** unit (99.95% de linhas) e **228/228** Playwright (claro/escuro × pt-BR/en, axe). Os 5 e2e de standby passam nos 4 projetos. Aviso W3 |
| DoD | PASS | As 8 linhas Auto do CONTEXT passam como escritas, inclusive a 3 com o CI do `0e588e9`. As 3 Auto do PROJECT também. 2 Manual do PROJECT ficam para o corte de release |

### Detalhe do gate 5
| Check | Resultado |
|---|---|
| 5.1 dependências do core | PASS: só `thiserror` |
| 5.2 I/O e threads no core | PASS: nada. O `record_stored` novo usa só a porta `ArchiveStore` |
| 5.3 ports | PASS. Nenhuma impl de porta no core. Traits públicas fora do core: as mesmas de antes (`Clock`, `Pause`, `Monotonic`, `Wire`, `Pace`, `Pictures`, `MediaSetup`), todas auxiliares de adapter |
| 5.4 adapters na composição | PASS: nada fora de `main.rs`/`lib.rs` e testes |
| 5.5 `unsafe` | PASS: nenhum novo. O `bezel-power/src/session.rs:23` continua com `// SAFETY:` |
| 5.6 panics | PASS: os 2 `unwrap` novos estão em módulos de teste (`bezel-cli/src/standby.rs` `mod tests`, `turing_rev_c.rs` `mod tests`) |
| 5.7 escrita no dispositivo | PASS. `Confirm::Yes` fora de teste: só em `Confirmed::require`/`MonitorModeConfirmed::require` e doc, como antes. O `Confirmed::recorded` continua `pub(crate)`. Agora o `RecordedChoice` também é `pub(crate)` e o `recorded_choice` é privado; dois `compile_fail` provam isso (`app/standby.rs:233`, `:237`, ambos `ok` no doctest) |
| 5.8 protocolo | PASS. Conferi `SET_BRIGHTNESS` 0x7B (`protocol/turing_rev_c.rs:43`, doc `:103`) e `RESTART` 0x84 (`:51`, doc `:108`). O §16 explica o 0x7B do `album` (`protocol-turing-rev-c.md:614-623`) |
| 5.9 caminhos no core | PASS: só doc e fixtures de teste antigos |
| 5.10 comandos síncronos | PASS: os não-async são os de antes (`list_fonts`, `preferences`, `cancel_job`…), e nenhum toca a tela nem o catálogo |
| 5.11 supply chain | PASS: `cargo audit` limpo; nenhum segredo; no `Cargo.lock` só `+name = "bezel-power"` |

### Achados da iteração 2
| Achado | Situação | Evidência |
|---|---|---|
| Crítico, linha 2 (`keep` com uma tela acordada fora do ao vivo) | **Resolvido** | `linux_keep_opens_no_awake_screen_that_is_not_live` monta um 5" rev C acordado e fora do ao vivo (`with_a_five`, conector `Apart`), com `keep` nas duas telas, e exige que o 5" não ouça nada, nem `connect`. A mutação acima prova que o teste falha sem a cláusula. O simétrico `linux_a_choice_opens_an_awake_screen_that_is_not_live` (`off` no 5") exige `["connect", "turn_off_now"]`, então o 5" é de fato descoberto e abrível |
| W1 0x7B do `album` sem registro | **Resolvido** | D-2026-10-03-power-off-standby-9, e o §16 do protocolo explica o motivo |
| W2 token compartilhado da UI | Resolvido | `createAnswers()` (`src/standby.js:232-242`): uma escrita para outra tela não pega ficha (`writing` → `null`) e deixa a leitura da tela mostrada ser desenhada. Teste unitário `a write for a screen no longer shown…`. Resíduo em W3 |
| W3 `recorded_choice` público | Resolvido | `at_shutdown(link, store, key)` lê a escolha no próprio core. `recorded_choice` é privado e `RecordedChoice` é `pub(crate)`. A doc agora diz o que é garantido: o limite fica na porta, e o resto é revisão de código (D-6 (3)) |
| W4 estado final só depois de ler o atraso | Resolvido | `backend.enter_final_state()` no anúncio, antes de `deadline_after` (`power.rs:182`). O `shut_down` entra de novo, o que é idempotente (`storage.rs:205`, `studio.rs:1275`). O teste `linux_the_final_state_starts_before_the_delay_is_read` usa um logind que leva 2 s para responder |
| W5 `stored` desatualizado depois do `album` | Resolvido | `record_stored` (`app/standby.rs:308-320`) grava o plano B do álbum quando o registro dizia outro e não salva quando já diz. Testes: `the_album_at_shutdown_records_the_plan_b_it_stores` no core (1 save, depois 0, `boot` mantido) e `a_shutdown_opens_the_awake_screens_and_wakes_none` no studio, com o catálogo em disco |
| W6 `set` sem o brilho | Resolvido | `set_result` recebe `StoredPlanB`. O teste exige `…start mode 1, sleep timer off, brightness 40%` |

### Mudança de contrato e o catálogo durante a ação (pedido do despacho)
- **`at_shutdown(link, store, key)`:** o studio não entrega mais a escolha ao core.
  - O `apply` (`power.rs:337-343`) passa só o link, o catálogo e a chave.
  - O `keep` volta `Applied::Nothing` antes de `ensure_supported`, então nada é enviado.
  - Um catálogo que não se lê vira `ShutdownChoiceFailed`. Antes era `ShutdownCatalogNotRead`, que continua valendo
    na leitura do `apply_choices`.
- **O que fica travado durante a ação:** é o `MutexGuard` em processo do `storage.archive()` (`power.rs:339`), não
  uma trava de arquivo. O `DiskArchive` não tem trava entre processos; ele grava por arquivo temporário e `rename`.
- **Julgamento:** isso não trava nada dentro do prazo do desligamento.
  - **CLI:** é outro processo e não disputa esse mutex.
  - **Thread principal do studio:** não toma o mutex. Os comandos síncronos (5.10) não tocam o catálogo, e no
    Windows a thread principal só espera o `recv_timeout` do `shut_down`.
  - **Comandos async:** os que usam o catálogo rodam em threads de trabalho. No estado final, são recusados antes
    (`claim`, empréstimo do link) ou esperam só a ação.
  - **Ordem de travas:** enquanto segura o guard, a thread `bezel-shutdown` não toma nenhuma outra trava (o link já
    saiu do studio), então não há inversão.
  - **Várias telas:** as ações já eram sequenciais. Uma tela travada já atrasava a seguinte; o mutex não piora isso.
  - Resíduo fora do prazo: **W1**.
- **O `album` grava o catálogo depois do RESTART** (`app/standby.rs:300`). Está coberto pela D-9 (última frase) e
  documentado no `at_shutdown` e nos guias en/pt-BR.
  - Não quebra a D-2 (4): o OPTIONS modo 1 no desligamento já era a ação da D-3 (1) desde a iteração 1. O registro só
    passou a dizer a verdade, e o `boot` fica (o teste confere).
  - Resíduo: **W2**.

## Blockers
_(nenhum)_

## Warnings
- **W1** (menor) `apps/bezel-studio/src-tauri/src/power.rs:339`: o guard do catálogo fica preso durante o I/O da ação
  na tela.
  - Cenário: uma tela trava na ação e uma escrita fica parada até `WRITE_STALL` = 10 s (`bezel-devices/src/wire.rs:69`).
    O prazo passa, o fd é solto, e o logind cancela (`false`).
  - Resultado: as abas Armazenamento, Ajustes e o gerenciador esperam esse guard (em threads de trabalho) até a escrita
    desistir. Antes da iteração 3, a thread presa segurava só o link.
  - É limitado e raro. Uma correção barata: um `ArchiveStore` do studio que trave o mutex só em cada `load`/`save`.
- **W2** (menor) `crates/bezel-core/src/app/standby.rs:300`: se o `save` do catálogo falhar depois do RESTART, o
  `at_shutdown` devolve erro e o studio diz `ShutdownChoiceFailed` (`power.rs:340-342`). Só que a tela reiniciou no
  álbum. O caso está documentado na doc da função; só o diagnóstico engana.
- **W3** (menor) `apps/bezel-studio/src/ui/standby.js:360`, `:374`: uma escrita para A pedida com B na tela não pega
  ficha (`null`).
  - Se o usuário volta para A antes da resposta, a resposta da escrita nunca é desenhada.
  - A leitura que começou na volta pode ter lido o catálogo antes da gravação. Nesse caso, A mostra a escolha antiga
    até a próxima leitura (trocar de aba ou de tela).
  - É um caso estreito e visual.

## DoD Checklist (gate 8)
| # | Criterion | Source | Type | Status | Evidence |
|---|---|---|---|---|---|
| 1 | Core e driver rev C: plano B e ações = pacotes exatos; OPTIONS inteiro; keepalive; impossível→TURNOFF | CONTEXT | Auto | PASS | `OK`, exit 0. `standby_` **8 passed** (≥4); core `--test standby` **10 passed** (≥6; novo `the_album_at_shutdown_records_the_plan_b_it_stores`) |
| 2 | Linux: 1 inibidor *delay*; `true` aplica e só então fecha o fd; `keep`/ticks/reconexão sem chamadas; `liveScreen` fica; prazo; `false` retoma | CONTEXT | Auto | PASS | `OK`, exit 0. `power::tests::linux_` **12 passed** (≥6). Novos: `linux_keep_opens_no_awake_screen_that_is_not_live` (falha sem `!= Keep`, provado por mutação), `linux_a_choice_opens_an_awake_screen_that_is_not_live` e `linux_the_final_state_starts_before_the_delay_is_read` |
| 3 | Windows: o `Exit` de uma sessão que acaba aplica, sair e outros eventos não; `rust-windows` verde no último commit de código (D-8) | CONTEXT | Auto | PASS | `OK`, exit 0. Os 2 `--exact` dão `ok. 2 passed`. `git log -1 -- . ':!.jdi'` = `0e588e9` → run `37157542418` → job `rust-windows` `success` (clippy e testes do Windows verdes; 940/0/12) |
| 4 | Studio: `Confirm::No` sem chamadas; `keep` desfaz; catálogo compartilhado; álbum lista/adiciona; substituir e remover só confirmados | CONTEXT | Auto | PASS | `OK`, exit 0. `standby::tests` **10 passed** (≥5) |
| 5 | CLI: sem `--yes` nada na tela nem no registro; com `--yes` o brilho e depois o plano B; `album add` horizontal e vertical | CONTEXT | Auto | PASS | `OK`, exit 0. `bezel --test standby` **6 passed** (≥5); os 2 `--exact` dão `ok. 2 passed` |
| 6 | UI: lógica pura, i18n, 5 e2e nomeados × 4 projetos com axe | CONTEXT | Auto | PASS | `OK`, exit 0. `standby.test.mjs` tap `# pass 28`, `# fail 0`; `test:unit` 265/0; `e2e-passed`: "5 tests × 4 projects, 20/20 runs passed with axe" |
| 7 | Guardas: comandos, fonte, sem logger, início silencioso; monitor vê só logind; nenhum pacote novo | CONTEXT | Auto | PASS | `OK`, exit 0, na 1ª execução. Os 4 `--exact` dão `ok. 4 passed`; `speaks_only_to_logind_on_its_bus` 1 passed; no lock só `+name = "bezel-power"`. O script deu `studio-starts-silent: OK: in 4 runs of 12s …` nas 2 execuções |
| 8 | Guia en/pt-BR, §19/§20, CHANGELOG, check-docs | CONTEXT | Auto | PASS | `OK`, exit 0; `check-docs: 16 pages in English and Portuguese, links and privacy checked; all checks passed` (exit 0) |
| P1 | `cargo test --workspace` | PROJECT | Auto | PASS | 1041/0/12, exit 0 |
| P2 | Cobertura ≥ 80% | PROJECT | Auto | PASS | 94.90% (comando do PROJECT, sem filtro), `OK`, exit 0 |
| P3 | Sem TODO/FIXME sem issue | PROJECT | Auto | PASS | `OK` |
| P4 | CHANGELOG por release | PROJECT | Manual | MANUAL_REQUIRED (release) | O `[Unreleased]` tem a entrada. Evidência sugerida: `## [x.y.z] - <data>` no corte |
| P5 | README descreve o comportamento atual | PROJECT | Manual | MANUAL_REQUIRED (release) | Evidência sugerida: diff do README revisado no PR |

Fica para o PR (CONTEXT, não é blocker):
- desligar de verdade com o 8.8" em cada opção e religar;
- desligar no Windows;
- revisão visual;
- fotos em pé nas duas orientações no 8.8";
- tema parado com temporizador de 1 min não dorme (D-5).

## Recommendation
A fase pode seguir para `/jdi-ship power-off-standby`.

A linha 2 do crítico está provada: o teste novo falha quando a cláusula `!= Keep` sai, e o simétrico mostra que o
5" é aberto quando a escolha não é `keep`. Os W1–W6 da iteração 2 estão resolvidos; o W1 virou a D-9.

O que se manteve sólido no diff novo:
- a escolha do desligamento sai só do catálogo, lida no core;
- o estado final começa no anúncio;
- o catálogo diz o plano B que a tela de fato tem;
- os gates, as 8 linhas do DoD e o CI do Windows estão verdes.

Os três avisos são menores e podem ficar para depois. Se for mexer antes do PR, o W1 tem a melhor relação entre
valor e custo: um store que trave por chamada.

## DoD Critic (enhanced)

_(nenhuma linha oca: as 8 linhas Auto provam o critério; a correção da linha 2 da iteração 2 se mantém)_

**Verdict:** APPROVED
