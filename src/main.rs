use fixedbitset::FixedBitSet;
use std::fmt;
use std::io::Write;
use std::str;

fn main() {
    let guesses = words(include_str!("wordlist_guesses.txt"));
    let solutions = words(include_str!("wordlist_solutions.txt"));
    cheat(&guesses, solutions);
    //println!("{}", best_guess(&solutions, &guesses));
}

fn words(s: &str) -> Vec<Word> {
    s.lines().map(Word::new).collect()
}

fn cheat(guesses: &[Word], mut solutions: Vec<Word>) {
    let mut guess = "arise".to_string();

    loop {
        println!("# {} to refine {}", guess, summarise_words(&solutions));

        print!("> ");
        std::io::stdout().flush().unwrap();
        let mut buf = String::new();
        std::io::stdin().read_line(&mut buf).unwrap();
        let score = Score::from_str(buf, guess.as_bytes()).unwrap();
        solutions = solutions.into_iter().filter(|w| score.matches(w)).collect();
        match solutions.len() {
            0 => unreachable!(),
            1 => break,
            _ => (),
        }
        guess = best_guess(
            &solutions,
            if solutions.len() > 3 {
                guesses
            } else {
                &solutions
            },
        );
    }
    println!("only solution remaining is {}", solutions[0]);
}

fn summarise_words(words: &[Word]) -> String {
    if words.len() > 10 {
        format!("{} words", words.len())
    } else {
        words
            .iter()
            .map(|w| format!("{}", w))
            .collect::<Vec<String>>()
            .join(",")
    }
}

// Return the guess which minimises the maximum number of possible solutions
// remaining after that guess.
fn best_guess(solutions: &[Word], guesses: &[Word]) -> String {
    let g = guesses
        .iter()
        .min_by_key(|guess| {
            solutions
                .iter()
                .map(|s| {
                    // How many solutions match the score with this guess+solution?
                    let score = Score::new(s, guess);
                    solutions.iter().filter(|w| score.matches(w)).count()
                })
                .max()
                .unwrap()
        })
        .unwrap();
    format!("{}", g)
}

type Letter = usize;
type Position = usize;

#[derive(Debug, Clone)]
struct Word {
    orig: Vec<Letter>,
    letters: FixedBitSet,
}

impl Word {
    fn new(s: &str) -> Self {
        let orig: Vec<Letter> = s.bytes().map(|ch| (ch - b'a') as usize).collect();
        let letters: FixedBitSet = orig.iter().copied().collect();
        Self { letters, orig }
    }
}

impl fmt::Display for Word {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            self.orig
                .iter()
                .map(|ch| (*ch as u8 + b'a') as char)
                .collect::<String>()
        )
    }
}

#[derive(Debug)]
struct Score {
    present: FixedBitSet,
    missing: FixedBitSet,
    good_loc: Vec<(Letter, Position)>,
    bad_loc: Vec<(Letter, Position)>,
}

impl Score {
    fn new(solution: &Word, guess: &Word) -> Self {
        let mut score = Self::empty();

        for (pos, ch) in guess.orig.iter().enumerate() {
            (if solution.letters.contains(*ch) {
                (if solution.orig[pos] == *ch {
                    &mut score.good_loc
                } else {
                    &mut score.bad_loc
                })
                .push((*ch, pos));
                &mut score.present
            } else {
                &mut score.missing
            })
            .insert(*ch)
        }
        score
    }

    fn empty() -> Self {
        Self {
            present: FixedBitSet::with_capacity(26),
            missing: FixedBitSet::with_capacity(26),
            good_loc: Vec::with_capacity(5),
            bad_loc: Vec::with_capacity(5),
        }
    }

    // Generate score from string like g..y.
    // g (green) for letter in correct spot
    // y (yellow) for letter in wrong spot
    fn from_str(s: String, guess: &[u8]) -> Result<Self, String> {
        let s = s.trim();
        if s.len() != 5 {
            return Err(format!("expected 5 chars, got {}", s.len()));
        }
        let mut score = Self::empty();

        for (pos, code) in s.bytes().enumerate() {
            let (loc, present) = match code {
                b'g' => (Some(&mut score.good_loc), &mut score.present),
                b'y' => (Some(&mut score.bad_loc), &mut score.present),
                b'.' => (None, &mut score.missing),
                _ => unreachable!(),
            };
            let ch = (guess[pos] - b'a') as usize;
            if let Some(loc) = loc {
                loc.push((ch, pos));
            }
            present.insert(ch);
        }
        Ok(score)
    }
    fn matches(&self, w: &Word) -> bool {
        self.present.is_subset(&w.letters)
            && self.missing.is_disjoint(&w.letters)
            && self.good_loc.iter().all(|(ch, pos)| w.orig[*pos] == *ch)
            && !self.bad_loc.iter().any(|(ch, pos)| w.orig[*pos] == *ch)
    }
}
