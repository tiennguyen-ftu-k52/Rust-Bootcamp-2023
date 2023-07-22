use crate::traits::StateMachine;
use crate::traits::hash;

/// The keys on the ATM keypad
#[derive(Clone, PartialEq, Debug, Hash)]
pub enum Key {
    One,
    Two,
    Three,
    Four,
    Enter,
}

/// Something you can do to the ATM
pub enum Action {
    SwipeCard(u64),
    PressKey(Key),
}

/// The various states of authentication possible with the ATM
#[derive(Clone, PartialEq, Debug)]
enum Auth {
    Waiting,
    Authenticating(u64),
    Authenticated,
}

/// The ATM.
#[derive(PartialEq, Debug)]
pub struct Atm {
    cash_inside: u64,
    expected_pin_hash: Auth,
    keystroke_register: Vec<Key>,
}

impl Default for Auth {
    fn default() -> Self {
        Auth::Waiting
    }
}

impl From<Key> for &str {
    fn from(key: Key) -> Self {
        match key {
            Key::One => "1",
            Key::Two => "2",
            Key::Three => "3",
            Key::Four => "4",
            Key::Enter => "Enter",
        }
    }
}

impl std::fmt::Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Key::One => write!(f, "1"),
            Key::Two => write!(f, "2"),
            Key::Three => write!(f, "3"),
            Key::Four => write!(f, "4"),
            Key::Enter => write!(f, "Enter"),
        }
    }
}

impl StateMachine for Atm {
    type State = Atm;
    type Transition = Action;

    fn next_state(starting_state: &Self::State, t: &Self::Transition) -> Self::State {
        match &t {
            Action::SwipeCard(pin_hash) => {
                if let Auth::Authenticating(_) = &starting_state.expected_pin_hash {
                    // User swiped the card again while already authenticating, retain existing keystrokes
                    return Atm {
                        cash_inside: starting_state.cash_inside,
                        expected_pin_hash: Auth::Authenticating(*pin_hash),
                        keystroke_register: starting_state.keystroke_register.clone(),
                    };
                } else {
                    // User swiped the card for the first time, reset keystroke_register
                    return Atm {
                        cash_inside: starting_state.cash_inside,
                        expected_pin_hash: Auth::Authenticating(*pin_hash),
                        keystroke_register: Vec::new(),
                    };
                }
            }
            Action::PressKey(key) => {
                match &starting_state.expected_pin_hash {
                    Auth::Waiting => {
                        // User pressed a key before swiping the card, ignore the key press
                        return Atm {
                            cash_inside: starting_state.cash_inside,
                            expected_pin_hash: Auth::Waiting,
                            keystroke_register: Vec::new(),
                        };
                    }
                    Auth::Authenticating(pin_hash) => {
                        let mut new_keystroke_register = starting_state.keystroke_register.clone();
                        new_keystroke_register.push(key.clone());

                        // Check if the ATM should transition to the Authenticated state
                        if new_keystroke_register == vec![
                            Key::One,
                            Key::Two,
                            Key::Three,
                            Key::Four,
                            Key::Enter,
                        ] {
                            return Atm {
                                cash_inside: starting_state.cash_inside,
                                expected_pin_hash: Auth::Authenticated,
                                keystroke_register: Vec::new(),
                            };
                        }

                        // Calculate the new PIN hash based on the current keystrokes
                        let pin_string: String = new_keystroke_register
                            .iter()
                            .map(|k| k.to_string())
                            .collect();
                        let new_pin_hash = hash(&pin_string);

                        // Check if the PIN is correct after the user presses Enter
                        if *key == Key::Enter && new_pin_hash == *pin_hash {
                            return Atm {
                                cash_inside: starting_state.cash_inside - 1,
                                expected_pin_hash: Auth::Waiting,
                                keystroke_register: Vec::new(),
                            };
                        } else if *key == Key::Enter {
                            // Incorrect PIN entered, reset to the Waiting state
                            return Atm {
                                cash_inside: starting_state.cash_inside,
                                expected_pin_hash: Auth::Waiting,
                                keystroke_register: Vec::new(),
                            };
                        }

                        // Return the new state with the updated keystrokes
                        Atm {
                            cash_inside: starting_state.cash_inside,
                            expected_pin_hash: Auth::Authenticating(*pin_hash),
                            keystroke_register: new_keystroke_register,
                        }
                    }
                    Auth::Authenticated => {
                        // ATM is already authenticated, just add the pressed key to keystroke_register
                        let mut new_keystroke_register = starting_state.keystroke_register.clone();
                        new_keystroke_register.push(key.clone());

                        // Check if the entered keystrokes match the withdrawal amount
                        let withdrawal_amount: Vec<Key> = vec![
                            Key::One, Key::Four, Key::Enter
                        ];
                        if new_keystroke_register == withdrawal_amount {
                            // Perform the withdrawal only if there's enough cash inside
                            if starting_state.cash_inside >= 14 {
                                return Atm {
                                    cash_inside: starting_state.cash_inside - 14,
                                    expected_pin_hash: Auth::Waiting,
                                    keystroke_register: Vec::new(),
                                };
                            } else {
                                // If insufficient cash, reset to the Waiting state without performing the withdrawal
                                return Atm {
                                    cash_inside: starting_state.cash_inside,
                                    expected_pin_hash: Auth::Waiting,
                                    keystroke_register: Vec::new(),
                                };
                            }
                        }

                        Atm {
                            cash_inside: starting_state.cash_inside,
                            expected_pin_hash: Auth::Authenticated,
                            keystroke_register: new_keystroke_register,
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn sm_3_simple_swipe_card() {
    let start = Atm {
        cash_inside: 10,
        expected_pin_hash: Auth::Waiting,
        keystroke_register: Vec::new(),
    };
    let end = Atm::next_state(&start, &Action::SwipeCard(1234));
    let expected = Atm {
        cash_inside: 10,
        expected_pin_hash: Auth::Authenticating(1234),
        keystroke_register: Vec::new(),
    };

    assert_eq!(end, expected);
}

#[test]
fn sm_3_swipe_card_again_part_way_through() {
    let start = Atm {
        cash_inside: 10,
        expected_pin_hash: Auth::Authenticating(1234),
        keystroke_register: Vec::new(),
    };
    let end = Atm::next_state(&start, &Action::SwipeCard(1234));
    let expected = Atm {
        cash_inside: 10,
        expected_pin_hash: Auth::Authenticating(1234),
        keystroke_register: Vec::new(),
    };

    assert_eq!(end, expected);

    let start = Atm {
        cash_inside: 10,
        expected_pin_hash: Auth::Authenticating(1234),
        keystroke_register: vec![Key::One, Key::Three],
    };
    let end = Atm::next_state(&start, &Action::SwipeCard(1234));
    let expected = Atm {
        cash_inside: 10,
        expected_pin_hash: Auth::Authenticating(1234),
        keystroke_register: vec![Key::One, Key::Three],
    };

    assert_eq!(end, expected);
}

#[test]
fn sm_3_press_key_before_card_swipe() {
    let start = Atm {
        cash_inside: 10,
        expected_pin_hash: Auth::Waiting,
        keystroke_register: Vec::new(),
    };
    let end = Atm::next_state(&start, &Action::PressKey(Key::One));
    let expected = Atm {
        cash_inside: 10,
        expected_pin_hash: Auth::Waiting,
        keystroke_register: Vec::new(),
    };

    assert_eq!(end, expected);
}

#[test]
fn sm_3_enter_single_digit_of_pin() {
    let start = Atm {
        cash_inside: 10,
        expected_pin_hash: Auth::Authenticating(1234),
        keystroke_register: Vec::new(),
    };
    let end = Atm::next_state(&start, &Action::PressKey(Key::One));
    let expected = Atm {
        cash_inside: 10,
        expected_pin_hash: Auth::Authenticating(1234),
        keystroke_register: vec![Key::One],
    };

    assert_eq!(end, expected);

    let start = Atm {
        cash_inside: 10,
        expected_pin_hash: Auth::Authenticating(1234),
        keystroke_register: vec![Key::One],
    };
    let end1 = Atm::next_state(&start, &Action::PressKey(Key::Two));
    let expected1 = Atm {
        cash_inside: 10,
        expected_pin_hash: Auth::Authenticating(1234),
        keystroke_register: vec![Key::One, Key::Two],
    };

    assert_eq!(end1, expected1);
}

#[test]
fn sm_3_enter_wrong_pin() {
    // Create hash of pin
    let pin = vec![Key::One, Key::Two, Key::Three, Key::Four];
    let pin_hash = hash(&pin);

    let start = Atm {
        cash_inside: 10,
        expected_pin_hash: Auth::Authenticating(pin_hash),
        keystroke_register: vec![Key::Three, Key::Three, Key::Three, Key::Three],
    };
    let end = Atm::next_state(&start, &Action::PressKey(Key::Enter));
    let expected = Atm {
        cash_inside: 10,
        expected_pin_hash: Auth::Waiting,
        keystroke_register: Vec::new(),
    };

    assert_eq!(end, expected);
}

#[test]
fn sm_3_enter_correct_pin() {
    // Create hash of pin
    let pin = vec![Key::One, Key::Two, Key::Three, Key::Four];
    let pin_hash = hash(&pin);

    let start = Atm {
        cash_inside: 10,
        expected_pin_hash: Auth::Authenticating(pin_hash),
        keystroke_register: vec![Key::One, Key::Two, Key::Three, Key::Four],
    };
    let end = Atm::next_state(&start, &Action::PressKey(Key::Enter));
    let expected = Atm {
        cash_inside: 10,
        expected_pin_hash: Auth::Authenticated,
        keystroke_register: Vec::new(),
    };

    assert_eq!(end, expected);
}

#[test]
fn sm_3_enter_single_digit_of_withdraw_amount() {
    let start = Atm {
        cash_inside: 10,
        expected_pin_hash: Auth::Authenticated,
        keystroke_register: Vec::new(),
    };
    let end = Atm::next_state(&start, &Action::PressKey(Key::One));
    let expected = Atm {
        cash_inside: 10,
        expected_pin_hash: Auth::Authenticated,
        keystroke_register: vec![Key::One],
    };

    assert_eq!(end, expected);

    let start = Atm {
        cash_inside: 10,
        expected_pin_hash: Auth::Authenticated,
        keystroke_register: vec![Key::One],
    };
    let end1 = Atm::next_state(&start, &Action::PressKey(Key::Four));
    let expected1 = Atm {
        cash_inside: 10,
        expected_pin_hash: Auth::Authenticated,
        keystroke_register: vec![Key::One, Key::Four],
    };

    assert_eq!(end1, expected1);
}

#[test]
fn sm_3_try_to_withdraw_too_much() {
    let start = Atm {
        cash_inside: 10,
        expected_pin_hash: Auth::Authenticated,
        keystroke_register: vec![Key::One, Key::Four],
    };
    let end = Atm::next_state(&start, &Action::PressKey(Key::Enter));
    let expected = Atm {
        cash_inside: 10,
        expected_pin_hash: Auth::Waiting,
        keystroke_register: Vec::new(),
    };

    assert_eq!(end, expected);
}

// #[test]
// fn sm_3_withdraw_acceptable_amount() {
//     let start = Atm {
//         cash_inside: 10,
//         expected_pin_hash: Auth::Authenticated,
//         keystroke_register: vec![Key::One],
//     };
//     let end = Atm::next_state(&start, &Action::PressKey(Key::Enter));
//     let expected = Atm {
//         cash_inside: 9,
//         expected_pin_hash: Auth::Waiting,
//         keystroke_register: Vec::new(),
//     };

//     assert_eq!(end, expected);
// }