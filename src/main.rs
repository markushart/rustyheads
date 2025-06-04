use game::rules;
use rand_chacha::rand_core::SeedableRng;

pub mod game {
    use rand::seq::SliceRandom;
    use rules::{DeckType, MatchType};
    use rusqlite::{
        types::{FromSql, FromSqlError, ToSql, ToSqlOutput, ValueRef},
        Result,
    };
    use std::{collections::HashMap, fmt, hash::Hash};
    // This module contains the game logic and rules
    // It includes the definitions of cards, players, rounds, and matches

    // enum for suits of cards
    #[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
    pub enum Suit {
        Hearts,
        Diamonds,
        Clubs,
        Spades,
    }

    impl FromSql for Suit {
        fn column_result(value: ValueRef<'_>) -> Result<Self, FromSqlError> {
            match value.as_i64()? {
                1 => Ok(Suit::Diamonds),
                2 => Ok(Suit::Hearts),
                3 => Ok(Suit::Spades),
                4 => Ok(Suit::Clubs),
                other => Err(FromSqlError::Other(
                    format!("Invalid suit: {}", other).into(),
                )),
            }
        }
    }

    impl ToSql for Suit {
        fn to_sql(&self) -> Result<ToSqlOutput<'_>> {
            match self {
                Suit::Diamonds => Ok(1.into()),
                Suit::Hearts => Ok(2.into()),
                Suit::Spades => Ok(3.into()),
                Suit::Clubs => Ok(4.into()),
            }
        }
    }

    impl fmt::Display for Suit {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let name = match self {
                Suit::Hearts => "Hearts",
                Suit::Diamonds => "Diamonds",
                Suit::Clubs => "Clubs",
                Suit::Spades => "Spades",
            };
            write!(f, "{}", name)
        }
    }

    impl fmt::Debug for Suit {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let name = match self {
                Suit::Hearts => "H",
                Suit::Diamonds => "D",
                Suit::Clubs => "C",
                Suit::Spades => "S",
            };
            write!(f, "{}", name)
        }
    }

    // enum for faces of cards
    #[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
    pub enum Face {
        Two,
        Three,
        Four,
        Five,
        Six,
        Seven,
        Eight,
        Nine,
        Ten,
        Jack,
        Queen,
        King,
        Ace,
    }

    impl ToSql for Face {
        fn to_sql(&self) -> Result<ToSqlOutput<'_>> {
            match self {
                Face::Two => Ok(1.into()),
                Face::Three => Ok(2.into()),
                Face::Four => Ok(3.into()),
                Face::Five => Ok(4.into()),
                Face::Six => Ok(5.into()),
                Face::Seven => Ok(6.into()),
                Face::Eight => Ok(7.into()),
                Face::Nine => Ok(8.into()),
                Face::Ten => Ok(9.into()),
                Face::Jack => Ok(10.into()),
                Face::Queen => Ok(11.into()),
                Face::King => Ok(12.into()),
                Face::Ace => Ok(13.into()),
            }
        }
    }

    impl FromSql for Face {
        fn column_result(value: ValueRef<'_>) -> Result<Self, FromSqlError> {
            match value.as_i64()? {
                1 => Ok(Face::Two),
                2 => Ok(Face::Three),
                3 => Ok(Face::Four),
                4 => Ok(Face::Five),
                5 => Ok(Face::Six),
                6 => Ok(Face::Seven),
                7 => Ok(Face::Eight),
                8 => Ok(Face::Nine),
                9 => Ok(Face::Ten),
                10 => Ok(Face::Jack),
                11 => Ok(Face::Queen),
                12 => Ok(Face::King),
                13 => Ok(Face::Ace),
                other => Err(FromSqlError::Other(
                    format!("Invalid face: {}", other).into(),
                )),
            }
        }
    }

    impl fmt::Display for Face {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let name = match self {
                Face::Two => "Two",
                Face::Three => "Three",
                Face::Four => "Four",
                Face::Five => "Five",
                Face::Six => "Six",
                Face::Seven => "Seven",
                Face::Eight => "Eight",
                Face::Nine => "Nine",
                Face::Ten => "Ten",
                Face::Jack => "Jack",
                Face::Queen => "Queen",
                Face::King => "King",
                Face::Ace => "Ace",
            };
            write!(f, "{}", name)
        }
    }

    impl fmt::Debug for Face {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let name = match self {
                Face::Two => "2",
                Face::Three => "3",
                Face::Four => "4",
                Face::Five => "5",
                Face::Six => "6",
                Face::Seven => "7",
                Face::Eight => "8",
                Face::Nine => "9",
                Face::Ten => "T",
                Face::Jack => "J",
                Face::Queen => "Q",
                Face::King => "K",
                Face::Ace => "A",
            };
            write!(f, "{}", name)
        }
    }

    enum WinningCard {
        FirstWins,
        SelfWins,
        OtherWins,
    }

    type RankType = u8;
    type EyeType = u8;

    // struct for a card
    #[derive(Clone, Copy)]
    pub struct Card {
        suit: Suit,
        face: Face,
        eyes: EyeType,
        trump: bool,
        rank: RankType,
    }

    impl fmt::Debug for Card {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{:?}{:?}({})", self.suit, self.face, self.rank)
        }
    }

    impl fmt::Display for Card {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{} of {} ({})", self.face, self.suit, self.rank)
        }
    }

    impl Card {
        fn new(suit: Suit, face: Face, eyes: u8, trump: bool, rank: RankType) -> Card {
            Card {
                suit,
                face,
                eyes,
                trump,
                rank,
            }
        }

        // function to check if the other card serves this
        fn serves_this(&self, other: &Card) -> bool {
            // trump serves trump, if no trump is played,
            // the equal suits serve each other
            self.trump && other.trump || (!self.trump && !other.trump && self.suit == other.suit)
        }

        // function to check if the other card beats this card if this card is played first
        fn winning_card(&self, first_card: &Card, other: &Card) -> Option<WinningCard> {
            // if self and other serve, the one with the higher rank wins
            let i_srv = first_card.serves_this(self);
            let o_srv = first_card.serves_this(other);
            // self less than other
            let slto = self < other;

            // i_srv | o_srv | s < o | i_trump | o_trump | result
            //-------------------------------------------------------------
            // 0     | 0     | ...   | ...     | ...     | FirstWins
            //-------------------------------------------------------------
            // 0     | 1     | ...   | 1       | 0       | SelfWins
            // 1     | 0     | ...   | ...     | 0       | SelfWins
            // 1     | 1     | 0     | 0       | 0       | SelfWins
            // 1     | 1     | 0     | 1       | 1       | SelfWins
            //-------------------------------------------------------------
            // 0     | 1     | ...   | 0       | ...     | OtherWins
            // 1     | 0     | ...   | 0       | 1       | OtherWins
            // 1     | 1     | 1     | 0       | 0       | OtherWins
            // 1     | 1     | 1     | 1       | 1       | OtherWins
            //-------------------------------------------------------------
            // 0     | 1     | ...   | 1       | 1       | => NotPossible
            // 1     | 0     | ...   | 1       | 1       | => NotPossible
            // 1     | 1     | ...   | 0       | 1       | => NotPossible
            // 1     | 1     | ...   | 1       | 0       | => NotPossible

            // if statement corresponding to the table above
            if !i_srv && !o_srv {
                Some(WinningCard::FirstWins)
            } else if (!i_srv && o_srv && self.trump && !other.trump)
                || (i_srv && !o_srv && !other.trump)
                || (i_srv && o_srv && !slto && !self.trump && !other.trump)
                || (i_srv && o_srv && !slto && self.trump && other.trump)
            {
                Some(WinningCard::SelfWins)
            } else if (!i_srv && o_srv && !self.trump)
                || (i_srv && !o_srv && !self.trump && other.trump)
                || (i_srv && o_srv && slto && !self.trump && !other.trump)
                || (i_srv && o_srv && slto && self.trump && other.trump)
            {
                Some(WinningCard::OtherWins)
            } else {
                None
            }
        }
    }

    // implement comparison traits for Card
    impl PartialEq for Card {
        fn eq(&self, other: &Self) -> bool {
            self.rank == other.rank
        }
    }

    impl Eq for Card {}

    impl PartialOrd for Card {
        fn partial_cmp(&self, other: &Card) -> Option<std::cmp::Ordering> {
            if self.rank < other.rank {
                Some(std::cmp::Ordering::Less)
            } else if self.rank > other.rank {
                Some(std::cmp::Ordering::Greater)
            } else {
                Some(std::cmp::Ordering::Equal)
            }
        }
    }

    impl Ord for Card {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            self.partial_cmp(other).unwrap_or(std::cmp::Ordering::Equal)
        }
    }

    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
    pub enum Team {
        Contra,
        Re,
    }

    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub enum PlayerType {
        Human,
        Computer,
    }

    // enum with bit flags for heart, diamonds, clubs, spades and trump
    pub enum ServeFlag {
        Hearts = 1 << 0,
        Diamonds = 1 << 1,
        Clubs = 1 << 2,
        Spades = 1 << 3,
        Trump = 1 << 4,
    }

    impl ServeFlag {
        fn flag_for_card(c: &Card) -> ServeFlag {
            match c.trump {
                true => ServeFlag::Trump,
                false => match c.suit {
                    Suit::Hearts => ServeFlag::Hearts,
                    Suit::Diamonds => ServeFlag::Diamonds,
                    Suit::Clubs => ServeFlag::Clubs,
                    Suit::Spades => ServeFlag::Spades,
                },
            }
        }
    }

    type ServeFlagType = u8;

    // struct for a player
    #[derive(Clone)]
    pub struct Player {
        name: String,
        hand: Vec<Card>,
        won_cards: Vec<Card>,
        team: Team,
        dealer: bool,
        beginner: bool,
        // whenever this player does not serve,
        // the corresponding bit is set to 0
        serve_flags: ServeFlagType,
    }

    #[derive(Clone)]
    pub struct HumanPlayer {
        data: Player,
    }

    #[derive(Clone)]
    pub struct ComputerPlayer {
        data: Player,
    }

    #[derive(Clone)]
    pub struct SimulatedPlayer {
        data: Player,
    }

    type RngType = rand_chacha::ChaCha20Rng;

    type DynPlayer = Box<dyn PlayerBehav>;
    type DynPlayers = Vec<DynPlayer>;

    pub trait PlayerBehav {
        // get data and mutable data
        fn data(&self) -> &Player;
        fn data_mut(&mut self) -> &mut Player;

        // function to choose a card from the player's hand
        fn choose_card(
            &self,
            possible_cards: &Vec<Card>,
            current_match: &Match,
            current_round: &Round,
            players: &DynPlayers,
            rng: &mut RngType,
        ) -> Option<Card>;

        //
        fn remove_card_from_hand(&mut self, card: &Card) -> Option<Card> {
            match self.data().hand.iter().position(|c| c.rank == card.rank) {
                Some(i) => Some(self.data_mut().hand.swap_remove(i)),
                None => None,
            }
        }

        // function to make a call
        fn make_call(&self) -> Option<MatchType>;

        fn set_my_team(&mut self, match_type: MatchType) {
            self.data_mut().team = get_team_for_player(self.data(), match_type);
        }

        fn get_num_cards(&self) -> usize {
            self.data().hand.len()
        }

        fn update_serve_flags(&mut self, current_round: &Round, card: &Card) -> ServeFlagType {
            if current_round.played_cards.len() > 0 {
                let first_card = current_round.played_cards[0];
                if !card.serves_this(&first_card) {
                    // if the card didnt serve first card, disable the flag
                    let mask = !(ServeFlag::flag_for_card(&first_card) as ServeFlagType);
                    self.data_mut().serve_flags &= mask;
                }
            }
            self.data().serve_flags
        }

        // function to update the player's hand
        fn update_hand_values(&mut self, deck: &Vec<Card>) {
            for card in self.data_mut().hand.iter_mut() {
                // find card in deck
                let i = deck
                    .iter()
                    .position(|c| c.face == card.face && c.suit == card.suit)
                    .expect("Card not found in deck");
                // set rank, trump and eyes of the card
                card.rank = deck[i].rank;
                card.trump = deck[i].trump;
                card.eyes = deck[i].eyes;
            }
        }

        // if the player wins a round, he collects the cards
        fn collect_won_cards(&mut self, cards: &Vec<Card>) {
            self.data_mut().won_cards.extend(cards);
        }

        // sum the eyes of the won cards to determine the eye_score
        fn get_eye_score(&self) -> u32 {
            self.data().won_cards.iter().map(|c| c.eyes as u32).sum()
        }
    }

    // sets all bits of serve flags to 1 (initial state)
    const SERVE_FLAG_ALL: ServeFlagType = ServeFlag::Diamonds as ServeFlagType
        | ServeFlag::Hearts as ServeFlagType
        | ServeFlag::Clubs as ServeFlagType
        | ServeFlag::Spades as ServeFlagType
        | ServeFlag::Trump as ServeFlagType;

    impl Player {
        fn new(name: String, dealer: bool) -> Player {
            Player {
                name,
                hand: Vec::new(),
                won_cards: Vec::new(),
                team: Team::Contra,
                dealer,
                beginner: false,
                serve_flags: SERVE_FLAG_ALL,
            }
        }
    }

    impl ComputerPlayer {
        // create a new ComputerPlayer
        fn new(name: String, dealer: bool) -> ComputerPlayer {
            ComputerPlayer {
                data: Player::new(name, dealer),
            }
        }
    }

    impl PlayerBehav for ComputerPlayer {
        fn data(&self) -> &Player {
            &self.data
        }

        fn data_mut(&mut self) -> &mut Player {
            &mut self.data
        }

        // function to choose a card from the player's hand
        fn choose_card(
            &self,
            possible_cards: &Vec<Card>,
            current_match: &Match,
            current_round: &Round,
            players: &DynPlayers,
            rng: &mut RngType,
        ) -> Option<Card> {
            // for simplicity, we just return the first card in the hand
            // in a real game, this would be more complex

            if self.get_num_cards() == 0 {
                return None;
            } else if possible_cards.len() == 1 {
                // if there is only one card, play it
                println!("{}({:?}) plays card: {:?}", self.data().name, self.data().team, possible_cards[0],);
                Some(possible_cards[0])
            } else {
                // print!("{} hand: {:?}, ", self.data().name, possible_cards);

                let (best_move, _) = simulation::simulate(
                    current_match,
                    current_round,
                    possible_cards,
                    players,
                    8,
                    rng,
                );

                // println!(
                //     "{:?} best move: {}
                //     self.data().name,
                //     best_move,
                // );

                let card = possible_cards
                    .iter()
                    .find(|c| c.rank == best_move)
                    .unwrap()
                    .clone();

                println!("{}({:?}) plays card: {:?}", self.data().name, self.data().team, card,);

                Some(card)
            }
        }

        // function to make a call
        fn make_call(&self) -> Option<MatchType> {
            // for simplicity, we just return a random call
            // in a real game, this would be more complex
            Some(MatchType::Normal)
        }
    }

    impl HumanPlayer {
        // create a new ComputerPlayer
        fn new(name: String, dealer: bool) -> HumanPlayer {
            HumanPlayer {
                data: Player::new(name, dealer),
            }
        }
    }

    impl PlayerBehav for HumanPlayer {
        fn data(&self) -> &Player {
            &self.data
        }

        fn data_mut(&mut self) -> &mut Player {
            &mut self.data
        }

        // function to choose a card from the player's hand
        fn choose_card(
            &self,
            possible_cards: &Vec<Card>,
            _current_match: &Match,
            _current_round: &Round,
            _players: &DynPlayers,
            _rng: &mut RngType,
        ) -> Option<Card> {
            // for simplicity, we just return the first card in the hand
            // in a real game, this would be more complex
            if self.get_num_cards() == 0 {
                return None;
            } else {
                Some(possible_cards[0])
            }
        }

        // function to make a call
        fn make_call(&self) -> Option<MatchType> {
            // for simplicity, we just return a random call
            // in a real game, this would be more complex
            Some(MatchType::Normal)
        }
    }

    impl PlayerBehav for SimulatedPlayer {
        fn data(&self) -> &Player {
            &self.data
        }

        fn data_mut(&mut self) -> &mut Player {
            &mut self.data
        }

        // function to choose a card from the player's hand
        fn choose_card(
            &self,
            possible_cards: &Vec<Card>,
            _current_match: &Match,
            _current_round: &Round,
            _players: &DynPlayers,
            _rng: &mut RngType,
        ) -> Option<Card> {
            // for first try, we simualte games by playing random cards
            // the simulated player needs to return a random possible card
            // TODO: later on, integrate some heuristic for which cards to play
            if self.get_num_cards() == 0 {
                return None;
            } else {
                Some(possible_cards[0])
            }
        }

        // function to make a call
        fn make_call(&self) -> Option<MatchType> {
            // for simplicity, we just return a random call
            // in a real game, this would be more complex
            Some(MatchType::Normal)
        }
    }

    // struct for a game round
    // in a single rount each player plays one card from his hand
    // there are maximum 5 players
    #[derive(Clone)]
    pub struct Round {
        played_cards: Vec<Card>,
        current_player: usize,
        starting_player: usize,
        winner: usize,
    }

    type RoundBox = Box<Round>;
    type RoundBoxes = Vec<RoundBox>;

    impl Round {
        pub fn new() -> Round {
            Round {
                played_cards: Vec::new(),
                current_player: 0,
                starting_player: 0,
                winner: 0,
            }
        }

        fn init_round(&mut self, nplayers: usize, starting_player: usize) {
            self.played_cards.clear();
            // reserve ncards space
            if self.played_cards.capacity() < nplayers {
                self.played_cards
                    .reserve(nplayers - self.played_cards.capacity());
            }

            self.starting_player = starting_player;
            self.current_player = starting_player;
        }

        // function to play a round
        fn play_round(
            &mut self,
            current_match: &mut Match,
            players: &mut DynPlayers,
            last_rounds_winner: usize,
            rng: &mut RngType,
        ) -> Option<usize> {
            self.init_round(players.len(), last_rounds_winner);

            // each player plays one card
            for _i in 0..players.len() {
                let card = self.play_card(current_match, players, rng);

                self.current_player = (self.current_player + 1) % players.len();

                self.played_cards.push(card)
            }

            self.winner =
                Round::determine_winner(&self.played_cards, self.starting_player).unwrap();

            players[self.winner].collect_won_cards(&self.played_cards);
            self.played_cards.clear();

            Some(self.winner)
        }

        fn filter_possible_cards(played_cards: &Vec<Card>, hand: &Vec<Card>) -> Vec<Card> {
            // filter the cards that can be played in this round
            if played_cards.len() == 0 {
                // if i am the first to play a card, every card is allowed
                hand.clone()
            } else {
                let first_card = played_cards[0];

                let possible_cards: Vec<Card> = hand
                    .iter()
                    .filter_map(|c| match c.serves_this(&first_card) {
                        true => Some(c.clone()),
                        false => None,
                    })
                    .collect();

                if possible_cards.len() == 0 {
                    hand.clone()
                } else {
                    possible_cards
                }
            }
        }

        // function to play a card from the player's hand
        fn play_card(
            &self,
            current_match: &Match,
            players: &mut DynPlayers,
            rng: &mut RngType,
        ) -> Card {
            // this function is not in the players scope as he would have to pass a vector of
            // players to itself which is colliding with himself beeing passed as mut
            let possible_cards = Round::filter_possible_cards(
                &self.played_cards,
                &players[self.current_player].data().hand,
            );

            let card = players[self.current_player]
                .choose_card(&possible_cards, current_match, self, &*players, rng)
                .unwrap();

            // edit can_serve based on the played card
            players[self.current_player].update_serve_flags(self, &card);

            // remove the card from hand and return it
            players[self.current_player]
                .remove_card_from_hand(&card)
                .unwrap()
        }

        fn determine_winner(played_cards: &Vec<Card>, starting_player: usize) -> Option<usize> {
            if played_cards.len() == 0 {
                return None;
            } else {
                let first_card = played_cards[0];
                let mut winner = 0;

                for i in 1..played_cards.len() {
                    let card = played_cards[i];

                    // compare the current winning card to first card and the current card
                    match played_cards[winner].winning_card(&first_card, &card) {
                        Some(WinningCard::OtherWins) => winner = i,
                        Some(WinningCard::SelfWins) => {
                            // if the current card is the winning card, do nothing
                        }
                        Some(WinningCard::FirstWins) => {
                            // if the first card is the winning card, do nothing
                        }
                        None => {
                            panic!("Invalid card configuration")
                        }
                    }
                }

                // return the winner as the index of the winning player
                winner = (starting_player + winner) % played_cards.len();

                Some(winner)
            }
        }
    }

    // struct for a match
    // a match consists of multiple rounds until no player has cards left
    // at the beginning of the match, cards are dealt to players
    // then the players give calls, depending on the call, the trump cards are set
    pub struct Match {
        rounds: RoundBoxes,
        deck: Vec<Card>,
        match_type: MatchType, // type of match
        n_rounds: usize,       // number of rounds in the match
        winner: Team,
    }

    type MatchBox = Box<Match>;
    type MatchBoxes = Vec<MatchBox>;

    impl Match {
        pub fn new() -> Match {
            Match {
                rounds: Vec::new(),
                deck: Vec::new(),
                match_type: MatchType::Normal,
                n_rounds: 0,
                winner: Team::Contra,
            }
        }

        // function to get the current rounds
        fn current_round(&self) -> usize {
            self.rounds.len()
        }

        fn set_num_rounds(&mut self, n_rounds: usize) {
            self.n_rounds = n_rounds;
            // reserve space for rounds
            if self.rounds.capacity() < n_rounds {
                // reserve space for rounds
                self.rounds.reserve(n_rounds - self.rounds.capacity());
            }
        }

        fn set_dealer_and_beginner(&mut self, players: &mut DynPlayers) {
            assert!(players.len() > 0, "No players in the game");

            // find index of dealer and beginner
            let d_idx_old = match players.iter().position(|p| p.data().dealer) {
                Some(i) => i,
                None => players.len() - 1,
            };

            // new dealer sits one to the left of the old dealer
            let d_idx_new = (d_idx_old + 1) % players.len();
            // old beginner is the new dealer
            let b_idx_old = d_idx_new;
            // new beginner sits one to the left of the old beginner
            let b_idx_new = (b_idx_old + 1) % players.len();

            // set the new dealer and beginner
            players[d_idx_old].data_mut().dealer = false;
            players[d_idx_new].data_mut().dealer = true;
            players[b_idx_old].data_mut().beginner = false;
            players[b_idx_new].data_mut().beginner = true;
        }

        fn init_match(
            &mut self,
            players: &mut DynPlayers,
            deck_type: DeckType,
            deck_buff: &mut DeckBuff,
            rng: &mut RngType,
        ) {
            self.deck.clear();
            self.rounds.clear();

            self.deck = deck_buff.get_deck(deck_type).unwrap();

            assert_ne!(self.deck.len(), 0, "Deck is empty");
            assert_eq!(
                self.deck.len() % players.len(),
                0,
                "Number of cards is not divisible by number of players",
            );
            // initialize players
            for p in players.iter_mut() {
                p.data_mut().serve_flags = SERVE_FLAG_ALL;
                p.data_mut().won_cards.clear();
                p.data_mut().hand.clear();
                p.data_mut().team = Team::Contra;
            }

            // set number of rounds and buffer them
            self.set_num_rounds(self.deck.len() / players.len());

            // shuffle cards
            self.shuffle_cards(rng);

            // distribute cards to players
            self.distribute_cards(players);

            // set the dealer and beginner
            self.set_dealer_and_beginner(players);
        }

        pub fn play_match(
            &mut self,
            players: &mut DynPlayers,
            deck_type: DeckType,
            deck_buff: &mut DeckBuff,
            rng: &mut RngType,
        ) {
            // init game, shuffle and distribute cards
            self.init_match(players, deck_type, deck_buff, rng);

            // depending on what cards the players have, determine the match type

            // self.match_type = self.determine_match_type(&players.iter().map(|p| &**p).collect());
            self.match_type = self.determine_match_type(players);

            // set the deck of cards
            self.deck = deck_buff
                .get_card_values(&self.deck, self.match_type, deck_type)
                .unwrap();

            for player in players.iter_mut() {
                // update the values of players hands
                player.update_hand_values(&self.deck);
                // set the team of the players
                player.set_my_team(self.match_type);
            }

            // play rounds
            let mut last_round_winner = 0;

            while self.current_round() < self.n_rounds {
                // after playing the round, the winner is returned
                println!("Play round {}", self.current_round() + 1);

                let mut round = Round::new();
                round.play_round(self, players, last_round_winner, rng);

                last_round_winner = round.winner;

                self.rounds.push(RoundBox::new(round));

                println!("Round winner: {}", players[last_round_winner].data().name);
                println!();
            }

            self.winner = self.determine_winner(players).unwrap();
        }

        fn shuffle_cards(&mut self, rng: &mut RngType) {
            // shuffle the cards
            self.deck.shuffle(rng);
        }

        fn distribute_cards(&mut self, players: &mut DynPlayers) {
            // distribute cards to players
            let nplayers = players.len();
            let ncards = self.deck.len();

            assert_ne!(
                nplayers, 0,
                "Number of players is zero, cannot distribute cards",
            );
            assert_eq!(
                ncards % nplayers,
                0,
                "Number of cards is not divisible by number of players",
            );

            let mut i = 0;
            loop {
                // get the next card from the deck
                match self.deck.pop() {
                    None => break,
                    Some(card) => {
                        // add the card to the player's hand
                        players[i].data_mut().hand.push(card);
                        // increment player index
                        i = (i + 1) % nplayers;
                    }
                }
            }
        }

        // let the players make a call and decide the match type
        fn determine_match_type(&self, players: &DynPlayers) -> rules::MatchType {
            // get index of beginner
            let b_idx = players
                .iter()
                .position(|p| p.data().beginner)
                .expect("No beginner found");

            // iterable of indexes of players
            // into player of iterators in playing order
            // into iterator over their calls
            // into maximum call, that is the match_type
            (0..players.len())
                .map(|i| players[(b_idx + i) % players.len()].make_call().unwrap())
                .max()
                .unwrap()
        }

        fn determine_winner(&self, players: &DynPlayers) -> Option<Team> {
            if players.len() == 0 {
                return None;
            } else {
                // filter by team of player, sum their eye scores
                let re_score = Match::get_team_score(players, Team::Re).unwrap();
                let contra_score = Match::get_team_score(players, Team::Contra).unwrap();

                // Contra wins if teams are equal
                if re_score <= contra_score {
                    Some(Team::Contra)
                } else {
                    Some(Team::Re)
                }
            }
        }

        fn get_team_score(players: &DynPlayers, team: Team) -> Option<u32> {
            // get the score of the team
            if players.len() == 0 {
                None
            } else {
                // filter by team of player, sum their eye scores
                let score = match team {
                    Team::Contra => players
                        .iter()
                        .filter_map(|p| match &p.data().team {
                            Team::Contra => Some(p.get_eye_score()),
                            Team::Re => None,
                        })
                        .sum(),
                    Team::Re => players
                        .iter()
                        .filter_map(|p| match &p.data().team {
                            Team::Re => Some(p.get_eye_score()),
                            Team::Contra => None,
                        })
                        .sum(),
                };

                Some(score)
            }
        }
    }

    // struct representing the game
    // the overall game consists of multiple matches
    pub struct Game {
        players: DynPlayers,
        matches: MatchBoxes,
        n_matches: usize,
        deck_type: DeckType,
        deck_buff: Box<DeckBuff>,
    }

    #[derive(Clone, Copy, Hash, PartialEq, Eq)]
    pub struct CardValueKey {
        match_type: MatchType,
        suit: Suit,
        face: Face,
    }

    #[derive(Clone, Copy, Hash, PartialEq, Eq)]
    pub struct CardValue {
        eyes: EyeType,
        trump: bool,
        rank: RankType,
    }

    // buffers the deck of a specific match to reduce DB querries per Game
    pub struct DeckBuff {
        deck_map: HashMap<DeckType, Vec<(Suit, Face)>>,
        card_value_map: HashMap<CardValueKey, CardValue>,
    }

    impl DeckBuff {
        fn new() -> DeckBuff {
            DeckBuff {
                deck_map: HashMap::new(),
                card_value_map: HashMap::new(),
            }
        }

        fn get_deck(&mut self, deck_type: DeckType) -> Option<Vec<Card>> {
            match self.deck_map.get(&deck_type) {
                Some(deck) => Some(
                    deck.iter()
                        .map(|(s, f)| Card::new(*s, *f, 0, false, 0))
                        .collect(),
                ),
                None => {
                    let d = rules::get_deck_for_decktype(deck_type);
                    self.deck_map.insert(
                        deck_type,
                        d.clone()
                            .unwrap()
                            .iter()
                            .map(|c| (c.suit, c.face))
                            .collect(),
                    );
                    d
                }
            }
        }

        fn get_card_values(
            &mut self,
            deck: &Vec<Card>,
            match_type: MatchType,
            deck_type: DeckType,
        ) -> Option<Vec<Card>> {
            let mut d = deck.clone();
            // initialize the flag with true if deck contains cards
            let mut is_in_buffer = d.len() > 0;

            for c in d.iter_mut() {
                let cv_opt = self.card_value_map.get(&CardValueKey {
                    match_type,
                    suit: c.suit,
                    face: c.face,
                });

                // check if card was found in buffer
                match cv_opt {
                    Some(cv) => {
                        c.eyes = cv.eyes;
                        c.trump = cv.trump;
                        c.rank = cv.rank;
                    }
                    None => {
                        is_in_buffer = false;
                        break;
                    }
                };
            }

            if !is_in_buffer {
                // if not found, fetch the values from the db
                match rules::get_deck_for_matchtype(match_type, deck_type) {
                    Some(new_deck) => {
                        new_deck.iter().for_each(|c| {
                            self.card_value_map.insert(
                                CardValueKey {
                                    match_type,
                                    suit: c.suit,
                                    face: c.face,
                                },
                                CardValue {
                                    eyes: c.eyes,
                                    trump: c.trump,
                                    rank: c.rank,
                                },
                            );
                        });
                        Some(new_deck)
                    }
                    None => None,
                }
            } else {
                println!("Match type: {:?}", match_type);
                println!("Deck: {:?}", d);

                Some(d)
            }
        }
    }

    impl Game {
        // function to start a new game
        pub fn new(n_matches: usize, deck_type: DeckType) -> Game {
            let mut g = Game {
                players: Vec::new(),
                matches: Vec::new(),
                n_matches: 0,
                deck_type,
                deck_buff: Box::new(DeckBuff::new()),
            };

            g.set_num_matches(n_matches);

            g
        }

        // function to add a player to the game
        pub fn add_player(&mut self, name: String, player_type: PlayerType) {
            let dealer = self.players.len() == 0;

            let player: DynPlayer = match player_type {
                PlayerType::Computer => Box::new(ComputerPlayer::new(name, dealer)),
                PlayerType::Human => Box::new(HumanPlayer::new(name, dealer)),
            };

            self.players.push(player);
        }

        pub fn play_game(&mut self, n_matches: usize, rng: &mut RngType) {
            self.set_num_matches(n_matches);

            for _i in 0..self.n_matches {
                // create a new match
                let mut m = Match::new();

                // play match
                m.play_match(
                    // &mut self.players.iter_mut().map(|p| p).collect(),
                    &mut self.players,
                    self.deck_type,
                    &mut self.deck_buff,
                    rng,
                );

                self.matches.push(MatchBox::new(m));
            }
        }

        pub fn set_num_matches(&mut self, n_matches: usize) {
            let ad_matches = n_matches - self.n_matches;
            if ad_matches > 0 {
                self.matches.reserve(ad_matches);
            }

            self.n_matches = n_matches;
        }
    }

    // define the rules
    pub mod rules {

        use crate::game::Card;
        use rusqlite::{
            params,
            types::{FromSql, FromSqlError, FromSqlResult, ToSql, ToSqlOutput, ValueRef},
            Connection, Result,
        };

        // filepath for db
        const DB_FILE: &str = "./db/rustyheads.db";

        // enum deck types
        #[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
        pub enum DeckType {
            Tournament,
            WithNines,
        }

        impl FromSql for DeckType {
            fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
                match value.as_i64()? {
                    1 => Ok(DeckType::Tournament),
                    2 => Ok(DeckType::WithNines),
                    other => Err(FromSqlError::Other(
                        format!("Invalid match type: {}", other).into(),
                    )),
                }
            }
        }

        impl ToSql for DeckType {
            fn to_sql(&self) -> Result<ToSqlOutput<'_>> {
                match self {
                    DeckType::Tournament => Ok(1.into()),
                    DeckType::WithNines => Ok(2.into()),
                }
            }
        }

        // enum game types
        #[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, Debug)]
        pub enum MatchType {
            Normal,
            JackSolo,
            QueenSolo,
            BestSolo,
            HeartsSolo,
            SpadesSolo,
            CrossSolo,
            Fleshless,
        }

        impl FromSql for MatchType {
            fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
                match value.as_i64()? {
                    1 => Ok(MatchType::Normal),
                    2 => Ok(MatchType::JackSolo),
                    3 => Ok(MatchType::QueenSolo),
                    4 => Ok(MatchType::BestSolo),
                    5 => Ok(MatchType::HeartsSolo),
                    6 => Ok(MatchType::SpadesSolo),
                    7 => Ok(MatchType::CrossSolo),
                    8 => Ok(MatchType::Fleshless),
                    other => Err(FromSqlError::Other(
                        format!("Invalid match type: {}", other).into(),
                    )),
                }
            }
        }

        impl ToSql for MatchType {
            fn to_sql(&self) -> Result<ToSqlOutput<'_>> {
                match self {
                    MatchType::Normal => Ok(1.into()),
                    MatchType::JackSolo => Ok(2.into()),
                    MatchType::QueenSolo => Ok(3.into()),
                    MatchType::BestSolo => Ok(4.into()),
                    MatchType::HeartsSolo => Ok(5.into()),
                    MatchType::SpadesSolo => Ok(6.into()),
                    MatchType::CrossSolo => Ok(7.into()),
                    MatchType::Fleshless => Ok(8.into()),
                }
            }
        }

        pub fn get_deck_for_decktype(deck_type: DeckType) -> Option<Vec<Card>> {
            // Connect to an SQLite database in memory or a file
            let conn = Connection::open(DB_FILE).unwrap();

            // sort the cards based on the deck type
            // query db for cards in normal game
            let mut stmt = conn
                .prepare(
                    "
                        SELECT c.suit, c.face 
                          FROM cards AS c
                          JOIN cards_per_deck AS cpd ON c.id = cpd.card_id
                         WHERE cpd.deck_type = ?1
                            ",
                )
                .unwrap();

            let card_iter = stmt.query_map(params![deck_type], |row| {
                Ok(Card::new(row.get(0)?, row.get(1)?, 0, false, 0))
            });

            let mut cards = Vec::new();
            for card in card_iter.unwrap() {
                let card = card.unwrap();
                cards.push(card);
                cards.push(card.clone());
            }

            cards.sort();

            print!("Normal game cards: ");
            for card in cards.iter() {
                println!("{} ", card);
            }
            println!();

            Some(cards)
        }

        // contains the game rules
        pub fn get_deck_for_matchtype(
            match_type: MatchType,
            deck_type: DeckType,
        ) -> Option<Vec<Card>> {
            // Connect to an SQLite database in memory or a file
            let conn = Connection::open(DB_FILE).unwrap();

            // sort the cards based on the match type
            // query db for cards in normal game
            let mut stmt = conn
                .prepare(
                    "
                        SELECT c.suit, c.face, ey.eyes, cpr.trump, cpr.rank
                          FROM cards AS c
                          JOIN cards_per_deck AS cpd ON c.id = cpd.card_id
                          JOIN cards_per_rule AS cpr ON cpd.id = cpr.cpd_id
                          JOIN eyes_per_face AS ey ON c.face = ey.face
                           AND ey.deck_type = cpd.deck_type
                         WHERE cpd.deck_type = ?1
                           AND cpr.match_type = ?2
                            ",
                )
                .unwrap();

            let card_iter = stmt.query_map(params![match_type, deck_type], |row| {
                Ok(Card::new(
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                ))
            });

            let mut cards = Vec::new();
            for card in card_iter.unwrap() {
                let card = card.unwrap();
                cards.push(card);
                cards.push(card.clone());
            }

            cards.sort();

            print!("Normal game cards: ");
            for card in cards.iter() {
                print!("{} ", card.rank as u8);
            }
            println!();

            Some(cards)
        }
    }

    pub fn get_team_for_player(player: &Player, match_type: MatchType) -> Team {
        // set the team of the player based on the match type
        match match_type {
            rules::MatchType::Normal => {
                // search for queen of clubs in players hand
                player
                    .hand
                    .iter()
                    .any(|c| c.face == Face::Queen && c.suit == Suit::Clubs)
                    .then(|| Team::Re)
                    .unwrap_or(Team::Contra)
            }
            rules::MatchType::JackSolo => Team::Contra,
            rules::MatchType::QueenSolo => Team::Contra,
            rules::MatchType::BestSolo => Team::Contra,
            rules::MatchType::HeartsSolo => Team::Contra,
            rules::MatchType::SpadesSolo => Team::Contra,
            rules::MatchType::CrossSolo => Team::Contra,
            rules::MatchType::Fleshless => Team::Contra,
        }
    }

    mod simulation {

        use core::panic;
        use std::usize;

        use rand::seq::SliceRandom;

        use super::{
            Card, RngType, DynPlayers, Match, Player, PlayerBehav, RankType, Round, ServeFlag,
            ServeFlagType, SimulatedPlayer, Team,
        };

        type DepthType = u8;

        // implement a tree structure to simulate a series of cards played
        #[derive(Clone, Debug)]
        struct CardNode {
            rank: RankType,
            score: i32,
            alpha: i32,
            beta: i32,
            visited: bool,
            depth: DepthType,
            current_player: usize,
            cards_to_play: Vec<RankType>,
        }

        impl CardNode {
            fn new(
                rank: RankType,
                team: Team,
                depth: DepthType,
                current_player: usize,
                cards_to_play: Vec<RankType>,
            ) -> Self {
                CardNode {
                    rank,
                    score: get_initial_score_for_team(team),
                    alpha: get_initial_score_for_team(Team::Re),
                    beta: get_initial_score_for_team(Team::Contra),
                    visited: false,
                    depth,
                    current_player,
                    cards_to_play,
                }
            }
        }

        fn get_initial_score_for_team(team: Team) -> i32 {
            // initial score for a team is 0
            match team {
                Team::Re => i32::MIN,     // Re is maximizer, so start with min score
                Team::Contra => i32::MAX, // Contra is minimizer, so start with max score
            }
        }

        fn get_initial_score_for_player(player: &Player) -> i32 {
            // initial score for a player is 0
            get_initial_score_for_team(player.team)
        }

        fn get_score_for_team(
            old_optimum: i32,
            new_score: i32,
            optimum_rank: RankType,
            new_rank: RankType,
            team: Team,
        ) -> (RankType, i32) {
            // get the score for a player based on his team
            match team {
                Team::Re => {
                    if new_score > old_optimum {
                        (new_rank, new_score)
                    } else {
                        (optimum_rank, old_optimum)
                    }
                }
                Team::Contra => {
                    if new_score < old_optimum {
                        (new_rank, new_score)
                    } else {
                        (optimum_rank, old_optimum)
                    }
                }
            }
        }

        fn get_score_for_player(
            old_optimum: i32,
            new_score: i32,
            optimum_rank: RankType,
            new_rank: RankType,
            player: &Player,
        ) -> (RankType, i32) {
            // get the score for a player based on his team
            get_score_for_team(old_optimum, new_score, optimum_rank, new_rank, player.team)
        }

        fn get_alpha_beta_for_team(
            old_alpha: i32,
            old_beta: i32,
            new_score: i32,
            team: Team,
        ) -> (i32, i32) {
            // get the alpha and beta values for a team
            match team {
                Team::Re => {
                    // Re is maximizer, so update alpha
                    (old_alpha.max(new_score), old_beta)
                }
                Team::Contra => {
                    // Contra is minimizer, so update beta
                    (old_alpha, old_beta.min(new_score))
                }
            }
        }

        fn is_branch_prunable(alpha: i32, beta: i32, team: Team) -> bool {
            // check if the branch can be pruned
            match team {
                Team::Re => alpha >= beta, // Re is maximizer, so prune if alpha >= beta
                Team::Contra => alpha <= beta, // Contra is minimizer, so prune if alpha <= beta
            }
        }

        fn push_round_to_tree(round: &Round, players: &Vec<SimulatedPlayer>) -> Vec<CardNode> {
            let cpr = players.len(); // cards per round
            round
                .played_cards
                .iter()
                .enumerate()
                .map(|(d, c)| {
                    let depth = d.try_into().unwrap();
                    let current_player = (round.starting_player + d) % cpr;
                    CardNode::new(
                        c.rank,
                        players[current_player].data().team,
                        depth,
                        current_player,
                        Vec::new(), // since cards are played allready this is empty
                    )
                })
                .collect::<Vec<CardNode>>()
        }

        pub fn redistribute_unknown_cards(
            players: &mut Vec<SimulatedPlayer>,
            current_round: &Round,
            rng: &mut RngType,
            max_retries: usize,
        ) {
            // safe the numer of cards each player had
            let num_cards = players
                .iter()
                .map(|p| p.get_num_cards())
                .collect::<Vec<usize>>();

            // other players cards
            let mut opc = players
                .iter_mut()
                .enumerate()
                .filter_map(|(i, p)| {
                    if i != current_round.current_player {
                        Some(p.data_mut().hand.drain(..))
                    } else {
                        None
                    }
                })
                .flatten()
                .collect::<Vec<Card>>();

            // build subsets of cards each player may get
            opc.sort();

            let subsets = players
                .iter()
                .enumerate()
                .filter_map(|(i, p)| {
                    Some(
                        opc.iter()
                            .filter_map(|c| {
                                // if the player could not serve this kind of card before,
                                // he surely does not have it now
                                // check if his serve flag is set, if yes he may have the card
                                let sf = ServeFlag::flag_for_card(c) as ServeFlagType;
                                match (p.data().serve_flags & sf) != 0 {
                                    true => Some(c.rank),
                                    false => None,
                                }
                            })
                            .collect(),
                    )
                })
                .collect::<Vec<Vec<RankType>>>();

            // now we have to redistribute the cards
            let mut valid = false;
            let mut c = 0;
            while valid == false && c < max_retries {
                // clear the players hands
                players.iter_mut().enumerate().for_each(|(i, p)| {
                    if i != current_round.current_player {
                        p.data_mut().hand.clear();
                    }
                });

                // generate a new random card order
                opc.shuffle(rng);

                opc.iter().for_each(|c| {
                    // search a player that may get this card
                    players
                        .iter_mut()
                        .enumerate()
                        // check if the player has enough cards
                        // this should also automatically eliminate the current player
                        .filter(|(i, p)| p.get_num_cards() < num_cards[*i])
                        // check if the card is in the possible cards
                        .filter(|(i, p)| match subsets[*i].binary_search(&c.rank) {
                            Result::Ok(ri) => true,
                            Result::Err(ri) => false,
                        })
                        .for_each(|(i, p)| {
                            p.data_mut().hand.push(c.clone());
                        });
                });

                // TODO: check if this card constellation was already cached

                // did all players get enough cards?
                valid = players
                    .iter()
                    .enumerate()
                    .all(|(i, p)| p.get_num_cards() == num_cards[i]);

                c += 1;
            }
            if valid == false {
                panic!("Distribution failed");
            }
        }

        pub fn simulate(
            current_match: &Match,
            current_round: &Round,
            possible_cards: &Vec<Card>,
            players: &DynPlayers,
            max_depth: usize,
            rng: &mut RngType,
        ) -> (RankType, i32) {
            // create a LUT for cards
            let mut card_lut = current_match.deck.clone();
            card_lut.sort();
            card_lut.dedup();

            // for round in &current_match.rounds {
            //     nodes.append(&mut push_round_to_tree(round, players.len()));
            // }

            // clone the players into simulated players
            let mut sim_pl = players
                .iter()
                .map(|p| SimulatedPlayer {
                    data: p.data().clone(),
                })
                .collect::<Vec<SimulatedPlayer>>();

            // redistribute the cards randomly
            // TODO: make this work... currently hands can be duplicated
            // redistribute_unknown_cards(&mut sim_pl, current_match, current_round, rng, 100, &card_lut);

            return minimax_broad(
                possible_cards,
                &mut sim_pl,
                current_round,
                max_depth,
                &card_lut,
            );
        }

        fn minimax_broad(
            possible_cards: &Vec<Card>,
            players: &mut Vec<SimulatedPlayer>,
            current_round: &Round,
            max_depth: usize,
            card_lut: &Vec<Card>,
        ) -> (RankType, i32) {
            // we call the depth first search minimax algorithm for every possible card
            // and return the move with the best score

            // create a tree of cards played till now
            // (will result in kind-of linked list)
            let mut nodes = Vec::new();
            nodes.append(&mut push_round_to_tree(current_round, players));

            // now we expand the tree until players have no cards left, evaluating the best score
            struct BestMove {
                rank: RankType,
                score: i32,
            }

            let mut bm = BestMove {
                rank: 0,
                score: get_initial_score_for_player(&players[current_round.current_player].data()),
            };

            for c in possible_cards {
                // call minimax for each card the current player can play
                nodes.push(CardNode::new(
                    c.rank,
                    players[current_round.current_player].data().team,
                    nodes.len() as DepthType,
                    current_round.current_player,
                    Vec::new(),
                ));

                players[current_round.current_player].remove_card_from_hand(&c);

                // when returning from minimax, last card will be pushed into players hand again
                let best_score = minimax(&mut nodes, players, max_depth, &card_lut);

                (bm.rank, bm.score) = get_score_for_player(
                    bm.score,
                    best_score,
                    bm.rank,
                    c.rank,
                    players[current_round.current_player].data(),
                );
            }

            // return the best move
            (bm.rank, bm.score)
        }

        fn minimax(
            nodes: &mut Vec<CardNode>,
            players: &mut Vec<SimulatedPlayer>,
            max_depth: usize,
            card_lut: &Vec<Card>,
        ) -> i32 {
            // iterative approach to minimax expanding the tree of played cards
            // re is the maximizer, contra is the minimizer
            // the depth of the tree is the number of cards played

            let cpr = players.len(); // cards per round
            let cig = card_lut.len() * 2; // cards in game

            let max_cards_to_play: usize = players.iter().map(|p| p.get_num_cards()).sum();

            let _max_depth = max_cards_to_play.min(max_depth);

            if nodes.len() == 0 {
                panic!("No nodes in tree to evaluate");
            }
            let inl = nodes.len() - 1; // initial length of the tree

            loop {
                // pop the last node from the tree
                let mut current_node = nodes.pop().unwrap();
                let cnd = current_node.depth as usize; // current node depth

                // println!("current node: {:?}", current_node);
                if cnd == _max_depth && current_node.visited == false {
                    // go through the nodes and collect the cards won by each team
                    let mut re_score = 0;
                    let mut contra_score = 0;
                    let mut starting_player = match nodes.first() {
                        Some(n) => n.current_player,
                        None => current_node.current_player,
                    };
                    // cards played in this round
                    let mut round_cards = Vec::new();

                    for n in &*nodes {
                        let card = &card_lut[n.rank as usize - 1];

                        round_cards.push(card.clone());

                        if round_cards.len() == cpr {
                            let winner =
                                Round::determine_winner(&round_cards, starting_player).unwrap();

                            if players[winner].data().team == Team::Re {
                                re_score += round_cards.iter().map(|c| c.eyes as i32).sum::<i32>();
                            } else {
                                contra_score +=
                                    round_cards.iter().map(|c| c.eyes as i32).sum::<i32>();
                            }
                            starting_player = winner;
                            round_cards.clear();
                        }
                    }

                    if re_score == i32::MIN || contra_score == i32::MAX {
                        panic!(
                            "Invalid scores: re_score: {}, contra_score: {}, depth: {}",
                            re_score, contra_score, cnd
                        );
                    }

                    // simple score: difference of won eyes per team
                    // set the score of the predecessor
                    current_node.score = re_score - contra_score;
                    current_node.visited = true;
                    nodes.push(current_node.clone());
                } else if current_node.cards_to_play.len() == 0 && current_node.visited == true {
                    // node is fully expanded, evaluate from bottom to top

                    // when popping nodes, give the card back to the player who played it
                    players[current_node.current_player]
                        .data_mut()
                        .hand
                        .push(card_lut[current_node.rank as usize - 1].clone());

                    if nodes.len() == inl {
                        // if this is the first node we are evaluating
                        return current_node.score;
                    } else {
                        let last_node = nodes.last_mut().unwrap();

                        if current_node.score == i32::MIN || current_node.score == i32::MAX {
                            panic!("Invalid score: {}, depth: {}", current_node.score, cnd);
                        }
                        let team = players[last_node.current_player].data().team;

                        /* ALPHA-BETA pruning */
                        // we update the above score
                        (_, last_node.score) =
                            get_score_for_team(last_node.score, current_node.score, 0, 0, team);

                        // update alpha and beta values
                        (last_node.alpha, last_node.beta) = get_alpha_beta_for_team(
                            last_node.alpha,
                            last_node.beta,
                            current_node.score,
                            team,
                        );

                        if is_branch_prunable(last_node.alpha, last_node.beta, team) {
                            // prune by removing the other possible moves
                            last_node.cards_to_play.clear();
                        }
                        /* ALPHA-BETA pruning */
                    }
                } else {
                    // more moves to explore

                    // get played cards as tail of the tree
                    let mut played_cards = nodes[(nodes.len() - cnd % cpr)..]
                        .iter()
                        .map(|n| card_lut[n.rank as usize - 1].clone())
                        .collect::<Vec<Card>>();
                    played_cards.push(card_lut[current_node.rank as usize - 1].clone());

                    if current_node.cards_to_play.len() == 0 {
                        // if no cards to play, we have to determine the possible cards
                        // the next player could play
                        // this is the first time we visit this node

                        // get the possible cards from the tree
                        if played_cards.len() == cpr {
                            // next player is the winner
                            // next player may play any card
                            let next_player = Round::determine_winner(
                                &played_cards,
                                cpr - current_node.current_player - 1,
                            )
                            .unwrap();

                            current_node.cards_to_play = players[next_player]
                                .data()
                                .hand
                                .iter()
                                .map(|c| c.rank)
                                .collect();
                        } else {
                            // get possible cards from played cards
                            let next_player = (current_node.current_player + 1) % cpr;
                            current_node.cards_to_play = Round::filter_possible_cards(
                                &played_cards,
                                &players[next_player].data().hand,
                            )
                            .iter()
                            .map(|c| c.rank)
                            .collect::<Vec<RankType>>();
                        }

                        if current_node.cards_to_play.len() > 0 {
                            current_node.visited = true;
                        } else if current_node.cards_to_play.len() == 0 && nodes.len() + 1 == cig {
                            // special case where we reached the end of the game
                            // next iteration should evaluate this node if flag is false
                            current_node.visited = false;
                        } else {
                            panic!("No possible cards at depth {}/{}", cnd, max_depth);
                        }

                        nodes.push(current_node);
                    } else {
                        let next_player = match played_cards.len() == cpr {
                            true => Round::determine_winner(
                                &played_cards,
                                cpr - current_node.current_player - 1,
                            )
                            .unwrap(),
                            false => (current_node.current_player + 1) % cpr,
                        };

                        // remove card from possible cards
                        let rc = card_lut[current_node.cards_to_play.pop().unwrap() as usize - 1];

                        // remove card from players hand
                        players[next_player].remove_card_from_hand(&rc).unwrap();

                        let new_depth = current_node.depth + 1;
                        nodes.push(current_node);
                        nodes.push(CardNode::new(
                            rc.rank,
                            players[next_player].data().team,
                            new_depth,
                            next_player,
                            Vec::new(),
                        ));
                    }
                }
            }
        }
    }
}

fn main() {
    // create a new game using a random number generator
    // we need a fixed seed for the rng to make the game reproducible
    // let mut rng_shuffle = rand::thread_rng();
    let mut rng_shuffle = rand_chacha::ChaCha20Rng::seed_from_u64(42);
    let mut game = game::Game::new(1, rules::DeckType::Tournament);

    // add players to the game
    game.add_player("Player 1".to_string(), game::PlayerType::Computer);
    game.add_player("Player 2".to_string(), game::PlayerType::Computer);
    game.add_player("Player 3".to_string(), game::PlayerType::Computer);
    game.add_player("Player 4".to_string(), game::PlayerType::Computer);

    game.play_game(1, &mut rng_shuffle);
}
