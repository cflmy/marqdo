/- Abstract stateful ABI and bounded-loop semantics.
   These are proof rules, not an axiom that an arbitrary ABI plugin
   satisfies its declared contract. Implementation witnesses are required. -/
namespace MarqdoEffects

inductive Outcome (σ α ε : Type) where
  | ok : α → σ → Outcome σ α ε
  | error : ε → σ → Outcome σ α ε
  | unknown : Outcome σ α ε

def bind {σ α β ε : Type}
    (first : Outcome σ α ε)
    (next : α → σ → Outcome σ β ε) : Outcome σ β ε :=
  match first with
  | .ok value state => next value state
  | .error e state => .error e state
  | .unknown => .unknown

structure AbiContract (σ α β : Type) where
  requires : σ → α → Prop
  ensures : σ → α → σ → β → Prop

def Implements {σ α β ε : Type}
    (run : σ → α → Outcome σ β ε)
    (contract : AbiContract σ α β) : Prop :=
  ∀ state arg, contract.requires state arg →
    ∃ value updated, run state arg = .ok value updated ∧
      contract.ensures state arg updated value

theorem impl_requires_witness {σ α β ε : Type}
    (run : σ → α → Outcome σ β ε) (contract : AbiContract σ α β)
    (h : Implements run contract) (state : σ) (arg : α)
    (pre : contract.requires state arg) :
    ∃ value updated, run state arg = .ok value updated := by
  obtain ⟨v, s, hr, _⟩ := h state arg pre
  exact ⟨v, s, hr⟩

-- The plugin's failure case remains observable. A declared contract alone
-- never rules out this case, unless a concrete implementation proof exists.
theorem error_not_abi_success {σ α ε : Type}
    (reason : ε) (state : σ) :
    ¬ (∃ v s, (Outcome.error reason state : Outcome σ α ε) =
      Outcome.ok v s) := by
  intro h
  obtain ⟨v, s, hs⟩ := h
  cases hs

theorem unknown_not_abi_success {σ α ε : Type} :
    ¬ (∃ v s, (Outcome.unknown : Outcome σ α ε) =
      Outcome.ok v s) := by
  intro h
  obtain ⟨v, s, hs⟩ := h
  cases hs

theorem bind_preserves_error {σ α β ε : Type}
    (next : α → σ → Outcome σ β ε) (reason : ε) (state : σ) :
    bind (Outcome.error reason state) next = Outcome.error reason state := by
  rfl

theorem bind_preserves_unknown {σ α β ε : Type}
    (next : α → σ → Outcome σ β ε) :
    bind Outcome.unknown next = Outcome.unknown := by
  rfl

-- Running out of fuel is UNRESOLVED, not a proof of termination or divergence.
def runFuel {σ ε : Type}
    (step : σ → Outcome σ σ ε) : Nat → σ → Outcome σ σ ε
  | 0, _ => .unknown
  | n + 1, state =>
    match step state with
    | .ok _ updated => runFuel step n updated
    | .error e updated => .error e updated
    | .unknown => .unknown

theorem fuel_zero_not_success {σ ε : Type}
    (step : σ → Outcome σ σ ε) (state : σ) :
    ¬ (∃ v s, runFuel step 0 state = Outcome.ok v s) := by
  intro h
  obtain ⟨v, s, hs⟩ := h
  cases hs

end MarqdoEffects
