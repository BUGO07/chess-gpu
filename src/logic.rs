pub const STARTING_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PieceKind {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Piece {
    pub kind: PieceKind,
    pub white: bool,
}

impl Piece {
    pub fn from_char(c: char) -> Self {
        let (kind, white) = match c {
            'P' => (PieceKind::Pawn, true),
            'N' => (PieceKind::Knight, true),
            'B' => (PieceKind::Bishop, true),
            'R' => (PieceKind::Rook, true),
            'Q' => (PieceKind::Queen, true),
            'K' => (PieceKind::King, true),
            'p' => (PieceKind::Pawn, false),
            'n' => (PieceKind::Knight, false),
            'b' => (PieceKind::Bishop, false),
            'r' => (PieceKind::Rook, false),
            'q' => (PieceKind::Queen, false),
            'k' => (PieceKind::King, false),
            x => panic!("Invalid piece {x}"),
        };
        Self { kind, white }
    }
    pub fn to_char(&self) -> char {
        match (&self.kind, self.white) {
            (PieceKind::Pawn, true) => 'P',
            (PieceKind::Knight, true) => 'N',
            (PieceKind::Bishop, true) => 'B',
            (PieceKind::Rook, true) => 'R',
            (PieceKind::Queen, true) => 'Q',
            (PieceKind::King, true) => 'K',
            (PieceKind::Pawn, false) => 'p',
            (PieceKind::Knight, false) => 'n',
            (PieceKind::Bishop, false) => 'b',
            (PieceKind::Rook, false) => 'r',
            (PieceKind::Queen, false) => 'q',
            (PieceKind::King, false) => 'k',
        }
    }
    pub fn to_idx(&self) -> u32 {
        match (&self.kind, self.white) {
            (PieceKind::Rook, true) => 0,
            (PieceKind::Knight, true) => 1,
            (PieceKind::Bishop, true) => 2,
            (PieceKind::Queen, true) => 3,
            (PieceKind::King, true) => 4,
            (PieceKind::Pawn, true) => 5,
            (PieceKind::Rook, false) => 6,
            (PieceKind::Knight, false) => 7,
            (PieceKind::Bishop, false) => 8,
            (PieceKind::Queen, false) => 9,
            (PieceKind::King, false) => 10,
            (PieceKind::Pawn, false) => 11,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BoardState {
    pub pieces: Vec<Option<Piece>>,
    pub white_to_play: bool,
    pub white_can_oo: bool,
    pub white_can_ooo: bool,
    pub black_can_oo: bool,
    pub black_can_ooo: bool,
    pub en_passant_square: Option<u8>,
    pub halfmove_clock: u64,
    pub fullmove_number: u64,
    pub game_over: u32, // 0 = ongoing, 1 = white wins, 2 = black wins, 3 = draw
}

impl BoardState {
    pub fn legal_moves(&self, square: u8) -> Vec<u8> {
        let mut moves = Vec::new();
        let Some(king_square) = self
            .pieces
            .iter()
            .position(|p| matches!(p, Some(piece) if piece.kind == PieceKind::King && piece.white == self.white_to_play))
            else {return moves};
        let king_square = king_square as u8;
        if let Some(piece) = &self.pieces[square as usize] {
            let checked_squares = if self.white_to_play == piece.white {
                self.checked_squares()
            } else {
                Vec::new()
            };
            match piece.kind {
                PieceKind::Pawn => {
                    let direction: i8 = if piece.white { 1 } else { -1 };
                    let start_rank: u8 = if piece.white { 1 } else { 6 };
                    let rank = square / 8;
                    let file = square % 8;

                    let forward_square = (rank as i8 + direction) * 8 + file as i8;
                    if (0..64).contains(&forward_square)
                        && self.pieces[forward_square as usize].is_none()
                    {
                        moves.push(forward_square as u8);

                        if rank == start_rank {
                            let double_forward_square =
                                (rank as i8 + 2 * direction) * 8 + file as i8;
                            if (0..64).contains(&double_forward_square)
                                && self.pieces[double_forward_square as usize].is_none()
                            {
                                moves.push(double_forward_square as u8);
                            }
                        }
                    }

                    for &df in &[-1, 1] {
                        let capture_file = file as i8 + df;
                        let capture_rank = rank as i8 + direction;
                        if (0..8).contains(&capture_file) && (0..8).contains(&capture_rank) {
                            let capture_square = capture_rank * 8 + capture_file;
                            if let Some(target_piece) = &self.pieces[capture_square as usize] {
                                if target_piece.white != piece.white {
                                    moves.push(capture_square as u8);
                                }
                            } else if Some(capture_square as u8) == self.en_passant_square {
                                moves.push(capture_square as u8);
                            }
                        }
                    }
                }
                PieceKind::Bishop => {
                    for &(dr, df) in &[(-1, -1), (-1, 1), (1, -1), (1, 1)] {
                        let mut rank = (square / 8) as i8;
                        let mut file = (square % 8) as i8;
                        loop {
                            rank += dr;
                            file += df;
                            if !(0..8).contains(&rank) || !(0..8).contains(&file) {
                                break;
                            }
                            let target_square = rank * 8 + file;
                            if let Some(target_piece) = &self.pieces[target_square as usize] {
                                if target_piece.white != piece.white {
                                    moves.push(target_square as u8);
                                }
                                break;
                            } else {
                                moves.push(target_square as u8);
                            }
                        }
                    }
                }
                PieceKind::Rook => {
                    for &(dr, df) in &[(-1, 0), (1, 0), (0, -1), (0, 1)] {
                        let mut rank = (square / 8) as i8;
                        let mut file = (square % 8) as i8;
                        loop {
                            rank += dr;
                            file += df;
                            if !(0..8).contains(&rank) || !(0..8).contains(&file) {
                                break;
                            }
                            let target_square = rank * 8 + file;
                            if let Some(target_piece) = &self.pieces[target_square as usize] {
                                if target_piece.white != piece.white {
                                    moves.push(target_square as u8);
                                }
                                break;
                            } else {
                                moves.push(target_square as u8);
                            }
                        }
                    }
                }
                PieceKind::Queen => {
                    for &(dr, df) in &[
                        (-1, -1),
                        (-1, 1),
                        (1, -1),
                        (1, 1),
                        (-1, 0),
                        (1, 0),
                        (0, -1),
                        (0, 1),
                    ] {
                        let mut rank = (square / 8) as i8;
                        let mut file = (square % 8) as i8;
                        loop {
                            rank += dr;
                            file += df;
                            if !(0..8).contains(&rank) || !(0..8).contains(&file) {
                                break;
                            }
                            let target_square = rank * 8 + file;
                            if let Some(target_piece) = &self.pieces[target_square as usize] {
                                if target_piece.white != piece.white {
                                    moves.push(target_square as u8);
                                }
                                break;
                            } else {
                                moves.push(target_square as u8);
                            }
                        }
                    }
                }
                PieceKind::Knight => {
                    let rank = (square / 8) as i8;
                    let file = (square % 8) as i8;
                    for &(dr, df) in &[
                        (-2, -1),
                        (-2, 1),
                        (-1, -2),
                        (-1, 2),
                        (1, -2),
                        (1, 2),
                        (2, -1),
                        (2, 1),
                    ] {
                        let new_rank = rank + dr;
                        let new_file = file + df;
                        if (0..8).contains(&new_rank) && (0..8).contains(&new_file) {
                            let target_square = new_rank * 8 + new_file;
                            if let Some(target_piece) = &self.pieces[target_square as usize] {
                                if target_piece.white != piece.white {
                                    moves.push(target_square as u8);
                                }
                            } else {
                                moves.push(target_square as u8);
                            }
                        }
                    }
                }
                PieceKind::King => {
                    let rank = (square / 8) as i8;
                    let file = (square % 8) as i8;
                    for &(dr, df) in &[
                        (-1, -1),
                        (-1, 0),
                        (-1, 1),
                        (0, -1),
                        (0, 1),
                        (1, -1),
                        (1, 0),
                        (1, 1),
                    ] {
                        let new_rank = rank + dr;
                        let new_file = file + df;
                        if (0..8).contains(&new_rank) && (0..8).contains(&new_file) {
                            let target_square = new_rank * 8 + new_file;

                            if self.white_to_play == piece.white
                                && checked_squares.contains(&(target_square as u8))
                            {
                                continue;
                            }
                            if let Some(target_piece) = &self.pieces[target_square as usize] {
                                if target_piece.white != piece.white {
                                    moves.push(target_square as u8);
                                }
                            } else {
                                moves.push(target_square as u8);
                            }
                        }
                    }
                    if !checked_squares.contains(&square) {
                        if piece.white {
                            if self.white_can_ooo
                                && self.pieces[1].is_none()
                                && self.pieces[2].is_none()
                                && self.pieces[3].is_none()
                                && !checked_squares.contains(&2)
                                && !checked_squares.contains(&3)
                                && !moves.contains(&2)
                            {
                                moves.push(2);
                            }
                            if self.white_can_oo
                                && self.pieces[5].is_none()
                                && self.pieces[6].is_none()
                                && !checked_squares.contains(&5)
                                && !checked_squares.contains(&6)
                                && !moves.contains(&6)
                            {
                                moves.push(6);
                            }
                        } else {
                            if self.black_can_ooo
                                && self.pieces[57].is_none()
                                && self.pieces[58].is_none()
                                && self.pieces[59].is_none()
                                && !checked_squares.contains(&58)
                                && !checked_squares.contains(&59)
                                && !moves.contains(&58)
                            {
                                moves.push(58);
                            }
                            if self.black_can_oo
                                && self.pieces[61].is_none()
                                && self.pieces[62].is_none()
                                && !checked_squares.contains(&61)
                                && !checked_squares.contains(&62)
                                && !moves.contains(&62)
                            {
                                moves.push(62);
                            }
                        }
                    }
                }
            }

            if piece.kind != PieceKind::King
                && piece.white == self.white_to_play
                && checked_squares.contains(&king_square)
            {
                let mut new_moves = Vec::new();
                for &mv in moves.iter() {
                    let mut temp_board = self.clone();
                    temp_board.make_move(square, mv);
                    temp_board.white_to_play = !temp_board.white_to_play;
                    if !temp_board.checked_squares().contains(&king_square)
                        && !new_moves.contains(&mv)
                    {
                        new_moves.push(mv);
                    }
                }
                return new_moves;
            }
        }
        moves
    }

    pub fn make_move(&mut self, from: u8, to: u8) {
        let piece = self.pieces[from as usize].take();
        if let Some(pc) = &piece {
            match pc.kind {
                PieceKind::Pawn => {
                    if let Some(ep_square) = self.en_passant_square
                        && to == ep_square
                    {
                        let capture_square = if pc.white { to - 8 } else { to + 8 };
                        self.pieces[capture_square as usize] = None;
                    }
                    self.en_passant_square = None;
                    if (pc.white && from / 8 == 1 && to / 8 == 3)
                        || (!pc.white && from / 8 == 6 && to / 8 == 4)
                    {
                        self.en_passant_square = Some((from + to) / 2);
                    }
                    if (to / 8 == 7 && pc.white) || (to / 8 == 0 && !pc.white) {
                        self.pieces[to as usize] = Some(Piece {
                            kind: PieceKind::Queen, // TODO  make selectable
                            white: pc.white,
                        });
                        self.white_to_play = !self.white_to_play;
                    } else {
                        self.pieces[to as usize] = piece.clone();
                    }
                }
                PieceKind::King => {
                    if pc.white {
                        self.white_can_oo = false;
                        self.white_can_ooo = false;
                        // move rook
                        if from == 4 && to == 2 {
                            self.pieces[3] = self.pieces[0].take();
                        } else if from == 4 && to == 6 {
                            self.pieces[5] = self.pieces[7].take();
                        }
                    } else {
                        self.black_can_oo = false;
                        self.black_can_ooo = false;
                        // move rook
                        if from == 60 && to == 58 {
                            self.pieces[59] = self.pieces[56].take();
                        } else if from == 60 && to == 62 {
                            self.pieces[61] = self.pieces[63].take();
                        }
                    }
                }
                PieceKind::Rook => {
                    if pc.white {
                        if from == 0 {
                            self.white_can_ooo = false;
                        } else if from == 7 {
                            self.white_can_oo = false;
                        }
                    } else if from == 56 {
                        self.black_can_ooo = false;
                    } else if from == 63 {
                        self.black_can_oo = false;
                    }
                }
                _ => {}
            }

            if !matches!(pc.kind, PieceKind::Pawn) {
                self.en_passant_square = None;
                self.pieces[to as usize] = piece;
            }
        }
        self.white_to_play = !self.white_to_play;
        self.game_over = self.is_game_over();
        // TODO make game over menu
        match self.game_over {
            1 => {
                println!(
                    "checkmate {} wins",
                    if !self.white_to_play {
                        "white"
                    } else {
                        "black"
                    }
                );
            }
            2 => {
                println!("stalemate");
            }
            3 => {
                println!("draw?");
            }
            _ => {}
        }
    }

    pub fn checked_squares(&self) -> Vec<u8> {
        let mut squares = Vec::new();
        for (i, square) in self.pieces.iter().enumerate() {
            if let Some(piece) = square
                && piece.white != self.white_to_play
            {
                for mv in self.legal_moves(i as u8) {
                    if piece.kind == PieceKind::Pawn {
                        if piece.white {
                            if i as u8 + 8 == mv || i as u8 + 16 == mv {
                                continue;
                            }
                        } else if i as u8 - 8 == mv || i as u8 - 16 == mv {
                            continue;
                        }
                    }
                    if !squares.contains(&mv) {
                        squares.push(mv);
                    }
                }
            }
        }
        squares
    }

    pub fn is_game_over(&self) -> u32 {
        let Some(king_square) = self
            .pieces
            .iter()
            .position(|p| matches!(p, Some(piece) if piece.kind == PieceKind::King && piece.white == self.white_to_play))
            else {
                println!("{:?}", self.pieces);
                return 3
            };
        for (i, square) in self.pieces.iter().enumerate() {
            if let Some(piece) = square
                && piece.white == self.white_to_play
                && !self.legal_moves(i as u8).is_empty()
            {
                return 0;
            }
        }

        if self.checked_squares().contains(&(king_square as u8)) {
            1
        } else {
            2
        }
    }
}
