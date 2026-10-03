# Phase 12: Review  (slug: power-off-standby)

**Verdict:** APPROVED_WITH_WARNINGS

> Revisão em modo `verify`, iteração 2 do loop (`/jdi-issue`). Branch `jdi/power-off-standby`, `HEAD` = `c90bc41`
> (35 commits desde `origin/main`); último commit de código `5746fe9`. A árvore ficou limpa durante toda a revisão.
>
> - **Escopo julgado:**
>   - o diff novo `cdcd804..5746fe9` (16 commits, 35 arquivos, +1450/−302), lido por inteiro;
>   - comparado com CONTEXT, PLAN, SUMMARY, PROJECT e as decisões D-2026-10-03-power-off-standby-1..8;
>   - cada achado da iteração 1 (iteração 1: BLOCKED; B1, B2, W1–W9 e as linhas 3/4/5 do crítico de DoD) conferido
>     no código e por execução.
> - **Números:** todos das minhas execuções nesta sessão; nenhum copiado do SUMMARY. Os `Verify:` do CONTEXT rodaram
>   exatamente como escritos: extraídos do arquivo e passados a `bash -c`, sem edição.
> - **CI (D-8):** o run `37154388067` (workflow_dispatch, `headSha` `5746fe9`) terminou `success`, e `gh run watch
>   --exit-status` deu exit 0.
>   - `rust-windows` (job `111294705649`): `success`, com os passos `cargo clippy` e `cargo test com cobertura` verdes.
>   - No log do Windows: 939 passed, 0 failed, 12 ignored, incluindo
>     `power::tests::a_session_end_applies_the_choice_and_a_quit_does_not ... ok`.
>   - `rust-linux` e `node-ui`: `success`.
> - **Hardware e studio do usuário:** nada abriu `/dev/ttyACM*`; nenhum teste `#[ignore]`/`BEZEL_HW_TESTS` rodou.
>   - O studio instalado (PID 570720) seguiu vivo e intocado.
>   - O `studio-starts-silent.sh` isolou cada execução (compositor, barramento e pastas próprios) e passou 2 de 2. A
>     queda do KWin da iteração 1 não se repetiu.
>   - A compatibilidade do catálogo foi testada só com `bezel --fake` e um `XDG_DATA_HOME` no scratchpad.

## Gates
| Gate | Status | Details |
|---|---|---|
| Build | PASS | `cargo build --workspace --locked`: exit 0 |
| Tests | PASS | **1036 passed, 0 failed, 12 ignored** (hardware/ffmpeg real/KLIPY real). Na iteração 1 eram 1030: +6, nenhum removido |
| Coverage | PASS | **94.93%** linhas (TOTAL, sem `main.rs`/`build.rs`), exit 0. Pelo comando do PROJECT, sem filtro: **94.88%**. Arquivos tocados: `power.rs` 84.41%, `standby.rs` (studio) 97.66%, `lib.rs` (studio) 95.60%, core `app/standby.rs` 99.27% / `domain/standby.rs` 100% / `domain/archive.rs` 99.83%, `archive/disk.rs` 94.81%, CLI `standby.rs` 98.91%, driver rev C 97.35%, `fake.rs` (devices) 98.69%, `bezel-power` `fake.rs` 94.46% |
| Lint | PASS | `cargo fmt --all --check` e `cargo clippy --workspace --all-targets --locked -- -D warnings`: exit 0. Clippy `x86_64-pc-windows-msvc` dos 8 crates sem studio (`bezel-core`, `-devices`, `-sensors`, `-render`, `bezel`, `-themes`, `-media`, `-power`): exit 0. Clippy do studio no Windows: o passo `cargo clippy` do `rust-windows` passou (B1 resolvido). Nenhum `allow` novo no diff |
| Hexagonal/Safety/Protocol/Hygiene | PASS | 5.1–5.11 limpos (detalhe abaixo). `cargo audit`: nenhuma vulnerabilidade (1290 advisories, 621 crates) |
| Consistency | PASS (com aviso) | Escopos: 33 commits `power-off-standby` + 2 `chore(jdi)`. Contratos novos registrados no SUMMARY (Deviations). Nenhuma D-XX quebrada; um pacote a mais no `album` do desligamento sem registro de decisão: **W1** |
| UI Validation | PASS (com aviso) | `npm test`: **264/264** unit (99.95% de linhas) e **228/228** Playwright (claro/escuro × pt-BR/en, axe). Os 5 e2e de standby passam nos 4 projetos. Aviso W2 |
| DoD | PASS | As 8 linhas Auto do CONTEXT passam como escritas, inclusive a 3 com o CI. As 3 Auto do PROJECT também. 2 Manual do PROJECT são do corte de release |

### Detalhe do gate 5
| Check | Resultado |
|---|---|
| 5.1 dependências do core | PASS: só `thiserror` |
| 5.2 I/O e threads no core | PASS: nada |
| 5.3 ports | PASS. Nenhuma impl de porta no core. Traits públicas fora do core: as mesmas da iteração 1 (`Pause`, `Monotonic`, `Wire`, `Clock`, `Pace`, `Pictures`, `MediaSetup`), todas auxiliares de adapter |
| 5.4 adapters na composição | PASS: nada fora de `main.rs`/`lib.rs` e testes |
| 5.5 `unsafe` | PASS: nenhum novo. O `bezel-power/src/session.rs:23` continua com `// SAFETY:` |
| 5.6 panics | PASS: os `unwrap`/`expect` novos estão todos em código de teste |
| 5.7 escrita no dispositivo | PASS. `Confirm::Yes` fora de teste: só em `Confirmed::require`/`MonitorModeConfirmed::require` e doc. O `album_add` agora passa ao `Manager::upload` a confirmação própria de substituir (`standby.rs:416`). O `upload` exige `Confirm::Yes` quando `plan.replaces` existe (`app/manager.rs:204-206`) e reconfere a presença com `Confirm::No` (`app/storage.rs:208-210`). O `leave_desktop_mode` é recusado no estado final (`backend.rs:427`) |
| 5.8 protocolo | PASS. Conferi `SET_BRIGHTNESS` 0x7B (`protocol/turing_rev_c.rs:43`, doc `:103`). A tabela do §16 agora lista as consultas e o 0x7B que vão antes das ações (`protocol-turing-rev-c.md:611-612`), e o teste `standby_album_restarts_at_the_level_stored_with_the_plan_b` fixa esses bytes |
| 5.9 caminhos no core | PASS: só doc e fixtures de teste antigos |
| 5.10 comandos síncronos | PASS: o `album_add` segue `async` + `blocking`; os não-async listados são os de antes e não tocam a tela |
| 5.11 supply chain | PASS: `cargo audit` limpo; nenhum segredo; no `Cargo.lock` só `+name = "bezel-power"` |

### Achados da iteração 1
| Achado | Situação | Evidência |
|---|---|---|
| B1 clippy do studio no Windows | **Resolvido** | Agora `if let Some(held) = lock.take() { held.release(); }` (`power.rs:181-183`). O CI do `5746fe9` passou no `cargo clippy` e nos testes do Windows |
| B2 substituir foto sem confirmação | **Resolvido** | O servidor decide o conflito: o `PhotoAsked.replace` (`standby.rs:124`) é o `Confirm` do `Manager::upload` (detalhes em 5.7). Teste `a_photo_replaces_its_namesake_only_when_asked…`: sem `replace` vem `notConfirmed` (`replacing sd/image/beach.png`), nada é enviado e o catálogo fica igual. A UI espera a listagem (`ui/standby.js:496`) e transforma um `notConfirmed` no botão de perigo "Substituir" (`:575-596`). O demo e o teste unitário fazem o mesmo |
| Crítico, linha 3 | **Resolvido** | `tests::only_the_exit_event_of_an_ending_session_applies_the_choice` passa pelo `on_run_event` real no runtime mock: `ExitRequested` dá 0 TURNOFF; `Exit` com a sessão terminando dá 1 TURNOFF e o estado final; `Exit` ao sair dá 0. O `run()` liga `bezel_power::session_ending` (`lib.rs:320`). O `rust-windows` voltou a ser obrigatório (D-8) |
| Crítico, linha 4 | **Resolvido** | O teste do studio passou a exigir a recusa sem `replace` |
| Crítico, linha 5 | **Resolvido** | `FakeLog::kept` grava brilho e OPTIONS numa só ordem. `set_with_yes_sends_the_brightness_then_the_plan_b` exige `[Brightness(40), Options(album, Some(40))]`: inverter a ordem quebra o teste |
| W1 fechar durante o envio | Resolvido | `modal().hold`: botões desabilitados e Esc ignorado; `added` corre quando a foto chega ao cartão. O e2e cobre Esc, Cancelar e X |
| W2 motivo de remoção apagado | Resolvido | `reload(note)` mantém o motivo |
| W3 resposta de `setStandby` em outra tela | Resolvido | A chave é capturada antes do diálogo e há token. Ver o novo W2 |
| W4 brilho do álbum | Resolvido no comportamento | O nível vai no `StoredPlanB` e o `album` do desligamento o reenvia. Ver W1 |
| W5 plano B sem o studio | Resolvido | Guias en/pt-BR e CHANGELOG |
| W6 `at_shutdown` aceitava qualquer `Standby` | Resolvido em parte | `RecordedChoice` com campos privados e `compile_fail`. Ver W3 |
| W7 `show` com plano B errado | Resolvido | `ScreenRecord.stored` / `planB`. Resíduo em W5 |
| W8 KWin instável | Não reproduzido | 2/2 OK nesta revisão |
| W9 menores | Resolvidos, salvo 1 | Enter nunca substitui (e2e); vídeos com tamanho (`offer` consulta cada um); comentários do `bezel-media` e do `StartMode`; prazo contado do sinal (teste novo `linux_the_deadline_counts_from_the_announcement`); `leave_desktop_mode` dá `busy` no estado final (teste); guias (§16, logout no Linux, "repouso", keepalive); `set keep` em tela não rev C dá "rev C screens only" (teste). O item de Ajustes virou todo em `.jdi/todos/2026-10-03-power-off-standby.md`. O e2e "keep undoes" ficou igual (`tests/e2e/standby.spec.mjs:258-262`), o que é aceitável: a releitura após trocar de aba tem valor |

### Compatibilidade do `catalog.json` (v0.15.0)
- O `ScreenDto` só ganhou campos opcionais (`standby` e `planB`, ambos com `#[serde(default, skip_serializing_if =
  "Option::is_none")]`). O schema continua 1 e não há `deny_unknown_fields`, nem na v0.15.0 nem agora.
- **Prova:** um catálogo no formato da v0.15.0 (`model`, `boot: internal/video/bezel_demo.mp4`, `entries: []`), num
  `XDG_DATA_HOME` do scratchpad, com `bezel --fake`:
  - `standby show` leu o catálogo: `keep`, plano B `start mode 2` (do `boot`);
  - `standby set album --brightness 40 --yes` manteve `boot` e `entries` e acrescentou
    `"standby":{"choice":"album"}` e `"planB":{"startMode":1,"sleepMinutes":0,"brightness":40}`;
  - o `show` seguinte disse `start mode 1, sleep timer off, brightness 40%`.
- Arquivos danificados (modo 3, temporizador 11, brilho 101) são recusados com o nome da tela
  (`archive/tests.rs`). Uma v0.15.0 lê o arquivo novo ignorando os campos que não conhece.

## Blockers
_(nenhum)_

## Warnings
- **W1** `crates/bezel-core/src/app/standby.rs:265-267`: com um nível gravado no plano B, o `album` do desligamento
  envia SET_BRIGHTNESS 0x7B antes do OPTIONS.
  - A D-2026-10-03-power-off-standby-3 (1) e o `PLAN.md:25` definem o `album` como "OPTIONS modo 1 + RESTART 0x84".
  - Não quebra a D-2 (5): o 0x7B não é persistente, nem de armazenamento, nem disruptivo, e é o meio de o OPTIONS
    levar o "brilho do link" que a D-2 (3) pede.
  - Está documentado no §16 (`protocol-turing-rev-c.md:612`) e fixado por teste.
  - Mesmo assim, é um pacote a mais na sequência travada sem registro (a D-8 só trata do Windows). Registrar como
    decisão (D-9) ou como desvio no SUMMARY antes do PR.
- **W2** `apps/bezel-studio/src/ui/standby.js:355-371`: o `write()` incrementa o token compartilhado `view.loads`
  mesmo quando grava para uma tela que já não é a mostrada.
  - Cenário: duas rev C, diálogo de confirmação de A aberto, A sai. O `update()` troca para B e inicia o `load()` de B.
    O usuário confirma A.
  - O `load()` de B volta com o token velho e é descartado (`:133`). A seção de B fica em "Carregando…" até outra
    troca ou o próximo `show()`.
  - Correção: um token só da escrita, ou incrementar só quando `view.key === key`.
- **W3** `crates/bezel-core/src/app/standby.rs:204` (+ doc em `domain/standby.rs:190-200`): o `recorded_choice` é
  `pub` sobre qualquer `ArchiveStore`.
  - Um adapter ainda obtém um `RecordedChoice` de um store que ele mesmo preenche (um `MemoryArchive`). Então a frase
    "no adapter can make one up" vale só contra a construção literal, que o `compile_fail` cobre.
  - É uma melhora real sobre a iteração 1, e um contorno deliberado fica na revisão de código (D-6 (3)). Ajustar a doc
    para o que de fato garante.
- **W4** `apps/bezel-studio/src-tauri/src/power.rs:175-177`, `:209-214`: o prazo agora conta do sinal, mas o estado
  final só começa depois da leitura do `InhibitDelayMaxUSec`, que tem até ~2 s de timeout de D-Bus.
  - Nesse intervalo, a sessão ao vivo ainda pode mandar quadros ou reconectar.
  - Ler o atraso em `hold` (`:219`), quando a trava é tomada, eliminaria a janela.
- **W5** (resíduo da W7 anterior) `crates/bezel-core/src/app/standby.rs:261-273`, `domain/archive.rs:370-376`: o
  `album` do desligamento grava o modo 1 na tela (ação da D-3), mas o catálogo não atualiza o `stored`.
  - Cenário: `album` escolhido, depois `storage boot` de um vídeo (`stored` = modo 2), depois um desligamento. A tela
    fica no modo 1 e liga com o álbum, enquanto `bezel standby show` e o studio dizem "start mode 2".
  - Só a exibição erra.
- **W6** (menor) `crates/bezel-cli/src/standby.rs:548`: a saída de `standby set … --brightness 40` diz `plan B stored
  on the screen: start mode 1, sleep timer off` sem o brilho, enquanto o `show` diz `…, brightness 40%`
  (`StoredPlanB`). Cosmético.

## DoD Checklist (gate 8)
| # | Criterion | Source | Type | Status | Evidence |
|---|---|---|---|---|---|
| 1 | Core e driver rev C: plano B e ações = pacotes exatos; OPTIONS inteiro; keepalive; impossível→TURNOFF | CONTEXT | Auto | PASS | `OK`, exit 0. `standby_` **8 passed** (≥4, inclui o novo `standby_album_restarts_at_the_level_stored_with_the_plan_b`); core `--test standby` **9 passed** (≥6) |
| 2 | Linux: 1 inibidor *delay*; `true` aplica e só então fecha o fd; `keep`/ticks/reconexão sem chamadas; `liveScreen` fica; prazo; `false` retoma | CONTEXT | Auto | PASS | `OK`, exit 0. `power::tests::linux_` **9 passed** (≥6; novo: `linux_the_deadline_counts_from_the_announcement`) |
| 3 | Windows: o `Exit` de uma sessão que acaba aplica, sair e outros eventos não; `rust-windows` verde no último commit de código (D-8) | CONTEXT | Auto | PASS | `OK`, exit 0. Os 2 `--exact` dão `ok. 2 passed`. `git log -1 -- . ':!.jdi'` = `5746fe9` → run `37154388067` → job `rust-windows` `success` (clippy e testes do Windows verdes; 939/0/12) |
| 4 | Studio: `Confirm::No` sem chamadas; `keep` desfaz; catálogo compartilhado; álbum lista/adiciona; substituir e remover só confirmados | CONTEXT | Auto | PASS | `OK`, exit 0. `standby::tests` **10 passed** (≥5), com `a_photo_replaces_its_namesake_only_when_asked_and_nothing_goes_without_a_card` |
| 5 | CLI: sem `--yes` nada na tela nem no registro; com `--yes` o brilho e depois o plano B; `album add` horizontal e vertical | CONTEXT | Auto | PASS | `OK`, exit 0. `bezel --test standby` **6 passed** (≥5); os 2 `--exact` (`set_with_yes_sends_the_brightness_then_the_plan_b`, `set_without_yes_opens_nothing_and_says_so`) dão `ok. 2 passed` |
| 6 | UI: lógica pura, i18n, 5 e2e nomeados × 4 projetos com axe | CONTEXT | Auto | PASS | `OK`, exit 0. tap `# fail 0`; `test:unit` 264/0; `e2e-passed`: "5 tests × 4 projects, 20/20 runs passed with axe" |
| 7 | Guardas: comandos, fonte, sem logger, início silencioso; monitor vê só logind; nenhum pacote novo | CONTEXT | Auto | PASS | `OK`, exit 0, na 1ª execução. Os 4 `--exact` dão `ok. 4 passed`; `speaks_only_to_logind_on_its_bus` 1 passed; no lock só `+name = "bezel-power"`. O script deu `studio-starts-silent: OK: in 4 runs of 12s …` nas 2 execuções desta revisão |
| 8 | Guia en/pt-BR, §19/§20, CHANGELOG, check-docs | CONTEXT | Auto | PASS | `OK`, exit 0; `check-docs: 16 pages in English and Portuguese, links and privacy checked; all checks passed` (exit 0) |
| P1 | `cargo test --workspace` | PROJECT | Auto | PASS | 1036/0/12, exit 0 |
| P2 | Cobertura ≥ 80% | PROJECT | Auto | PASS | 94.88% (sem filtro), exit 0 |
| P3 | Sem TODO/FIXME sem issue | PROJECT | Auto | PASS | `OK` |
| P4 | CHANGELOG por release | PROJECT | Manual | MANUAL_REQUIRED (release) | O `[Unreleased]` tem a entrada (plano B sem o studio, `--brightness`). Evidência sugerida: `## [x.y.z] - <data>` no corte |
| P5 | README descreve o comportamento atual | PROJECT | Manual | MANUAL_REQUIRED (release) | Evidência sugerida: diff do README revisado no PR |

Fica para o PR (CONTEXT, não é blocker):
- desligar de verdade com o 8.8" em cada opção e religar;
- desligar no Windows;
- revisão visual;
- fotos em pé nas duas orientações no 8.8";
- tema parado com temporizador de 1 min não dorme (D-5).

## Recommendation
A fase pode seguir para `/jdi-ship power-off-standby`. Os dois blockers e as três linhas do crítico da iteração 1
estão resolvidos e provados:
- **B1:** o clippy e os testes do Windows passaram no CI do último commit de código.
- **B2:** o servidor decide a substituição, como a CLI e a aba Armazenamento.
- **Linhas 3/4/5:** cada prova agora falha se o comportamento regredir.

Antes do PR, em ordem de valor:
1. **W1:** registrar o 0x7B do `album` no desligamento (D-9 ou desvio no SUMMARY).
2. **W2:** separar o token da escrita do token da leitura.

W3–W6 podem ficar para depois.

O que se manteve sólido no diff novo:
- o catálogo continua compatível nos dois sentidos;
- o `RecordedChoice` e o `refuse_while_shutting_down` estreitaram os caminhos que chegam à tela no desligamento;
- as guardas de privacidade (o monitor do barramento, o início silencioso) seguem verdes.

## DoD Critic (enhanced)

- DoD row «2»: `worth_opening` (`power.rs:257-262`) é o único ponto que impede abrir uma rev C acordada e fora do ao vivo quando a escolha é `keep`; sem a cláusula `!= Standby::Keep` (aparentemente redundante, `apply` em `power.rs:338` já pula o `keep`), o desligamento chama `connector.connect` (HELLO, STOP_MEDIA, PRE_UPDATE_BITMAP — `turing_rev_c.rs:218,246-247`) e para o vídeo que a tela toca sozinha, e todos os `linux_` passam: `linux_keep_releases_the_lock_at_once_and_sends_nothing` (`power/tests.rs:941`) só tem o 8.8" ao vivo, que `apply_choices` pula antes de consultar `worth_opening`; nenhum teste combina `keep` com uma tela acordada fora do ao vivo.

**Verdict:** BLOCKED
