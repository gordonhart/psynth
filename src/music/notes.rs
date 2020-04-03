use std::fmt;


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


impl fmt::Display for Pitch {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match self {
            Pitch::Flat => "♭",
            Pitch::Natural => "",
            Pitch::Sharp => "♯",
        })
    }
}


#[derive(Debug, Copy, Clone, Eq, PartialEq)]
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
}
