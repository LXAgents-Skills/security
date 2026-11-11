# 2.0.2

**Released:** 2026-11-11

Every tool ID now starts with `lxagents_skills_security_`. A new tool,
`lxagents_skills_security_automation`, lists every other tool and the condition that activates it.
**The text each tool returns is unchanged, and so are the routes, the environment variables and
the bins.** The owner chose a patch number for this release. Any client that calls a tool by its
old name has to change, so treat it as a breaking change for those clients.

**Clients must:**

1. **Call tools by their new IDs.** Every old name gets the prefix:
   `skill` → `lxagents_skills_security_skill`,
   `python_flask_web_server_security` → `lxagents_skills_security_python_flask_web_server_security`.
2. **Four JavaScript guides were also shortened** so that every ID fits in 64 characters:

   | Old | New |
   |---|---|
   | `javascript_general_web_frontend_security` | `lxagents_skills_security_js_general_web_frontend_security` |
   | `javascript_typescript_nextjs_web_server_security` | `lxagents_skills_security_js_ts_nextjs_web_server_security` |
   | `javascript_typescript_react_web_frontend_security` | `lxagents_skills_security_js_ts_react_web_frontend_security` |
   | `javascript_typescript_vue_web_frontend_security` | `lxagents_skills_security_js_ts_vue_web_frontend_security` |

3. **Start at `lxagents_skills_security_automation`** rather than at `skill`.
4. **Restart any open session**, so that the client reloads the tool list.

## Added

- **`content/automation.md`**: the routing hub, served as `lxagents_skills_security_automation`. A
  test fails if the hub and the served tools disagree.
- **`ID_PREFIX`** and **`MAX_PUBLISHED_ID_LENGTH`**, exported from
  `src/instructions/from-content.js`.

## Changed

- **Tool IDs** are now `{org_name}_{repo_name}_{base}`, using underscores only.
- **`TOOL_NAME`** accepts up to 128 characters, and the test suite caps every published ID at 64.
- **The server `instructions`** now point to the automation tool first.
- **The tool count** goes from 11 to 12. Tests go from 63 to 64.
