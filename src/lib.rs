extern crate rand;
use self::rand::{thread_rng, Rng};

#[derive(Clone)]
pub struct DiceRolls {
    rolls: Vec<DiceRoll>,
    formula: Vec<String>,
    rolls_formula: Vec<String>,
}

impl DiceRolls {
    /// Returns an i32 as the result of the formula including any calculational
    /// operators.
    pub fn get_result(&self) -> i32 {
        return solve_rpn_formula(self.formula.clone());
    }

    /// Returns an i32 as the simple sum of all rolls.
    pub fn get_sum_of_rolls(&self) -> i32 {
        let mut total = 0;
        for roll in &self.rolls {
            total += roll.result;
        }
        return total;
    }

    /// Returns a formatted String showing the dice and the rolled results.
    ///
    /// # Remarks
    ///
    /// From my current experimentation, this appears to be close to ~O(c^n).
    /// While it doesn't get slow until ludicrous numbers to dice rolls (1+ million),
    /// there's probably a better way to do this.
    pub fn get_rolls_string(&self) -> String {
        let mut rolls_string = String::new();
        for (i, roll) in self.rolls.iter().enumerate() {
            if i == self.rolls.len() - 1 {
                rolls_string.push_str(format!("d{} -> [{}]", roll.sides, roll.result).as_ref());
                break;
            }
            rolls_string.push_str(format!("d{} -> [{}], ", roll.sides, roll.result).as_ref());
        }
        return rolls_string;
    }

    /// Returns a postfix formatted String showing the formula.
    pub fn get_formula_string_as_rpn(&self) -> String {
        let mut formula_string = String::new();
        for (i, fragment) in self.formula.iter().enumerate() {
            if match_token(fragment) > 0 {
                formula_string.push_str(format!("{} ", fragment).as_ref());
                continue;
            }

            if i == self.formula.len() - 1 {
                formula_string.push_str(format!("[{}]", fragment).as_ref());
                break;
            }

            formula_string.push_str(format!("[{}] ", fragment).as_ref());
        }
        return formula_string;
    }

    /// Returns an infix formatted String showing the formula.
    pub fn get_formula_string_as_infix(&self) -> String {
        return parse_into_infix(self.formula.clone()).replace("( ", "[").replace(" )", "]");
    }

    /// Returns a postfix formatted String showing the formula withthe original dice notation instead of the rolled result.
    pub fn get_rolls_formula_string_as_rpn(&self) -> String {
        let mut formula_string = String::new();
        for (i, fragment) in self.rolls_formula.iter().enumerate() {
            if match_token(fragment) > 0 {
                formula_string.push_str(format!("{} ", fragment).as_ref());
                continue;
            }

            if i == self.rolls_formula.len() - 1 {
                formula_string.push_str(format!("[{}]", fragment).as_ref());
                break;
            }

            formula_string.push_str(format!("[{}] ", fragment).as_ref());
        }
        return formula_string;
    }

    /// Returns a infix formatted String showing the formula withthe original dice notation instead of the rolled result.
    pub fn get_rolls_formula_string_as_infix(&self) -> String {
        return parse_into_infix(self.rolls_formula.clone()).replace("( ", "[").replace(" )", "]");
    }
}

#[derive(Clone, Copy)]
pub struct DiceRoll {
    sides: i32,
    result: i32,
}

/// Returns a DiceRolls object based on the provided formula.
///
/// # Arguments
/// * `input` - A string that provides the dice notation to work off.
pub fn roll(input: String) -> DiceRolls {
    let formula_vector = parse_into_rpn(input.trim().as_ref());
    return resolve_rolls_vector(formula_vector);
}

/// Returns a Vector of Strings with each element containing a token or an operator in postfix (rpn) format.
///
/// # Arguments
/// * `input_formula` - A string that provides the notation to work off.
///
/// # Example values
///
/// * `3 + 4 * 6` -> `["3", "4", "6", "*", "+"]`
/// * `2d4 + d6 + d4` -> `["2d4", "d6", "d4", "+", "+"]`
/// * `xv * (ab + dc)` -> `["xv", "ab", "dc", "+", "*"]`
pub fn parse_into_rpn(input_formula: &str) -> Vec<String> {
    let formula = input_formula.replace(" ", "").replace("_", "");
    let mut formula_vector: Vec<String> = Vec::new();
    let mut active_segment = String::new();
    let mut operator_stack: Vec<String> = Vec::new();
    let mut lorb = false;

    for c in formula.chars() {
        let cs = c.to_string();
        let precedence = match_token(cs.as_ref());

        match precedence {
            // Current token is an operator token
            p if p > 0 => if active_segment.len() > 0 {
                formula_vector.push(active_segment.clone());
                active_segment = String::new();
                while let Some(top) = operator_stack.pop() {
                    if match_token(top.as_ref()) >= precedence {
                        formula_vector.push(top.to_string());
                    } else {
                        operator_stack.push(top);
                        break;
                    }
                }
                operator_stack.push(cs);
            } else if lorb {
                operator_stack.push(cs);
            } else {
                active_segment.push(c);
            },
            // Current token is a left bracket token
            p if p == -1 => {
                lorb = false;
                operator_stack.push(cs);
            }
            // Current token is a right bracket token
            p if p == -2 => {
                if active_segment.len() > 0 {
                    formula_vector.push(active_segment.clone());
                    active_segment = String::new();
                    lorb = true;
                }
                while let Some(top) = operator_stack.pop() {
                    if match_token(top.as_ref()) == -1 {
                        break;
                    }
                    formula_vector.push(top.to_string());
                }
            }
            // Current token is a standard token
            _ => {
                lorb = false;
                active_segment.push(c);
            }
        }
    }

    if active_segment.len() > 0 {
        formula_vector.push(active_segment);
    }

    while let Some(top) = operator_stack.pop() {
        formula_vector.push(top.to_string());
    }

    return formula_vector;
}

#[test]
fn parse_rpn_formula() {
    assert_eq!(vec!["3", "4", "+"], parse_into_rpn("3 + 4"));
    assert_eq!(
        vec!["3", "4", "2", "1", "−", "×", "+"],
        parse_into_rpn("3 + 4 × (2 − 1)")
    );
    assert_eq!(
        vec!["2", "1", "−", "3", "×", "4", "+"],
        parse_into_rpn("(2 − 1) × 3 + 4")
    );
    assert_eq!(vec!["x", "y", "+"], parse_into_rpn("x + y"));
    assert_eq!(
        vec!["ab", "cd", "ef", "gh", "−", "×", "+"],
        parse_into_rpn("ab + cd × (ef − gh)")
    );
    assert_eq!(
        vec!["2d5", "1d6", "−", "3d6", "×", "2d12", "+"],
        parse_into_rpn("(2d5 − 1d6) × 3d6 + 2d12")
    );
}

/// Returns a Vector of Strings with each element containing a token or an operator in bracketed infix format.
///
/// # Arguments
/// * `input_formula` - A Vector of Strings that provides the postfix formatted notation to work off.
/// See [rfyl::parse_into_rpn()](fn.parse_into_rpn.html) for more details.
///
/// # Example values
///
/// * `["3", "4", "6", "*", "+"]` -> `["(", "3", "+", "(", "4", "*", "6", ")", ")"]`
pub fn parse_into_infix(input_formula: Vec<String>) -> String {
    let mut formula_vector: Vec<String> = Vec::new();
    let mut formula_string = String::new();

    for e in input_formula {
        let precedence = match_token(e.as_ref());

        match precedence {
            // Operator
            p if p > 0 => if formula_vector.len() < 2 {
                panic!("Insufficient values in expression start");
            } else {
                if let Some(a) = formula_vector.pop() {
                    if let Some(b) = formula_vector.pop() {
                        formula_vector.push(format!("( {0} {1} {2} )", b, e, a));
                    } else {
                        panic!("Right hand token in evaluation doesn't exist");
                    }
                } else {
                    panic!("Left hand token in evaluation doesn't exist");
                }
            },
            // Non-operator
            _ => {
                formula_vector.push(e);
            }
        }
    }

    if formula_vector.len() == 1 {
        formula_string = formula_vector[0].to_string();
    } else if formula_vector.len() > 1 {
        panic!("Too many values in postfix formula. Please verify the formula.");
    } else if formula_vector.len() < 1 {
        panic!("Not enough values in postfix formula. Please verify the formula.");
    }

    return formula_string;
}

#[test]
fn parse_infix_formula() {
    assert_eq!(
        "( 3 + 4 )",
        parse_into_infix(vec!["3".to_string(), "4".to_string(), "+".to_string()])
    );
    assert_eq!(
        "( 3 + ( 4 × ( 2 − 1 ) ) )",
        parse_into_infix(vec![
            "3".to_string(),
            "4".to_string(),
            "2".to_string(),
            "1".to_string(),
            "−".to_string(),
            "×".to_string(),
            "+".to_string(),
        ])
    );
    assert_eq!(
        "( ( ( 2 − 1 ) × 3 ) + 4 )",
        parse_into_infix(vec![
            "2".to_string(),
            "1".to_string(),
            "−".to_string(),
            "3".to_string(),
            "×".to_string(),
            "4".to_string(),
            "+".to_string(),
        ])
    );
}

/// Returns an i32 as the result of a postfix (rpn) formula.
///
/// # Arguments
/// * `formula` - A Vector of Strings that provides the postfix formatted notation to work off.
/// See [rfyl::parse_into_rpn()](fn.parse_into_rpn.html) for more details.
///
/// # Example values
///
/// * `["3", "4", "6", "*", "+"]` -> `27`
pub fn solve_rpn_formula(formula: Vec<String>) -> i32 {
    let mut working_stack: Vec<i32> = Vec::new();
    let mut total: i32 = 0;
    for e in formula.iter() {
        if e.parse::<i32>().is_ok() {
            working_stack.push(e.parse::<i32>().unwrap());
        } else {
            if let Some(a) = working_stack.pop() {
                if let Some(b) = working_stack.pop() {
                    match match_token(e) {
                        4 => {
                            if a == 0 {panic!("Divide by zero: `{} / {}` is undefined", b, a);}
                            working_stack.push((b as f32 / a as f32).round() as i32)
                        },
                        3 => working_stack.push(b * a),
                        2 => working_stack.push(b + a),
                        1 => working_stack.push(b - a),
                        _ => panic!("Invalid operator: `{}`", e),
                    }
                } else {
                    panic!("Right hand token in evaluation doesn't exist");
                }
            } else {
                panic!("Left hand token in evaluation doesn't exist");
            }
        }
    }
    if let Some(t) = working_stack.pop() {
        total = t;
    }
    return total;
}

#[test]
fn solve_rpn() {
    assert_eq!(
        6,
        solve_rpn_formula(vec![
            "4".to_string(),
            "2".to_string(),
            "+".to_string(),
        ])
    );
    assert_eq!(
        5,
        solve_rpn_formula(vec![
            "2".to_string(),
            "2".to_string(),
            "*".to_string(),
            "4".to_string(),
            "4".to_string(),
            "*".to_string(),
            "+".to_string(),
            "4".to_string(),
            "/".to_string(),
        ])
    );
}

fn resolve_rolls_vector(rolls_vector: Vec<String>) -> DiceRolls {
    let mut formula_vector: Vec<String> = Vec::new();
    let mut formula_vector_with_rolls: Vec<String> = Vec::new();
    let mut dice_rolls: Vec<DiceRoll> = Vec::new();

    for element in rolls_vector {
        // Ignore if element is recognised as a token.
        if match_token(element.as_ref()) > 0 {
            formula_vector.push(element.clone());
            formula_vector_with_rolls.push(element);
            continue;
        }

        let roll = resolve_roll_fragment(element.as_ref());

        for i_roll in roll.clone().rolls {
            dice_rolls.push(i_roll);
        }

        formula_vector.push(roll.get_sum_of_rolls().to_string());
        formula_vector_with_rolls.push(element);
    }

    return DiceRolls {
        rolls: dice_rolls,
        formula: formula_vector,
        rolls_formula: formula_vector_with_rolls,
    };
}

fn resolve_roll_fragment(input_fragment: &str) -> DiceRolls {
    let mut rng = thread_rng();
    let mut dice_count_str = String::new();
    let mut dice_sides_str = String::new();
    let mut d_switch: bool = false;
    let mut dice_rolls: Vec<DiceRoll> = Vec::new();
    let mut sum: i32 = 0;
    let dice_count: i32;
    let dice_sides: i32;

    if input_fragment.parse::<i32>().is_ok() {
        let current_roll = DiceRoll {
            sides: 0,
            result: input_fragment.parse::<i32>().unwrap(),
        };

        dice_rolls.push(current_roll);
        sum += current_roll.result;
    } else {
        for (i, c) in input_fragment.chars().enumerate() {
            if !d_switch {
                if c.to_string() == "d" {
                    d_switch = true;
                    if i == 0 {
                        dice_count_str.push_str("1");
                    }
                    continue;
                }
                dice_count_str.push(c);
            } else {
                dice_sides_str.push(c);
            }
        }

        if dice_count_str.parse::<i32>().is_ok() {
            dice_count = dice_count_str.parse::<i32>().unwrap();            
        } else {
            panic!("Dice count value: `{}` is invalid", dice_count_str);
        }

        if dice_sides_str.parse::<i32>().is_ok() {
            dice_sides = dice_sides_str.parse::<i32>().unwrap();            
        } else if match_token(dice_sides_str.as_ref()) == -3 {
            dice_sides = 100;
        } else {
            panic!("Dice sides value: `{}` is invalid", dice_sides_str);            
        }
                
        for _ in 0..dice_count {
            let current_roll = DiceRoll {
                sides: dice_sides,
                result: rng.gen_range(1, dice_sides),
            };

            dice_rolls.push(current_roll);
            sum += current_roll.result;
        }
    }

    return DiceRolls {
        rolls: dice_rolls,
        formula: vec![sum.to_string()],
        rolls_formula: vec![input_fragment.to_string()],
    };
}

fn match_token(token: &str) -> i32 {
    match token {
        "/" => return 4,
        "÷" => return 4,
        "*" => return 3,
        "×" => return 3,
        "+" => return 2,
        "−" => return 1,
        "-" => return 1,
        "(" => return -1,
        ")" => return -2,
        "%" => return -3,
        _ => return 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roll_from_string() {
        println!();
        let roll0 = roll("2d4".to_string());
        println!("Rolls:             {}", roll0.get_rolls_string());
        println!("RPN Formula:       {}", roll0.get_formula_string_as_rpn());
        println!("Formula:           {}", roll0.get_formula_string_as_infix());
        println!("RPN Rolls Formula: {}", roll0.get_rolls_formula_string_as_rpn());
        println!("Rolls Formula:     {}", roll0.get_rolls_formula_string_as_infix());
        println!("Result:            {}", roll0.get_result());
        println!();

        let roll1 = roll("(2d6 - 1d8) * (3d4 + 4d12)".to_string());
        println!("Rolls:             {}", roll1.get_rolls_string());
        println!("RPN Formula:       {}", roll1.get_formula_string_as_rpn());
        println!("Formula:           {}", roll1.get_formula_string_as_infix());
        println!("RPN Rolls Formula: {}", roll1.get_rolls_formula_string_as_rpn());
        println!("Rolls Formula:     {}", roll1.get_rolls_formula_string_as_infix());
        println!("Result:            {}", roll1.get_result());
        println!();

        let roll2 = roll("3d% + d%".to_string());
        println!("Rolls:             {}", roll2.get_rolls_string());
        println!("RPN Formula:       {}", roll2.get_formula_string_as_rpn());
        println!("Formula:           {}", roll2.get_formula_string_as_infix());
        println!("RPN Rolls Formula: {}", roll2.get_rolls_formula_string_as_rpn());
        println!("Rolls Formula:     {}", roll2.get_rolls_formula_string_as_infix());
        println!("Result:            {}", roll2.get_result());
        println!();

        let roll3 = roll("d100 / 15".to_string());
        println!("Rolls:             {}", roll3.get_rolls_string());
        println!("RPN Formula:       {}", roll3.get_formula_string_as_rpn());
        println!("Formula:           {}", roll3.get_formula_string_as_infix());
        println!("RPN Rolls Formula: {}", roll3.get_rolls_formula_string_as_rpn());
        println!("Rolls Formula:     {}", roll3.get_rolls_formula_string_as_infix());
        println!("Result:            {}", roll3.get_result());
        println!();

        let roll4 = roll("1d4 + 2d6 * 3d2 / 4d8 + (2d6 + 3d8) - 16 * (1 / 1d4)".to_string());
        println!("Rolls:             {}", roll4.get_rolls_string());
        println!("RPN Formula:       {}", roll4.get_formula_string_as_rpn());
        println!("Formula:           {}", roll4.get_formula_string_as_infix());
        println!("RPN Rolls Formula: {}", roll4.get_rolls_formula_string_as_rpn());
        println!("Rolls Formula:     {}", roll4.get_rolls_formula_string_as_infix());
        println!("Result:            {}", roll4.get_result());
        println!();
    }
}
