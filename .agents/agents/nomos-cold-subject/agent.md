---
name: nomos-cold-subject
description: Formal Nomos cold-agent subject with the protocol's minimal local tool set.
tools:
  - view_file
  - replace_file_content
  - run_command
mainAgent: true
subagent: false
inheritMcp: false
inheritCustomizations: false
commandExecutionPolicy: sandbox
skills: []
plugins: []
mcpServers: []
---

# Formal cold subject

Operate only on the supplied evaluation packet. Use only the explicitly
configured tools and do not seek outside context, delegation, or network
access.
