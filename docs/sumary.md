# Interpreter Design: Lessons and Principles

The main lesson from this implementation is to keep three concepts separate:
**values, control flow, and environments**. Most of the difficult bugs came from
treating two of these as the same thing.

## 1. Values: What did this expression produce?

Expressions evaluate to values:

```rust
fn execute_expr(
    &mut self,
    expr: &Expr,
    env: &EnvRef,
) -> MSPResult<Value>;
```

Examples include numbers, strings, arrays, and the result of a function call.

Statements usually perform an action without producing a useful value:

```text
x = 10;       // changes a binding
say(10);      // evaluates a call and discards its result
fn say(x) {}  // registers a function
```

An early mistake was returning `Flow::Return` for an expression statement. That
would make an ordinary assignment or call exit its enclosing function. Only an
explicit `return` statement should create that signal under the current language
rules.

## 2. Flow: What should execute next?

```rust
enum Flow {
    Normal,
    Break,
    Continue,
    Return(Value),
}
```

These are successful execution outcomes, so they belong inside `Ok(...)`.
Runtime failures belong in `Err(...)`.

```rust
fn execute_stmt(...) -> MSPResult<Flow>;
```

The important principle is:

> A construct handles the control-flow signals it owns and propagates the others.

| Executor | `Normal` | `Break` | `Continue` | `Return(value)` |
|---|---|---|---|---|
| Statement sequence / block | Execute next statement | Propagate | Propagate | Propagate |
| `if` | Finish normally | Propagate | Propagate | Propagate |
| `for` | Run update, then check condition | Exit loop | Run update, then check condition | Propagate |
| `while` | Check condition again | Exit loop | Check condition again | Propagate |
| Function call | Produce `null` on fallthrough | Reject | Reject | Produce the returned value |
| Script | Execute next statement | Reject | Reject | Reject |

`execute_stmts()` can ignore `Normal` because it means “this statement finished;
keep going”:

```rust
for stmt in stmts {
    match self.execute_stmt(stmt, env)? {
        Flow::Normal => {}
        flow => return Ok(flow),
    }
}
Ok(Flow::Normal)
```

It must propagate the other signals immediately. Otherwise, statements after a
`return`, `break`, or `continue` would execute.

The original Rust `continue` continued the Rust loop over statements, rather than
the language's loop. Explicit signals let the correct enclosing executor make
that decision.

This also gives nested loops the right behavior: the inner loop consumes its own
`Break`; the outer loop receives `Normal`.

## 3. Environments: Which binding does this name refer to?

```rust
type EnvRef = Rc<RefCell<Env>>;

struct Env {
    parent: Option<EnvRef>,
    values: HashMap<String, Value>,
}
```

Each part has one purpose:

- `HashMap` stores this scope's bindings.
- `parent` supports lookup in enclosing scopes.
- `Rc` allows several scopes or functions to share an environment.
- `RefCell` allows mutation through those shared handles.

Lookup should search the current scope first, then parents. Returning an owned
`Value` avoids references tied to temporary borrows:

```rust
fn get(&self, name: &str) -> Option<Value>;
```

Separate defining a binding from updating one:

| Operation | Behavior |
|---|---|
| Define | Insert into the current scope |
| Lookup | Find the nearest binding |
| Assign existing | Update the nearest binding |
| General assignment rule for this language | Update the nearest binding; create locally if none exists |

Parameters must be defined locally even if a parent has the same name.

The `append` bug illustrated this distinction: lookup found the enclosing array,
but writing with local `store()` created a new iteration-local array. The
enclosing array stayed unchanged.

## 4. Cloning an environment handle does not create a scope

With `EnvRef`:

```rust
let same_scope = env.clone();
```

This clones the `Rc` handle. Both handles refer to the same environment, so they
see the same mutations.

Use it when:

- A function captures its definition environment.
- A child environment needs to retain its parent.
- An owned handle is needed without holding a borrow of `self.env`.

Creating a scope is a different operation:

```rust
let child_scope = Env::new(Some(env.clone()));
```

This creates an empty local map linked to the enclosing scope.

When a method already accepts `&EnvRef`, simply pass `env`. Writing `&env.clone()`
is usually unnecessary.

## 5. Create scopes according to language rules

A consistent scope policy for this language would be:

| Situation | Environment to use |
|---|---|
| Script execution | Persistent global environment |
| Function definition | Register in and capture the current environment |
| Function arguments | Evaluate in the caller's environment |
| Function invocation | Fresh parameter environment whose parent is the definition environment |
| Ordinary block / selected `if` branch | Child of the current environment |
| `for` initializer, condition, update | One shared loop environment |
| Each loop-body iteration | Fresh child of the loop environment |

For a `for` loop:

```text
enclosing scope
  └─ loop scope: initializer variable
       └─ body scope: new each iteration
```

The loop variable survives across iterations. Body-local variables do not.

These are language-design choices, not universal requirements. The
implementation must apply the chosen rules consistently.

Give each helper a clear scope contract. For example:

```text
execute_stmts: execute in the supplied environment
execute_block: create a child, then execute its statements
function call: create parameter scope, execute body in that scope
```

This avoids accidentally creating two scopes—or none. If the loop executor
creates the body scope itself, it should execute the body's statements directly
in that scope instead of asking another helper to create it again.

## 6. Keep mutable borrows away from recursive execution

Evaluate code without holding a `RefCell` borrow:

```rust
let function = env.borrow().get(name);
// Borrow has ended.

let argument = self.execute_expr(expr, env)?;
```

Avoid holding `borrow()` while evaluating an argument or body that may call
`borrow_mut()` on the same environment. That causes a runtime borrow panic.

For mutation, the usual pattern is:

1. Resolve or read what is needed.
2. Release the borrow.
3. Evaluate expressions.
4. Borrow briefly to write.

Also define evaluation order. Arguments may have side effects, so taking an array
snapshot before evaluating them and writing it afterward can overwrite those
effects.

## 7. Mistakes worth remembering

| Mistake | Consequence | Guiding rule |
|---|---|---|
| Using ordinary values for control flow | Nested returns and loop exits disappeared | Give control flow an explicit type |
| Registering functions in a temporary child | Later calls could not find them | Define functions in their enclosing scope |
| Parenting every scope to globals | Nested locals became inaccessible | Use the actual enclosing environment |
| Executing the body inside argument binding | Only the first argument was bound | Bind all arguments before execution |
| Filtering out evaluation errors | Invalid programs appeared to succeed | Propagate errors with `?` |
| Debug-printing captured environments | Recursive formatting caused stack overflow | Debug functions by name, not their entire environment |
| Unchecked indexes and argument counts | Rust panics escaped the interpreter | Turn invalid language operations into evaluation errors |
| Assuming parser support implied runtime support | Accepted syntax failed during execution | Test the full lexer → parser → interpreter path |

Captured environments also create reference cycles when an environment contains
a function that captures it. `Rc` does not collect those cycles automatically;
memory management will need attention as the interpreter grows.

## 8. Principles for extending the interpreter

Before implementing a construct, specify four things:

1. Its value semantics: what values it accepts or produces.
2. Its scope semantics: where names are read, defined, and updated.
3. Its control-flow semantics: which signals it consumes or propagates.
4. Its failure semantics: what produces a language error rather than a Rust panic.

Then test observable behavior. Useful tests distinguish nearby mistakes:

- An accumulator updated across iterations.
- `continue` followed by a side effect.
- `return` inside an `if` inside a loop.
- Recursive calls.
- Functions called from a different scope than where they were defined.

That discipline makes each executor small: evaluate its operands, apply its scope
rules, handle the signals it owns, and propagate everything else.
