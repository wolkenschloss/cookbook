use spucky::*;

spec! {
    example {
        case a {
            let a = 1;
            let want = 1;
        }

        case b {
            let a = 2;
            let want = 4;
        }

        case c {
            let a = 3;
            let want = 9;
        }

        let got = a * a;
        assert_eq!(got, want)
    }
}