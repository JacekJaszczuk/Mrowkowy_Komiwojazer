use std::collections::BTreeSet;
use std::collections::BinaryHeap;

/// Graf pozwala zapisać dane w sposób macierzowy
#[derive(Debug)]
struct Graf<T> {
    liczba_wezlow: usize,
    macierz: Vec<Vec<T>>,
}

#[derive(Debug)]
struct Mrowka<'a> {
    graf: &'a Graf<u32>,
    feromony: &'a mut Graf<f32>,
    nieodwiedzone_miasta: BTreeSet<u32>,
}

impl<'a> Mrowka<'a> {
    fn new(graf: &'a Graf<u32>, feromony: &'a mut Graf<f32>, miasto_startowe: u32) -> Mrowka<'a> {
        Mrowka {
            graf,
            feromony,
            nieodwiedzone_miasta: BTreeSet::new(),
        }
    }
}

fn algorytm_mrowkowy(graf: & Graf<u32>, liczba_iteracji: u32, waga_losowosci: f32, zostawiany_feromon: f32, ulatnianie_feromonu: f32) {
    // Wyznacz wagę feromonu:
    let waga_feromonu: f32 = 1.0 - waga_losowosci;

    // Twórz graf z feromonami:
    let mut feromony: Graf<f32> = Graf {
        liczba_wezlow: graf.liczba_wezlow,
        macierz: vec![vec![0.0; graf.liczba_wezlow]; graf.liczba_wezlow],
    };
    //println!("{:?}", feromony);

    // Twórz wektor z mrówkami, każda mrówka zaczyna z osobnego miasta:
    let mrowki: Vec<Mrowka> = Vec::new();

    println!("Rozsyłam mrówki po grafie.");

}

fn main() {
    let graf1: Graf<u32> = Graf {
        liczba_wezlow: 6,
        macierz: vec![
            vec![ 0,  5, 13, 16, 13,  5],
            vec![ 5,  0,  4, 13, 20, 16],
            vec![13,  4,  0,  5, 16, 20],
            vec![16, 13,  5,  0,  5, 13],
            vec![13, 20, 16,  5,  0,  4],
            vec![ 5, 16, 20, 13,  4,  0],
            ],
    };
    println!("Witaj w świecie mrówek!");

    algorytm_mrowkowy(&graf1, 20, 0.3, 0.15, 0.01);
}

/*
fn main2() {
    println!("Hello, world!");
    let x = Graf {
        liczba_wezlow: 65,
        macierz: vec![vec![23]],
    };
    referencja(&x);
    referencja(&x);
    niereferencja(x);
    //referencja(&x);
}

fn referencja(graf: & Graf) {
    println!("{}", graf.liczba_wezlow);
}

fn niereferencja(graf: Graf) {
    println!("{}", graf.liczba_wezlow);
}
*/
