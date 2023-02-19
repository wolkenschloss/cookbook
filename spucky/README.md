# Spucky Testhelfer

Spucky ist eine Sammlung mit Helfern zum Testen von Rust Code. Es ist kein Test Framework.

## Beispiel für spec! Makro

```rust

fn square(a: i32) -> i32 {
    a * a
}

#[cfg(test)] 
mod test {
    use spucky::spec;
    
    spec! {
        square_tests {
            case test1 {
                let a = 1;
                let want = 1;
            }
            
            case test4 {
                let a = 4;
                let want = 16;
            }
            
            let got = square(a);
            assert_eq!(want, get)
        }
    }
}
```


## Bekannte Probleme

* Generierte Test können in der IDE nicht angeklickt werden
* Attribute `#[ignore]` und `#[should_panic]` werden nicht unterstützt.
