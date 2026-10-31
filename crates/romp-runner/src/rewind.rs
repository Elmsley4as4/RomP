//! The history the rewind button walks back through.
//!
//! A libretro core has no rewind of its own: the frontend keeps save states and puts
//! an older one back instead of running a frame. States are taken every few frames
//! rather than every frame, because serializing a core costs real time and holding the
//! button back through a coarser history still plays smoothly.

use std::collections::VecDeque;

/// How many frames pass between two points the rewind can return to. Rewinding then
/// runs at about this many times the game's speed.
const EVERY: u32 = 6;

/// The most states kept, which is the length of the history at small state sizes.
const MOST_POINTS: usize = 600;

/// The most memory the history may hold. Big-state cores keep fewer, shorter points.
const MOST_BYTES: usize = 96 * 1024 * 1024;

/// Always keep this many points, however big the states are, so a core with a huge
/// state still rewinds a little rather than not at all.
const LEAST_POINTS: usize = 8;

pub struct Rewind {
    points: VecDeque<Vec<u8>>,
    bytes: usize,
    frames: u32,
    on: bool,
}

impl Default for Rewind {
    fn default() -> Self {
        Self::new()
    }
}

impl Rewind {
    pub fn new() -> Self {
        Self {
            points: VecDeque::new(),
            bytes: 0,
            frames: 0,
            on: false,
        }
    }

    pub fn is_on(&self) -> bool {
        self.on
    }

    /// Turns the history on or off. Turning it on starts from nothing, so a game never
    /// rewinds into another game's states; asking for what is already set changes nothing.
    pub fn set_on(&mut self, on: bool) {
        if self.on == on {
            return;
        }
        self.on = on;
        self.clear();
    }

    pub fn clear(&mut self) {
        self.points.clear();
        self.bytes = 0;
        self.frames = 0;
    }

    /// Whether the frame about to run is one to keep a state of.
    pub fn wants_state(&mut self) -> bool {
        if !self.on {
            return false;
        }
        self.frames = self.frames.wrapping_add(1);
        self.frames % EVERY == 0
    }

    /// Keeps one state, dropping the oldest until it fits.
    pub fn keep(&mut self, state: Vec<u8>) {
        self.bytes += state.len();
        self.points.push_back(state);
        while self.points.len() > MOST_POINTS
            || (self.bytes > MOST_BYTES && self.points.len() > LEAST_POINTS)
        {
            let Some(old) = self.points.pop_front() else {
                break;
            };
            self.bytes = self.bytes.saturating_sub(old.len());
        }
    }

    /// The newest point, which is where the game goes back to. It is taken out of the
    /// history so the next step back is further back still, and so the game never
    /// returns to a point it has already replayed.
    pub fn take(&mut self) -> Option<Vec<u8>> {
        let state = self.points.pop_back()?;
        self.bytes = self.bytes.saturating_sub(state.len());
        Some(state)
    }

    pub fn len(&self) -> usize {
        self.points.len()
    }

    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn due_frames(rewind: &mut Rewind, frames: u32) -> u32 {
        (0..frames).filter(|_| rewind.wants_state()).count() as u32
    }

    #[test]
    fn nothing_is_kept_while_the_history_is_off() {
        let mut rewind = Rewind::new();
        assert_eq!(due_frames(&mut rewind, 60), 0);
        rewind.set_on(true);
        assert_eq!(due_frames(&mut rewind, 60), 60 / EVERY);
    }

    #[test]
    fn states_come_back_newest_first_and_only_once() {
        let mut rewind = Rewind::new();
        rewind.set_on(true);
        for i in 0..3u8 {
            rewind.keep(vec![i]);
        }
        assert_eq!(rewind.take(), Some(vec![2]));
        assert_eq!(rewind.take(), Some(vec![1]));
        assert_eq!(rewind.take(), Some(vec![0]));
        assert_eq!(rewind.take(), None);
        assert!(rewind.is_empty());
    }

    #[test]
    fn the_oldest_states_go_when_the_history_is_full() {
        let mut rewind = Rewind::new();
        rewind.set_on(true);
        for i in 0..(MOST_POINTS as u32 + 10) {
            rewind.keep(i.to_le_bytes().to_vec());
        }
        assert_eq!(rewind.len(), MOST_POINTS);
        assert_eq!(
            rewind.take(),
            Some((MOST_POINTS as u32 + 9).to_le_bytes().to_vec())
        );
    }

    #[test]
    fn big_states_still_keep_a_few_points() {
        let mut rewind = Rewind::new();
        rewind.set_on(true);
        let state = vec![0u8; MOST_BYTES / 4];
        for _ in 0..20 {
            rewind.keep(state.clone());
        }
        assert_eq!(rewind.len(), LEAST_POINTS);
    }

    #[test]
    fn turning_the_history_on_starts_it_afresh() {
        let mut rewind = Rewind::new();
        rewind.set_on(true);
        rewind.keep(vec![1]);
        // Saying it again is not a new game, so the points stay.
        rewind.set_on(true);
        assert_eq!(rewind.len(), 1);
        rewind.set_on(false);
        rewind.set_on(true);
        assert!(rewind.is_empty());
    }
}
