use core::fmt::Debug;
use serde::Serialize;
use std::time::{Duration, Instant};

pub use serde;
pub use wasmi_profiling_macro::WasmiProfiling;

/// Derived and implemented by the Wasmi instruction type to track profiling data during execution.
pub trait WasmiProfiling {
    /// The profiling struct holding all relevant data.
    type Data: Debug + Default + Copy + Clone + Serialize + SelectInstr;

    /// Creates new profiling data initialized with default values.
    fn data() -> Self::Data;
}

/// Implemented by types that allow to select an instruction.
///
/// This is automatically generated by the [`WasmiProfiling`] macro for the generated type
/// and allows for the `data.instr().i32_add().start()` API.
pub trait SelectInstr {
    /// The particular instruction selector.
    type Selector<'a>
    where
        Self: 'a;

    /// Starts instruction selection for profiling.
    fn instr(&mut self) -> Self::Selector<'_>;
}

/// Implemented by the generated instruction data to compute the total time spent executing instructions.
pub trait InstrsTotalTime {
    /// Returns the total amount of time spent executing instructions.
    fn instrs_total_time(&self) -> Duration;
}

/// Implemented by the generated instruction data to compute number of executed instructions.
pub trait InstrsCount {
    /// Returns the total amount of instructions executed.
    fn instrs_count(&self) -> u64;
}

/// Type blueprint to collect profiling data.
///
/// Generic over the type representing the Wasmi instructions which is generated by a macro.
#[derive(Debug, Default, Copy, Clone, Serialize)]
pub struct ProfilingData<T> {
    /// Profiling data for Wasmi instruction dispatch.
    pub dispatch: DispatchTracker,
    /// Profiling data for host function calls.
    pub host: InstrTracker,
    /// Measures time between events.
    #[serde(skip)]
    pub ticker: Ticker,
    /// Profiling data for Wasmi bytecode instructions.
    pub instr: T,
}

impl<T> ProfilingData<T> {
    /// Start profiling a Wasmi execution run.
    ///
    /// # Note
    ///
    /// This should be invoked right before the first instruction dispatch.
    #[inline]
    pub fn start(&mut self) {
        self.ticker.tick();
    }
}

/// A tracker of profiling data for dispatching of Wasmi bytecode.
#[derive(Debug, Default, Copy, Clone, Serialize)]
pub struct DispatchTracker {
    /// The total time spent during Wasmi bytecode dispatch.
    pub total_time: Duration,
}

/// A tracker of profiling data for a single type of instruction.
#[derive(Debug, Default, Copy, Clone, Serialize)]
pub struct InstrTracker {
    /// The number of times the associated instruction got executed.
    pub count: u64,
    /// The total time spent during execution of the associated instruction.
    pub total_time: Duration,
}

impl InstrTracker {
    /// Start instruction execution measurement.
    #[inline]
    pub fn start(&mut self, ticker: &mut Ticker, dispatch: &mut DispatchTracker) {
        dispatch.total_time += ticker.tick();
        self.count = self.count.wrapping_add(1);
    }

    /// End instruction execution measurement.
    #[inline]
    pub fn stop(&mut self, ticker: &mut Ticker) {
        self.total_time += ticker.tick();
    }

    /// Returns `true` if the associated Wasmi instruction has never been called.
    ///
    /// # Note
    ///
    /// This is primarily used by `serde` to filter out instructions during
    /// serialization that have never been used during execution profiling.
    pub fn is_never_called(&self) -> bool {
        self.count == 0
    }
}

/// A stop-watch ticker for measuring time between events.
#[derive(Debug, Copy, Clone)]
pub struct Ticker {
    /// The time stamp of the last call to [`Ticker::tick`] or its creation.
    last: Instant,
}

impl Default for Ticker {
    fn default() -> Self {
        Self {
            last: Instant::now(),
        }
    }
}

impl Ticker {
    /// Returns the [`Duration`] since the last call to [`Ticker::tick`].
    #[inline]
    pub fn tick(&mut self) -> Duration {
        let old = self.last;
        self.last = Instant::now();
        self.last - old
    }
}

/// Blueprint type for instruction selection via [`SelectInstr`] trait.
#[derive(Debug)]
pub struct SelectedInstr<'a> {
    /// The surrounding dispatch data.
    dispatch: &'a mut DispatchTracker,
    /// The associated ticker to measure time.
    ticker: &'a mut Ticker,
    /// The profiling data of the selected instruction.
    instr: &'a mut InstrTracker,
}

impl<'a> SelectedInstr<'a> {
    /// Creates a new [`SelectedInstr`] from the given parts.
    #[inline]
    pub fn new(
        dispatch: &'a mut DispatchTracker,
        ticker: &'a mut Ticker,
        instr: &'a mut InstrTracker,
    ) -> Self {
        Self {
            dispatch,
            ticker,
            instr,
        }
    }

    /// Starts to measure execution time for the selected instruction.
    #[inline]
    pub fn start(self) {
        self.instr.start(self.ticker, self.dispatch);
    }

    /// Stops to measure execution time for the selected instruction.
    #[inline]
    pub fn stop(self) {
        self.instr.stop(self.ticker);
    }
}
