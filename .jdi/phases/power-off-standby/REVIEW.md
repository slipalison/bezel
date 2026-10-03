# Phase 12: Review  (slug: power-off-standby)

**Verdict:** BLOCKED

> Revisão em modo `verify`, iteração 1 do loop (`/jdi-issue`). Branch `jdi/power-off-standby`, `HEAD` = `cdcd804`
> (15 commits desde `origin/main`), árvore limpa durante toda a revisão (`git status`: só o `LOOP.md` não versionado,
> mais este arquivo).
>
> - **Escopo julgado:** `git diff origin/main...HEAD` inteiro (87 arquivos, +12649/−284), contra CONTEXT, PLAN,
>   SUMMARY, PROJECT e as decisões D-2026-10-03-power-off-standby-1..7, D-2026-09-30-device-protocols-2 e
>   D-2026-10-01-gif-sticker-search-15..19. Li por inteiro o core (`domain/standby`, `app/standby`, portas, `Confirmed`),
>   o driver rev C, o `bezel-power`, o `power.rs`/`standby.rs`/`studio.rs`/`storage.rs`/`lib.rs` do studio e a CLI;
>   a UI e os guias foram lidos em paralelo por dois revisores só-leitura, e cada defeito que eles apontaram foi
>   conferido por mim no código antes de entrar aqui.
> - **Números:** todos das minhas execuções nesta sessão; nenhum copiado do SUMMARY. Os `Verify:` do CONTEXT rodaram
>   exatamente como escritos (`bash -c`, em sequência, sem edição).
> - **Hardware e studio do usuário:** nada abriu `/dev/ttyACM*`; nenhum teste `#[ignore]`/`BEZEL_HW_TESTS`. O studio
>   instalado (PID 570720, iniciado às 14:39) seguiu vivo e intocado. O `studio-starts-silent.sh` isolou cada
>   execução (compositor, barramento e pastas próprios); nenhum processo de teste ficou vivo no fim.
> - **CI (D-7, informativo):** o run `37150326673` do `cdcd804` estava em andamento; o job `rust-windows` já tinha
>   **falhado no passo `cargo clippy`** (log do job). O veredito não depende do CI: o defeito B1 foi reproduzido aqui,
>   de forma independente.

## Gates
| Gate | Status | Details |
|---|---|---|
| Build | PASS | `cargo build --workspace --locked`: exit 0. O `--locked` aceitou o lock (só ganhou `bezel-power`) |
| Tests | PASS | **1030 passed, 0 failed, 12 ignored** (hardware/ffmpeg real/KLIPY real), 43 binários. Eram 945 na revisão de `gif-sticker-search`: +85, nenhum removido |
| Coverage | PASS | **94.92%** lines (TOTAL, sem `main.rs`/`build.rs`), exit 0. Pelo comando do PROJECT, sem filtro: **94.87%**. Novos: `power.rs` 85.16%, `studio/standby.rs` 97.67%, `bezel-power` `logind.rs` 95.56% / `fake.rs` 94.35% / `session.rs` 100%, core `domain/standby.rs` 100% / `app/standby.rs` 99.19%, `photo.rs` 95.35%, CLI `standby.rs` 98.83%, driver rev C 97.29%. Nenhuma lógica em `main.rs` |
| Lint | **FAIL** | Linux: `cargo fmt --all --check` e `cargo clippy --workspace --all-targets --locked -- -D warnings`: exit 0. Clippy `x86_64-pc-windows-msvc` dos 8 crates sem studio (`bezel-core`, `-devices`, `-sensors`, `-render`, `bezel`, `-themes`, `-media`, `-power`; `target/wincheck`): exit 0. **Mas o clippy do `bezel-studio` no Windows falha** (`clippy::drop_non_drop`, `power.rs:178`): **B1**. Todo `allow` novo tem `reason =` |
| Hexagonal/Safety/Protocol/Hygiene | PASS | 5.1–5.11 limpos (detalhe abaixo). `cargo audit`: exit 0 (1290 advisories, 621 crates) |
| Consistency | **FAIL** | Escopos: 15 commits `power-off-standby` + 2 `chore(jdi)` do orquestrador. Arquivos fora do PLAN registrados como desvio no SUMMARY. **D-2026-10-03-power-off-standby-4 (3)** ("um nome que já existe pede a confirmação de substituir") quebra num caminho reproduzível: **B2** |
| UI Validation | PASS (com avisos) | `npm test`: **264/264** unit (99.95% de linhas) e **228/228** Playwright (claro/escuro × pt-BR/en, axe). i18n com paridade de chaves, sem `window.confirm`, `invoke(`/`__TAURI__` só no `bridge.js`, movimento reduzido respeitado. Avisos W1–W3 |
| DoD | PASS | As 8 linhas Auto do CONTEXT passam como escritas; as 3 Auto do PROJECT também. 2 Manual do PROJECT são do corte de release |

### Detalhe do gate 5
| Check | Resultado |
|---|---|
| 5.1 dependências do core | PASS: só `thiserror` |
| 5.2 I/O e threads no core | PASS: nada. `Confirmed::recorded` é `pub(crate)`, puro |
| 5.3 ports | PASS. Nenhuma impl de porta no core. Trait pública nova fora do core: só `Monotonic` (relógio do keepalive, auxiliar do driver, como `Pause`/`Clock`) |
| 5.4 adapters na composição | PASS: nada fora de `main.rs`/`lib.rs` e testes |
| 5.5 `unsafe` | PASS: o único novo é `bezel-power/src/session.rs:23` (`GetSystemMetrics`), com `// SAFETY:` e `allow(unsafe_code, reason = …)` sob `#![deny(unsafe_code)]`; nenhum em core/CLI/studio |
| 5.6 panics | PASS: nenhum `unwrap`/`expect`/`panic!` fora de teste nos arquivos novos (`power.rs`, `standby.rs`, `app/standby.rs`, `domain/standby.rs`, `photo.rs`, `bezel-power`, CLI `standby.rs`) |
| 5.7 escrita no dispositivo | PASS. `Confirm::Yes` fora de teste: só em `Confirmed::require`/`MonitorModeConfirmed::require` e na CLI/`commands.rs`. OPTIONS sai só de `choose` (confirmado), `write_boot_media` (confirmado) e `restart_into_album`; 0x84 só de `restart_into_album`; o único chamador de `at_shutdown` é `power.rs:255`, com a escolha relida do catálogo (ver W6) |
| 5.8 protocolo | PASS. `keepalive_run` tem teste de bytes (`80000010ffff00`, forma comprimida `80000010fcff`). Conferi `TURN_OFF` 0x83, `RESTART` 0x84, `SET_OPTIONS` 0x7D (bytes 10..14 = brilho, modo, 0, flip, temporizador) e o loop do `PLAY_VIDEO` (byte 7) contra §6.2/§17.2/§19 |
| 5.9 caminhos no core | PASS: só doc e fixtures de teste antigos |
| 5.10 comandos síncronos | PASS: os 5 comandos novos são `async` (os 4 que tocam a tela via `blocking`) |
| 5.11 supply chain | PASS: `cargo audit` exit 0; nenhum segredo; no `Cargo.lock` só `+name = "bezel-power"` |

## Blockers

### B1 — o studio não passa no clippy do Windows (`apps/bezel-studio/src-tauri/src/power.rs:178`)
- **Regra:** gate 4 (`cargo clippy -- -D warnings` limpo, Linux **e** Windows: PROJECT, Global constraints) e
  D-2026-10-03-power-off-standby-7 (2) (o job `rust-windows` continua obrigatório no PR).
- **O defeito:** fora do Linux, `bezel_power::Inhibitor` envolve `sys::Lock(Infallible)`
  (`crates/bezel-power/src/logind.rs:68` e `:333`), que não tem cola de `Drop`. Então `drop(lock.take())`
  (`power.rs:178`) dispara `clippy::drop_non_drop`, e com `-D warnings` o `bezel-studio` (lib e lib test) não compila
  sob clippy no Windows.
- **Reprodução:**
  - O job `rust-windows` do `cdcd804` falhou no passo `cargo clippy --all-targets --all-features -- -D warnings`:
    `error: call to std::mem::drop with a value that does not implement Drop … --> apps\bezel-studio\src-tauri\src\power.rs:178:17`.
    Por isso os testes e o empacotamento do Windows nem rodaram, e o teste de fim de sessão nunca rodou no Windows.
  - Aqui, o clippy do studio para msvc não compila (`ring` exige `lib.exe`). Por isso reproduzi com uma crate mínima
    no scratchpad, com os mesmos tipos e a mesma chamada: o clippy 1.98 dá o mesmo erro.
- **Por que a DoD não pegou:** a linha 3 só roda o clippy msvc do `bezel-power`. O pressuposto da D-7 ("o código
  específico do Windows fica no `bezel-power`, então o clippy local cobre") não vale aqui: o lint depende do tipo
  condicional do `bezel-power`, mas aparece no studio.
- **Correção sugerida:** soltar sem `drop` (`if let Some(held) = lock.take() { held.release(); }` ou `lock = None;`)
  ou dar `Drop` ao `Inhibitor`.

### B2 — uma foto do álbum pode substituir outra sem a confirmação de substituir (D-4 (3))
- **Regras:** D-2026-10-03-power-off-standby-4 (3) ("um nome que já existe pede a confirmação de substituir de hoje")
  e PROJECT ("nenhum comando destrutivo … sem confirmação explícita").
- **O defeito:** quem detecta o conflito de nome é só a UI (`albumClash(photos, name)`,
  `apps/bezel-studio/src/ui/standby.js:542-552`). O backend aceita `confirmed: true` de um envio como permissão para
  substituir (`apps/bezel-studio/src-tauri/src/standby.rs:365-447` → `Manager::upload(..., Confirm::Yes)`), mesmo
  quando o `prepare_upload` já sabe que existe `plan.replaces`. A CLI faz certo (`replaces` sem `--yes` = recusa,
  `crates/bezel-cli/src/standby.rs:706`), e a aba Armazenamento também (o plano do servidor pede a confirmação).
- **Reprodução 1 (lista falha):**
  1. A listagem do álbum falha: o `managerOverview` rejeita, por exemplo com `busy` transitório. O `photos` fica `[]`
     (`ui/standby.js:419-428`) e só aparece o erro; "Adicionar foto…" continua habilitado.
  2. O usuário adiciona uma foto cujo nome sugerido já existe (por exemplo `praia.png`).
  3. O diálogo diz "Enviar", sem aviso de substituição, e o arquivo existente é sobrescrito.
- **Reprodução 2 (sem falha alguma, junto de W1):**
  1. O usuário envia `praia.png` e aperta Esc durante "Enviando para o cartão…". O envio termina, mas a lista não é
     recarregada.
  2. O usuário adiciona a mesma foto de novo: de novo sem aviso, e sobrescreve.
- **Correção sugerida:** o servidor decide o conflito, como na aba Armazenamento: `album_add` recusa (ou devolve um
  plano) quando `plan.replaces` existe e não veio um `replace` explícito. Além disso, a UI desabilita "Adicionar"
  enquanto a lista não carregou.

## Warnings
- **W1** `apps/bezel-studio/src/ui/standby.js:524-540` (+ `modal`, `:255-268`): durante "Enviando para o cartão…",
  Cancelar, o X e Esc continuam ativos e fecham o diálogo, mas o `album_add` segue e grava. Fica sem toast, sem
  `reload()` e sem `storageChanged()`: a UI diz que nada foi adicionado, e a foto está no cartão. A correção é
  desabilitar fechar/cancelar durante o envio ou ligar o Cancelar ao `cancel_job`.
- **W2** `ui/standby.js:440-447`: um `deleteStored` que falha escreve o erro no `status`, e o `reload()` seguinte
  apaga a mensagem. A foto fica na lista sem explicação.
- **W3** `ui/standby.js:335-348` e `:468`: `write()` grava em `view.data` a resposta de `setStandby` sem token de
  carga nem conferência da chave.
  - Cenário: duas rev C acordadas. O usuário confirma `off` em A e troca o seletor para B enquanto grava. Se o
    `standby_overview(B)` volta antes do `set_standby(A)`, a seção de B passa a mostrar a escolha e os vídeos de A.
    O "Usar o álbum" usa `view.key`, não `standing.key`.
- **W4** `docs/user/power-off.md:82`, `:87-90` (pt-BR `:83`, `:88-90`): `bezel standby set album --brightness 40
  --yes  # starts at 40% brightness` não vale quando o studio aplica o álbum no desligamento.
  - O `restart_into_album` (`crates/bezel-core/src/app/standby.rs:234`) reescreve o OPTIONS com o brilho do link:
    o do studio na tela ao vivo, 170 (cerca de 67%) numa tela aberta só para o desligamento.
  - Isso segue o "brilho do link" da D-2 (3), mas o guia promete o contrário.
- **W5** `docs/user/power-off.md:27-28`, `:38-39` (e pt-BR; mais brando no `CHANGELOG.md`): o guia diz que, "com só
  `bezel run`", o que acontece no desligamento é o plano B.
  - Para `video`/`album` o plano B é só um modo de início, que vale quando a tela reinicia ou perde energia.
  - Com a USB alimentada e sem o studio, a tela fica congelada, que é justamente o problema da fase. Só o temporizador
    do `off` age sozinho.
- **W6** `crates/bezel-core/src/app/standby.rs:199` e `:234-235`, `domain/storage.rs:413-416`: `at_shutdown` é
  `pub`, aceita qualquer `&Standby` e cunha `Confirmed::recorded` para ele.
  - Que OPTIONS e 0x84 venham de uma escolha registrada e confirmada (D-2 (5)) depende só do único chamador
    (`power.rs:250-256`, que relê o catálogo).
  - Um adapter novo forjaria a prova numa linha. Sugestão: `at_shutdown(link, store, key)` lê a escolha ele mesmo.
- **W7** `crates/bezel-core/src/app/standby.rs:94` (`standby.plan_b(boot.start_mode())`) e
  `domain/archive.rs:362-365`: o `bezel standby show` pode mostrar um plano B que não está na tela.
  - Cenário: `standby set album --yes` e depois `storage boot sd/video/x.mp4 --yes`. A tela fica no modo 2
    (`PlanB::with_boot`), e o `show` diz "start mode 1".
  - O registro não guarda qual das duas ações foi a última. O comportamento real segue a D-2 (4); só a exibição erra.
- **W8** `scripts/ci/studio-starts-silent.sh`: das 3 execuções completas desta revisão, 1 falhou.
  - Na execução 3/4 ("no key, hidden"), o `kwin_wayland --virtual` caiu (`KCrash: Application 'kwin_wayland'
    crashing`), e o studio saiu com 139 ao perder o display.
  - A execução da linha 7 da DoD e outras 2 completas deram OK. A queda é do ambiente, não atribuível à fase, mas a
    prova fica instável.
- **W9 (menores):**
  - **UI:**
    - Enter no campo do nome aciona "Substituir" (botão de perigo), contra a regra do projeto de que a resposta
      destrutiva nunca é o padrão (`ui/standby.js:555-559`).
    - No app real, todo vídeo da lista mostra "—" de tamanho (`dto.rs`: `StoredFileDto::at(path, None)`), enquanto o
      demo mostra tamanhos.
    - O e2e "keep undoes" relê um estado que já tinha verificado (`tests/e2e/standby.spec.mjs:252-255`).
  - **Comentários desatualizados:**
    - `crates/bezel-media/Cargo.toml:16` ("no decoding"; o `photo.rs` decodifica).
    - `crates/bezel-core/src/domain/storage.rs:320-323`: o `StartMode` diz "a última imagem/vídeo tocado",
      contrariando o §19.
  - **`power.rs`:**
    - O prazo é contado depois do `Get InhibitDelayMaxUSec` (até 2 s de timeout), não da chegada do
      `PrepareForShutdown` (`power.rs:203-209`). Ler o atraso no início evitaria isso.
    - `leave_desktop_mode` (`backend.rs:425`) não respeita o estado final. É uma operação HID confirmada pelo usuário,
      fora da lista da D-3.
  - **Ajustes:** mostrar Tela › Ajustes de uma rev C acordada e não ao vivo abre a tela (HELLO, STOP_VIDEO,
    STOP_MEDIA, 0x86 e as listagens), como já faz a aba Armazenamento (`app.js:208-216`). Isso para a reprodução
    autônoma dela.
  - **Guias:**
    - A tabela do §16 omite o GET_FILE_SIZE, o STOP e o GET_STORAGE_INFO que vão antes das ações
      (`protocol-turing-rev-c.md:610-611`).
    - O logout no Linux não aparece como não coberto.
    - O keepalive aparece como fato, mas a confirmação no 8.8" ficou para o PR (D-5).
    - O pt-BR diz "temporizador de descanso" na tabela, e a UI diz "Temporizador de repouso".
    - `bezel standby set keep --yes` numa tela não rev C sem registro responde "already left as it is" em vez de
      "not supported" (`crates/bezel-cli/src/standby.rs:480-486`).

## DoD Checklist (gate 8)
| # | Criterion | Source | Type | Status | Evidence |
|---|---|---|---|---|---|
| 1 | Core e driver rev C: plano B e ações = pacotes exatos; OPTIONS inteiro; keepalive; impossível→TURNOFF | CONTEXT | Auto | PASS | `standby_` **7 passed** (≥4); core `--test standby` **8 passed** (≥6); saída `OK` |
| 2 | Linux: 1 inibidor *delay*; `true` aplica e só então fecha o fd; `keep`/ticks/reconexão sem chamadas; `liveScreen` fica; prazo; `false` retoma | CONTEXT | Auto | PASS | `power::tests::linux_` **8 passed** (≥6), os 8 nomes conferidos; `OK` |
| 3 | Windows: fim de sessão aplica, sair não; `RunEvent::Exit`; `bezel-power` no clippy msvc | CONTEXT | Auto | PASS | `--exact …a_session_end_applies_the_choice_and_a_quit_does_not`: 1 passed; `RunEvent::Exit` em `lib.rs`; clippy msvc do `bezel-power` exit 0; `OK`. O studio no Windows falha fora desta linha (B1) |
| 4 | Studio: `Confirm::No` sem chamadas; `keep` desfaz; catálogo compartilhado; álbum lista/adiciona/remove | CONTEXT | Auto | PASS | `standby::tests` **10 passed** (≥5); `OK` |
| 5 | CLI: sem `--yes` nada; com `--yes` plano B e registro; `album add` com EXIF nas duas orientações | CONTEXT | Auto | PASS | `bezel --test standby` **6 passed** (≥5); `OK` |
| 6 | UI: lógica pura, i18n, 5 e2e nomeados × 4 projetos com axe | CONTEXT | Auto | PASS | tap `# fail 0`; `test:unit` 264/0; `e2e-passed`: "5 tests × 4 projects, 20/20 runs passed with axe"; `OK` |
| 7 | Guardas: comandos, fonte, sem logger, início silencioso; monitor vê só logind; nenhum pacote novo | CONTEXT | Auto | PASS | 4 `--exact` **4 passed**; `speaks_only_to_logind_on_its_bus` 1 passed; lock só `+name = "bezel-power"`; `studio-starts-silent: OK: in 4 runs of 12s …`; `OK` (instabilidade do KWin em W8) |
| 8 | Guia en/pt-BR, §19/§20, CHANGELOG, check-docs | CONTEXT | Auto | PASS | `OK`; `check-docs: 16 pages in English and Portuguese, links and privacy checked; all checks passed` |
| P1 | `cargo test --workspace` | PROJECT | Auto | PASS | 1030/0/12, exit 0 |
| P2 | Cobertura ≥ 80% | PROJECT | Auto | PASS | 94.87% (comando do PROJECT, sem filtro), exit 0 |
| P3 | Sem TODO/FIXME sem issue | PROJECT | Auto | PASS | `OK` |
| P4 | CHANGELOG por release | PROJECT | Manual | MANUAL_REQUIRED | `[Unreleased]` tem a entrada; o `## [versão]` sai no corte de release |
| P5 | README descreve o comportamento atual | PROJECT | Manual | MANUAL_REQUIRED | Revisar o diff do README no PR (escolha "quando o computador desligar", guias) |

Fica para o PR (CONTEXT, não é blocker): desligar de verdade com o 8.8" em cada opção e religar; desligar no Windows;
visual; fotos em pé nas duas orientações no 8.8"; tema parado com temporizador de 1 min não dorme (D-5).

## Recommendation
Corrigir B1 e B2 e rodar de novo `/jdi-do power-off-standby` → `/jdi-verify power-off-standby`.

- **B1:** é uma linha em `power.rs:178`, ou um `Drop` no `Inhibitor`. Vale confirmar com o job `rust-windows` antes
  do PR, porque o clippy do studio para Windows só roda lá.
- **B2:** levar a decisão de substituir para o servidor no `album_add`, como fazem a CLI e a aba Armazenamento.

Na mesma rodada, valem W1 (fechar durante o envio) e W4/W5 (texto do guia sobre brilho e plano B). Os demais avisos
podem ficar para depois.

O resto da fase está sólido:
- O fluxo de desligamento segue a D-3: estado final em todos os pontos onde um link nasce ou é emprestado, job
  cancelado e esperado, prazo de `delay_max − 0,5 s`, `keep` solta na hora, `false` re-inibe e retoma, e o
  `liveScreen` fica.
- As sequências de pacotes batem com o §19 no simulador.
- As guardas de privacidade não afrouxaram: o monitor do barramento vê só `Hello`/`AddMatch`/`Inhibit`/`Get`.

## DoD Critic (enhanced)

- DoD row «3»: só o `bezel-power` passa pelo clippy msvc e o studio falha em `power.rs:178` (B1); além disso `grep -q 'RunEvent::Exit'` casa o comentário de `lib.rs:324`, e trocar `lib.rs:331` por `RunEvent::ExitRequested { .. }` passaria — o tao não envia `ExitRequested` no `WM_ENDSESSION`, e nenhum teste roda `on_run_event` (`power/tests.rs:430` chama `at_exit` direto).
- DoD row «4»: `standby/tests.rs:518` (`a_photo_replaces_its_namesake...`) exige que `album_add(..., Confirm::Yes)` sobrescreva `sd/image/beach.png` em silêncio; `album_add` (`standby.rs:365-447`) nunca expõe `plan.replaces` — a violação da D-4 (3) (B2) passa e fica travada pelo teste.
- DoD row «5»: `tests/standby.rs` nunca observa o que chegou à tela `--fake`: "nada enviado" só pelo texto do stderr (`:187`) e o plano B só pelo eco do stdout; `:271-272` passa `--brightness 50` e confere só "start mode 1, sleep timer off" — mover `link.set_brightness(level)` (`src/standby.rs:492`) para depois do `choose` (`:494`) gravaria o OPTIONS com o brilho errado (D-2 (3)) e passaria; o teste de lib em `src/standby.rs:895` olha brilho e Options em logs separados.

**Verdict:** BLOCKED
