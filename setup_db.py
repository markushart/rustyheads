import sqlite3

DB_PATH = "./db/rustyheads.db"

if __name__ == "__main__":
    with sqlite3.connect(DB_PATH) as conn:
        cursor = conn.cursor()

        cursor.execute('DROP TABLE IF EXISTS users;')

        # Create a new table
        cursor.execute('''
            CREATE TABLE IF NOT EXISTS users (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                age INTEGER NOT NULL
            )
        ''')

        cursor.execute('DROP TABLE IF EXISTS faces;')

        cursor.execute(''' 
            CREATE TABLE IF NOT EXISTS faces (
                id INTEGER PRIMARY KEY,
                face_key INTEGER NOT NULL,
                name TEXT NOT NULL
            )
        ''')

        cursor.execute('DROP TABLE IF EXISTS suits;')

        cursor.execute('''
            CREATE TABLE IF NOT EXISTS suits (
                id INTEGER PRIMARY KEY,
                suit_key INTEGER NOT NULL,
                name TEXT NOT NULL
            )
        ''')

        cursor.execute('DROP TABLE IF EXISTS cards;')

        cursor.execute('''
            CREATE TABLE IF NOT EXISTS cards (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                face INTEGER NOT NULL,
                suit INTEGER NOT NULL,
                FOREIGN KEY (face) REFERENCES faces(id),
                FOREIGN KEY (suit) REFERENCES suits(id),
                UNIQUE(face, suit)
            )
        ''')

        cursor.execute('DROP TABLE IF EXISTS match_types;')

        cursor.execute('''
            CREATE TABLE IF NOT EXISTS match_types (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT NOT NULL,
                solo BOOLEAN NOT NULL
            )
        ''')

        cursor.execute('DROP TABLE IF EXISTS cards_per_rule;')

        cursor.execute('''
            CREATE TABLE IF NOT EXISTS cards_per_rule (
                id INTEGER PRIMARY KEY,
                match_type INTEGER NOT NULL,
                card_id INTEGER NOT NULL,
                rank INTEGER NOT NULL,
                trump BOOLEAN NOT NULL,
                FOREIGN KEY ( match_type ) REFERENCES match_types(id),
                FOREIGN KEY ( card_id ) REFERENCES cards(id)
            )
        ''')

        cursor.execute('DROP TABLE IF EXISTS eyes_per_face;')

        cursor.execute('''
            CREATE TABLE IF NOT EXISTS eyes_per_face (
                id INTEGER PRIMARY KEY,
                match_type INTEGER NOT NULL,
                face INTEGER NOT NULL,
                eyes INTEGER NOT NULL,
                FOREIGN KEY ( match_type ) REFERENCES match_types(id),
                FOREIGN KEY ( face ) REFERENCES faces(id),
                UNIQUE(match_type, face)
            )
        ''')

        cursor.execute('''
            INSERT INTO faces (face_key, name) VALUES
                (1, 'Two'),
                (2, 'Three'),
                (3, 'Four'),
                (4, 'Five'),
                (5, 'Six'),
                (6, 'Seven'),
                (7, 'Eight'),
                (8, 'Nine'),
                (9, 'Ten'),
                (10, 'Jack'),
                (11, 'Queen'),
                (12, 'King'),
                (13, 'Ace')
        ''')

        cursor.execute('''
            INSERT INTO suits (suit_key, name) VALUES
                (1, 'Diamonds'),
                (2, 'Hearts'),
                (3, 'Spades'),
                (4, 'Clubs')
        ''')

        cursor.execute('''
            INSERT INTO cards (id, name, face, suit) VALUES
                (  1, 'Two of Diamonds', 1, 1),
                (  2, 'Three of Diamonds', 2, 1),
                (  3, 'Four of Diamonds', 3, 1),
                (  4, 'Five of Diamonds', 4, 1),
                (  5, 'Six of Diamonds', 5, 1),
                (  6, 'Seven of Diamonds', 6, 1),
                (  7, 'Eight of Diamonds', 7, 1),
                (  8, 'Nine of Diamonds', 8, 1),
                (  9, 'Ten of Diamonds', 9, 1),
                ( 10, 'Jack of Diamonds', 10, 1),
                ( 11, 'Queen of Diamonds', 11, 1),
                ( 12, 'King of Diamonds', 12, 1),
                ( 13, 'Ace of Diamonds', 13, 1),

                ( 14,  'Two of Hearts', 1, 2),
                ( 15, 'Three of Hearts', 2, 2),
                ( 16, 'Four of Hearts', 3, 2),
                ( 17, 'Five of Hearts', 4, 2),
                ( 18, 'Six of Hearts', 5, 2),
                ( 19, 'Seven of Hearts', 6, 2),
                ( 20, 'Eight of Hearts', 7, 2),
                ( 21, 'Nine of Hearts', 8, 2),
                ( 22, 'Ten of Hearts', 9, 2),
                ( 23, 'Jack of Hearts', 10, 2),
                ( 24, 'Queen of Hearts', 11, 2),
                ( 25, 'King of Hearts', 12, 2),
                ( 26, 'Ace of Hearts', 13, 2),

                ( 27, 'Two of Spades', 1, 3),
                ( 28, 'Three of Spades', 2, 3),
                ( 29, 'Four of Spades', 3, 3),
                ( 30, 'Five of Spades', 4, 3),
                ( 31, 'Six of Spades', 5, 3),
                ( 32, 'Seven of Spades', 6, 3),
                ( 33, 'Eight of Spades', 7, 3),
                ( 34, 'Nine of Spades', 8, 3),
                ( 35, 'Ten of Spades', 9, 3),
                ( 36, 'Jack of Spades', 10, 3),
                ( 37, 'Queen of Spades', 11, 3),
                ( 38, 'King of Spades', 12, 3),
                ( 39, 'Ace of Spades', 13, 3),

                ( 40, 'Two of Clubs', 1, 4),
                ( 41, 'Three of Clubs', 2, 4),
                ( 42, 'Four of Clubs', 3, 4),
                ( 43, 'Five of Clubs', 4, 4),
                ( 44, 'Six of Clubs', 5, 4),
                ( 45, 'Seven of Clubs', 6, 4),
                ( 46, 'Eight of Clubs', 7, 4),
                ( 47, 'Nine of Clubs', 8, 4),
                ( 48, 'Ten of Clubs', 9, 4),
                ( 49, 'Jack of Clubs', 10, 4),
                ( 50, 'Queen of Clubs', 11, 4),
                ( 51, 'King of Clubs', 12, 4),
                ( 52, 'Ace of Clubs', 13, 4)
        ''')

        cursor.execute('''
            INSERT INTO match_types (name, description, solo) VALUES
            ('Normal', 'Normal game with no special rules', 0),
            ('JackSolo', 'The caller plays alone, Jacks are trump', 1)
        ''')

        cursor.execute('''
            INSERT INTO eyes_per_face (match_type, face, eyes) VALUES
            (1,  9, 10),
            (1, 10,  2),
            (1, 11,  3),
            (1, 12,  4),
            (1, 13, 11),

            (2,  9, 10),
            (2, 10,  2),
            (2, 11,  3),
            (2, 12,  4),
            (2, 13, 11)
        ''')

        cursor.execute('''
            INSERT INTO cards_per_rule (match_type, card_id, rank, trump) VALUES
                (1,  9,  10, 1), -- Ten of Diamonds
                (1, 10,  12, 1), -- Jack of Diamonds
                (1, 11,  16, 1), -- Queen of Diamonds
                (1, 12,  20, 1), -- King of Diamonds
                (1, 13,  11, 1), -- Ace of Diamonds

                (1, 22,  4, 0),  -- Ten of Hearts
                (1, 23, 13, 1),  -- Jack of Hearts
                (1, 24, 17, 1),  -- Queen of Hearts
                (1, 25,  1, 0),  -- King of Hearts
                (1, 26,  7, 0),  -- Ace of Hearts

                (1, 35,  5, 0),  -- Ten of Spades
                (1, 36, 14, 1),  -- Jack of Spades
                (1, 37, 18, 1),  -- Queen of Spades
                (1, 38,  2, 0),  -- King of Spades
                (1, 39,  8, 0),  -- Ace of Spades

                (1, 48,  6, 0),  -- Ten of Clubs
                (1, 49, 15, 1),  -- Jack of Clubs
                (1, 50, 19, 1),  -- Queen of Clubs
                (1, 51,  3, 0),  -- King of Clubs
                (1, 52,  9, 0),  -- Ace of Clubs

                (2,  9,  9, 0),
                (2, 10, 17, 1),
                (2, 11,  1, 1),
                (2, 12,  5, 0),
                (2, 13, 13, 0),

                (2, 22, 10, 0),
                (2, 23, 18, 1),
                (2, 24,  2, 1),
                (2, 25,  6, 0),
                (2, 26, 14, 0),

                (2, 35, 11, 0),
                (2, 36, 19, 1),
                (2, 37,  3, 1),
                (2, 38,  7, 0),
                (2, 39, 15, 0),

                (2, 48, 12, 0),
                (2, 49, 20, 1),
                (2, 50,  4, 1),
                (2, 51,  8, 0),
                (2, 52, 16, 0)
        ''')

        conn.commit()
        print("Database setup complete.")

        # test insertion by pulling the cards of the first game type
        cursor.execute('''
            SELECT cards.name, cards.face, cards.suit, cards_per_rule.rank, cards_per_rule.trump
            FROM cards_per_rule
            JOIN cards ON cards.id = cards_per_rule.card_id
            JOIN eyes_per_face ON cards.face = eyes_per_face.face
                               AND cards_per_rule.match_type = eyes_per_face.match_type
            WHERE cards_per_rule.match_type = 2
        ''')

        rows = cursor.fetchall()

        # print cards sorted by rank
        for row in sorted(rows, key=lambda x: x[3]):
            print(row)
