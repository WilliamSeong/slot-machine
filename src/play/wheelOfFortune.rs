use rand::Rng;
use rusqlite::Connection;
use std::io;
use std::thread;
use std::time::Duration;
use crate::interfaces::user::User;
use crate::interfaces::menus::menu_generator;
use crate::logger::logger;

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
        // println!("Your Wallet: ${}", player_money);

        // Check if player has funds and can still play
        // if player_money < MIN_BET {
        //     println!("You don't have enough money for the minimum bet (${}).", MIN_BET);
        //     println!("Thanks for playing!");
        //     break;
        // }

        // get bet from user
        // let bet = get_player_bet(bet);

        // check if user has money 
        // if bet == 0 {
        //     break;
        // }

        //Sub bet from wallet
        // player_money -= bet;
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
        } else {
            println!("\nCongratulations! You won ${}", winnings);
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
fn get_player_bet(current_money: u32) -> u32 {
    //see if uer chooses max bet
    let current_max_bet = current_money.min(MAX_BET);

    loop {
        println!(
            "Enter your bet (min: ${}, max: ${})",
            MIN_BET, current_max_bet
        );
        println!("(You have ${}) or type 'q' to quit:", current_money);

        
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");

        // Check for quit
        if input.trim().eq_ignore_ascii_case("q") {
            return 0; //to quit or exit
        }

        //check user input make sure its correct type
        match input.trim().parse::<u32>() {
            Ok(bet_amount) => {
                // Check bet constraints
                if bet_amount < MIN_BET {
                    println!("Bet is too small! Minimum bet is ${}.", MIN_BET);
                } else if bet_amount > current_max_bet {
                    println!(
                        "Bet is too large! Your max bet is ${}.",
                        current_max_bet
                    );
                } else {
                    // Valid bet
                    return bet_amount;
                }
            }
            Err(_) => {
                println!("That's not a valid number. Please try again.");
            }
        }
    }
}

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
//ag
