#[cfg(test)]
mod kat_tests {
    use heap_unranking::*;

    #[test]
    fn rank_kats() {
        let s = precompute(20);
        assert_eq!(0, rank(&s, [0, 1, 2].into()));
        assert_eq!(5, rank(&s, [2, 1, 0].into()));
        assert_eq!(20, rank(&s, [1, 3, 2, 0].into()));
        assert_eq!(9, rank(&s, [3, 0, 1, 2, 4].into()));
        assert_eq!(
            39916798,
            rank(&s, [1, 10, 2, 3, 4, 5, 6, 7, 8, 9, 0].into())
        );
        assert_eq!(
            154423787521,
            rank(
                &s,
                [7, 2, 10, 14, 12, 13, 5, 0, 6, 3, 1, 4, 9, 8, 11].into()
            )
        );
    }

    #[test]
    fn unrank_kats() {
        let s = precompute(14);
        assert_eq!(unrank(&s, 1, 0), [0].into());
        assert_eq!(unrank(&s, 2, 0), [0, 1].into());
        assert_eq!(unrank(&s, 2, 1), [1, 0].into());
        assert_eq!(unrank(&s, 3, 0), [0, 1, 2].into());
        assert_eq!(unrank(&s, 3, 1), [1, 0, 2].into());
        assert_eq!(unrank(&s, 3, 2), [2, 0, 1].into());
        assert_eq!(unrank(&s, 3, 3), [0, 2, 1].into());
        assert_eq!(unrank(&s, 3, 4), [1, 2, 0].into());
        assert_eq!(unrank(&s, 3, 5), [2, 1, 0].into());
        assert_eq!(unrank(&s, 4, 0), [0, 1, 2, 3].into());
        assert_eq!(unrank(&s, 4, 1), [1, 0, 2, 3].into());
        assert_eq!(unrank(&s, 4, 2), [2, 0, 1, 3].into());
        assert_eq!(unrank(&s, 4, 3), [0, 2, 1, 3].into());
        assert_eq!(unrank(&s, 4, 4), [1, 2, 0, 3].into());
        assert_eq!(unrank(&s, 4, 5), [2, 1, 0, 3].into());
        assert_eq!(unrank(&s, 4, 6), [3, 1, 0, 2].into());
        assert_eq!(unrank(&s, 4, 7), [1, 3, 0, 2].into());
        assert_eq!(unrank(&s, 4, 8), [0, 3, 1, 2].into());
        assert_eq!(unrank(&s, 4, 9), [3, 0, 1, 2].into());
        assert_eq!(unrank(&s, 4, 10), [1, 0, 3, 2].into());
        assert_eq!(unrank(&s, 4, 11), [0, 1, 3, 2].into());
        assert_eq!(unrank(&s, 4, 12), [0, 2, 3, 1].into());
        assert_eq!(unrank(&s, 4, 13), [2, 0, 3, 1].into());
        assert_eq!(unrank(&s, 4, 14), [3, 0, 2, 1].into());
        assert_eq!(unrank(&s, 4, 15), [0, 3, 2, 1].into());
        assert_eq!(unrank(&s, 4, 16), [2, 3, 0, 1].into());
        assert_eq!(unrank(&s, 4, 17), [3, 2, 0, 1].into());
        assert_eq!(unrank(&s, 4, 18), [3, 2, 1, 0].into());
        assert_eq!(unrank(&s, 4, 19), [2, 3, 1, 0].into());
        assert_eq!(unrank(&s, 4, 20), [1, 3, 2, 0].into());
        assert_eq!(unrank(&s, 4, 21), [3, 1, 2, 0].into());
        assert_eq!(unrank(&s, 4, 22), [2, 1, 3, 0].into());
        assert_eq!(unrank(&s, 4, 23), [1, 2, 3, 0].into());
        assert_eq!(unrank(&s, 5, 0), [0, 1, 2, 3, 4].into());
        assert_eq!(unrank(&s, 5, 1), [1, 0, 2, 3, 4].into());
        assert_eq!(unrank(&s, 5, 2), [2, 0, 1, 3, 4].into());
        assert_eq!(unrank(&s, 5, 3), [0, 2, 1, 3, 4].into());
        assert_eq!(unrank(&s, 5, 4), [1, 2, 0, 3, 4].into());
        assert_eq!(unrank(&s, 5, 5), [2, 1, 0, 3, 4].into());
        assert_eq!(unrank(&s, 5, 6), [3, 1, 0, 2, 4].into());
        assert_eq!(unrank(&s, 5, 7), [1, 3, 0, 2, 4].into());
        assert_eq!(unrank(&s, 5, 8), [0, 3, 1, 2, 4].into());
        assert_eq!(unrank(&s, 5, 9), [3, 0, 1, 2, 4].into());
        assert_eq!(unrank(&s, 5, 10), [1, 0, 3, 2, 4].into());
        assert_eq!(unrank(&s, 5, 11), [0, 1, 3, 2, 4].into());
        assert_eq!(unrank(&s, 5, 12), [0, 2, 3, 1, 4].into());
        assert_eq!(unrank(&s, 5, 13), [2, 0, 3, 1, 4].into());
        assert_eq!(unrank(&s, 5, 14), [3, 0, 2, 1, 4].into());
        assert_eq!(unrank(&s, 5, 15), [0, 3, 2, 1, 4].into());
        assert_eq!(unrank(&s, 5, 16), [2, 3, 0, 1, 4].into());
        assert_eq!(unrank(&s, 5, 17), [3, 2, 0, 1, 4].into());
        assert_eq!(unrank(&s, 5, 18), [3, 2, 1, 0, 4].into());
        assert_eq!(unrank(&s, 5, 19), [2, 3, 1, 0, 4].into());
        assert_eq!(unrank(&s, 5, 20), [1, 3, 2, 0, 4].into());
        assert_eq!(unrank(&s, 5, 21), [3, 1, 2, 0, 4].into());
        assert_eq!(unrank(&s, 5, 22), [2, 1, 3, 0, 4].into());
        assert_eq!(unrank(&s, 5, 23), [1, 2, 3, 0, 4].into());
        assert_eq!(unrank(&s, 5, 24), [4, 2, 3, 0, 1].into());
        assert_eq!(unrank(&s, 5, 25), [2, 4, 3, 0, 1].into());
        assert_eq!(unrank(&s, 5, 26), [3, 4, 2, 0, 1].into());
        assert_eq!(unrank(&s, 5, 27), [4, 3, 2, 0, 1].into());
        assert_eq!(unrank(&s, 5, 28), [2, 3, 4, 0, 1].into());
        assert_eq!(unrank(&s, 5, 29), [3, 2, 4, 0, 1].into());
        assert_eq!(unrank(&s, 6, 0), [0, 1, 2, 3, 4, 5].into());
        assert_eq!(unrank(&s, 6, 1), [1, 0, 2, 3, 4, 5].into());
        assert_eq!(unrank(&s, 6, 2), [2, 0, 1, 3, 4, 5].into());
        assert_eq!(unrank(&s, 6, 3), [0, 2, 1, 3, 4, 5].into());
        assert_eq!(unrank(&s, 6, 4), [1, 2, 0, 3, 4, 5].into());
        assert_eq!(unrank(&s, 6, 5), [2, 1, 0, 3, 4, 5].into());
        assert_eq!(unrank(&s, 6, 6), [3, 1, 0, 2, 4, 5].into());
        assert_eq!(unrank(&s, 6, 7), [1, 3, 0, 2, 4, 5].into());
        assert_eq!(unrank(&s, 6, 8), [0, 3, 1, 2, 4, 5].into());
        assert_eq!(unrank(&s, 6, 9), [3, 0, 1, 2, 4, 5].into());
        assert_eq!(unrank(&s, 6, 10), [1, 0, 3, 2, 4, 5].into());
        assert_eq!(unrank(&s, 6, 11), [0, 1, 3, 2, 4, 5].into());
        assert_eq!(unrank(&s, 6, 12), [0, 2, 3, 1, 4, 5].into());
        assert_eq!(unrank(&s, 6, 13), [2, 0, 3, 1, 4, 5].into());
        assert_eq!(unrank(&s, 6, 14), [3, 0, 2, 1, 4, 5].into());
        assert_eq!(unrank(&s, 6, 15), [0, 3, 2, 1, 4, 5].into());
        assert_eq!(unrank(&s, 6, 16), [2, 3, 0, 1, 4, 5].into());
        assert_eq!(unrank(&s, 6, 17), [3, 2, 0, 1, 4, 5].into());
        assert_eq!(unrank(&s, 6, 18), [3, 2, 1, 0, 4, 5].into());
        assert_eq!(unrank(&s, 6, 19), [2, 3, 1, 0, 4, 5].into());
        assert_eq!(unrank(&s, 6, 20), [1, 3, 2, 0, 4, 5].into());
        assert_eq!(unrank(&s, 6, 21), [3, 1, 2, 0, 4, 5].into());
        assert_eq!(unrank(&s, 6, 22), [2, 1, 3, 0, 4, 5].into());
        assert_eq!(unrank(&s, 6, 23), [1, 2, 3, 0, 4, 5].into());
        assert_eq!(unrank(&s, 6, 24), [4, 2, 3, 0, 1, 5].into());
        assert_eq!(unrank(&s, 6, 25), [2, 4, 3, 0, 1, 5].into());
        assert_eq!(unrank(&s, 6, 26), [3, 4, 2, 0, 1, 5].into());
        assert_eq!(unrank(&s, 6, 27), [4, 3, 2, 0, 1, 5].into());
        assert_eq!(unrank(&s, 6, 28), [2, 3, 4, 0, 1, 5].into());
        assert_eq!(unrank(&s, 6, 29), [3, 2, 4, 0, 1, 5].into());
        assert_eq!(unrank(&s, 7, 0), [0, 1, 2, 3, 4, 5, 6].into());
        assert_eq!(unrank(&s, 7, 1), [1, 0, 2, 3, 4, 5, 6].into());
        assert_eq!(unrank(&s, 7, 2), [2, 0, 1, 3, 4, 5, 6].into());
        assert_eq!(unrank(&s, 7, 3), [0, 2, 1, 3, 4, 5, 6].into());
        assert_eq!(unrank(&s, 7, 4), [1, 2, 0, 3, 4, 5, 6].into());
        assert_eq!(unrank(&s, 7, 5), [2, 1, 0, 3, 4, 5, 6].into());
        assert_eq!(unrank(&s, 7, 6), [3, 1, 0, 2, 4, 5, 6].into());
        assert_eq!(unrank(&s, 7, 7), [1, 3, 0, 2, 4, 5, 6].into());
        assert_eq!(unrank(&s, 7, 8), [0, 3, 1, 2, 4, 5, 6].into());
        assert_eq!(unrank(&s, 7, 9), [3, 0, 1, 2, 4, 5, 6].into());
        assert_eq!(unrank(&s, 7, 10), [1, 0, 3, 2, 4, 5, 6].into());
        assert_eq!(unrank(&s, 7, 11), [0, 1, 3, 2, 4, 5, 6].into());
        assert_eq!(unrank(&s, 7, 12), [0, 2, 3, 1, 4, 5, 6].into());
        assert_eq!(unrank(&s, 7, 5038), [1, 6, 2, 3, 4, 5, 0].into());
        assert_eq!(unrank(&s, 7, 5039), [6, 1, 2, 3, 4, 5, 0].into());
        assert_eq!(unrank(&s, 8, 40318), [6, 5, 1, 2, 3, 4, 7, 0].into());
        assert_eq!(unrank(&s, 8, 40319), [5, 6, 1, 2, 3, 4, 7, 0].into());
        assert_eq!(unrank(&s, 9, 1), [1, 0, 2, 3, 4, 5, 6, 7, 8].into());
        assert_eq!(unrank(&s, 9, 362878), [1, 8, 2, 3, 4, 5, 6, 7, 0].into());
        assert_eq!(unrank(&s, 9, 362879), [8, 1, 2, 3, 4, 5, 6, 7, 0].into());
        // 10
        assert_eq!(unrank(&s, 10, 1), [1, 0, 2, 3, 4, 5, 6, 7, 8, 9].into());
        assert_eq!(
            unrank(&s, 10, 3628798),
            [8, 7, 1, 2, 3, 4, 5, 6, 9, 0].into()
        );
        assert_eq!(
            unrank(&s, 10, 1048577),
            [4, 3, 7, 5, 9, 2, 8, 6, 0, 1].into()
        );
        assert_eq!(
            unrank(&s, 10, 2097153),
            [6, 9, 2, 3, 0, 5, 1, 8, 7, 4].into()
        );
        assert_eq!(
            unrank(&s, 10, 3145729),
            [4, 0, 2, 6, 3, 8, 9, 1, 5, 7].into()
        );
        assert_eq!(
            unrank(&s, 10, 3628799),
            [7, 8, 1, 2, 3, 4, 5, 6, 9, 0].into()
        );
        assert_eq!(
            unrank(&s, 10, 3628799),
            [7, 8, 1, 2, 3, 4, 5, 6, 9, 0].into()
        );
        // 11
        assert_eq!(
            unrank(&s, 11, 39916798),
            [1, 10, 2, 3, 4, 5, 6, 7, 8, 9, 0].into()
        );
        assert_eq!(
            unrank(&s, 11, 39916799),
            [10, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0].into()
        );
        assert_eq!(
            unrank(&s, 11, 39916799),
            [10, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0].into()
        );
        // 12
        assert_eq!(
            unrank(&s, 12, 479001598),
            [10, 9, 1, 2, 3, 4, 5, 6, 7, 8, 11, 0].into()
        );
        assert_eq!(
            unrank(&s, 12, 479001599),
            [9, 10, 1, 2, 3, 4, 5, 6, 7, 8, 11, 0].into()
        );
        assert_eq!(
            unrank(&s, 12, 100663297),
            [4, 9, 5, 2, 7, 6, 0, 8, 11, 10, 3, 1].into()
        );
        assert_eq!(
            unrank(&s, 12, 52428801),
            [1, 9, 6, 4, 0, 3, 2, 7, 11, 8, 5, 10].into()
        );
        assert_eq!(
            unrank(&s, 12, 146800641),
            [8, 11, 0, 9, 3, 1, 4, 6, 5, 7, 10, 2].into()
        );
        assert_eq!(
            unrank(&s, 12, 477102081),
            [7, 2, 1, 8, 4, 5, 9, 10, 6, 3, 11, 0].into()
        );
        // 13
        assert_eq!(
            unrank(&s, 13, 319815681),
            [2, 1, 6, 5, 10, 0, 9, 4, 3, 8, 11, 7, 12].into()
        );
        assert_eq!(
            unrank(&s, 13, 634388481),
            [10, 7, 2, 6, 11, 12, 3, 4, 5, 0, 8, 1, 9].into()
        );
        assert_eq!(
            unrank(&s, 13, 1347420161),
            [11, 12, 10, 3, 4, 0, 1, 7, 9, 2, 5, 6, 8].into()
        );
        assert_eq!(
            unrank(&s, 13, 2107637761),
            [1, 9, 8, 0, 2, 10, 7, 3, 12, 5, 4, 11, 6].into()
        );
        assert_eq!(
            unrank(&s, 13, 2144337921),
            [8, 11, 2, 5, 3, 1, 0, 4, 7, 12, 9, 10, 6].into()
        );
        assert_eq!(
            unrank(&s, 13, 3523215361),
            [1, 8, 12, 5, 2, 7, 6, 4, 0, 10, 11, 9, 3].into()
        );
        assert_eq!(
            unrank(&s, 13, 5022679041),
            [9, 8, 6, 3, 2, 0, 1, 5, 12, 4, 11, 7, 10].into()
        );
        assert_eq!(
            unrank(&s, 13, 6223298561),
            [6, 10, 5, 12, 3, 2, 4, 1, 7, 8, 9, 11, 0].into()
        );
        // 14
        assert_eq!(
            unrank(&s, 14, 3344957441),
            [8, 10, 3, 11, 12, 7, 9, 0, 2, 6, 1, 5, 4, 13].into()
        );
        assert_eq!(
            unrank(&s, 14, 4902092801),
            [9, 12, 8, 3, 6, 2, 7, 5, 1, 11, 0, 4, 10, 13].into()
        );
        assert_eq!(
            unrank(&s, 14, 7932477441),
            [11, 5, 1, 8, 3, 4, 9, 10, 6, 13, 0, 2, 7, 12].into()
        );
        assert_eq!(
            unrank(&s, 14, 16420700161),
            [8, 3, 5, 10, 0, 12, 6, 4, 13, 11, 9, 7, 2, 1].into()
        );
        assert_eq!(
            unrank(&s, 14, 16583229441),
            [13, 10, 12, 5, 11, 6, 3, 4, 9, 7, 8, 0, 2, 1].into()
        );
        // 15
        assert_eq!(
            unrank(&s, 15, 2233466881),
            [5, 9, 12, 0, 3, 4, 10, 1, 7, 11, 8, 2, 6, 13, 14].into()
        );
        assert_eq!(
            unrank(&s, 15, 11141120001),
            [6, 9, 13, 3, 8, 1, 7, 4, 2, 11, 0, 5, 10, 12, 14].into()
        );
        assert_eq!(
            unrank(&s, 15, 14905507841),
            [13, 12, 6, 10, 4, 0, 7, 9, 2, 3, 11, 8, 5, 1, 14].into()
        );
        assert_eq!(
            unrank(&s, 15, 24363663361),
            [4, 10, 0, 9, 12, 8, 1, 5, 3, 7, 6, 13, 11, 2, 14].into()
        );
        assert_eq!(
            unrank(&s, 15, 31111249921),
            [13, 6, 2, 7, 10, 5, 12, 1, 8, 9, 4, 11, 0, 3, 14].into()
        );
        assert_eq!(
            unrank(&s, 15, 44281364481),
            [11, 4, 8, 1, 5, 10, 13, 7, 0, 12, 3, 2, 9, 6, 14].into()
        );
        assert_eq!(
            unrank(&s, 15, 55343841281),
            [5, 12, 4, 13, 0, 10, 1, 8, 2, 9, 3, 6, 11, 7, 14].into()
        );
        assert_eq!(
            unrank(&s, 15, 66091745281),
            [5, 8, 6, 11, 13, 0, 4, 12, 1, 7, 10, 3, 2, 9, 14].into()
        );
        assert_eq!(
            unrank(&s, 15, 80625008641),
            [9, 5, 10, 1, 6, 7, 4, 3, 8, 12, 13, 2, 0, 11, 14].into()
        );
        assert_eq!(
            unrank(&s, 15, 89474990081),
            [9, 12, 8, 6, 7, 2, 1, 4, 13, 10, 14, 3, 5, 0, 11].into()
        );
        assert_eq!(
            unrank(&s, 15, 140005867521),
            [1, 12, 5, 10, 9, 3, 4, 7, 13, 0, 14, 8, 2, 6, 11].into()
        );
        assert_eq!(
            unrank(&s, 15, 154423787521),
            [7, 2, 10, 14, 12, 13, 5, 0, 6, 3, 1, 4, 9, 8, 11].into()
        );
    }
}
