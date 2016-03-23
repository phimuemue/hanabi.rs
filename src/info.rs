use std::cmp::Eq;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::hash::Hash;

use cards::*;

// trait representing information about a card
pub trait CardInfo {
    // get all a-priori possibilities
    fn get_all_possibilities(&self) -> Vec<Card> {
        let mut v = Vec::new();
        for &color in COLORS.iter() {
            for &value in VALUES.iter() {
                v.push(Card::new(color, value));
            }
        }
        v
    }
    // whether the card is possible
    fn is_possible(&self, card: &Card) -> bool;

    // TODO: have a borrow_possibilities to allow for more efficiency?

    // mark all current possibilities for the card
    fn get_possibilities(&self) -> Vec<Card> {
        let mut v = Vec::new();
        for &color in COLORS.iter() {
            for &value in VALUES.iter() {
                let card = Card::new(color, value);
                if self.is_possible(&card) {
                    v.push(card);
                }
            }
        }
        v
    }
    // get probability weight for the card
    #[allow(unused_variables)]
    fn get_weight(&self, card: &Card) -> u32 {
        1
    }
    fn get_weighted_possibilities(&self) -> Vec<(Card, u32)> {
        let mut v = Vec::new();
        for card in self.get_possibilities() {
            let weight = self.get_weight(&card);
            v.push((card, weight));
        }
        v
    }

    // mark a whole color as false
    fn mark_color_false(&mut self, color: &Color);
    // mark a color as correct
    fn mark_color_true(&mut self, color: &Color) {
        for other_color in COLORS.iter() {
            if other_color != color {
                self.mark_color_false(other_color);
            }
        }
    }
    fn mark_color(&mut self, color: &Color, is_color: bool) {
        if is_color {
            self.mark_color_true(color);
        } else {
            self.mark_color_false(color);
        }
    }

    // mark a whole value as false
    fn mark_value_false(&mut self, value: &Value);
    // mark a value as correct
    fn mark_value_true(&mut self, value: &Value) {
        for other_value in VALUES.iter() {
            if other_value != value {
                self.mark_value_false(other_value);
            }
        }
    }
    fn mark_value(&mut self, value: &Value, is_value: bool) {
        if is_value {
            self.mark_value_true(value);
        } else {
            self.mark_value_false(value);
        }
    }
}


// Represents hinted information about possible values of type T
pub trait Info<T> where T: Hash + Eq + Clone {
    // get all a-priori possibilities
    fn get_all_possibilities() -> Vec<T>;

    // get map from values to whether it's possible
    // true means maybe, false means no
    fn get_possibility_set(&self) -> &HashSet<T>;
    fn get_mut_possibility_set(&mut self) -> &mut HashSet<T>;

    // get what is now possible
    fn get_possibilities(&self) -> Vec<T> {
        self.get_possibility_set().iter().map(|t| t.clone()).collect::<Vec<T>>()
    }

    fn is_possible(&self, value: &T) -> bool {
        self.get_possibility_set().contains(value)
    }

    fn initialize() -> HashSet<T> {
        let mut possible_map : HashSet<T> = HashSet::new();
        for value in Self::get_all_possibilities().iter() {
            possible_map.insert(value.clone());
        }
        possible_map
    }

    fn mark_true(&mut self, value: &T) {
        let possible = self.get_mut_possibility_set();
        possible.clear();
        possible.insert(value.clone());
    }

    fn mark_false(&mut self, value: &T) {
        self.get_mut_possibility_set().remove(value);
    }

    fn mark(&mut self, value: &T, info: bool) {
        if info {
            self.mark_true(value);
        } else {
            self.mark_false(value);
        }
    }
}

#[derive(Debug)]
pub struct ColorInfo(HashSet<Color>);
impl ColorInfo {
    pub fn new() -> ColorInfo { ColorInfo(ColorInfo::initialize()) }
}
impl Info<Color> for ColorInfo {
    fn get_all_possibilities() -> Vec<Color> { COLORS.to_vec() }
    fn get_possibility_set(&self) -> &HashSet<Color> { &self.0 }
    fn get_mut_possibility_set(&mut self) -> &mut HashSet<Color> { &mut self.0 }
}

#[derive(Debug)]
pub struct ValueInfo(HashSet<Value>);
impl ValueInfo {
    pub fn new() -> ValueInfo { ValueInfo(ValueInfo::initialize()) }
}
impl Info<Value> for ValueInfo {
    fn get_all_possibilities() -> Vec<Value> { VALUES.to_vec() }
    fn get_possibility_set(&self) -> &HashSet<Value> { &self.0 }
    fn get_mut_possibility_set(&mut self) -> &mut HashSet<Value> { &mut self.0 }
}

// represents information only of the form:
// this color is/isn't possible, this value is/isn't possible
#[derive(Debug)]
pub struct SimpleCardInfo {
    pub color_info: ColorInfo,
    pub value_info: ValueInfo,
}
impl SimpleCardInfo {
    pub fn new() -> SimpleCardInfo {
        SimpleCardInfo {
            color_info: ColorInfo::new(),
            value_info: ValueInfo::new(),
        }
    }
}
impl CardInfo for SimpleCardInfo {
    fn get_possibilities(&self) -> Vec<Card> {
        let mut v = Vec::new();
        for &color in self.color_info.get_possibilities().iter() {
            for &value in self.value_info.get_possibilities().iter() {
                v.push(Card::new(color, value));
            }
        }
        v
    }
    fn is_possible(&self, card: &Card) -> bool {
        self.color_info.is_possible(&card.color) &&
        self.value_info.is_possible(&card.value)

    }
    fn mark_color_false(&mut self, color: &Color) {
        self.color_info.mark_false(color);

    }
    fn mark_value_false(&mut self, value: &Value) {
        self.value_info.mark_false(value);
    }
}
impl fmt::Display for SimpleCardInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut string = String::new();
        for color in &COLORS {
            if self.color_info.is_possible(color) {
                string.push(display_color(color));
            }
        }
        // while string.len() < COLORS.len() + 1 {
        string.push(' ');
        //}
        for value in &VALUES {
            if self.value_info.is_possible(value) {
                string.push_str(&format!("{}", value));
            }
        }
        f.pad(&string)
    }
}

// Can represent information of the form:
// this card is/isn't possible
// also, maintains weights for the cards
#[derive(Clone)]
pub struct CardPossibilityTable {
    possible: HashMap<Card, u32>,
}
impl CardPossibilityTable {
    pub fn new() -> CardPossibilityTable {
        let mut possible = HashMap::new();
        for &color in COLORS.iter() {
            for &value in VALUES.iter() {
                possible.insert(
                    Card::new(color, value),
                    get_count_for_value(&value)
                );
            }
        }
        CardPossibilityTable {
            possible: possible,
        }
    }

    // mark a possible card as false
    fn mark_false(&mut self, card: &Card) {
        self.possible.remove(card);
    }
}
impl CardInfo for CardPossibilityTable {
    fn is_possible(&self, card: &Card) -> bool {
        self.possible.contains_key(card)
    }
    fn get_possibilities(&self) -> Vec<Card> {
        let mut cards = self.possible.keys().map(|card| {card.clone() }).collect::<Vec<_>>();
        cards.sort();
        cards
    }
    fn mark_color_false(&mut self, color: &Color) {
        for &value in VALUES.iter() {
            self.mark_false(&Card::new(color, value));
        }

    }
    fn mark_value_false(&mut self, value: &Value) {
        for &color in COLORS.iter() {
            self.mark_false(&Card::new(color, value.clone()));
        }
    }
    fn get_weight(&self, card: &Card) -> u32 {
        *self.possible.get(card).unwrap_or(&0)
    }
}
impl fmt::Display for CardPossibilityTable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for card in self.get_possibilities() {
            try!(f.write_str(&format!("{}, ", card)));
        }
        Ok(())
    }
}
