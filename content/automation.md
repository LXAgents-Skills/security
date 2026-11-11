---
name: Read this tool every session
description: Read this tool every session, first — it lists every tool and the one condition that activates each, so you load only what the request needs.
---

# Automation

Read this once, at the start of every session. It is the only tool you read unprompted.

* Call no other tool yet. Match each step of the request against the conditions below, and
  call a tool only when its condition is true, and only that tool.
* Never call a tool "just in case". Each tool is complete on its own, so one call gives you
  everything that tool has.
* Several conditions can be true at once: a Next.js app with a React frontend needs both of
  those guides. Call them in the order the work reaches them, not all at the start.
* Read the general guide for a language before its framework guide. They are written to be
  read in that order.
* No tool takes an argument.

## The workflow

- `skill` - the request asks for security best practices, a security review or report, or secure-by-default code in Python, JavaScript/TypeScript, or Go. It names the workflow for finding the languages and frameworks in scope.

## Go

- `golang_general_backend_security` - the code in scope is Go: a backend, a service, or anything on `net/http`.

## JavaScript and TypeScript, server side

- `javascript_express_web_server_security` - the code in scope is an Express server on Node.js.
- `javascript_typescript_nextjs_web_server_security` - the code in scope is a Next.js application: its server components, route handlers, middleware, or server actions.

## JavaScript and TypeScript, browser side

- `javascript_general_web_frontend_security` - the code in scope runs in the browser, with or without a framework. Read it before any frontend framework guide.
- `javascript_jquery_web_frontend_security` - the frontend uses jQuery.
- `javascript_typescript_react_web_frontend_security` - the frontend uses React, including the client side of a Next.js application.
- `javascript_typescript_vue_web_frontend_security` - the frontend uses Vue.

## Python

- `python_django_web_server_security` - the code in scope is a Django application.
- `python_fastapi_web_server_security` - the code in scope is a FastAPI application.
- `python_flask_web_server_security` - the code in scope is a Flask application.
