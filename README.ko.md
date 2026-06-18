# harness lint

[![CI](https://img.shields.io/github/actions/workflow/status/CorrectRoadH/harness-lint/ci.yml?branch=main&label=CI&style=flat-square)](https://github.com/CorrectRoadH/harness-lint/actions/workflows/ci.yml)
[![release](https://img.shields.io/github/v/release/CorrectRoadH/harness-lint?label=release&style=flat-square)](https://github.com/CorrectRoadH/harness-lint/releases)
[![rust](https://img.shields.io/badge/rust-2024-blue?style=flat-square)](Cargo.toml)
[![homebrew](https://img.shields.io/badge/homebrew-CorrectRoadH%2Ftap-fbb040?style=flat-square)](https://github.com/CorrectRoadH/homebrew-tap)

[English](README.md) | [简体中文](README.zh.md) | [日本語](README.ja.md) | [한국어](README.ko.md)

harness-lint는 Harness Engineering을 위한 차세대 lint 도구입니다. vibe coding에서는 여러 번 고쳐 말하거나 `AGENTS.md`에 지시를 적어 두어도 AI가 그 지시를 무시하는 일이 자주 생깁니다. 이 도구는 Lint Driven Development 방식으로 그 문제를 해결합니다. 사용자가 AI Agent에게 하지 말아야 할 일을 말하면, 먼저 그 요구를 고정된 lint 규칙으로 바꾸고, 빠르고 엄격한 검사로 AI가 같은 실수를 반복하지 못하게 합니다.

기존 lint 도구와 비교하면 harness lint의 규칙은 사람이 읽고 이해하기 쉽게 작성됩니다. 또한 AI coding 워크플로와 베스트 프랙티스에 맞게 설계되어 있습니다.

## 설치

```sh
brew install getgrit/tap/grit
brew install CorrectRoadH/tap/harness-lint
```

함께 쓰는 Codex skill을 설치하려면:

```sh
npx skills add CorrectRoadH/harness-lint
```

## Agent용 리포지토리 초기화

```text
READ https://raw.githubusercontent.com/CorrectRoadH/harness-lint/refs/heads/main/INIT.md and install harness lint for this code repo
```

## Agent 플러그인 (Claude Code 및 Codex)

`AGENTS.md`에 적은 정적 지시는 한 번만 읽히고, agent가 실제로 작업하는 순간과는 멀리 떨어져 있습니다. [`plugins/`](plugins/)의 플러그인은 대신 라이프사이클 훅을 사용해 세션마다 Lint Driven Development 지침을 다시 주입하고, 프롬프트마다 `harness-lint check --changed`를 실행해 **현재 실제 위반**을 agent에게 전달함으로써 다음 줄을 작성하기 전에 고치도록 합니다.

Claude Code:

```text
/plugin marketplace add CorrectRoadH/harness-lint
/plugin install harness-lint@harness-lint
```

Codex (프로젝트 로컬 훅을 `.codex/`에 배치):

```sh
mkdir -p .codex/hooks
cp plugins/codex/hooks.json .codex/hooks.json
cp plugins/codex/hooks/*.sh .codex/hooks/
chmod +x .codex/hooks/*.sh
```

둘 다 `/harness-lint-capture` 명령을 함께 제공합니다. 세션의 피드백을 검토해 재사용 가능한 지적을 규칙으로 정착시킵니다(LDD의 나머지 절반). 자세한 내용은 [`plugins/README.md`](plugins/README.md)와 `~/.codex` 전역 설정을 참고하세요.

## 자주 쓰는 명령

```sh
harness-lint check --changed
harness-lint check --all
harness-lint rule list
harness-lint search "python typing"
harness-lint list --available
harness-lint install python
harness-lint install python-pep8
harness-lint outdated
harness-lint update
harness-lint remove python
```

## 설정 데모

`harness.toml`은 어떤 파일을 검사할지, 로컬 규칙을 어디에 둘지, 어떤 규칙 팩을 설치할지, 어떤 규칙 결과를 다르게 다룰지를 제어합니다.

```toml
# 생성된 설정에 표시되는 선택적 프로젝트 이름.
[project]
name = "my-service"

# 기본 lint 동작.
[lint]
# warn은 문제를 보고하고, error는 검사를 실패시킵니다.
default_level = "warn"
# `harness-lint check --changed`에서 사용합니다.
changed_base = "origin/main"
# 실행 간에 파일 단위 결과를 재사용합니다.
cache = true

# 프로젝트 소유의 로컬 규칙 파일.
[rules]
local = ["rules"]

# 설치하고 복원할 공유 규칙 팩.
[packs]
typescript = "github:CorrectRoadH/harness-lint@main#packs/typescript"

# 규칙 파일을 편집하지 않고 한 규칙의 레벨을 변경합니다.
[overrides]
"typescript.no-console-log" = "error"

# 특정 규칙을 끕니다.
[disabled]
rules = ["typescript.no-explicit-any"]

# 모든 규칙에서 이 경로를 건너뜁니다. 아무것도 스캔하지 않습니다.
[ignore]
paths = ["dist/**", "coverage/**"]

# 대부분의 규칙이 건너뛰어야 하지만 일부 규칙이 필요로 하는 이름 있는 파일 영역.
# default_rules = false는 이를 default 영역에서 제거하므로 일반 규칙은
# 스캔하지 않습니다. provides는 공유 팩 규칙이 당신의 레이아웃을
# 하드코딩하지 않고 대상으로 삼을 수 있는 이식 가능한 개념 이름을 나열합니다.
[file_sets.generated]
paths = ["backend/gen/**/*.pb.go", "packages/proto/gen/**"]
default_rules = false
provides = ["generated"]

# 일치하는 경로에 대해서만 한 규칙을 숨깁니다. 다른 규칙은 그 파일을 계속 검사합니다.
[[exceptions]]
rule = "typescript.no-console-log"
paths = ["src/generated/**"]
reason = "Generated SDK code is checked in and emits debug output during local mocks."
```

규칙은 frontmatter의 `runs_on`으로 영역에 옵트인합니다. `runs_on`이 없으면 규칙은 **default** 영역(`default_rules = false` 세트가 요구하지 않는, 보이는 모든 것)을 스캔합니다.

```markdown
---
id: local.proto-no-id-getter
title: Proto messages must generate GetId
language: go
runs_on: ["generated"]   # generated 영역만. 일반 소스는 절대 스캔하지 않음
---
```

### 설정이 어떻게 합성되는가

harness-lint는 세 가지 독립된 질문에 이 순서로 답합니다. 이들을 분리해 두는 것이 위의 손잡이들을 예측 가능하게 쌓을 수 있게 하는 이유입니다.

1. **규칙이 켜져 있는가?** 팩의 기본 비활성 목록과 `[disabled]`는 규칙을 완전히 끕니다. `[overrides]`는 심각도만 바꿉니다. 꺼진 규칙은 나머지를 건너뜁니다.
2. **규칙이 어떤 파일을 스캔하는가?** 리포지토리에서 시작해 우선순위 순서로 적용합니다.
   - 구조적 제외 — `.git`, `node_modules`, `target`, `.harness`, 규칙 디렉터리, 그리고 `.gitignore`된 파일은 절대 스캔 대상이 되지 않으며 무엇도 이를 덮어쓰지 못합니다.
   - `[ignore].paths` — 모든 규칙에서 제거됩니다. 무엇도 다시 옵트인할 수 없습니다.
   - **file sets** — 남은 파일이 분할됩니다. `default_rules = false` 세트는 `default` 영역에서 제거되고, 규칙은 `runs_on`에서 그 세트(또는 그것이 `provides`하는 개념)를 명시했을 때만 도달합니다. `runs_on`이 없는 규칙은 `default`를 스캔합니다.
   - 규칙의 언어와 GritQL `$filename` 술어가 남은 것을 더 좁힙니다.
3. **결과가 보고되는가?** `[[exceptions]]`는 일치하는 경로에서 스캔된 규칙의 진단을 숨깁니다.

`runs_on`은 배타적 범위이지 뒷문이 아닙니다. 규칙이 기본적으로 닫힌 파일 세트에 도달하는 것은 그것을 요청했기 때문이며, 오직 그 규칙만입니다. 세트의 *위치*(`paths`)는 `harness.toml`에서 프로젝트 소유이지만, 규칙의 *대상*은 이식 가능한 이름(`generated`)입니다. 그래서 공유 팩 규칙은 생성된 코드가 어디 있는지 몰라도 `runs_on: ["generated"]`를 출시할 수 있고, 당신은 하나의 `provides`로 둘을 연결합니다. 파일 세트는 자유롭게 이름을 바꿔도 됩니다. 그 `provides`가 개념을 계속 나열하는 한 모든 팩 규칙은 계속 동작합니다. 일반 소스와 영역 둘 다 필요한가요? 둘 다 나열합니다: `runs_on: ["default", "generated"]`.

harness-lint는 자체 설정 무결성도 검사합니다. 더 이상 존재하지 않는 `[[exceptions]]` / `[ignore]` / `[file_sets.*]` 경로, `[ignore]`와 겹치거나 경로가 없는 파일 세트, 알 수 없는 규칙을 명시하는 `[disabled]` / `[overrides]` 항목, 그리고 `runs_on`이 무엇도 제공하지 않는 파일 세트나 개념을 명시하는 모든 규칙 — 이 모두가 보고됩니다(기본은 warn, 파일 세트 / 실행 대상 구조적 오류는 error. id별로 `[overrides]`로 조정합니다).

## 로컬 규칙

프로젝트별 커스텀 규칙은 기본적으로 `rules/*.md`에 둡니다. 다른 위치를 사용하려면 `harness.toml`에서 설정할 수 있습니다.

```toml
[rules]
local = ["custom-rules"]
```

`harness-lint rule create`는 설정된 첫 번째 로컬 규칙 디렉터리에 새 규칙을 작성합니다. 로컬 규칙은 생성 시점에 실행 가능한 GritQL을 포함해야 합니다.

```sh
harness-lint rule create "Avoid print debugging" --language python --grit '`print($value)`'
```

피드백을 신뢰할 수 있는 GritQL pattern으로 표현할 수 없다면 harness-lint rule을 만들지 마세요. 그런 제약은 agent 지침, review checklist, 또는 프로젝트 문서에 남겨두세요.

규칙을 만든 뒤에는 더 넓은 체크에 의존하기 전에, 그 규칙만 실행해서 예상한 파일이 보고되는지 확인하세요. `check`에 path를 넘겨 rule scope를 흉내 내지 마세요. 특정 파일에만 적용해야 한다면 GritQL의 `$filename`으로 표현하세요.

```sh
harness-lint rule verify local.no-print
harness-lint check --all --rule local.no-print
```

규칙 파일 예시:

````markdown
---
id: local.no-print
title: Avoid print debugging
language: python
level: warn
skill: tdd
tags: [local, python]
---

# Avoid print debugging

Use logging instead of committed print calls.

```grit
language python
`print($value)`
```

## Bad

```python
print(user)
```

## Good

```python
logger.info("user=%s", user)
```
````

규칙을 특정 파일 범위로 제한하려면 GritQL에서 `$filename` 조건을 직접 작성하세요.

```grit
`console.log($value)` where {
  $filename <: r".*src/.*\.ts",
  !$filename <: r".*\.test\.ts"
}
```
