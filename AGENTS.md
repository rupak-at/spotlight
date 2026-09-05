# Project workflow

After every completed logical change, run the relevant checks, then stage and commit the relevant files. Do not leave completed project changes uncommitted at handoff.

Use these commands, replacing the paths and message with the actual change:

```sh
git add <relevant-file-or-directory> [<another-relevant-path> ...]
git commit -m "<type>: <clear description of the completed change>"
```

- Use focused, descriptive messages such as `feat: add folder search`, `fix: prevent stale results from launching`, or `docs: explain Ubuntu shortcut setup`.
- Stage only files belonging to the current change; preserve unrelated user work.
- Verify the commit succeeded and include the commit hash when reporting completion.
- If sandbox permissions block Git writes, request the necessary tool escalation and complete the authorized commit.
