---
markup: occlusion
title: Call one function: the same document is the tool
description: v2.2.0 marqdo call runs a function inside any .mq.md. The explanation, the implementation and the call are one page.
---

# main

v2.2.0 adds one command: `marqdo call`. It adds no Markdown marker and no skill system. An ordinary .mq.md file is already a document, a piece of knowledge and a program. What was missing is a way to call one of its functions from outside.

The usual command runs the whole file. Functions inside it run only when the entry calls them. Trying one calculation meant writing an entry, a print and a check. The smallest unit you can execute is now a function.

```text
marqdo call examples/calculator.mq.md 计算器.加法 a=2 b=3
```

Standard output is 5. `--json` is for programs and agents: success carries the result, failure carries a diagnostic code, and the process exits non-zero. `--backend tree` and `--backend bytecode` both run that function. Another entry in the file is not used as a fallback.

The second half of this page is itself a calculator. This command calls this page:

```text
marqdo call public/features/22-call.mq.md calculator.add a=2 b=3
```

---

The change is how the file is used.

A person reads the sentences next to the function and sees what add computes. The compiler reads the same markers and checks parameters and the return. A developer composes the function into another program. A test passes inputs to this function and looks at the output, without a wrapper whose only job is to print. An agent reads what the function is for and runs the implementation that is already there, instead of reading the page and rewriting it.

Those four uses are one call. Knowledge becomes a tool because knowledge can already run, now at function scope. The file does not need a special name, and it does not need to declare itself a skill.

---

The command keeps a boundary.

An external call runs a pure function. Writing files, starting a process, using the network or loading a plugin is refused before the function runs, so the side effect does not happen. Wider permission stays on the checks the project already has. An instance method needs its receiver; this command does not invent one.

The function must resolve to exactly one declaration. No match, or one short name that hits two functions, is a structured error. The command does not pick the first hit. Arguments bind through the signature and contract that the function already has. Missing, unknown and ill-typed arguments are diagnostics.

After editing a function, call it again with the inputs that failed. A passing check is a reason to keep the edit. A test does not prove every input. It gives the edit a check that can be repeated.

# calculator

This is an ordinary Marqdo document.

It holds a little reusable arithmetic.

## add

Add the numbers `a` and `b`.

*[result](a + b)*

## multiply

Multiply the numbers `a` and `b`.

*[result](a * b)*
