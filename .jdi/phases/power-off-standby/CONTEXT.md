# Phase 12: Tela com o computador desligado (apagada, vídeo do cartão ou álbum) — Context  (slug: power-off-standby)

## Goal
Por tela: apagar, vídeo guardado em loop ou álbum do cartão quando o PC desliga (Linux e Windows), com plano B na tela.

## Locked decisions
- D-1: pedido de 2026-10-03 e fatos medidos no 8.8" (card).
- D-2: rev C; `keep` (padrão)/`off`/`video`/`album` no catálogo compartilhado; mudar = tela conectada + `Confirm::Yes` e grava o plano B (OPTIONS inteiro); `keep` desfaz.
- D-3: Linux: inibidor *delay* + `PrepareForShutdown`; Windows: `RunEvent::Exit` + `SM_SHUTTINGDOWN`; estado final, prazo, `liveScreen` fica; crate `bezel-power`.
- D-4: álbum = `sd/image`; foto com EXIF, enquadrada na orientação da tela, PNG nativo.
- D-5: keepalive rev C: 30 s sem tráfego → parcial mínimo.
- D-6: Tela › Configurações; `bezel standby`; provas e critérios de ausência.
- D-7: contagens do DoD são mínimas (≥ N, 0 falhas); Windows provado por teste + clippy msvc local, CI no PR.

## Canonical refs
- Card: pedido de 2026-10-03; `D-2026-10-03-power-off-standby-{1..6}`, device-protocols-2, gif-sticker-search-15; `protocol-turing-rev-c.md` §6.2, §16, §19, §20

## Out of scope
- `.jdi/todos/2026-10-03-power-off-standby.md` (suspensão, `bezel run`, logout…); slideshow: fase `slideshow-builder`.

## Definition of Done

### Auto-verifiable
- [ ] Core e driver rev C (simulador de firmware): plano B e ações = sequência exata dos pacotes medidos, nenhum outro; OPTIONS inteiro; keepalive; impossível→TURNOFF
      **Verify:** `cargo test -p bezel-devices --locked --lib -- driver::turing_rev_c::tests::standby_ 2>&1 | grep -qE 'test result: ok\. ([4-9]|[1-9][0-9]+) passed' && cargo test -p bezel-core --locked --test standby 2>&1 | grep -qE 'test result: ok\. ([6-9]|[1-9][0-9]+) passed' && echo OK`
      **Source:** CONTEXT
- [ ] Linux (`setup` real, `dbus-daemon` privado, logind falso): 1 inibidor *delay*; `true` aplica e só então fecha o fd; `keep`, e após a ação com ticks e reconexão devidos: zero chamadas no fake (estado final; contorno deliberado fora, D-6); `liveScreen` fica; prazo com tela travada; `false` retoma
      **Verify:** `cargo test -p bezel-studio --locked --lib -- power::tests::linux_ 2>&1 | grep -qE 'test result: ok\. ([6-9]|[1-9][0-9]+) passed' && echo OK`
      **Source:** CONTEXT
- [ ] Windows: fim de sessão aplica, sair não; `RunEvent::Exit` ligado; `bezel-power` compila e passa no clippy para Windows (o CI do PR roda o resto)
      **Verify:** `cargo test -p bezel-studio --locked --lib -- --exact power::tests::a_session_end_applies_the_choice_and_a_quit_does_not 2>&1 | grep -q 'ok. 1 passed' && grep -q 'RunEvent::Exit' apps/bezel-studio/src-tauri/src/lib.rs && cargo clippy -p bezel-power --locked --target x86_64-pc-windows-msvc --all-targets -- -D warnings && echo OK`
      **Source:** CONTEXT
- [ ] Studio: com `Confirm::No` zero chamadas e catálogo igual; `keep` desfaz; lê o catálogo compartilhado; álbum lista, adiciona enquadrado na orientação da tela, remove confirmado
      **Verify:** `cargo test -p bezel-studio --locked --lib -- standby::tests 2>&1 | grep -qE 'test result: ok\. ([5-9]|[1-9][0-9]+) passed' && echo OK`
      **Source:** CONTEXT
- [ ] CLI (binário e catálogo em disco): sem `--yes` nada enviado nem registrado; com `--yes` plano B e registro que o studio lê; `album add` horizontal e vertical, com EXIF
      **Verify:** `cargo test -p bezel --locked --test standby 2>&1 | grep -qE 'test result: ok\. ([5-9]|[1-9][0-9]+) passed' && echo OK`
      **Source:** CONTEXT
- [ ] UI: lógica pura, i18n e os 5 testes nomeados nos 4 projetos com axe
      **Verify:** `set -o pipefail; cd apps/bezel-studio && node --test --test-reporter=tap tests/ui/standby.test.mjs | awk '/^# pass [1-9]/{p=1} /^# fail 0$/{f=1} END{exit !(p&&f)}' && npm run test:unit >/dev/null && node scripts/e2e-passed.mjs "standby › four choices, each explained" "standby › plan B asks first" "standby › keep undoes" "standby › album: vertical and horizontal" "standby › album: remove asks first" && echo OK`
      **Source:** CONTEXT
- [ ] Guardas: comandos, fonte, sem logger, início silencioso (shim); um monitor do barramento vê só mensagens ao logind (outro crate falando D-Bus de propósito: fora, D-6); nenhum pacote externo novo
      **Verify:** `cargo test -p bezel-studio --locked --lib -- --exact tests::every_command_is_allowed_by_name tests::nothing_in_the_app_forges_an_invocation tests::the_studio_installs_no_logger tests::the_app_setup_sends_nothing_at_start 2>&1 | grep -q 'ok. 4 passed' && cargo test -p bezel-power --locked --lib -- --exact logind::tests::speaks_only_to_logind_on_its_bus 2>&1 | grep -q 'ok. 1 passed' && ! git diff "$(git merge-base HEAD origin/main)" -- Cargo.lock | grep '^+name = ' | grep -qv '"bezel-power"' && bash scripts/ci/studio-starts-silent.sh | grep -q '^studio-starts-silent: OK: ' && echo OK`
      **Source:** CONTEXT
- [ ] Guia en/pt-BR, §19/§20 do protocolo, CHANGELOG e check-docs
      **Verify:** `f=docs/reverse-engineering/protocol-turing-rev-c.md; grep -q 'bezel standby' docs/user/power-off.md && grep -q 'bezel standby' docs/user/pt-BR/power-off.md && grep -q '"power-off.md"' scripts/ci/check-docs.sh && [ "$(sed -n '/^## 19\./,/^## 20\./p' $f | grep -cE '^\| (Start mode [12]|Sleep timer|RESTART 0x84|Video after the host) ')" -ge 5 ] && ! grep -q 'which file start modes' $f && sed -n '/^## \[Unreleased\]/,/^## \[[0-9]/p' CHANGELOG.md | grep -qiE 'shuts? down' && bash scripts/ci/check-docs.sh >/dev/null && echo OK`
      **Source:** CONTEXT

### Manual
- _(none)_

## Deferred to PR review
- PC desligado de verdade com o 8.8" em cada opção, e religado retomando o tema.
- Desligar de verdade no Windows.
- Visual: a escolha e o gerenciador do álbum.
- 8.8": fotos do álbum em pé nas duas orientações; tema parado com timer de 1 min não dorme (D-5).

## Notes
- Ordem: core → devices → `bezel-power` → studio → UI → CLI → docs; `install-local.sh` ao fim; só fakes.
