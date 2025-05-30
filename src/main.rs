use game::rules;

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
            write!(f, "{:?}{:?}", self.suit, self.face)
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

    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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
            rng: &mut rand::rngs::ThreadRng,
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
            rng: &mut rand::rngs::ThreadRng,
        ) -> Option<Card> {
            // for simplicity, we just return the first card in the hand
            // in a real game, this would be more complex

            simulation::simulate(current_match, current_round, players, rng);

            if self.get_num_cards() == 0 {
                return None;
            } else {
                println!("{} hand: {:?}", self.data().name, possible_cards);
                println!("{} plays: {}", self.data().name, possible_cards[0]);
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
            current_match: &Match,
            current_round: &Round,
            players: &DynPlayers,
            rng: &mut rand::rngs::ThreadRng,
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

    impl SimulatedPlayer {
        // create a new SimulatedPlayer
        fn new(name: String, dealer: bool) -> SimulatedPlayer {
            SimulatedPlayer {
                data: Player::new(name, dealer),
            }
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
            current_match: &Match,
            current_round: &Round,
            players: &DynPlayers,
            rng: &mut rand::rngs::ThreadRng,
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
            rng: &mut rand::rngs::ThreadRng,
        ) -> Option<usize> {
            self.init_round(players.len(), last_rounds_winner);

            // each player plays one card
            for i in 0..players.len() {
                let card = self.play_card(current_match, players, rng);

                self.current_player = (self.current_player + 1) % players.len();

                self.played_cards.push(card)
            }

            self.winner = self.determine_winner().unwrap();

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
            rng: &mut rand::rngs::ThreadRng,
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

        fn determine_winner(&self) -> Option<usize> {
            if self.played_cards.len() == 0 {
                return None;
            } else {
                let first_card = self.played_cards[0];
                let mut winner = 0;

                for i in 1..self.played_cards.len() {
                    let card = self.played_cards[i];

                    // compare the current winning card to first card and the current card
                    match self.played_cards[winner].winning_card(&first_card, &card) {
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
                winner = (self.starting_player + winner) % self.played_cards.len();

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
            rng: &mut rand::rngs::ThreadRng,
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
            rng: &mut rand::rngs::ThreadRng,
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

        fn shuffle_cards(&mut self, rng: &mut rand::rngs::ThreadRng) {
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

    type GameBox = Box<Game>;
    type GameBoxes = Vec<GameBox>;

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

        pub fn play_game(&mut self, n_matches: usize, rng: &mut rand::rngs::ThreadRng) {
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

        use std::{fmt::Pointer, ops::Index, usize};

        use rand::seq::SliceRandom;

        use super::{
            Card, DynPlayer, DynPlayers, Match, Player, PlayerBehav, RankType, Round, RoundBoxes,
            ServeFlag, ServeFlagType, SimulatedPlayer, Team,
        };

        type DepthType = u8;

        // implement a tree structure to simulate a series of cards played
        #[derive(Debug)]
        struct CardNode {
            rank: RankType,
            score: i32,
            best_move: RankType,
            depth: DepthType,
            current_player: usize,
            cards_to_play: Vec<RankType>,
        }

        impl CardNode {
            fn new(
                rank: RankType,
                score: i32,
                best_move: RankType,
                depth: DepthType,
                current_player: usize,
                cards_to_play: Vec<RankType>,
            ) -> Self {
                CardNode {
                    rank,
                    score,
                    best_move,
                    depth,
                    current_player,
                    cards_to_play,
                }
            }

            fn new_initial() -> Self {
                // return an initial Node
                Self::new(0, 0, 0, 0, 0, Vec::new())
            }
        }

        fn push_round_to_tree(round: &Round, cpr: usize) -> Vec<CardNode> {
            round
                .played_cards
                .iter()
                .enumerate()
                .map(|(d, c)| {
                    let depth = d.try_into().unwrap();
                    CardNode::new(
                        c.rank,
                        0,
                        c.rank,
                        depth,
                        (round.starting_player + d) % cpr,
                        Vec::new(), // since cards are played allready this is empty
                    )
                })
                .collect::<Vec<CardNode>>()
        }

        fn tree_to_rounds(
            nodes: &Vec<CardNode>,
            cpr: usize, // cards per round
            cig: usize, // cards in game
            match_starter: usize,
            card_lut: &Vec<Card>,
        ) -> Vec<Round> {
            // reconstruct the rounds from tree
            let mut rounds = Vec::new();
            let mut lrw = match_starter; // last round winner

            for n in &*nodes {
                if n.rank == 0 {
                    panic!("Invalid node with rank 0 in tree: {:?}", n);
                } else if (n.depth as usize) % cpr == 0 {
                    // first card of the round
                    let nr = Round {
                        played_cards: Vec::new(),
                        starting_player: lrw,
                        current_player: lrw,
                        winner: 0,
                    };
                    rounds.push(nr);
                }

                // add the card to the current round
                let r = rounds.last_mut().unwrap();
                r.played_cards.push(card_lut[(n.rank - 1) as usize]);

                // update winner and current player
                r.current_player = (r.current_player + 1) % cpr;

                if r.played_cards.len() == cpr {
                    // if all players played a card, determine the winner
                    r.winner = r.determine_winner().unwrap();
                    lrw = r.winner;
                }
            }

            rounds
        }

        pub fn redistribute_unknown_cards(
            players: &mut Vec<SimulatedPlayer>,
            current_match: &Match,
            current_round: &Round,
            rng: &mut rand::rngs::ThreadRng,
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
            players: &DynPlayers,
            rng: &mut rand::rngs::ThreadRng,
        ) {
            // create a LUT for cards
            let mut card_lut = current_match.deck.clone();
            card_lut.sort();
            card_lut.dedup();

            // create a tree of cards played till now
            // (will result in kind-of linked list)
            let mut nodes = Vec::new();

            for round in &current_match.rounds {
                nodes.append(&mut push_round_to_tree(round, players.len()));
            }

            nodes.append(&mut push_round_to_tree(current_round, players.len()));

            // clone the players into simulated players
            let mut sim_pl = players
                .iter()
                .map(|p| SimulatedPlayer {
                    data: p.data().clone(),
                })
                .collect::<Vec<SimulatedPlayer>>();

            // redistribute the cards randomly
            redistribute_unknown_cards(&mut sim_pl, current_match, current_round, rng, 100);

            let match_starter = match current_match.rounds.first() {
                Some(r) => r.starting_player,
                None => current_round.starting_player,
            };

            // now we expand the tree until players have no cards left, evaluating the best score
            for c in Round::filter_possible_cards(
                &current_round.played_cards,
                &players[current_round.current_player].data().hand,
            ) {
                // call minimax for each card the current player can play
                nodes.push(CardNode::new(
                    c.rank,
                    0,
                    c.rank,
                    nodes.len() as DepthType,
                    current_round.current_player,
                    // TODO: replace this or do expansion in minimax
                    Vec::new(),
                ));
                minimax(&mut nodes, &mut sim_pl, 40, &card_lut, match_starter);
            }
        }

        fn possible_cards_from_tree(nodes: &Vec<CardNode>, cards_played: usize, ) -> Vec<Card> {
            if cards_played == 0 {
                // player may play every card
                current_player.data().hand.clone()
            } else {
                // get cards_played predecessors of this node
                let played_cards = nodes
                    .iter()
                    .rev()
                    .take(cards_played)
                    .map(|n| card_lut[n.rank as usize - 1].clone())
                    .rev()
                    .collect::<Vec<Card>>();

                Round::filter_possible_cards(&played_cards, &current_player.data().hand)
            }
        }

        fn winner_from_tree(nodes: &Vec<CardNode>) -> Option<usize> {
            // get the winner of current round to be the next starter
            if played_cards.len() == cpr {
                Round {
                    played_cards: played_cards.clone(),
                    starting_player,
                    current_player: next_player,
                    winner: 0,
                }
                .determine_winner()
            } else {
                None
            };
        }

        fn minimax(
            nodes: &mut Vec<CardNode>,
            players: &mut Vec<SimulatedPlayer>,
            max_depth: usize,
            card_lut: &Vec<Card>,
            match_starter: usize,
        ) -> i32 {
            // iterative approach to minimax expanding the tree of played cards
            // re is the maximizer, contra is the minimizer
            // the depth of the tree is the number of cards played

            let cpr = players.len(); // cards per round
            let cig = card_lut.len() * 2; // cards in game

            while let mut current_node = nodes.pop().unwrap() {
                let cnd = current_node.depth as usize; // current node depth

                println!("current node: {:?}", current_node);

                if cnd == max_depth || cnd == cig {
                    // simple score: difference of won eyes per team
                    let scores = [Team::Re, Team::Contra]
                        .iter()
                        .map(|t| {
                            players
                                .iter_mut()
                                .filter(|p| p.data().team == *t)
                                .map(|p| {
                                    p.data()
                                        .won_cards
                                        .iter()
                                        .map(|c| c.eyes as i32)
                                        .sum::<i32>()
                                })
                                .sum()
                        })
                        .collect::<Vec<i32>>();

                    // append the score to the predecessor
                    nodes.last_mut().unwrap().score = scores[0] - scores[1];
                    nodes.last_mut().unwrap().best_move = current_node.rank;

                    // when popping nodes, give back the card to the player who played it
                    players[current_node.current_player]
                        .data_mut()
                        .hand
                        .push(card_lut[current_node.rank as usize - 1].clone());
                } else if current_node.cards_to_play.len() == 0 {
                    // node is fully expanded
                    let current_player = &mut players[current_node.current_player];
                    let maximizing_player = current_player.data().team == Team::Re;

                    let mut pred_n = nodes.last_mut().unwrap(); // predecessor

                    if maximizing_player {
                        // if the player is the maximizer, we want to maximize the score
                        if current_node.score > pred_n.score {
                            pred_n.score = current_node.score;
                            pred_n.best_move = current_node.rank;
                        }
                    } else {
                        // if the player is the minimizer, we want to minimize the score
                        if current_node.score < pred_n.score {
                            pred_n.score = current_node.score;
                            pred_n.best_move = current_node.rank;
                        }
                    }

                    // when popping nodes, give back the card to the player who played it
                    current_player
                        .data_mut()
                        .hand
                        .push(card_lut[current_node.rank as usize - 1].clone());
                } else {
                    // more moves to explore
                    let current_player = &mut players[current_node.current_player];

                    // we have to distinct between a finished round and a running round
                    let mut possible_cards = Vec::new();

                    // in case this is the last player to play a card, determine the winner
                    let mut next_player = (current_node.current_player + 1) % cpr;
                    let cards_played = cnd % cpr; // cards played in this round

                    if cards_played == 0 {
                        // player may play every card
                        possible_cards = current_player.data().hand.clone();
                    } else {
                        // get cards_played predecessors of this node
                        println!("card nodes: {:?}", nodes);
                        let played_cards = nodes
                            .iter()
                            .rev()
                            .take(cards_played)
                            .map(|n| card_lut[n.rank as usize - 1].clone())
                            .rev()
                            .collect::<Vec<Card>>();

                        possible_cards = Round::filter_possible_cards(
                            &played_cards,
                            &current_player.data().hand,
                        );

                        // get the winner of current round to be the next starter
                        if played_cards.len() == cpr {
                            let round = Round {
                                played_cards: played_cards.clone(),
                                starting_player: match_starter,
                                current_player: next_player,
                                winner: 0,
                            };
                            next_player = round.determine_winner().unwrap();
                        };
                    }

                    if possible_cards.len() == 0 {
                        panic!("No possible cards");
                    }

                    possible_cards.sort();

                    println!("current node: {:?}", current_node);
                    println!(
                        "{} cards: {:?}",
                        current_player.data().name,
                        current_player.data().hand
                    );
                    println!(
                        "{} possible cards: {:?}",
                        current_player.data().name,
                        possible_cards
                    );

                    // remove card from possible cards
                    println!(
                        "cards to play: {:?}",
                        current_node
                            .cards_to_play
                            .iter()
                            .map(|r| card_lut[*r as usize - 1])
                            .collect::<Vec<Card>>()
                    );
                    let rc = card_lut[current_node.cards_to_play.pop().unwrap() as usize - 1];
                    // remove card from players hand
                    current_player.remove_card_from_hand(&rc);

                    let new_depth = current_node.depth + 1;

                    nodes.push(current_node);
                    nodes.push(CardNode::new(
                        rc.rank,
                        0,
                        0,
                        new_depth,
                        next_player,
                        players[next_player]
                            .data()
                            .hand
                            .iter()
                            .map(|c| c.rank)
                            .collect(),
                    ));
                }
            }
            0
        }
    }
}

fn main() {
    // create a new game using a random number generator
    let mut rng_shuffle = rand::thread_rng();
    let mut game = game::Game::new(1, rules::DeckType::Tournament);

    // add players to the game
    game.add_player("Player 1".to_string(), game::PlayerType::Computer);
    game.add_player("Player 2".to_string(), game::PlayerType::Computer);
    game.add_player("Player 3".to_string(), game::PlayerType::Computer);
    game.add_player("Player 4".to_string(), game::PlayerType::Computer);

    game.play_game(1, &mut rng_shuffle);
}
