use std::convert::TryFrom;
use std::fmt;

use anyhow::{anyhow, Result, Context};
use num_enum::TryFromPrimitive;


pub type Hz = f32;


#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Note {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
}


impl Note {
    pub fn next(&self) -> Self {
        use Note::*;
        match self {
            A => B,
            B => C,
            C => D,
            D => E,
            E => F,
            F => G,
            G => A,
        }
    }

    pub fn prev(&self) -> Self {
        use Note::*;
        match self {
            A => G,
            B => A,
            C => B,
            D => C,
            E => D,
            F => E,
            G => F,
        }
    }
}


impl TryFrom<char> for Note {
    type Error = anyhow::Error;
    fn try_from(c: char) -> Result<Self, Self::Error> {
        use Note::*;
        match c {
            'A' => Ok(A),
            'B' => Ok(B),
            'C' => Ok(C),
            'D' => Ok(D),
            'E' => Ok(E),
            'F' => Ok(F),
            'G' => Ok(G),
            _ => Err(anyhow!("unable to create Note from '{}'", c)),
        }
    }
}


impl fmt::Display for Note {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}


#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Pitch {
    Flat,
    Natural,
    Sharp,
}


impl Pitch {
    pub fn try_next(&self) -> Result<Self> {
        use Pitch::*;
        match self {
            Flat => Ok(Natural),
            Natural => Ok(Sharp),
            Sharp => Err(anyhow!("unable to shift pitch 'Sharp' up")),
        }
    }

    pub fn try_prev(&self) -> Result<Self> {
        use Pitch::*;
        match self {
            Flat => Err(anyhow!("unable to shift pitch 'Flat' down")),
            Natural => Ok(Flat),
            Sharp => Ok(Natural),
        }
    }
}


impl TryFrom<char> for Pitch {
    type Error = anyhow::Error;
    fn try_from(c: char) -> Result<Self, Self::Error> {
        match c {
            'b' | '♭' => Ok(Pitch::Flat),
            '♮' => Ok(Pitch::Natural),
            '#' | '♯' => Ok(Pitch::Sharp),
            _ => Err(anyhow!("unable to create Pitch from '{}'", c)),
        }
    }
}


impl fmt::Display for Pitch {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match self {
            Pitch::Flat => "♭",
            Pitch::Natural => "",
            Pitch::Sharp => "♯",
        })
    }
}


#[derive(Debug, Copy, Clone, Eq, PartialEq, TryFromPrimitive)]
#[repr(usize)]
pub enum Octave {
    Zero,
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
}


impl Octave {
    pub fn try_next(&self) -> Result<Self> {
        Ok(Octave::try_from(((*self) as i32 + 1) as usize)
            .with_context(|| format!("unable to shift '{:?}' up an octave", self))?)
    }

    pub fn try_prev(&self) -> Result<Self> {
        let self_i = (*self) as i32; // manually check bounds to combat underflow
        if self_i == 0 {
            Err(anyhow!("unable to shift '{:?}' down an octave", self))
        } else {
            Ok(Octave::try_from((self_i - 1) as usize).expect("already checked bounds"))
        }
    }
}


impl TryFrom<char> for Octave {
    type Error = anyhow::Error;
    fn try_from(c: char) -> Result<Self, Self::Error> {
        let err_f = || anyhow!("unable to create a Octave from '{}'", c);
        let octave_digit = c.to_digit(10).ok_or_else(err_f)? as usize;
        // wrap this result in Ok(..?) to allow anyhow to work its magic in Result conversion
        Ok(Octave::try_from(octave_digit)?)
    }
}


impl fmt::Display for Octave {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // unicode for subscript
        let codepoints = vec![0xe2, 0x82, 0x80 + ((*self) as i32) as u8];
        write!(f, "{}", std::str::from_utf8(codepoints.as_slice())
            .unwrap_or_else(|_| panic!("bad unicode for Octave '{:?}'", self)))
    }
}


/// Hiding the fields of `Tone` and providing getters like this allows for full external visibility
/// but blocked direct instantiation, which is very important as many implemented operations will
/// fail on invalid `Tone`s.
#[derive(Debug, PartialEq)]
pub struct Tone(Note, Pitch, Octave);


impl Tone {
    pub const FIXED_HZ: Hz = 440.0;
    pub const FIXED_TONE: Self = Tone(Note::A, Pitch::Natural, Octave::Four);

    pub fn new(note: Note, pitch: Pitch, octave: Octave) -> Result<Self> {
        let new_tone = Self(note, pitch, octave);
        // block creation for notes that fail semitone_rank (do not exist, e.g. Cb3)
        if new_tone.semitone_rank() < 0 {
            Err(anyhow!("specified Tone '{:?}' is invalid", new_tone))
        } else {
            Ok(new_tone)
        }
    }

    pub fn note(&self) -> Note {
        self.0
    }

    pub fn pitch(&self) -> Pitch {
        self.1
    }

    pub fn octave(&self) -> Octave {
        self.2
    }

    // NOTE: can't impl TryFrom for generic type param (like AsRef<str>):
    // https://github.com/rust-lang/rust/issues/50133
    pub fn try_from<S>(s: S) -> Result<Self>
    where
        S: AsRef<str>,
    {
        let s_str = s.as_ref();
        let s_len = s_str.chars().count(); // TODO: this will fail for certain unicode glyphs
        if s_len == 2 {  // e.g. A0
            let note = Note::try_from(s_str.chars().nth(0).unwrap_or('_'))?;
            let octave = Octave::try_from(s_str.chars().nth(1).unwrap_or('_'))?;
            Tone::new(note, Pitch::Natural, octave)
        } else if s_len == 3 { // e.g. F#7
            let note = Note::try_from(s_str.chars().nth(0).unwrap_or('_'))?;
            let pitch = Pitch::try_from(s_str.chars().nth(1).unwrap_or('_'))?;
            let octave = Octave::try_from(s_str.chars().nth(2).unwrap_or('_'))?;
            Tone::new(note, pitch, octave)
        } else {
            Err(anyhow!("unable to create a Tone from '{}'", s_str))
        }
    }

    pub fn semitone_rank(&self) -> i32 {
        use Note::*;
        use Pitch::*;
        match self {
            Tone(C, Natural, ..) => 1,
            Tone(C, Sharp,   ..) |
            Tone(D, Flat,    ..) => 2,
            Tone(D, Natural, ..) => 3,
            Tone(D, Sharp,   ..) |
            Tone(E, Flat,    ..) => 4,
            Tone(E, Natural, ..) => 5,
            Tone(F, Natural, ..) => 6,
            Tone(F, Sharp,   ..) |
            Tone(G, Flat,    ..) => 7,
            Tone(G, Natural, ..) => 8,
            Tone(G, Sharp,   ..) |
            Tone(A, Flat,    ..) => 9,
            Tone(A, Natural, ..) => 10,
            Tone(A, Sharp,   ..) |
            Tone(B, Flat,    ..) => 11,
            Tone(B, Natural, ..) => 12,
            _ => -1, // unreachable by anything but impls here
        }
    }

    pub fn semitone_distance_to(&self, to: &Tone) -> i32 {
        let inter_octave_dist = (self.octave() as i32) - (to.octave() as i32);
        let intra_octave_dist = self.semitone_rank() - to.semitone_rank();
        intra_octave_dist + (12 * inter_octave_dist)
    }

    /// Shift `self` to the specified `Octave`.
    pub fn with_octave(&self, octave: Octave) -> Self {
        Self(self.note(), self.pitch(), octave)
    }

    /// Shift `self` a semitone up.
    ///
    /// Fails if we've reached the highest note defined.
    // TODO: better to fail, or continue on indefinitely? Frequency is defined by a formula, so
    // there's no real reason to stop (besides being inaudible)
    pub fn semitone_up(&self) -> Result<Self> {
        let try_up = match self.pitch().try_next() {
            Ok(p) => Tone(self.note(), p, self.octave()),
            Err(_) => match self.note() {
                // B is the special case where we need to bump octave, can skip Cb
                Note::B => Tone(Note::C, Pitch::Natural, self.octave().try_next()?),
                n => Tone(n.next(), Pitch::Flat, self.octave()),
            },
        };
        let try_up_rank = try_up.semitone_rank();
        if try_up_rank > 0 && try_up_rank != self.semitone_rank() {
            Ok(try_up)
        } else {
            try_up.semitone_up()
        }
    }

    /// Shift `self` a semitone down.
    // TODO: this is close enough to semitone_up to almost be copypasta, worth trying to merge?
    pub fn semitone_down(&self) -> Result<Self> {
        let try_down = match self.pitch().try_prev() {
            Ok(p) => Tone(self.note(), p, self.octave()),
            Err(_) => match self.note() {
                // C is the special case where we need to drop octave, can skip B#
                Note::C => Tone(Note::B, Pitch::Natural, self.octave().try_prev()?),
                n => Tone(n.prev(), Pitch::Sharp, self.octave()),
            },
        };
        let try_down_rank = try_down.semitone_rank();
        if try_down_rank > 0 && try_down_rank != self.semitone_rank() {
            Ok(try_down)
        } else {
            try_down.semitone_down()
        }
    }

    /// Return the equivalent `Tone` if it exists.
    pub fn doppelganger(&self) -> Option<Self> {
        use Note::*;
        use Pitch::*;
        match self {
            Tone(C, Sharp, octave) => Some(Tone(D, Flat,  *octave)),
            Tone(D, Flat,  octave) => Some(Tone(C, Sharp, *octave)),
            Tone(D, Sharp, octave) => Some(Tone(E, Flat,  *octave)),
            Tone(E, Flat,  octave) => Some(Tone(D, Sharp, *octave)),
            Tone(F, Sharp, octave) => Some(Tone(G, Flat,  *octave)),
            Tone(G, Flat,  octave) => Some(Tone(F, Sharp, *octave)),
            Tone(G, Sharp, octave) => Some(Tone(A, Flat,  *octave)),
            Tone(A, Flat,  octave) => Some(Tone(G, Sharp, *octave)),
            Tone(A, Sharp, octave) => Some(Tone(B, Flat,  *octave)),
            Tone(B, Flat,  octave) => Some(Tone(A, Sharp, *octave)),
            _ => None,
        }
    }
}


impl fmt::Display for Tone {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}{}", self.note(), self.pitch(), self.octave())
    }
}


/// This would have been painful to implement manually!
///
/// Formula from [this fantastic MTU resource](https://pages.mtu.edu/~suits/notefreqs.html).
// TODO: precompute?
impl From<&Tone> for Hz {
    fn from(tone: &Tone) -> Self {
        match tone {
            t if t == &Tone::FIXED_TONE => Tone::FIXED_HZ,
            t => {
                let dist = t.semitone_distance_to(&Tone::FIXED_TONE);
                Tone::FIXED_HZ * (2.0f32).powf(1.0 / 12.0).powi(dist)
            },
        }
    }
}


impl From<Tone> for Hz {
    fn from(tone: Tone) -> Self {
        Hz::from(&tone)
    }
}


#[cfg(test)]
mod test {
    use super::*;

    use Note::*;
    use Pitch::*;
    use Octave::*;

    static EPSILON: Hz = 0.05; // source numbers were not very precise

    macro_rules! assert_delta {
        ($left:expr, $right:expr, $delta:expr) => {
            if ($left - $right).abs() >= $delta {
                panic!("assertion failed:\nleft:  {}\nright: {}\ndelta: {}", $left, $right, $delta);
            }
        };
    }

    #[test]
    fn test_tone_conversion() {
        let c0 = Tone::new(C, Sharp, Zero).unwrap();
        assert_delta!(Hz::from(&c0), 17.32, EPSILON);
        assert_delta!(Hz::from(Tone::new(F, Natural, Six).unwrap()), 1396.91, EPSILON);
        assert_delta!(Hz::from(Tone::new(G, Flat, Eight).unwrap()), 5919.91, EPSILON);
    }

    #[test]
    fn test_semitone_distance() {
        assert_eq!(Tone::FIXED_TONE.semitone_distance_to(&Tone::FIXED_TONE), 0);
    }

    #[test]
    fn test_tone_try_from() {
        assert_eq!(Tone::FIXED_TONE, Tone::try_from("A4").unwrap());
        assert_eq!(Tone::new(C, Sharp, Zero).unwrap(), Tone::try_from("C#0").unwrap());
    }

    #[test]
    fn test_semitone_bump() {
        let fs6 = Tone::new(F, Sharp, Six).unwrap();
        assert_eq!(fs6.semitone_up().unwrap(), Tone::new(G, Natural, Six).unwrap());
        assert_eq!(fs6.semitone_down().unwrap(), Tone::new(F, Natural, Six).unwrap());

        let c0 = Tone::new(C, Natural, Zero).unwrap();
        if let Ok(n) = c0.semitone_down() {
            panic!("cannot shift bottom note C0 down a semitone (got: {:?})", n);
        }
        assert_eq!(c0.semitone_up().unwrap(), Tone::new(C, Sharp, Zero).unwrap());

        let b8 = Tone::new(B, Natural, Eight).unwrap();
        if let Ok(n) = b8.semitone_up() {
            panic!("cannot shift top note B8 up a semitone (got: {:?})", n);
        }
        assert_eq!(b8.semitone_down().unwrap(), Tone::new(B, Flat, Eight).unwrap());
    }
}
