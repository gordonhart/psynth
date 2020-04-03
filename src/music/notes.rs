use std::convert::TryFrom;
use std::fmt;

use anyhow::{anyhow, Result};
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


impl TryFrom<char> for Note {
    type Error = anyhow::Error;
    fn try_from(c: char) -> Result<Self, Self::Error> {
        match c {
            'A' => Ok(Note::A),
            'B' => Ok(Note::B),
            'C' => Ok(Note::C),
            'D' => Ok(Note::D),
            'E' => Ok(Note::E),
            'F' => Ok(Note::F),
            'G' => Ok(Note::G),
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
        write!(f, "{}", (*self) as i32)
    }
}


#[derive(Debug, PartialEq)]
pub struct Tone {
    pub note: Note,
    pub pitch: Pitch,
    pub octave: Octave,
}


impl Tone {
    const FIXED_HZ: Hz = 440.0;
    const FIXED_TONE: Self = Tone { note: Note::A, pitch: Pitch::Natural, octave: Octave::Four };

    pub fn new(note: Note, pitch: Pitch, octave: Octave) -> Self {
        Self { note, pitch, octave }
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
            Ok(Tone::new(note, Pitch::Natural, octave))
        } else if s_len == 3 { // e.g. F#7
            let note = Note::try_from(s_str.chars().nth(0).unwrap_or('_'))?;
            let pitch = Pitch::try_from(s_str.chars().nth(1).unwrap_or('_'))?;
            let octave = Octave::try_from(s_str.chars().nth(2).unwrap_or('_'))?;
            Ok(Tone::new(note, pitch, octave))
        } else {
            Err(anyhow!("unable to create a Tone from '{}'", s_str))
        }
    }

    pub fn semitone_rank(&self) -> i32 {
        use Note::*;
        use Pitch::*;
        match self {
            Tone { note: C, pitch: Natural, .. } => 1,
            Tone { note: C, pitch: Sharp,   .. } |
            Tone { note: D, pitch: Flat,    .. } => 2,
            Tone { note: D, pitch: Natural, .. } => 3,
            Tone { note: D, pitch: Sharp,   .. } |
            Tone { note: E, pitch: Flat,    .. } => 4,
            Tone { note: E, pitch: Natural, .. } => 5,
            Tone { note: F, pitch: Natural, .. } => 6,
            Tone { note: F, pitch: Sharp,   .. } |
            Tone { note: G, pitch: Flat,    .. } => 7,
            Tone { note: G, pitch: Natural, .. } => 8,
            Tone { note: G, pitch: Sharp,   .. } |
            Tone { note: A, pitch: Flat,    .. } => 9,
            Tone { note: A, pitch: Natural, .. } => 10,
            Tone { note: A, pitch: Sharp,   .. } |
            Tone { note: B, pitch: Flat,    .. } => 11,
            Tone { note: B, pitch: Natural, .. } => 12,
            t => panic!("does '{}' exist?", t),
        }
    }

    pub fn semitone_distance_to(&self, to: &Tone) -> i32 {
        let inter_octave_dist = (self.octave as i32) - (to.octave as i32);
        let intra_octave_dist = self.semitone_rank() - to.semitone_rank();
        intra_octave_dist + (12 * inter_octave_dist)
    }
}


impl fmt::Display for Tone {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}{}", self.note, self.pitch, self.octave)
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
        let c0 = Tone::new(Note::C, Pitch::Sharp, Octave::Zero);
        assert_delta!(Hz::from(&c0), 17.32, EPSILON);
        assert_delta!(Hz::from(Tone::new(Note::F, Pitch::Natural, Octave::Six)), 1396.91, EPSILON);
        assert_delta!(Hz::from(Tone::new(Note::G, Pitch::Flat, Octave::Eight)), 5919.91, EPSILON);
    }

    #[test]
    fn test_semitone_distance() {
        assert_eq!(Tone::FIXED_TONE.semitone_distance_to(&Tone::FIXED_TONE), 0);
    }

    #[test]
    fn test_tone_try_from() {
        assert_eq!(Tone::FIXED_TONE, Tone::try_from("A4").unwrap());
        assert_eq!(Tone::new(Note::C, Pitch::Sharp, Octave::Zero), Tone::try_from("C#0").unwrap());
    }
}
