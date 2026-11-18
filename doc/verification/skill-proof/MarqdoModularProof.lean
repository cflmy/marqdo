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
  rcases target with ⟨y, _, hy⟩
  have impossible : (0 : Nat) = 1 := by
    simpa [second, g, f] using hy
  exact Nat.zero_ne_one impossible

end MarqdoModularProof
