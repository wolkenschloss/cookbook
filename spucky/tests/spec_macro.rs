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

spec! {
    result {
        type Output = Result<(), String>;

        case four {
            let result = Ok(());
        }

        // case five {
        //     let result = Err("Fehler in Testfall five".to_string());
        // }

        result
    }
}

// Oder besser diese Syntax?
//
// spec! {
//     suite3() -> Result<(), String> {
//
//         case four {
//             let result = Ok(());
//         }
//
//         #[ignore]
//         case five {
//             let result = Err("Fehler in Testfall five".to_string());
//         }
//
//         #[should_panic]
//         case six {
//               panic!("Fehler in Testfall six");
//         }
//
//         result
//     }
// }
