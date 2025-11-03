use rand::Rng;
use rusqlite::Connection;
use std::thread;
use std::time::Duration;
use crate::interfaces::user::User;
use crate::interfaces::menus::menu_generator;
use crate::logger::logger;
use crate::db::dbqueries;

const STARTING_MONEY: u32 = 100;
const MIN_BET: u32 = 5;
const MAX_BET: u32 = 50;

//Shows state of wheel and uses as a multiplier 
struct Segment {
    display: &'static str,
    multiplier: f32,
}

//each value of a segment on the wheel
const WHEEL: [Segment; 8] = [
    Segment { display: "2x", multiplier: 2.0 },
    Segment { display: "BANKRUPT", multiplier: 0.0 },
    Segment { display: "1.5x", multiplier: 1.5 },
    Segment { display: "0.5x (Lose Half)", multiplier: 0.5 },
    Segment { display: "3x", multiplier: 3.0 },
    Segment { display: "BANKRUPT", multiplier: 0.0 },
    Segment { display: "1x (Bet Back)", multiplier: 1.0 },
    Segment { display: "JACKPOT 10x", multiplier: 10.0 },
];

//used some asii art to create wheel
const ANIMATION_FRAMES: [&str; 4] = [
    r"
          . - ~ ~ - .
      /               \
     |                 |
    <         |         >
     |                 |
      \               /
          ' - ~ ~ - '
    ",
    r"
          . - ~ ~ - .
      /               \
     |        /        |
    <                 >
     |        \        |
      \               /
          ' - ~ ~ - '
    ",
    r"
          . - ~ ~ - .
      /               \
     |                 |
    <        - -        >
     |                 |
      \               /
          ' - ~ ~ - '
    ",
    r"
          . - ~ ~ - .
      /               \
     |        \        |
    <                 >
     |        /        |
      \               /
          ' - ~ ~ - '
    ",
];

//game play public fun
pub fn gameplay_wheel(conn: &Connection, user: &User, bet: f64) -> bool{
    let mut rng = rand::rng();
    // let mut player_money = STARTING_MONEY;

    println!("--- ♛ Welcome to the Wheel of Fortune! ♛ ---");

    loop {
        println!("\n------------------------------------");
        println!("You bet ${}. Spinning the wheel...", bet);

        //animation once bet is entered 
        run_spin_animation(&mut rng);

        //result of play
        let result_segment = &WHEEL[rng.random_range(0..WHEEL.len())];
        // calculate winnings if hit multiplier run math
        let winnings = bet * result_segment.multiplier as f64;

        //let user know of win or lose and update wallet
        clearscreen::clear().expect("Failed to clear screen");
        println!("The wheel slows down... and lands on:");
        println!("\n      *** {} ***", result_segment.display);

        if winnings == 0.0 {
            println!("\nOh no! You lost your bet.");
            println!("Current balance is {}", dbqueries::transaction(conn, user, -(bet)));
            let _ = dbqueries::add_loss(conn, "wheel of fortune");
            let _ = dbqueries::add_user_loss(conn, user, "wheel of fortune");
        } else {
            println!("\nCongratulations! You won ${}", winnings);
            println!("Current balance is {}", dbqueries::transaction(conn, user, winnings));
            let _ = dbqueries::add_win(conn, "wheel of fortune");
            let _ = dbqueries::add_user_win(conn, user, "wheel of fortune", winnings);
        }

        // Show options to user
        let menu_options = vec!["Spin Again", "Change Bet", "Exit"];
        let user_input = menu_generator("═══ 🎰 Play Again? 🎰 ═══", &menu_options);

        match user_input.trim() {
            "Spin Again" => {
                logger::info(&format!("User ID: {} continuing with same bet", user.id));
                continue;
            }
            "Change Bet" => {
                logger::info(&format!("User ID: {} changing bet", user.id));
                return true;
            }
            "Exit" => {
                logger::info(&format!("User ID: {} exiting slots game", user.id));
                return false;
            }
            _ => {
                logger::info(&format!("User ID: {} made invalid selection, continuing game", user.id));
                println!("Playing again..."); 
                continue;
            }
        }

    }

    // println!("You leave the casino with ${}. Goodbye!", player_money);
}

//promt uer for bet 
// fn get_player_bet(current_money: u32) -> u32 {
//     //see if uer chooses max bet
//     let current_max_bet = current_money.min(MAX_BET);

//     loop {
//         println!(
//             "Enter your bet (min: ${}, max: ${})",
//             MIN_BET, current_max_bet
//         );
//         println!("(You have ${}) or type 'q' to quit:", current_money);

        
//         let mut input = String::new();
//         io::stdin()
//             .read_line(&mut input)
//             .expect("Failed to read line");

//         // Check for quit
//         if input.trim().eq_ignore_ascii_case("q") {
//             return 0; //to quit or exit
//         }

//         //check user input make sure its correct type
//         match input.trim().parse::<u32>() {
//             Ok(bet_amount) => {
//                 // Check bet constraints
//                 if bet_amount < MIN_BET {
//                     println!("Bet is too small! Minimum bet is ${}.", MIN_BET);
//                 } else if bet_amount > current_max_bet {
//                     println!(
//                         "Bet is too large! Your max bet is ${}.",
//                         current_max_bet
//                     );
//                 } else {
//                     // Valid bet
//                     return bet_amount;
//                 }
//             }
//             Err(_) => {
//                 println!("That's not a valid number. Please try again.");
//             }
//         }
//     }
// }

// Runs a spinning animation
fn run_spin_animation(rng: &mut impl Rng) {
    let total_frames = 25; // Total number of "ticks"
    let mut delay = Duration::from_millis(50); // Starting delay

    for i in 0..total_frames {
        clearscreen::clear().expect("Failed to clear screen");

        // ASCII art frame
        let frame_art = ANIMATION_FRAMES[i % ANIMATION_FRAMES.len()];

        //the wheel segments flying past

        let random_segment = &WHEEL[rng.random_range(0..WHEEL.len())];
        println!("Spinning the Wheel!");
        println!("{}", frame_art);
        println!("\n  >> {} <<", random_segment.display);

        thread::sleep(delay);

        // Slow down the animation towards the end may adjust 
        if i > total_frames - 7 {
            delay += Duration::from_millis(30);
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    
    #[test]
    fn test_wheel_constants() {
        // Test that the wheel has segments
        assert_eq!(WHEEL.len(), 8);

        // Test that there are two "BANKRUPT" segments
        let bankrupt_count = WHEEL.iter().filter(|s| s.display == "BANKRUPT").count();
        assert_eq!(bankrupt_count, 2);
        
        // Test that both BANKRUPT segments have a 0.0 multiplier
        assert!(WHEEL.iter()
                     .filter(|s| s.display == "BANKRUPT")
                     .all(|s| s.multiplier == 0.0));

        // Test that all multipliers are non-negative
        assert!(WHEEL.iter().all(|s| s.multiplier >= 0.0));

        // Test that the JACKPOT exists and is correct
        let jackpot_exists = WHEEL.iter().any(|s| s.display == "JACKPOT 10x" && s.multiplier == 10.0);
        assert!(jackpot_exists, "The 10x JACKPOT segment is missing or incorrect");
    }

    #[test]
    fn test_animation_frames_constant() {
        // Ensure there are 4 animation frames
        assert_eq!(ANIMATION_FRAMES.len(), 4);
        // Ensure no frame is an empty string
        assert!(ANIMATION_FRAMES.iter().all(|&frame| !frame.is_empty()));
    }

    fn calculate_spin_result(bet: f64, wheel_index: usize) -> (f64, &'static Segment) {
    
        let clamped_index = wheel_index % WHEEL.len();
        let result_segment = &WHEEL[clamped_index];
        
        // Use (bet * multiplier) as f64 for precision
        let winnings = bet * (result_segment.multiplier as f64);
        (winnings, result_segment)
    }

    #[test]
    fn test_calculate_spin_result_jackpot() {
        let bet = 10.0;
        // Find the index for the JACKPOT
        let jackpot_index = WHEEL.iter().position(|s| s.multiplier == 10.0).expect("JACKPOT segment not found");

        let (winnings, segment) = calculate_spin_result(bet, jackpot_index);

        assert_eq!(segment.display, "JACKPOT 10x");
        //note to self assert_eq! for floating-point comparisons
        assert_eq!(winnings, 100.0);
    }

    #[test]
    fn test_calculate_spin_result_bankrupt() {
        let bet = 50.0;
        // for BANKRUPT
        let bankrupt_index = WHEEL.iter().position(|s| s.multiplier == 0.0).expect("BANKRUPT segment not found");

        let (winnings, segment) = calculate_spin_result(bet, bankrupt_index);

        assert_eq!(segment.display, "BANKRUPT");
        assert_eq!(winnings, 0.0);
    }

    #[test]
    fn test_calculate_spin_result_lose_half() {
        let bet = 30.0;

        let lose_half_index = WHEEL.iter().position(|s| s.multiplier == 0.5).expect("0.5x segment not found");

        let (winnings, segment) = calculate_spin_result(bet, lose_half_index);

        assert_eq!(segment.display, "0.5x (Lose Half)");
        assert_eq!(winnings, 15.0);
    }

    #[test]
    fn test_calculate_spin_result_bet_back() {
        let bet = 25.0;

        let bet_back_index = WHEEL.iter().position(|s| s.multiplier == 1.0).expect("1x segment not found");

        let (winnings, segment) = calculate_spin_result(bet, bet_back_index);

        assert_eq!(segment.display, "1x (Bet Back)");
        assert_eq!(winnings, 25.0);
    }

    //test for win lose

    #[test]
    fn test_calculate_spin_result_2x() {
        let bet = 20.0;
        let index = WHEEL.iter().position(|s| s.display == "2x").expect("2x segment not found");
        let (winnings, segment) = calculate_spin_result(bet, index);

        assert_eq!(segment.display, "2x");
        assert_eq!(winnings, 40.0); 
    }

    #[test]
    fn test_calculate_spin_result_1_5x() {
        let bet = 20.0;
        let index = WHEEL.iter().position(|s| s.display == "1.5x").expect("1.5x segment not found");
        let (winnings, segment) = calculate_spin_result(bet, index);

        assert_eq!(segment.display, "1.5x");
        assert_eq!(winnings, 30.0);
    }

    #[test]
    fn test_calculate_spin_result_3x() {
        let bet = 20.0;
        let index = WHEEL.iter().position(|s| s.display == "3x").expect("3x segment not found");
        let (winnings, segment) = calculate_spin_result(bet, index);

        assert_eq!(segment.display, "3x");
        assert_eq!(winnings, 60.0);
    }

//check bounds
    #[test]
    fn test_calculate_spin_result_with_out_of_bounds_index() {
        let bet = 10.0;
        
        let invalid_index = 100; 
        
        let (winnings, segment) = calculate_spin_result(bet, invalid_index);

        assert_eq!(segment.display, "3x");
        assert_eq!(segment.multiplier, 3.0);
        assert_eq!(winnings, 30.0); 
    }

    
}