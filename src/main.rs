use game::rules;

pub mod game {
    use rand::seq::SliceRandom;
    use rules::{DeckType, MatchType};
    use rusqlite::{
        types::{FromSql, FromSqlError, ToSql, ToSqlOutput, ValueRef},
        Result,
    };
    use std::fmt;
    // This module contains the game logic and rules
    // It includes the definitions of cards, players, rounds, and matches

    // enum for suits of cards
    #[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
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
    #[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
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

    // struct for a card
    #[derive(Clone, Copy)]
    pub struct Card {
        suit: Suit,
        face: Face,
        eyes: u8,
        trump: bool,
        rank: u32,
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
        fn new(suit: Suit, face: Face, eyes: u8, trump: bool, rank: u32) -> Card {
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
            // 0     | 1     | ...   | 1       | 1       | => NotPosiible
            // 1     | 0     | ...   | 1       | 1       | => NotPosiible
            // 1     | 1     | ...   | 0       | 1       | => NotPosiible
            // 1     | 1     | ...   | 1       | 0       | => NotPosiible

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

    pub enum Team {
        Contra,
        Re,
    }

    pub enum PlayerType {
        Human,
        Computer,
    }

    // struct for a player
    pub struct Player {
        name: String,
        hand: Vec<Card>,
        won_cards: Vec<Card>,
        team: Team,
        dealer: bool,
        beginner: bool,
    }

    pub struct HumanPlayer {
        data: Player,
    }

    pub struct ComputerPlayer {
        data: Player,
    }

    type DynPlayer = Box<dyn PlayerBehav>;
    type DynPlayers = Vec<DynPlayer>;

    pub trait PlayerBehav {
        // get data and mutable data
        fn data(&self) -> &Player;
        fn data_mut(&mut self) -> &mut Player;

        // function to choose a card from the player's hand
        fn choose_card(&self, round: &Round, possible_cards: &Vec<Card>) -> Option<Card>;

        // function to make a call
        fn make_call(&self) -> Option<MatchType>;

        fn set_my_team(&mut self, match_type: MatchType) {
            self.data_mut().team = get_team_for_player(self.data(), match_type);
        }

        fn get_num_cards(&self) -> usize {
            self.data().hand.len()
        }

        // function to play a card from the player's hand
        fn play_card(&mut self, round: &Round) -> Card {
            let possible_cards = round.filter_possible_cards(&self.data().hand);

            let card = self.choose_card(round, &possible_cards).unwrap();

            match self.data().hand.iter().position(|c| c == &card) {
                Some(i) => self.data_mut().hand.swap_remove(i),
                None => panic!("Card not found in hand"),
            }
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

    impl ComputerPlayer {
        // create a new ComputerPlayer
        fn new(name: String, dealer: bool) -> Player {
            Player {
                name,
                hand: Vec::new(),
                won_cards: Vec::new(),
                team: Team::Contra,
                dealer,
                beginner: false,
            }
        }
    }

    impl PlayerBehav for Player {
        fn data(&self) -> &Player {
            self
        }

        fn data_mut(&mut self) -> &mut Player {
            &mut *self
        }

        // function to choose a card from the player's hand
        fn choose_card(&self, round: &Round, possible_cards: &Vec<Card>) -> Option<Card> {
            // for simplicity, we just return the first card in the hand
            // in a real game, this would be more complex
            if self.get_num_cards() == 0 {
                return None;
            } else {
                println!("{} hand: {:?}", self.name, possible_cards);
                println!("{} plays: {}", self.name, possible_cards[0]);
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

    impl PlayerBehav for ComputerPlayer {
        fn data(&self) -> &Player {
            &self.data
        }

        fn data_mut(&mut self) -> &mut Player {
            &mut self.data
        }

        // function to choose a card from the player's hand
        fn choose_card(&self, round: &Round, possible_cards: &Vec<Card>) -> Option<Card> {
            // for simplicity, we just return the first card in the hand
            // in a real game, this would be more complex
            if self.get_num_cards() == 0 {
                return None;
            } else {
                // println!("{} hand: {:?}", self.name, possible_cards);
                // println!("{} plays: {}", self.name, possible_cards[0]);
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
        fn new(name: String, dealer: bool) -> Player {
            Player {
                name,
                hand: Vec::new(),
                won_cards: Vec::new(),
                team: Team::Contra,
                dealer,
                beginner: false,
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
        fn choose_card(&self, round: &Round, possible_cards: &Vec<Card>) -> Option<Card> {
            // for simplicity, we just return the first card in the hand
            // in a real game, this would be more complex
            if self.get_num_cards() == 0 {
                return None;
            } else {
                // println!("{} hand: {:?}", self.name, possible_cards);
                // println!("{} plays: {}", self.name, possible_cards[0]);
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

    pub trait Winnable {
        type Winner;
        type CheckType;

        fn determine_winner(&self, player: &Vec<&Self::CheckType>) -> Option<Self::Winner>;
    }

    // struct for a game round
    // in a single rount each player plays one card from his hand
    // there are maximum 5 players
    pub struct Round {
        played_cards: Vec<Card>,
        current_player: usize,
        starting_player: usize,
    }

    impl Winnable for Round {
        type Winner = usize;
        type CheckType = Card;

        fn determine_winner(&self, check_values: &Vec<&Self::CheckType>) -> Option<Self::Winner> {
            if check_values.len() == 0 {
                return None;
            } else {
                let first_card = check_values[0];
                let mut winner = 0;

                for i in 1..check_values.len() {
                    let card = check_values[i];

                    // compare the current winning card to first card and the current card
                    match check_values[winner].winning_card(first_card, card) {
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
                winner = (self.starting_player + winner) % check_values.len();

                Some(winner)
            }
        }
    }

    impl Round {
        pub fn new() -> Round {
            Round {
                played_cards: Vec::new(),
                current_player: 0,
                starting_player: 0,
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
            players: &mut DynPlayers,
            last_rounds_winner: usize,
        ) -> Option<usize> {
            self.init_round(players.len(), last_rounds_winner);

            // each player plays one card
            for _i in 0..players.len() {
                let card = players[self.current_player].data_mut().play_card(self);

                self.current_player = (self.current_player + 1) % players.len();

                self.played_cards.push(card)
            }

            let winner = self
                .determine_winner(
                    &self
                        .played_cards
                        .iter()
                        .map(|c| &c as &Card)
                        .collect::<Vec<&Card>>(),
                )
                .unwrap();

            self.current_player = winner;

            players[winner]
                .data_mut()
                .collect_won_cards(&self.played_cards);
            self.played_cards.clear();

            Some(winner)
        }

        fn filter_possible_cards(&self, hand: &Vec<Card>) -> Vec<Card> {
            // filter the cards that can be played in this round
            if self.played_cards.len() == 0 {
                // if i am the first to play a card, every card is allowed
                hand.clone()
            } else {
                let first_card = self.played_cards[0];

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
    }

    // struct for a match
    // a match consists of multiple rounds until no player has cards left
    // at the beginning of the match, cards are dealt to players
    // then the players give calls, depending on the call, the trump cards are set
    pub struct Match {
        rounds: Vec<Round>,
        deck: Vec<Card>,
        match_type: MatchType, // type of match
        n_rounds: usize,       // number of rounds in the match
    }

    impl Winnable for Match {
        type Winner = Team;
        type CheckType = u32;

        fn determine_winner(&self, check_values: &Vec<&Self::CheckType>) -> Option<Self::Winner> {
            if check_values.len() < 2 {
                return None;
            } else {
                // // filter by team of player, sum their eye scores
                // let re_score = self.get_team_score(check_values, Team::Re);
                // let contra_score = self.get_team_score(check_values, Team::Contra);

                // Contra wins if teams are equal
                let contra_score = check_values[0];
                let re_score = check_values[1];
                if re_score <= contra_score {
                    Some(Team::Contra)
                } else {
                    Some(Team::Re)
                }
            }
        }
    }

    impl Match {
        pub fn new() -> Match {
            Match {
                rounds: Vec::new(),
                deck: Vec::new(),
                match_type: MatchType::Normal,
                n_rounds: 0,
            }
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
            rng: &mut rand::rngs::ThreadRng,
        ) {
            self.deck.clear();

            // TODO: implement buffer for cards_per_decktype and deck_per_rule
            self.deck = rules::get_deck_for_decktype(deck_type);

            assert_ne!(self.deck.len(), 0, "Deck is empty");
            assert_eq!(
                self.deck.len() % players.len(),
                0,
                "Number of cards is not divisible by number of players",
            );

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
            rng: &mut rand::rngs::ThreadRng,
        ) {
            // init game, shuffle and distribute cards
            self.init_match(players, deck_type, rng);

            // depending on what cards the players have, determine the match type

            // self.match_type = self.determine_match_type(&players.iter().map(|p| &**p).collect());
            self.match_type = self.determine_match_type(players);

            // set the deck of cards
            self.deck = rules::get_deck_for_matchtype(self.match_type, deck_type);

            // update the values of players hands
            for player in players.iter_mut() {
                player.update_hand_values(&self.deck);
            }

            // set the team of the players
            for player in players.iter_mut() {
                player.set_my_team(self.match_type);
            }

            // play rounds
            let mut last_round_winner = 0;
            for _i in 0..self.n_rounds {
                let mut r = Round::new();

                // after playing the roud, the winner is returned
                println!("Play round {}", _i);

                last_round_winner = r.play_round(players, last_round_winner).unwrap();

                println!("Round winner: {}", players[last_round_winner].data().name);
                println!();

                self.rounds.push(r);
            }
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
                .map(|i| players[(b_idx + i) % players.len()].data())
                .map(|p| p.make_call().unwrap())
                .max()
                .unwrap()
        }

        fn get_team_score(&self, players: &DynPlayers, team: Team) -> Option<u32> {
            // get the score of the team
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

    // struct representing the game
    // the overall game consists of multiple matches
    pub struct Game {
        players: DynPlayers,
        matches: Vec<Match>,
        n_matches: usize,
        deck_type: DeckType,
    }

    impl Game {
        // function to start a new game
        pub fn new(n_matches: usize, deck_type: DeckType) -> Game {
            let mut g = Game {
                players: Vec::new(),
                matches: Vec::new(),
                n_matches: 0,
                deck_type,
            };

            g.set_num_matches(n_matches);

            g
        }

        // function to add a player to the game
        pub fn add_player(&mut self, name: String, player_type: PlayerType) {
            let dealer = self.players.len() == 0;
            // let player = match player_type {
            //     PlayerType::Computer => ComputerPlayer::new(name, dealer),
            //     PlayerType::Human => HumanPlayer::new(name, dealer),
            // };
            // self.players.push(Box::new(player));
            match player_type {
                PlayerType::Computer => self.players.push(Box::new(ComputerPlayer::new(name, dealer))),
                PlayerType::Human => self.players.push(Box::new(HumanPlayer::new(name, dealer))),
            };
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
                    rng,
                );

                self.matches.push(m);
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
        #[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
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
        #[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
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

        pub fn get_deck_for_decktype(deck_type: DeckType) -> Vec<Card> {
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

            cards
        }

        // contains the game rules
        pub fn get_deck_for_matchtype(match_type: MatchType, deck_type: DeckType) -> Vec<Card> {
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

            cards
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
}

fn main() {
    // create a new game using a random number generator
    let mut rng_shuffle = rand::thread_rng();
    let mut game = game::Game::new(1, rules::DeckType::Tournament);

    // add players to the game
    game.add_player("Player 1".to_string(), crate::game::PlayerType::Computer);
    game.add_player("Player 2".to_string(), crate::game::PlayerType::Computer);
    game.add_player("Player 3".to_string(), crate::game::PlayerType::Computer);
    game.add_player("Player 4".to_string(), crate::game::PlayerType::Computer);

    game.play_game(1, &mut rng_shuffle);
}
