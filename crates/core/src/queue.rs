//! Reine Wiedergabe-Reihenfolge ohne Audio: Liste, Shuffle, Wiederholen und „Als Nächstes“.

use std::collections::VecDeque;

use rand::seq::SliceRandom;

use crate::model::Repeat;

#[derive(Debug, Default, Clone)]
pub struct Queue {
    items: Vec<i64>,
    /// Abspielreihenfolge als Indizes in `items`.
    order: Vec<usize>,
    /// Position in `order`.
    cursor: Option<usize>,
    /// Manuell eingereihte Titel, die vor dem Rest der Liste kommen.
    up_next: VecDeque<i64>,
    /// Gerade laufender Titel aus `up_next` (gehört nicht zu `items`).
    detached: Option<i64>,
    shuffle: bool,
    repeat: Repeat,
}

impl Queue {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn shuffle(&self) -> bool {
        self.shuffle
    }

    pub fn repeat(&self) -> Repeat {
        self.repeat
    }

    pub fn set_repeat(&mut self, repeat: Repeat) {
        self.repeat = repeat;
    }

    pub fn items(&self) -> &[i64] {
        &self.items
    }

    pub fn len(&self) -> usize {
        self.items.len() + self.up_next.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn current(&self) -> Option<i64> {
        self.detached
            .or_else(|| self.cursor.map(|c| self.items[self.order[c]]))
    }

    /// Index des aktuellen Titels in `items` (z. B. zum Speichern der Sitzung).
    pub fn current_index(&self) -> Option<usize> {
        self.cursor.map(|c| self.order[c])
    }

    /// Ersetzt die Liste und startet bei `start`.
    pub fn set(&mut self, items: Vec<i64>, start: usize) -> Option<i64> {
        self.items = items;
        self.detached = None;
        if self.items.is_empty() {
            self.order.clear();
            self.cursor = None;
            return None;
        }
        let start = start.min(self.items.len() - 1);
        self.rebuild_order(Some(start));
        self.current()
    }

    fn rebuild_order(&mut self, keep: Option<usize>) {
        self.order = (0..self.items.len()).collect();
        if self.shuffle {
            let mut rng = rand::thread_rng();
            self.order.shuffle(&mut rng);
            if let Some(k) = keep {
                let pos = self.order.iter().position(|&i| i == k).unwrap();
                self.order.swap(0, pos);
            }
            self.cursor = keep.map(|_| 0);
        } else {
            self.cursor = keep;
        }
    }

    pub fn set_shuffle(&mut self, on: bool) {
        if self.shuffle == on {
            return;
        }
        self.shuffle = on;
        let keep = self.current_index();
        self.rebuild_order(keep);
    }

    /// Nächster Titel. `auto` = der vorige Titel ist zu Ende gelaufen.
    pub fn next(&mut self, auto: bool) -> Option<i64> {
        if auto && self.repeat == Repeat::One {
            if let Some(cur) = self.current() {
                return Some(cur);
            }
        }
        if let Some(id) = self.up_next.pop_front() {
            self.detached = Some(id);
            return Some(id);
        }
        self.detached = None;
        let len = self.order.len();
        if len == 0 {
            return None;
        }
        let next = match self.cursor {
            None => Some(0),
            Some(c) if c + 1 < len => Some(c + 1),
            Some(_) if self.repeat != Repeat::Off => {
                if self.shuffle {
                    self.rebuild_order(None);
                }
                Some(0)
            }
            Some(_) => None,
        };
        match next {
            Some(n) => {
                self.cursor = Some(n);
                self.current()
            }
            None => None,
        }
    }

    pub fn prev(&mut self) -> Option<i64> {
        if self.detached.take().is_some() {
            return self.current();
        }
        let len = self.order.len();
        match self.cursor {
            Some(c) if c > 0 => self.cursor = Some(c - 1),
            Some(_) if self.repeat == Repeat::All => self.cursor = Some(len - 1),
            _ => {}
        }
        self.current()
    }

    /// Hängt einen Titel an die Liste an.
    pub fn append(&mut self, id: i64) {
        self.items.push(id);
        let idx = self.items.len() - 1;
        if self.shuffle && self.cursor.is_some() {
            let from = self.cursor.unwrap() + 1;
            let pos = rand::Rng::gen_range(&mut rand::thread_rng(), from..=self.order.len());
            self.order.insert(pos, idx);
        } else {
            self.order.push(idx);
        }
    }

    /// „Als Nächstes spielen“
    pub fn play_next(&mut self, id: i64) {
        self.up_next.push_front(id);
    }

    /// „Zur Warteschlange hinzufügen“
    pub fn enqueue(&mut self, id: i64) {
        self.up_next.push_back(id);
    }

    /// Alles nach dem laufenden Titel entfernen (eingereihte Titel und den Rest der Liste).
    pub fn clear_upcoming(&mut self) {
        self.up_next.clear();
        match self.current() {
            Some(cur) => {
                self.items = vec![cur];
                self.order = vec![0];
                self.cursor = Some(0);
                self.detached = None;
            }
            None => {
                self.items.clear();
                self.order.clear();
                self.cursor = None;
            }
        }
    }

    /// Die nächsten `n` Titel in Abspielreihenfolge (für die Anzeige).
    pub fn upcoming(&self, n: usize) -> Vec<i64> {
        let mut out: Vec<i64> = self.up_next.iter().copied().take(n).collect();
        if let Some(c) = self.cursor {
            let rest = self.order.iter().skip(c + 1).map(|&i| self.items[i]);
            out.extend(rest.take(n.saturating_sub(out.len())));
        } else if self.detached.is_none() {
            out.extend(self.order.iter().map(|&i| self.items[i]).take(n.saturating_sub(out.len())));
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plays_in_order_and_stops_at_end() {
        let mut q = Queue::new();
        assert_eq!(q.set(vec![1, 2, 3], 0), Some(1));
        assert_eq!(q.next(true), Some(2));
        assert_eq!(q.next(true), Some(3));
        assert_eq!(q.next(true), None);
        assert_eq!(q.current(), Some(3));
    }

    #[test]
    fn repeat_all_wraps_around() {
        let mut q = Queue::new();
        q.set_repeat(Repeat::All);
        q.set(vec![1, 2], 1);
        assert_eq!(q.next(true), Some(1));
        assert_eq!(q.prev(), Some(2));
    }

    #[test]
    fn repeat_one_only_on_auto_advance() {
        let mut q = Queue::new();
        q.set_repeat(Repeat::One);
        q.set(vec![1, 2], 0);
        assert_eq!(q.next(true), Some(1));
        assert_eq!(q.next(false), Some(2));
    }

    #[test]
    fn up_next_comes_first_then_list_continues() {
        let mut q = Queue::new();
        q.set(vec![1, 2, 3], 0);
        q.enqueue(10);
        q.play_next(20);
        assert_eq!(q.upcoming(5), vec![20, 10, 2, 3]);
        assert_eq!(q.next(true), Some(20));
        assert_eq!(q.next(true), Some(10));
        assert_eq!(q.next(true), Some(2));
    }

    #[test]
    fn shuffle_keeps_current_and_visits_all() {
        let mut q = Queue::new();
        q.set((1..=50).collect(), 9);
        q.set_shuffle(true);
        assert_eq!(q.current(), Some(10));
        let mut seen = vec![10];
        while let Some(id) = q.next(true) {
            seen.push(id);
        }
        seen.sort();
        assert_eq!(seen, (1..=50).collect::<Vec<_>>());
        q.set_shuffle(false);
        assert_eq!(q.current(), q.items().get(q.current_index().unwrap()).copied());
    }

    #[test]
    fn clear_upcoming_keeps_only_current() {
        let mut q = Queue::new();
        q.set(vec![1, 2, 3, 4], 1);
        q.enqueue(9);
        q.clear_upcoming();
        assert_eq!(q.current(), Some(2));
        assert!(q.upcoming(10).is_empty());
        assert_eq!(q.next(true), None);
        // auch mit Zufall und während ein eingereihter Titel läuft
        let mut q = Queue::new();
        q.set((1..=20).collect(), 0);
        q.set_shuffle(true);
        q.play_next(42);
        assert_eq!(q.next(false), Some(42));
        q.clear_upcoming();
        assert_eq!(q.current(), Some(42));
        assert!(q.upcoming(10).is_empty());
    }

    #[test]
    fn append_extends_list() {
        let mut q = Queue::new();
        q.set(vec![1], 0);
        q.append(2);
        assert_eq!(q.next(true), Some(2));
    }
}
