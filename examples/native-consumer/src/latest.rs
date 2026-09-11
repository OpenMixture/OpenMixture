//! One active request, one replaceable pending request, and one displayed result.
//! The caller owns the renderer/worker and returns each completion to this state.
//! No GPU cancellation, background thread or result cloning is performed here.
use std::{
    fs, io,
    path::{Path, PathBuf},
};

/// Consumer-local identity, deliberately separate from a semantic plan hash.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Generation(u64);
impl Generation {
    pub fn value(self) -> u64 {
        self.0
    }
}

pub struct Latest<R, O, E> {
    latest: u64,
    active: Option<Generation>,
    pending: Option<(Generation, R)>,
    displayed: Option<(Generation, O)>,
    failure: Option<(Generation, E)>,
}
impl<R, O, E> Default for Latest<R, O, E> {
    fn default() -> Self {
        Self {
            latest: 0,
            active: None,
            pending: None,
            displayed: None,
            failure: None,
        }
    }
}
impl<R, O, E> Latest<R, O, E> {
    /// Replaces pending work immediately. Overflow fails without changing state.
    pub fn request(&mut self, request: R) -> Result<Generation, &'static str> {
        let next = self
            .latest
            .checked_add(1)
            .ok_or("request generation exhausted")?;
        let generation = Generation(next);
        self.latest = next;
        self.pending = Some((generation, request));
        self.failure = None;
        Ok(generation)
    }
    /// Transfers work to the single caller-owned worker. Never starts a second job.
    pub fn start(&mut self) -> Option<(Generation, R)> {
        if self.active.is_some() {
            return None;
        }
        let work = self.pending.take()?;
        self.active = Some(work.0);
        Some(work)
    }
    /// Accepts only the active generation, publishing only if it is still newest.
    /// Returns an obsolete output for immediate drop or explicit filesystem cleanup.
    /// Duplicate/unknown completions cannot clear the active slot or replace display.
    pub fn complete(&mut self, generation: Generation, result: Result<O, E>) -> Option<O> {
        if self.active != Some(generation) {
            return result.ok();
        }
        self.active = None;
        if generation.0 != self.latest {
            return result.ok();
        }
        match result {
            Ok(output) => self
                .displayed
                .replace((generation, output))
                .map(|(_, old)| old),
            Err(error) => {
                self.failure = Some((generation, error));
                None
            }
        }
    }
    /// A retained display may be stale, including when the newest request failed.
    pub fn displayed(&self) -> Option<(Generation, &O, bool)> {
        self.displayed
            .as_ref()
            .map(|(g, o)| (*g, o, g.0 == self.latest))
    }
    pub fn failure(&self) -> Option<(Generation, &E)> {
        self.failure.as_ref().map(|(g, e)| (*g, e))
    }
    pub fn take_displayed(&mut self) -> Option<O> {
        self.displayed.take().map(|(_, output)| output)
    }
}

/// A fresh generation directory owned by this example. Never adopt an existing path.
/// Cleanup is explicit and fallible; no Drop implementation hides filesystem errors.
pub struct OutputDirectory {
    path: PathBuf,
}
impl OutputDirectory {
    pub fn create(parent: &Path, generation: Generation) -> io::Result<Self> {
        let path = parent.join(format!("generation-{}", generation.0));
        fs::create_dir(&path)?;
        Ok(Self { path })
    }
    pub fn path(&self) -> &Path {
        &self.path
    }
    pub fn remove(self) -> io::Result<()> {
        fs::remove_dir_all(self.path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn replaced_pending_stale_and_duplicate_completions_do_not_publish() {
        let mut s = Latest::<_, &str, &str>::default();
        let a = s.request("a").unwrap();
        assert_eq!(s.start(), Some((a, "a")));
        let b = s.request("b").unwrap();
        let c = s.request("c").unwrap();
        assert!(s.start().is_none());
        assert_eq!(s.complete(c, Ok("early")), Some("early")); // pending cannot finish
        assert!(s.start().is_none());
        assert_eq!(s.complete(a, Ok("old")), Some("old"));
        assert!(s.displayed().is_none());
        assert_eq!(s.start(), Some((c, "c"))); // b was never started
        assert_eq!(s.complete(b, Ok("unknown")), Some("unknown"));
        assert!(s.start().is_none());
        assert_eq!(s.complete(c, Ok("current")), None);
        assert_eq!(s.displayed(), Some((c, &"current", true)));
        assert_eq!(s.complete(a, Ok("late")), Some("late"));
        assert_eq!(s.complete(c, Ok("duplicate")), Some("duplicate"));
        assert_eq!(s.displayed(), Some((c, &"current", true)));
    }
    #[test]
    fn latest_failure_keeps_old_display_explicitly_stale() {
        let mut s = Latest::<_, &str, &str>::default();
        let a = s.request(()).unwrap();
        s.start();
        s.complete(a, Ok("display"));
        let b = s.request(()).unwrap();
        s.start();
        assert_eq!(s.displayed(), Some((a, &"display", false)));
        s.complete(b, Err("new request failed"));
        assert_eq!(s.failure(), Some((b, &"new request failed")));
        assert_eq!(s.complete(a, Ok("late success")), Some("late success"));
        assert_eq!(s.displayed(), Some((a, &"display", false)));
        let c = s.request(()).unwrap();
        s.start();
        assert!(s.failure().is_none());
        assert_eq!(s.complete(c, Ok("replacement")), Some("display"));
        assert_eq!(s.displayed(), Some((c, &"replacement", true)));
    }
    #[test]
    fn superseded_failure_does_not_report_newest_failure() {
        let mut s = Latest::<_, (), &str>::default();
        let a = s.request(()).unwrap();
        s.start();
        s.request(()).unwrap();
        s.complete(a, Err("old"));
        assert!(s.failure().is_none());
        assert!(s.start().is_some());
    }
    #[test]
    fn generation_exhaustion_preserves_pending_and_display() {
        let mut s = Latest::<_, (), ()> {
            latest: u64::MAX - 1,
            ..Default::default()
        };
        let g = s.request("last").unwrap();
        assert!(s.request("overflow").is_err());
        assert_eq!(s.start(), Some((g, "last")));
    }
    #[test]
    fn retained_outputs_and_pending_requests_are_dropped_promptly() {
        use std::{cell::Cell, rc::Rc};
        struct Count(Rc<Cell<usize>>);
        impl Count {
            fn new(n: &Rc<Cell<usize>>) -> Self {
                n.set(n.get() + 1);
                Self(n.clone())
            }
        }
        impl Drop for Count {
            fn drop(&mut self) {
                self.0.set(self.0.get() - 1);
            }
        }
        let n = Rc::new(Cell::new(0));
        let r = Rc::new(Cell::new(0));
        let mut s = Latest::<Count, Count, ()>::default();
        for _ in 0..10 {
            s.request(Count::new(&r)).unwrap();
            assert_eq!(r.get(), 1);
        }
        let (a, request) = s.start().unwrap();
        drop(request);
        s.complete(a, Ok(Count::new(&n)));
        assert_eq!(n.get(), 1);
        let b = s.request(Count::new(&r)).unwrap();
        let work = s.start().unwrap();
        s.request(Count::new(&r)).unwrap();
        assert_eq!(r.get(), 2);
        let output = Count::new(&n);
        assert_eq!(n.get(), 2);
        drop(s.complete(b, Ok(output)));
        assert_eq!(n.get(), 1);
        drop(work);
        let (c, work) = s.start().unwrap();
        drop(work);
        drop(s.complete(c, Ok(Count::new(&n))));
        assert_eq!(n.get(), 1);
        drop(s);
        assert_eq!(n.get(), 0);
        assert_eq!(r.get(), 0);
    }
    #[test]
    fn output_directories_are_isolated_and_only_owned_paths_removed() {
        let parent = std::env::temp_dir().join(format!("mixture-latest-{}", std::process::id()));
        fs::create_dir(&parent).unwrap();
        fs::write(parent.join("unrelated"), b"keep").unwrap();
        let a = OutputDirectory::create(&parent, Generation(1)).unwrap();
        let b = OutputDirectory::create(&parent, Generation(2)).unwrap();
        assert!(OutputDirectory::create(&parent, Generation(1)).is_err());
        fs::write(a.path().join("partial.png"), b"partial").unwrap();
        fs::write(b.path().join("current.png"), b"current").unwrap();
        a.remove().unwrap();
        assert_eq!(fs::read(b.path().join("current.png")).unwrap(), b"current");
        b.remove().unwrap();
        assert!(parent.join("unrelated").exists());
        fs::remove_dir_all(parent).unwrap();
    }
}
