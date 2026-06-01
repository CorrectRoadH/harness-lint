---
id: python-typing.no-any-import
title: Avoid importing Any as a default escape hatch
language: python
level: warn
status: warn
tags: [python, typing]
---

# Avoid importing Any as a default escape hatch

Reach for concrete models, protocols, unions, or `object` before introducing `Any`.

```grit
language python
`from typing import Any`
```

## Bad

```python
from typing import Any
```

## Good

```python
from typing import Protocol
```
