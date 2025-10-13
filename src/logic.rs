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

#[derive(Debug, Eq, PartialEq)]
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
}

impl BoardState {
    pub fn from_fen(fen: &str) -> anyhow::Result<BoardState> {
        let parts = fen.split(' ').collect::<Vec<_>>();
        if parts.len() != 6 {
            return Err(anyhow::anyhow!("invalid FEN: {fen:?}"));
        }

        let (white_can_oo, white_can_ooo, black_can_oo, black_can_ooo) =
            Self::parse_castling(parts[2]);

        Ok(BoardState {
            pieces: Self::parse_placement(parts[0])?,
            white_to_play: Self::parse_side_to_play(parts[1])?,
            white_can_oo,
            white_can_ooo,
            black_can_oo,
            black_can_ooo,
            en_passant_square: Self::parse_en_passant(parts[3])?,
            halfmove_clock: Self::parse_halfmove(parts[4])?,
            fullmove_number: Self::parse_fullmove(parts[5])?,
        })
    }

    fn parse_placement(placement_str: &str) -> anyhow::Result<Vec<Option<Piece>>> {
        let mut placement = vec![None; 64];
        let lines = placement_str.split('/').collect::<Vec<_>>();

        if lines.len() != 8 {
            return Err(anyhow::anyhow!("bad placement: {placement_str}"));
        }

        for (rank, pieces) in lines.iter().rev().enumerate() {
            let mut file = 0;

            for piece_char in pieces.chars() {
                if let Some(n) = piece_char.to_digit(10) {
                    Self::increment_file(&mut file, n as usize, pieces)?;
                } else {
                    let piece = Piece::from_char(piece_char);
                    placement[rank * 8 + file] = Some(piece);
                    file += 1;
                }
            }
        }

        Ok(placement)
    }

    fn increment_file(file: &mut usize, n: usize, rank: &str) -> anyhow::Result<()> {
        *file += n;
        if *file > 8 {
            Err(anyhow::anyhow!("too many pieces in rank {rank}"))
        } else {
            Ok(())
        }
    }

    fn parse_side_to_play(side_to_play: &str) -> anyhow::Result<bool> {
        match side_to_play {
            "w" => Ok(true),
            "b" => Ok(false),
            _ => Err(anyhow::anyhow!("no such side: {side_to_play}")),
        }
    }

    fn parse_castling(castling: &str) -> (bool, bool, bool, bool) {
        let white_oo = castling.contains('K');
        let white_ooo = castling.contains('Q');
        let black_oo = castling.contains('k');
        let black_ooo = castling.contains('q');

        (white_oo, white_ooo, black_oo, black_ooo)
    }

    fn parse_en_passant(en_passant: &str) -> anyhow::Result<Option<u8>> {
        if en_passant == "-" {
            return Ok(None);
        }

        if en_passant.len() != 2 {
            return Err(anyhow::anyhow!("bad en passant: {en_passant}"));
        }

        let chars = en_passant.chars().collect::<Vec<_>>();
        let (file, rank) = (chars[0], chars[1]);

        let file = match file {
            'a' => 0,
            'b' => 1,
            'c' => 2,
            'd' => 3,
            'e' => 4,
            'f' => 5,
            'g' => 6,
            'h' => 7,
            _ => return Err(anyhow::anyhow!("bad en passant: {en_passant}")),
        };

        let rank = match rank {
            '1' => 0,
            '2' => 1,
            '3' => 2,
            '4' => 3,
            '5' => 4,
            '6' => 5,
            '7' => 6,
            '8' => 7,
            _ => return Err(anyhow::anyhow!("bad en passant: {en_passant}")),
        };

        Ok(Some(file + rank * 8))
    }

    fn parse_halfmove(halfmove: &str) -> anyhow::Result<u64> {
        halfmove
            .parse()
            .map_err(|_| anyhow::anyhow!("bad halfmove: {halfmove}"))
    }

    fn parse_fullmove(fullmove: &str) -> anyhow::Result<u64> {
        fullmove
            .parse()
            .map_err(|_| anyhow::anyhow!("bad fullmove: {fullmove}"))
    }

    pub fn to_fen(&self) -> String {
        let placement = self.make_placement();
        let side_to_play = self.make_side_to_play();
        let castling = self.make_castling();
        let en_passant = self.make_en_passant();
        let halfmove = self.make_halfmove();
        let fullmove = self.make_fullmove();

        let parts = [
            placement,
            side_to_play,
            castling,
            en_passant,
            halfmove,
            fullmove,
        ];
        parts.join(" ")
    }

    fn make_placement(&self) -> String {
        let mut placement = String::new();

        for rank in (0..8).rev() {
            let mut blanks = 0;

            for file in 0..8 {
                match self.pieces[rank * 8 + file] {
                    Some(ref piece) => {
                        if blanks != 0 {
                            placement.push_str(&blanks.to_string());
                            blanks = 0;
                        }

                        placement.push(piece.to_char());
                    }

                    None => blanks += 1,
                }
            }

            if blanks != 0 {
                placement.push_str(&blanks.to_string());
            }

            if rank != 0 {
                placement.push('/');
            }
        }

        placement
    }

    fn make_side_to_play(&self) -> String {
        if self.white_to_play { "w" } else { "b" }.to_owned()
    }

    fn make_castling(&self) -> String {
        let mut castling = String::new();
        if self.white_can_oo {
            castling.push('K')
        }
        if self.white_can_ooo {
            castling.push('Q')
        }
        if self.black_can_oo {
            castling.push('k')
        }
        if self.black_can_ooo {
            castling.push('q')
        }

        if !castling.is_empty() {
            castling
        } else {
            "-".to_owned()
        }
    }

    fn make_en_passant(&self) -> String {
        match self.en_passant_square {
            Some(en_passant) => {
                let file = match en_passant % 8 {
                    0 => 'a',
                    1 => 'b',
                    2 => 'c',
                    3 => 'd',
                    4 => 'e',
                    5 => 'f',
                    6 => 'g',
                    7 => 'h',
                    _ => unreachable!(),
                };

                let rank = match en_passant / 8 {
                    0 => '1',
                    1 => '2',
                    2 => '3',
                    3 => '4',
                    4 => '5',
                    5 => '6',
                    6 => '7',
                    7 => '8',
                    _ => unreachable!(),
                };

                format!("{}{}", file, rank)
            }

            None => "-".to_owned(),
        }
    }

    fn make_halfmove(&self) -> String {
        self.halfmove_clock.to_string()
    }

    fn make_fullmove(&self) -> String {
        self.fullmove_number.to_string()
    }

    pub fn legal_moves(&self, square: u8) -> Vec<u8> {
        let mut moves = Vec::new();
        let king_square = self
            .pieces
            .iter()
            .position(|p| matches!(p, Some(piece) if piece.kind == PieceKind::King && piece.white == self.white_to_play))
            .unwrap() as u8;
        if let Some(piece) = &self.pieces[square as usize] {
            let checked_squares = if self.white_to_play == piece.white {
                self.checked_squares()
            } else {
                Vec::new()
            };
            if piece.kind != PieceKind::King && checked_squares.contains(&king_square) {
                return moves;
            }
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
                }
            }
        }
        moves
    }

    pub fn make_move(&mut self, from: u8, to: u8) {
        let piece = self.pieces[from as usize].take();
        if let Some(piece) = &piece {
            match piece.kind {
                PieceKind::Pawn => {
                    if let Some(ep_square) = self.en_passant_square
                        && to == ep_square
                    {
                        let capture_square = if piece.white { to - 8 } else { to + 8 };
                        self.pieces[capture_square as usize] = None;
                    }
                    self.en_passant_square = None;
                    if (piece.white && from / 8 == 1 && to / 8 == 3)
                        || (!piece.white && from / 8 == 6 && to / 8 == 4)
                    {
                        self.en_passant_square = Some((from + to) / 2);
                    }
                }
                PieceKind::King => {
                    if piece.white {
                        self.white_can_oo = false;
                        self.white_can_ooo = false;
                    } else {
                        self.black_can_oo = false;
                        self.black_can_ooo = false;
                    }
                }
                PieceKind::Rook => {
                    if piece.white {
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
        }
        self.pieces[to as usize] = piece;
        self.white_to_play = !self.white_to_play;
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
}
