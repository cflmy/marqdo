---
type: Constraint
id: C-014
title: Centralized config resolution
status: stable
applies_to:
  - config.resolve
---

# Database and config access

All configuration resolution must go through `resolve_config`.

Direct ad-hoc config parsing in handlers is prohibited.
