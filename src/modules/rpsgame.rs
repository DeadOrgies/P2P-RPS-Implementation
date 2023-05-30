pub use crate::modules::peer_to_peer;
pub use crate::modules::PeerCounter::PeerCounter;
use egui::{containers::*, *};
use eframe::{egui::*, epi};
use libp2p::Swarm;
use rand::Rng;
use std::{thread, time, io::{self, Write}};

//RPS moves
enum Choice {
    Rock,
    Paper,
    Scissors,
}

impl Choice {
    fn from_index(index: usize) -> Option<Self> {
        match index {
            0 => Some(Self::Rock),
            1 => Some(Self::Paper),
            2 => Some(Self::Scissors),
            _ => None,
        }
    }

    fn to_string(&self) -> &str {
        match self {
            Self::Rock => "Rock",
            Self::Paper => "Paper",
            Self::Scissors => "Scissors",
        }
    }
}


//struct for RPS Game that can be implemented for p2p clients as well
struct RPSGame {
    user_choice: Option<Choice>,
    opponent_choice: Option<Choice>,
    cpu_choice: Option<Choice>,
    result: Option<&'static str>,
    play_vs_cpu: bool,
    show_main_menu: bool,
    play_p2p: bool,
    match_p2p: bool,
    is_connected: bool,
}

impl Default for RPSGame {
    fn default() -> Self {
        Self {
            user_choice: None,
            opponent_choice: None,
            cpu_choice: None,
            result: None,
            play_vs_cpu: false,
            show_main_menu: true,
            play_p2p: false,
            match_p2p: false,
            is_connected: false,
        }
    }
}

impl RPSGame {
    //function that executes user versus cpu
    fn play_cpu(&mut self) {
        let cpu_choice = rand::thread_rng().gen_range(0..=2);
        self.cpu_choice = Choice::from_index(cpu_choice);

        if let (Some(user_choice), Some(cpu_choice)) = (self.user_choice.as_ref(), self.cpu_choice.as_ref()) {
            self.result = match (user_choice, cpu_choice) {
                (Choice::Rock, Choice::Scissors)
                | (Choice::Paper, Choice::Rock)
                | (Choice::Scissors, Choice::Paper) => Some("You Win!"),
                (Choice::Rock, Choice::Paper)
                | (Choice::Paper, Choice::Scissors)
                | (Choice::Scissors, Choice::Rock) => Some("You Lose!"),
                _ => Some("It's a Tie!"),
            };
        }
    }

    //function that executes peer versus peer
    fn play_p2p(&mut self) {
        if let Some(user_choice) = self.user_choice.as_ref() {
            if let Some(opponent_choice) = self.opponent_choice.as_ref() {
                self.result = match (user_choice, opponent_choice) {
                    (Choice::Rock, Choice::Scissors)
                    | (Choice::Paper, Choice::Rock)
                    | (Choice::Scissors, Choice::Paper) => Some("You Win!"),
                    (Choice::Rock, Choice::Paper)
                    | (Choice::Paper, Choice::Scissors)
                    | (Choice::Scissors, Choice::Rock) => Some("You Lose!"),
                    _ => Some("It's a Tie!"),
                };
            } else {
                println!("Opponent's choice is missing. Unable to determine the result.");
            }
        }
    }

    fn set_connected(&mut self) {
        self.is_connected = true;
    }

    //reset the application state
    fn reset(&mut self) {
        self.user_choice = None;
        self.opponent_choice = None;
        self.cpu_choice = None;
        self.result = None;
        self.play_vs_cpu = false;
        self.show_main_menu = true;
        self.play_p2p = false;
        self.match_p2p = false;
    }
}

fn receive_opponent_choice() -> Choice {
    println!("Enter your opponent's move: rock, paper, or scissors");
    let mut input = String::new();
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut input).unwrap();
    let opponent_choice = match input.trim().to_lowercase().as_str() {
        "rock" => Choice::Rock,
        "paper" => Choice::Paper,
        "scissors" => Choice::Scissors,
        _ => {
            println!("Invalid choice. Defaulting to Rock.");
            Choice::Rock
        }
    };
    opponent_choice
}
pub struct MyApp {
    rps_game: RPSGame,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            rps_game: RPSGame::default(),
        }
    }
}


impl epi::App for MyApp {
    fn name(&self) -> &str {
        "Rock Paper Scissors"
    }

    fn update(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame<'_>) {
        egui::CentralPanel::default().show(ctx, |ui: &mut Ui| {
            if self.rps_game.show_main_menu {
                ui.heading("Main Menu");
                if ui.button("Play vs CPU").clicked() {
                    self.rps_game.play_vs_cpu = true;
                    self.rps_game.play_p2p = false;
                    self.rps_game.show_main_menu = false;
                }
                if ui.button("Play P2P").clicked() {
                    self.rps_game.play_p2p = true;
                    self.rps_game.play_vs_cpu = false;
                    self.rps_game.show_main_menu = false;
                }
            } else if self.rps_game.play_p2p && !self.rps_game.match_p2p 
            {
                ui.heading("P2P Menu");
                if ui.button("Match with Other Players").clicked() {
                    // Match with other players logic goes here
                    self.rps_game.match_p2p = true;
                    thread::spawn(|| {
                    
                    peer_to_peer::p2pclient();
                
                   
                });
                }
                

                if ui.button("Go Back to Main Menu").clicked() {
                    self.rps_game.reset();
                    thread::sleep(time::Duration::from_millis(600));
                }
            } else if self.rps_game.play_p2p && self.rps_game.match_p2p 
            {
                if !self.rps_game.is_connected
                {
                    ui.heading("Matching...");

                }
                else if self.rps_game.is_connected
                {
                    ui.heading("Connected!");

                }
                


                if ui.button("Cancel").clicked() {
                    self.rps_game.reset();
                    thread::sleep(time::Duration::from_millis(600));
                }

            }
            else 
            {
                // Game logic
                if let Some(result) = self.rps_game.result {
                    ui.label(result);
                    if ui.button("Play Again").clicked() {
                        self.rps_game.user_choice = None;
                        self.rps_game.cpu_choice = None;
                        self.rps_game.result = None;
                    }
                    if ui.button("Quit to Main Menu").clicked() {
                        self.rps_game.reset();
                    }
                } else {
                    let choices = ["Rock", "Paper", "Scissors"];
                    ui.horizontal(|ui| {
                        for (index, choice) in choices.iter().enumerate() {
                            if ui.button(choice).clicked() {
                                self.rps_game.user_choice = Choice::from_index(index);
                                self.rps_game.play_cpu();
                            }
                        }
                    });
                    if ui.button("Quit to Main Menu").clicked() {
                        self.rps_game.reset();
                    }
                }

                // Show CPU choice if user has made a choice
                if let Some(user_choice) = self.rps_game.user_choice.as_ref() {
                    ui.label(format!("Your Choice: {}", user_choice.to_string()));
                    if let Some(cpu_choice) = self.rps_game.cpu_choice.as_ref() {
                        ui.label(format!("CPU Choice: {}", cpu_choice.to_string()));
                    }
                }
            }
        });

        // Request a UI repaint
        frame.repaint_signal();
    }
}


