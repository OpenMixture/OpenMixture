//! Private Stage B size policy and receipts, shared by lib and integration tests.
use super::GpuContext;
use serde_json::{Value, json};
pub(crate) const AMENDMENT: &str = "2026-10-04-software-size-scope";
pub(crate) const ALL: [[u32; 2]; 4] = [[256, 256], [1024, 1024], [2048, 2048], [257, 129]];
pub(crate) struct Scope {
    software: bool,
    frozen: Vec<[u32; 2]>,
    pub(crate) selected: Vec<[u32; 2]>,
}
impl Scope {
    pub(crate) fn new(context: &GpuContext, plan: &Value, frozen: Vec<[u32; 2]>) -> Self {
        assert_eq!(plan["structuralProbes"]["amendments"][3]["id"], AMENDMENT);
        let material: Vec<[u32; 2]> = serde_json::from_value(plan["sizes"].clone()).unwrap();
        Self::from_type(
            &context.report().adapter().unwrap().device_type,
            &material,
            frozen,
        )
    }
    fn from_type(device_type: &str, material: &[[u32; 2]], frozen: Vec<[u32; 2]>) -> Self {
        assert_eq!(
            material, ALL,
            "required material sizes, including hardware 2048, changed"
        );
        assert!(!frozen.is_empty());
        assert!(frozen.iter().all(|s| ALL.contains(s)));
        let software = device_type == "Cpu";
        let selected = frozen
            .iter()
            .copied()
            .filter(|s| !software || *s != [2048, 2048])
            .collect();
        Self {
            software,
            frozen,
            selected,
        }
    }
    pub(crate) fn omitted(&self, templates: &[Value]) -> Vec<Value> {
        if !self.software || !self.frozen.contains(&[2048, 2048]) {
            return vec![];
        }
        templates
            .iter()
            .map(|template| {
                let mut row = template.clone();
                row["size"] = json!([2048, 2048]);
                row["status"] = json!("notRunOnSoftware");
                row["passed"] = Value::Null;
                row["amendment"] = json!(AMENDMENT);
                row
            })
            .collect()
    }
    pub(crate) fn describe(&self, rows: &[Value], omitted: &[Value], complete: bool) -> Value {
        let executed: Vec<_> = self
            .selected
            .iter()
            .copied()
            .filter(|s| rows.iter().any(|r| r["size"] == json!(s)))
            .collect();
        assert!(
            rows.iter()
                .all(|r| self.selected.iter().any(|s| r["size"] == json!(s)))
        );
        if complete {
            assert_eq!(
                executed, self.selected,
                "required hardware/probe size did not execute"
            );
        }
        json!({"amendment":AMENDMENT,"classification":if self.software {"reportedCpu"} else {"hardwareSizeCoverage"},"materialHardwareSizes":ALL,"frozenProbeSizes":self.frozen,"selectedSizes":self.selected,"executedSizes":executed,"notRunOnSoftware":omitted,"complete":complete,"qualificationScope":"software does not qualify hardware; original per-probe scopes and odd-share observation-only rule retained"})
    }
    pub(crate) fn save(
        &self,
        context: &GpuContext,
        probe: &str,
        rows: &[Value],
        omitted: &[Value],
        complete: bool,
    ) {
        let receipt = json!({"probe":probe,"adapter":context.report().adapter(),"sizeScope":self.describe(rows,omitted,complete),"rows":rows,"completed":complete,"materialAccepted":false});
        eprintln!("woven scope {probe}: {}", receipt["sizeScope"]);
        if let Ok(directory) = std::env::var("MIXTURE_NODE_EVIDENCE_DIR") {
            std::fs::create_dir_all(&directory).unwrap();
            std::fs::write(
                std::path::Path::new(&directory).join(format!("woven-{probe}-scope.json")),
                serde_json::to_vec_pretty(&receipt).unwrap(),
            )
            .unwrap();
        }
    }
}
#[test]
fn hardware_four_software_three_and_explicit_omission() {
    for device in ["DiscreteGpu", "IntegratedGpu", "VirtualGpu", "Other"] {
        let scope = Scope::from_type(device, &ALL, ALL.to_vec());
        assert_eq!(scope.selected, ALL);
        assert!(scope.omitted(&[json!({"case":"plain"})]).is_empty());
    }
    let scope = Scope::from_type("Cpu", &ALL, ALL.to_vec());
    assert_eq!(scope.selected, vec![[256, 256], [1024, 1024], [257, 129]]);
    let omitted = scope.omitted(&[json!({"case":"plain"})]);
    assert_eq!(
        omitted,
        vec![
            json!({"case":"plain","size":[2048,2048],"status":"notRunOnSoftware","passed":null,"amendment":AMENDMENT})
        ]
    );
    let rows: Vec<_> = scope.selected.iter().map(|s| json!({"size":s})).collect();
    assert_eq!(
        scope.describe(&rows, &omitted, true)["executedSizes"],
        json!(scope.selected)
    );
    let odd = Scope::from_type("Cpu", &ALL, vec![[257, 129]]);
    assert!(odd.omitted(&[json!({"control":"crown"})]).is_empty());
}
#[test]
#[should_panic(expected = "required material sizes")]
fn missing_hardware_2048_is_rejected() {
    Scope::from_type(
        "DiscreteGpu",
        &[[256, 256], [1024, 1024], [257, 129]],
        ALL.to_vec(),
    );
}
#[test]
#[should_panic(expected = "required hardware/probe size did not execute")]
fn unexecuted_hardware_2048_is_rejected() {
    let scope = Scope::from_type("DiscreteGpu", &ALL, ALL.to_vec());
    let rows = vec![
        json!({"size":[256,256]}),
        json!({"size":[1024,1024]}),
        json!({"size":[257,129]}),
    ];
    scope.describe(&rows, &[], true);
}
