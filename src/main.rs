use axum::{
    routing::{get, post},
    http::StatusCode,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use postflop_solver::*;

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        .route("/preflop", get(preflop));
        // `POST /users` goes to `create_user`

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

fn utg_range() -> &'static str {
    "AA-22,AKo-AJo,AKs-ATs,KQo-KJo,KQs-KTs,QJs,JTs,T9s,98s,87s"
}

fn mp_range() -> &'static str {
    "AA-22,AKo-ATo,AKs-A2s,KQo-KTo,KQs-K7s,QJo-Q8o,QJs-QTs,JTs-J9s,T9s-T8s,98s-97s,87s-86s,76s"
}

fn co_range() -> &'static str {
    "AA-TT,AKo-A8o,AKs-A5s,KQo-K8o,KQs-K6s,QJo-Q7o,QJs-Q6s,JTo-J7o,JTs-J6s,T9o-T7o,T9s-T6s,98s-96s,87s-85s,76s-75s,65s"
}

fn btn_range() -> &'static str {
    "AA-22,AKo-A2o,AKs-A2s,KQo-K2o,KQs-K2s,QJo-Q2o,QJs-Q2s,JTo-J2o,JTs-J2s,T9o-T2o,T9s-T2s,98s-94s,87s-84s,76s-74s,65s-64s,54s"
}

fn sb_range() -> &'static str {
    "AA-22,AKo-A2o,AKs-A2s,KQo-K2o,KQs-K2s,QJo-Q2o,QJs-Q2s,JTo-J2o,JTs-J2s,T9o-T2o,T9s-T2s,98s-93s,87s-83s,76s-73s,65s-63s,54s-53s"
}

fn bb_range() -> &'static str {
    "AA-22,AKo-A2o,AKs-A2s,KQo-K2o,KQs-K2s,QJo-Q2o,QJs-Q2s,JTo-J2o,JTs-J2s,T9o-T2o,T9s-T2s,98s-92s,87s-82s,76s-72s,65s-62s,54s-52s"
}

async fn preflop(Json(player):Json<Player>) -> (StatusCode, Json<PreflopResult>) {
    let range = utg_range().parse::<Range>().unwrap();
    let card1 = card_from_str(&player.cards[0..2]).unwrap();
    let card2 = card_from_str(&player.cards[2..4]).unwrap();

    if range.get_weight_by_cards(card1, card2) == 0.0 {
        let result = PreflopResult {
            strategy: "Fold".parse().unwrap(),
        };
        (StatusCode::OK, Json(result)); }
    let result = PreflopResult {
        strategy: "Call".parse().unwrap(),
    };
    (StatusCode::OK, Json(result))
}

// basic handler that responds with a static string
async fn root() -> &'static str {
    let oop_range = "66+,A8s+,A5s-A4s,AJo+,K9s+,KQo,QTs+,JTs,96s+,85s+,75s+,65s,54s";
    let ip_range = "QQ-22,AQs-A2s,ATo+,K5s+,KJo+,Q8s+,J8s+,T7s+,96s+,86s+,75s+,64s+,53s+";

    let card_config = CardConfig {
        range: [oop_range.parse().unwrap(), ip_range.parse().unwrap()],
        flop: flop_from_str("Td9d6h").unwrap(),
        turn: card_from_str("Qc").unwrap(),
        river: NOT_DEALT,
    };

    let bet_sizes = BetSizeOptions::try_from(("60%, e, a", "2.5x")).unwrap();

    let tree_config = TreeConfig {
        initial_state: BoardState::Turn,
        starting_pot: 20,
        effective_stack: 100,
        rake_rate: 0.0,
        rake_cap: 0.0,
        flop_bet_sizes: [bet_sizes.clone(), bet_sizes.clone()],
        turn_bet_sizes: [bet_sizes.clone(), bet_sizes.clone()],
        river_bet_sizes: [bet_sizes.clone(), bet_sizes],
        turn_donk_sizes: None,
        river_donk_sizes: Some(DonkSizeOptions::try_from("50%").unwrap()),
        add_allin_threshold: 1.5,
        force_allin_threshold: 0.15,
        merging_threshold: 0.1,
    };

    let action_tree = ActionTree::new(tree_config).unwrap();
    let mut game = PostFlopGame::with_config(card_config, action_tree).unwrap();
    game.allocate_memory(false);

    let max_num_iterations = 1000;
    let target_exploitability = game.tree_config().starting_pot as f32 * 0.005;
    solve(&mut game, max_num_iterations, target_exploitability, true);
    game.cache_normalized_weights();

    let actions = game.available_actions();
    let ev = game.expected_values_detail(0);
    let oop_cards = game.private_cards(0);
    let oop_cards_str = holes_to_strings(oop_cards).unwrap();

    println!("{:?}", oop_cards_str);
    let ksjs = holes_to_strings(oop_cards)
        .unwrap()
        .iter()
        .position(|s| s == "KsJs")
        .unwrap();
    let strategy = game.strategy();

    println!("{:?}", actions);
    println!(
        "{:?} {:?} {:?} {:?}", strategy[ksjs], strategy[ksjs + 167], strategy[ksjs + 167 * 2], strategy[ksjs + 167 * 3]
    );

    println!(
        "{:?} {:?} {:?} {:?}", actions.len(), ev.len(), oop_cards.len(), oop_cards_str.get(20)
    );
    "Hello, World!"
}

async fn create_user(
    // this argument tells axum to parse the request body
    // as JSON into a `CreateUser` type
    Json(payload): Json<CreateUser>,
) -> (StatusCode, Json<User>) {
    // insert your application logic here
    let user = User {
        id: 1337,
        username: payload.username,
    };

    // this will be converted into a JSON response
    // with a status code of `201 Created`
    (StatusCode::CREATED, Json(user))
}

// the input to our `create_user` handler
#[derive(Deserialize)]
struct CreateUser {
    username: String,
}

#[derive(Deserialize, Debug)]
struct Player {
    cards: String,
    pos: String
}

// the output to our `create_user` handler
#[derive(Serialize)]
struct User {
    id: u64,
    username: String,
}

#[derive(Serialize)]
struct PreflopResult {
    strategy: String,
}