import Lean
import ReplayPolicy.Cursor
import ReplayPolicy.Identity

/-! JSON-lines ABI for the executable proved policy. JSON transport is an
explicit adapter; all validation decisions call the proved functions. -/
open Lean
open Marqdo.ReplayCursor

structure StepWire where
  q : String
  r : String
  deriving FromJson
structure PolicyCommand where
  op : String
  steps : List StepWire := []
  value : String := ""
  deriving FromJson

def reasonName : Reason → String
  | .missing => "missing"
  | .request => "request"
  | .reply => "reply"
  | .surplus => "surplus"
  | .phase => "phase"

def report (out : IO.FS.Stream) (c : Cursor String String) (allow : Bool)
    (reason : Option Reason := none) : IO Unit := do
  let why := match reason,c.phase with
    | some r,_ => some r
    | none,.stopped r => some r
    | _,_ => none
  let json := Json.mkObj [
    ("policy",toJson PolicyIdentity.digest), ("allow",toJson allow), ("matched",toJson c.matched),
    ("remaining",toJson c.remaining.length),
    ("reason",toJson (why.map reasonName))]
  out.putStrLn json.compress
  out.flush

def readCommand (input : IO.FS.Stream) : IO (Option PolicyCommand) := do
  let line ← input.getLine
  if line.isEmpty then return none
  let json ← IO.ofExcept (Json.parse line)
  return some (← IO.ofExcept (fromJson? json))

partial def loop (input out : IO.FS.Stream) (c : Cursor String String) : IO Unit := do
  let some command ← readCommand input | return
  match command.op with
  | "prepare" =>
    let (next,allowed) := prepare c command.value
    report out next allowed
    loop input out next
  | "receive" =>
    let (next,accepted) := receive c command.value
    report out next accepted
    loop input out next
  | "finish" =>
    let why := finish c
    report out c why.isNone why
  | _ => throw (IO.userError "replay policy: unknown command")

def main : IO Unit := do
  let input ← IO.getStdin
  let out ← IO.getStdout
  let some command ← readCommand input | throw (IO.userError "replay policy: missing initialization")
  unless command.op == "init" do throw (IO.userError "replay policy: expected initialization")
  let c : Cursor String String := ⟨command.steps.map (fun e => (e.q,e.r)),0,.ready⟩
  report out c true
  loop input out c
