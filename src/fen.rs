use crate::logic::{BoardState, Piece};

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
            game_over: 0,
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

    fn parse_en_passant(en_passant: &str) -> anyhow::Result<Option<u32>> {
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

    fn parse_halfmove(halfmove: &str) -> anyhow::Result<u32> {
        halfmove
            .parse()
            .map_err(|_| anyhow::anyhow!("bad halfmove: {halfmove}"))
    }

    fn parse_fullmove(fullmove: &str) -> anyhow::Result<u32> {
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
}
