//! Explicit evidence levels: a Lean proof of the mathematical model is
//! categorically different from a proof about the dynamic Marqdo runtime.
//! Neither source metadata nor an asserted plugin contract grants authority.
use serde::{Serialize,Deserialize};
use serde_json::{json,Value};

#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="snake_case")]
pub enum EvidenceGrade{
    /// Only when correspondence to supported runtime semantics was proven.
    Proved,
    /// Kernel checked theorem over an explicitly identified abstract model.
    ModelProved,
    /// Observed by supplied tests/experiments. Not a mathematical proof.
    Tested,
    /// Trust or environmental assertion, not proven from source.
    Assumed,
    /// Insufficient evidence, timeout, unsupported language, or no proof.
    Unknown,
    /// An actual counterexample, never just a missing Lean derivation.
    Refuted,
    /// The source, contract, toolchain, or dependency changed.
    Stale
}
impl EvidenceGrade{
    pub fn as_str(self)->&'static str{
        match self{
            Self::Proved=>"proved",Self::ModelProved=>"model_proved",
            Self::Tested=>"tested",Self::Assumed=>"assumed",
            Self::Unknown=>"unknown",Self::Refuted=>"refuted",
            Self::Stale=>"stale"
        }
    }
}
fn record(grade:EvidenceGrade,basis:&str,kernel_replayed:bool,
    runtime_correspondence:bool)->Value{
    let effective=if grade==EvidenceGrade::Proved && !runtime_correspondence{
        EvidenceGrade::ModelProved
    }else{grade};
    json!({
        "schema_version":1,"grade":effective.as_str(),"basis":basis,
        "model":"lean_int_abstract",
        "kernel_replayed":kernel_replayed,
        "runtime_correspondence_verified":runtime_correspondence,
        "full_marqdo_runtime_semantics_proved":false,
        "not_execution_authority":true
    })
}
pub fn model_proved()->Value{
    record(EvidenceGrade::ModelProved,"source_regenerated_lean_kernel",
        true,false)
}
pub fn unknown(reason:&str)->Value{
    record(EvidenceGrade::Unknown,reason,false,false)
}
pub fn assumed()->Value{
    record(EvidenceGrade::Assumed,"external_or_author_declaration",false,false)
}
pub fn for_receipt(status:&str,usable:bool,kernel_replayed:bool)->Value{
    if status=="stale"{
        return record(EvidenceGrade::Stale,"source_or_dependency_changed",
            false,false)
    }
    if !usable {
        return unknown(if status=="ambiguous"{"ambiguous_receipt"}
            else if status=="unproved"{"lean_not_proved"}
            else{"no_trusted_replayable_proof"})
    }
    if status=="proved"{
        return record(EvidenceGrade::ModelProved,
            "source_bound_lean_certificate",kernel_replayed,false)
    }
    unknown("unsupported_receipt_status")
}
pub fn with_grade(grade:EvidenceGrade,source:&str)->Value{
    // This constructor is for explicitly sourced tests/assumptions and is
    // intentionally never allowed to create runtime-level Proved by fiat.
    record(grade,source,false,false)
}

#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn grades_do_not_promote_abstract_integer_proofs(){
        let proved=for_receipt("proved",true,true);
        assert_eq!(proved["grade"],"model_proved");
        assert_eq!(proved["full_marqdo_runtime_semantics_proved"],false);
        assert_eq!(with_grade(EvidenceGrade::Proved,"asserted")["grade"],
            "model_proved");
        assert_eq!(for_receipt("stale",false,false)["grade"],"stale");
        assert_eq!(for_receipt("unproved",false,false)["grade"],"unknown");
        assert_eq!(for_receipt("ambiguous",false,false)["grade"],"unknown");
        assert_eq!(with_grade(EvidenceGrade::Refuted,
            "actual_counterexample")["grade"],"refuted");
        assert_eq!(assumed()["grade"],"assumed");
    }
}
