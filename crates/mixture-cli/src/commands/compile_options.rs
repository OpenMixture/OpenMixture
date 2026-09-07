//! Shared CLI syntax for the core compile request; semantics stay in mixture-core.
use mixture_core::{CompileError, CompileRequest, OutputChannel};
use std::{collections::BTreeMap, ffi::OsString, slice::Iter};
#[derive(Default)]
pub(super) struct CompileOptions {
    size: Option<[u32; 2]>,
    outputs: Option<String>,
    overrides: BTreeMap<String, serde_json::Value>,
}
impl CompileOptions {
    pub fn parse_option(
        &mut self,
        flag: &str,
        args: &mut Iter<'_, OsString>,
    ) -> Result<bool, String> {
        if !matches!(flag, "--size" | "--output" | "--set") {
            return Ok(false);
        }
        let value = args
            .next()
            .and_then(|v| v.to_str())
            .ok_or_else(|| format!("Missing UTF-8 value for {flag}."))?;
        match flag {
            "--size" if self.size.is_none() => {
                let (w, h) = value.split_once('x').unwrap_or((value, value));
                let number = |v: &str| {
                    v.parse::<u32>().map_err(|_| {
                        "Expected --size pixels or widthxheight using unsigned 32-bit integers."
                            .to_owned()
                    })
                };
                self.size = Some([number(w)?, number(h)?]);
            }
            "--output" if self.outputs.is_none() => self.outputs = Some(value.to_owned()),
            "--set" => {
                let (id, json) = value
                    .split_once('=')
                    .filter(|(id, _)| !id.is_empty())
                    .ok_or("Expected --set publicId=JSON.")?;
                let parsed = serde_json::from_str(json)
                    .map_err(|e| format!("Invalid JSON override for {id}: {e}"))?;
                if self.overrides.insert(id.to_owned(), parsed).is_some() {
                    return Err(format!("Duplicate override ID: {id}."));
                }
            }
            _ => return Err(format!("Duplicate {flag} option.")),
        }
        Ok(true)
    }
    pub fn request(self) -> Result<CompileRequest, CompileError> {
        let mut request = CompileRequest {
            overrides: self.overrides,
            ..Default::default()
        };
        if let Some(size) = self.size {
            request.size = size;
        }
        if let Some(outputs) = self.outputs {
            request.outputs = outputs
                .split(',')
                .map(str::parse::<OutputChannel>)
                .collect::<Result<_, _>>()?;
        }
        Ok(request)
    }
}
