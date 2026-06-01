---
id: python-pep8.no-wildcard-import
title: Avoid wildcard imports
language: python
level: warn
status: warn
tags: [python, pep8, imports]
---

# Avoid wildcard imports

Import explicit names so module dependencies stay readable and tooling can trace references.

```grit
language python
`from $module import *`
```

## Bad

```python
from models import *
```

## Good

```python
from models import User, Team
```
