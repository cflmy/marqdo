/-
  Core assume–guarantee composition lemma for Marqdo module contracts.

  This checks the *logic* of composing contracts, NOT that a particular
  Marqdo function implements one. The compiler must separately establish
  implementation correspondence and the call-site bridge obligation.
-/

namespace MarqdoModularProof

structure Contract (α β : Type) where
  requires : α → Prop
  ensures : α → β → Prop

def Implements {α β : Type} (f : α → β) (c : Contract α β) : Prop :=
  ∀ x, c.requires x → c.ensures x (f x)

def composed {α β γ : Type}
    (first : Contract α β) (second : Contract β γ) : Contract α γ where
  requires := first.requires
  ensures := fun x z =>
    ∃ y, first.ensures x y ∧ second.ensures y z

-- Caller obligation is explicit. In particular, callee contract validity
-- alone does not establish the precondition of the following component.
theorem compose_sound {α β γ : Type}
    (f : α → β) (g : β → γ)
    (first : Contract α β) (second : Contract β γ)
    (hf : Implements f first)
    (hg : Implements g second)
    (bridge : ∀ x, first.requires x →
      first.ensures x (f x) → second.requires (f x)) :
    Implements (fun x => g (f x)) (composed first second) := by
  intro x hx
  have hfirst := hf x hx
  have hnext := hg (f x) (bridge x hx hfirst)
  exact ⟨f x, hfirst, hnext⟩

-- Trust-boundary test: the bridge is genuinely needed. Without it,
-- the conclusion would fail for suitable partial contracts.
example :
    ¬ (∀ (f : Nat → Nat) (g : Nat → Nat)
        (first : Contract Nat Nat) (second : Contract Nat Nat),
      Implements f first →
      Implements g second →
      Implements (fun x => g (f x)) (composed first second)) := by
  intro h
  let f : Nat → Nat := fun _ => 0
  let g : Nat → Nat := fun y => y
  let first : Contract Nat Nat := {
    requires := fun _ => True
    ensures := fun _ _ => True
  }
  let second : Contract Nat Nat := {
    requires := fun y => y = 1
    ensures := fun _ z => z = 1
  }
  have hf : Implements f first := by
    intro _ _
    trivial
  have hg : Implements g second := by
    intro y hy
    simpa [second, g] using hy
  have hcomp := h f g first second hf hg
  have target := hcomp 0 True.intro
  cases target with
  | intro y hpair =>
    have impossible : (0 : Nat) = 1 := by
      simpa [second, g, f] using hpair.right
    exact Nat.zero_ne_one impossible

-- Execution is a *partial* result: host/plugin failures and divergence
-- must remain visible. Loops/recursion can be reasoned about only after
-- separate termination proofs, and ABI effects require a proven contract.
inductive Exit (α ε : Type) where
  | ok (value : α)
  | error (reason : ε)
  | diverged

def sequence {α β ε : Type}
    (first : Exit α ε) (next : α → Exit β ε) : Exit β ε :=
  match first with
  | .ok value => next value
  | .error e => .error e
  | .diverged => .diverged

structure TotalContract (α β : Type) where
  requires : α → Prop
  ensures : α → β → Prop

def Satisfies {α β ε : Type}
    (run : α → Exit β ε) (c : TotalContract α β) : Prop :=
  ∀ x, c.requires x → ∃ y, run x = .ok y ∧ c.ensures x y

def combined {α β γ : Type}
    (first : TotalContract α β)
    (second : TotalContract β γ) : TotalContract α γ where
  requires := first.requires
  ensures := fun x z => ∃ y, first.ensures x y ∧ second.ensures y z

theorem total_sequence_sound {α β γ ε : Type}
    (f : α → Exit β ε) (g : β → Exit γ ε)
    (cf : TotalContract α β) (cg : TotalContract β γ)
    (hf : Satisfies f cf)
    (hg : Satisfies g cg)
    (bridge : ∀ x y, cf.requires x → cf.ensures x y →
        cg.requires y) :
    Satisfies (fun x => sequence (f x) g) (combined cf cg) := by
  intro x hx
  obtain ⟨y, hrunF, hpostF⟩ := hf x hx
  obtain ⟨z, hrunG, hpostG⟩ := hg y (bridge x y hx hpostF)
  refine ⟨z, ?_, ⟨y, hpostF, hpostG⟩⟩
  simpa [sequence, hrunF] using hrunG

-- A crashing plugin and a diverging loop cannot satisfy a total contract.
theorem error_not_success {α ε : Type} (e : ε) :
    ¬ (∃ v : α, Exit.error e = Exit.ok v) := by
  intro h
  obtain ⟨v, hv⟩ := h
  cases hv

theorem diverged_not_success {α ε : Type} :
    ¬ (∃ v : α, (Exit.diverged : Exit α ε) = Exit.ok v) := by
  intro h
  obtain ⟨v, hv⟩ := h
  cases hv

end MarqdoModularProof
