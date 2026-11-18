import Lean

namespace Marqdo.ReplayCursor

inductive Reason where
  | missing | request | reply | surplus | phase
  deriving DecidableEq, Repr
inductive Phase where
  | ready | pending | stopped (reason : Reason)
  deriving DecidableEq, Repr
structure Cursor (Q R : Type) where
  remaining : List (Q × R)
  matched : Nat
  phase : Phase
  deriving Repr

def halt (c : Cursor Q R) (why : Reason) : Cursor Q R :=
  match c.phase with
  | .stopped _ => c
  | _ => {c with phase := .stopped why}

variable {Q R S V : Type} [DecidableEq Q] [DecidableEq R]

def prepare (c : Cursor Q R) (request : Q) : Cursor Q R × Bool :=
  match c.phase with
  | .stopped _ => (c,false)
  | .pending => (halt c .phase,false)
  | .ready => match c.remaining with
    | [] => (halt c .missing,false)
    | (wanted,_)::_ => if request = wanted then ({c with phase := .pending},true)
      else (halt c .request,false)

def receive (c : Cursor Q R) (actual : R) : Cursor Q R × Bool :=
  match c.phase with
  | .stopped _ => (c,false)
  | .ready => (halt c .phase,false)
  | .pending => match c.remaining with
    | [] => (halt c .phase,false)
    | (_,wanted)::rest =>
      let consumed := {c with remaining := rest}
      if actual = wanted then ({consumed with matched := c.matched+1, phase := .ready},true)
      else (halt consumed .reply,false)

def finish (c : Cursor Q R) : Option Reason :=
  match c.phase with
  | .stopped why => some why
  | .pending => some .phase
  | .ready => if c.remaining.isEmpty then none else some .surplus

def WellFormed (c : Cursor Q R) : Prop := c.phase = .pending → c.remaining ≠ []

omit [DecidableEq R] in
theorem prepare_wellFormed (c : Cursor Q R) (q : Q) : WellFormed (prepare c q).1 := by
  cases hp : c.phase with
  | stopped why => simp [prepare,hp,WellFormed]
  | pending => simp [prepare,hp,halt,WellFormed]
  | ready =>
    cases hs : c.remaining with
    | nil => simp [prepare,hp,hs,halt,WellFormed]
    | cons event rest =>
      rcases event with ⟨wanted,reply⟩
      by_cases h : q = wanted <;> simp [prepare,hp,hs,h,halt,WellFormed]

omit [DecidableEq Q] in
theorem receive_wellFormed (c : Cursor Q R) (r : R) : WellFormed (receive c r).1 := by
  cases hp : c.phase with
  | stopped why => simp [receive,hp,WellFormed]
  | ready => simp [receive,hp,halt,WellFormed]
  | pending =>
    cases hs : c.remaining with
    | nil => simp [receive,hp,hs,halt,WellFormed]
    | cons event rest =>
      rcases event with ⟨q,wanted⟩
      by_cases h : r = wanted <;> simp [receive,hp,hs,h,halt,WellFormed]

omit [DecidableEq R] in
theorem prepare_stopped (events : List (Q × R)) (n : Nat) (why : Reason) (q : Q) :
    prepare ⟨events,n,.stopped why⟩ q = (⟨events,n,.stopped why⟩,false) := rfl
omit [DecidableEq Q] in
theorem receive_stopped (events : List (Q × R)) (n : Nat) (why : Reason) (r : R) :
    receive ⟨events,n,.stopped why⟩ r = (⟨events,n,.stopped why⟩,false) := rfl

omit [DecidableEq R] in
/-- A permit identifies exactly the head request; no assumption on replies. -/
theorem permit_has_expected_request (c : Cursor Q R) (q : Q)
    (granted : (prepare c q).2 = true) :
    ∃ r rest, c.phase = .ready ∧ c.remaining = (q,r)::rest := by
  cases hp : c.phase with
  | stopped why => simp [prepare,hp] at granted
  | pending => simp [prepare,hp] at granted
  | ready =>
    cases hs : c.remaining with
    | nil => simp [prepare,hp,hs] at granted
    | cons event rest =>
      rcases event with ⟨wanted,reply⟩
      by_cases h : q = wanted
      · subst wanted; exact ⟨reply,rest,rfl,rfl⟩
      · simp [prepare,hp,hs,h] at granted


omit [DecidableEq Q] [DecidableEq R] in
theorem finish_ready (events : List (Q × R)) (n : Nat) :
    finish (Cursor.mk events n .ready) = none ↔ events = [] := by
  cases events <;> simp [finish]

omit [DecidableEq Q] in
theorem matching_reply_consumes_one (q : Q) (r : R) (rest : List (Q × R)) (n : Nat) :
    receive (Cursor.mk ((q,r)::rest) n .pending) r =
      (Cursor.mk rest (n+1) .ready,true) := by
  simp [receive]

omit [DecidableEq Q] in
theorem mismatching_reply_retains_match_count (q : Q) (wanted actual : R)
    (rest : List (Q × R)) (n : Nat) (different : actual ≠ wanted) :
    receive (Cursor.mk ((q,wanted)::rest) n .pending) actual =
      (Cursor.mk rest n (.stopped .reply),false) := by
  simp [receive,different,halt]

omit [DecidableEq R] in
theorem prepare_preserves_history (c : Cursor Q R) (q : Q) :
    (prepare c q).1.remaining = c.remaining ∧ (prepare c q).1.matched = c.matched := by
  cases hp : c.phase with
  | stopped why => simp [prepare,hp]
  | pending => simp [prepare,hp,halt]
  | ready =>
    cases hs : c.remaining with
    | nil => simp [prepare,hp,hs,halt]
    | cons event rest =>
      rcases event with ⟨wanted,r⟩
      by_cases h : q = wanted <;> simp [prepare,hp,hs,h,halt]

end Marqdo.ReplayCursor
