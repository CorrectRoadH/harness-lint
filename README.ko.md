# harness lint

[![CI](https://img.shields.io/github/actions/workflow/status/CorrectRoadH/harness-lint/ci.yml?branch=main&label=CI&style=flat-square)](https://github.com/CorrectRoadH/harness-lint/actions/workflows/ci.yml)
[![release](https://img.shields.io/github/v/release/CorrectRoadH/harness-lint?label=release&style=flat-square)](https://github.com/CorrectRoadH/harness-lint/releases)
[![version](https://img.shields.io/badge/version-0.2.3-orange?style=flat-square)](Cargo.toml)
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

## Agent용 리포지토리 초기화

```text
READ https://raw.githubusercontent.com/CorrectRoadH/harness-lint/refs/heads/main/INIT.md and install harness lint for this code repo
```

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

## 로컬 규칙

프로젝트별 커스텀 규칙은 기본적으로 `rules/*.md`에 둡니다. 다른 위치를 사용하려면 `harness.toml`에서 설정할 수 있습니다.

```toml
[rules]
local = ["custom-rules"]
```

`harness-lint rule create`는 설정된 첫 번째 로컬 규칙 디렉터리에 새 규칙을 작성합니다.

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
