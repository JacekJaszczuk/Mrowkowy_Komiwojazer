use std::rc::Rc;
use std::cell::RefCell;
use std::collections::BTreeSet;
use std::fs::File;
use rand::Rng;
use clap::{Arg, App, SubCommand};

#[macro_use]
extern crate serde_derive;

extern crate serde;
extern crate serde_json;

// Testy:
#[cfg(test)]
mod test {
    #[test]
    fn test_straznik_dobry() {
        let graf = super::Graf {
            liczba_wezlow: 3,
            macierz: vec![vec![1]],
        };
        let miasta = vec![0, 2, 1];
        super::straznik(& graf, & miasta);
    }

    #[test]
    #[should_panic]
        fn test_straznik_rozna_liczba_miast() {
        let graf = super::Graf {
            liczba_wezlow: 7,
            macierz: vec![vec![1]],
        };
        let miasta = vec![0, 2, 3, 1];
        super::straznik(& graf, & miasta);
    }

    #[test]
    #[should_panic]
        fn test_straznik_powtarzajace_sie_miasta() {
        let graf = super::Graf {
            liczba_wezlow: 4,
            macierz: vec![vec![1]],
        };
        let miasta = vec![0, 3, 3, 1];
        super::straznik(& graf, & miasta);
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Graf<T> {
    liczba_wezlow: usize,
    macierz: Vec<Vec<T>>,
}

impl<T> Graf<T> {
    fn losowy(size: usize, min: usize, max: usize) -> Graf<usize> {
        // Inicjalizuj generator liczb pseudolosowych:
        let mut dice = rand::thread_rng();

        // Twórz pola:
        let liczba_wezlow = size;
        let mut macierz = vec![vec![0; liczba_wezlow]; liczba_wezlow];

        // Generuj losowe wartości w macierzy:
        for x in &mut macierz {
            for x in x {
                *x = dice.gen_range(min, max);
            }
        }

        // Zwróć graf:
        Graf {
            liczba_wezlow,
            macierz,
        }
    }
}

#[derive(Debug)]
struct Mrowka<'a> {
    graf: &'a Graf<usize>,
    feromony: Rc<RefCell<Graf<f32>>>,
    odwiedzone_miasta: Vec<usize>,
    nieodwiedzone_miasta: Vec<(bool, f32)>,
    obecne_miasto: usize,
}

impl<'a> Mrowka<'a> {
    fn new(graf: &'a Graf<usize>, feromony: Rc<RefCell<Graf<f32>>>, miasto_startowe: usize) -> Mrowka<'a> {
        let mut odwiedzone_miasta = Vec::with_capacity(graf.liczba_wezlow);
        odwiedzone_miasta.push(miasto_startowe);
        Mrowka {
            graf,
            feromony,
            odwiedzone_miasta,
            nieodwiedzone_miasta: vec![(true, std::f32::MAX); graf.liczba_wezlow],
            obecne_miasto: miasto_startowe,
        }
    }
}

// Funkjca generuje graf i zapisuje go w katalogu grafy:
fn generuj_graf(liczba_miast: usize, min: usize, max: usize) {
    let graf = Graf::<usize>::losowy(liczba_miast, min, max);
    //println!("{:?}", graf);

    // Zapisz graf do pliku:
    let file = File::create("grafy/nowy_graf.json").unwrap();
    serde_json::to_writer(file, & graf).unwrap();
}

// Funkcja liczy długość trasy:
fn funkcja_celu(graf: & Graf<usize>, miasta: & Vec<usize>) -> usize {
    let mut sum = 0;
    let mut i = 0;

    // Dodaj długość ścieżek poza ścieżką z ostatniego miasta do pierwszego:
    while i < graf.liczba_wezlow - 1 {
        sum += graf.macierz[miasta[i]][miasta[i+1]];
        i += 1;
    }

    // Dodaj ścieżkę z ostatniego miasta do pierwszego:
    sum += graf.macierz[miasta[graf.liczba_wezlow-1]][miasta[0]];
    sum
}

// Funkjca sprawdza czy wynik algorytmu jest prawidłowy (zgodny z założeniami):
fn straznik(graf: & Graf<usize>, miasta: & Vec<usize>) {
    // Sprawdź czy długość wektora z miastami, zgadza się z liczbą węzłów grafu:
    if miasta.len() != graf.liczba_wezlow {
        panic!("Liczba miast w wektorze nie zgadza się z liczbą miast w grafie!");
    }

    // Sprawdź czy miasta się nie powtarzają:
    let mut miasta_zbior = BTreeSet::new();
    for miasto in miasta {
        miasta_zbior.insert(miasto);
    }
    if miasta_zbior.len() != miasta.len() {
        panic!("Miasta w wektorze się powtarzają!");
    }
}

// Funkjca algorytmu mrówkowego wczytująca graf z pliku:
fn algorytm_mrowkowy_file(nazwa_grafu: String, liczba_iteracji: usize, waga_losowosci: f32, zostawiany_feromon: f32, ulatnianie_feromonu: f32) {
    let file = File::open(nazwa_grafu).unwrap();
    let graf: Graf<usize> = serde_json::from_reader(file).unwrap();
    //println!("{:?}", graf);
    algorytm_mrowkowy(& graf, liczba_iteracji, waga_losowosci, zostawiany_feromon, ulatnianie_feromonu);
}

// Funkcja algorytmu mrówkowego:
fn algorytm_mrowkowy(graf: & Graf<usize>, liczba_iteracji: usize, waga_losowosci: f32, zostawiany_feromon: f32, ulatnianie_feromonu: f32) {
    // Wyznacz wagę feromonu:
    let waga_feromonu: f32 = 1.0 - waga_losowosci;

    // Inicjalizuj generator liczb pseudolosowych:
    let mut dice = rand::thread_rng();

    // Twórz graf z feromonami, na początku wszysztkie wartości 0.0:
    let feromony: Graf<f32> = Graf {
        liczba_wezlow: graf.liczba_wezlow,
        macierz: vec![vec![0.0; graf.liczba_wezlow]; graf.liczba_wezlow],
    };
    let feromony = Rc::new(RefCell::new(feromony));
    //println!("{:?}", feromony);

    // Twórz zmienną na najlepsze rozwiązanie:
    let mut the_best = (std::usize::MAX, vec![0]);

    // Iteracje algorytmu mrówkowego:
    let mut i = 0;
    while i < liczba_iteracji {
        // Twórz wektor z mrówkami, każda mrówka zaczyna z osobnego miasta:
        eprintln!("Rozsyłam mrówki po grafie.");
        let mut mrowki: Vec<Mrowka> = Vec::with_capacity(graf.liczba_wezlow);
        let mut k = 0;
        while k < graf.liczba_wezlow {
            mrowki.push(Mrowka::new(graf, Rc::clone(& feromony), k));
            k = k + 1;
        }
    
        // Przejdź się mrówkami po wszystkich miastach:
        let mut z = 0;
        while z < graf.liczba_wezlow - 1 {
            // Iteruj się po mrówkach w celu wykonania pojedynczego kroku:
            for mrowka in mrowki.iter_mut() {
                // Usuń obecne miasto z wektora odwiedzonych:
                mrowka.nieodwiedzone_miasta[mrowka.obecne_miasto].0 = false;

                // Licz atrakcyjność dla każdej ścieżki i szukaj najlepszego miasta:
                let mut best = (0usize, std::f32::MAX);
                let mut j = 0; // Indeks miasta.
                for miasto in mrowka.nieodwiedzone_miasta.iter_mut() {
                    // Jeżeli miasto nie było jeszcze odwiedzone to:
                    if miasto.0 == true {
                        miasto.1 = dice.gen_range(0.0, waga_losowosci) + waga_feromonu * mrowka.feromony.borrow().macierz[mrowka.obecne_miasto][j];
                        if miasto.1 < best.1 {
                            // Mamy najlepszy wynik, zapisujemy go:
                            best.0 = j;
                            best.1 = miasto.1;
                        }
                    }
                    j = j + 1;
                }

                // Zostaw feromon na ścieżce:
                let dzielnik_feromonu = mrowka.graf.macierz[mrowka.obecne_miasto][best.0] as f32; // Dzielnik feromonu to długość ścieżki.
                mrowka.feromony.borrow_mut().macierz[mrowka.obecne_miasto][best.0] += zostawiany_feromon / dzielnik_feromonu;

                // Jeżeli jest tam za dużo feromonu to przytnij jego wartość do 1.0:
                if mrowka.feromony.borrow_mut().macierz[mrowka.obecne_miasto][best.0] > 1.0 {
                    mrowka.feromony.borrow_mut().macierz[mrowka.obecne_miasto][best.0] = 1.0;
                }
                
                // Przemieść się do najlepszego miasta:
                mrowka.obecne_miasto = best.0;

                // Zapamiętaj najlepsze miasto na wektorze miast odwiedzonych:
                mrowka.odwiedzone_miasta.push(mrowka.obecne_miasto);
            }
            // Ulotnij feromony:
            for fer in feromony.borrow_mut().macierz.iter_mut() {
                for fer in fer.iter_mut() {
                    *fer -= ulatnianie_feromonu;

                    // Jeżeli jest ujemna liczba feromonów to ustaw 0.0:
                    if *fer < 0.0 {
                        *fer = 0.0;
                    }
                }
            }
            z = z + 1;
        }
        // W mrówkach są już rozwiązania, zbierz je i wybierz najlepsze:
        for mrowka in mrowki.iter() {
            // Jeżeli rozwiązanie jest lepsze to zapamiętaj je:
            let wartosc = funkcja_celu(graf, & mrowka.odwiedzone_miasta);
            if wartosc < the_best.0 {
                the_best.0 = wartosc;
                the_best.1 = mrowka.odwiedzone_miasta.clone();
                // Pochwal się rozwiązaniem:
                println!("Nowe rozwiązanie: {:?}", the_best);
            }
        }
        i = i + 1;
    }

    // Sprawdź najlepsze rozwiązanie:
    straznik(graf, & the_best.1);

    // Wypisz najlepsze rozwiązanie:
    println!("Rozkład feromonów:");
    println!("{:?}", feromony.borrow().macierz);
    println!("Najlepsze rozwiązanie to:");
    println!("{:?}", the_best);
}

// Funkjca algorytmu zachałannego wczytująca graf z pliku:
fn algorytm_zachlanny_file(nazwa_grafu: String) {
    let file = File::open(nazwa_grafu).unwrap();
    let graf: Graf<usize> = serde_json::from_reader(file).unwrap();
    //println!("{:?}", graf);
    algorytm_zachlanny(& graf);
}

fn algorytm_zachlanny(graf: & Graf<usize>) {

    // Obecne miasto:
    let mut obecne_miasto = 0usize;

    // Twórz wektor na najlepsze rozwiązanie:
    let mut the_best = (std::usize::MAX, Vec::new());
    the_best.1.push(obecne_miasto);

    // Wektor nie odwiedzonych miast:
    let mut nieodwiedzone_miasta = vec![true; graf.liczba_wezlow];
    nieodwiedzone_miasta[obecne_miasto] = false;

    // Dodawaj kolejne miasta do wektora:
    let mut i = 0;
    while i < graf.liczba_wezlow - 1 {
        let mut najlepsze_miasto = (0usize, std::usize::MAX);
        let mut j = 0;
        while j < graf.liczba_wezlow {
            if nieodwiedzone_miasta[j] == true {
                // Sprawdź czy to lepsze posunięcie:
                if graf.macierz[obecne_miasto][j] < najlepsze_miasto.1 {
                    najlepsze_miasto = (j, graf.macierz[obecne_miasto][j]);
                }
            }
            j += 1;
        }
        // Mamy najlepsze miasto w tym kroku zapisujemy je:
        obecne_miasto = najlepsze_miasto.0;
        nieodwiedzone_miasta[obecne_miasto] = false;
        the_best.1.push(obecne_miasto);
        i += 1;
    }

    // Sprawdź najlepsze rozwiązanie:
    straznik(graf, & the_best.1);

    // Wylicz wynik znalezionego rozwiązania:
    the_best.0 = funkcja_celu(graf, & the_best.1);

    // Wypisz najlepsze rozwiązanie:
    println!("Najlepsze rozwiązanie to:");
    println!("{:?}", the_best);
}

fn main() {
    println!("Witaj w świecie mrówek!");

    // Przykładowy graf:
    let graf1: Graf<usize> = Graf {
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

    // Zapisz przykładowy graf do pliku graf1:
    let file = File::create("grafy/graf1.json").unwrap();
    serde_json::to_writer(file, & graf1).unwrap();

    // Biblioteka Clap do uzyskania argumentów linii poleceń:
    let matches = App::new("Program mrówkowy")
        .version("0.0")
        .author("Jacek Jaszczuk 218320")
        .about("Zadaniem programu jest rozwiązwyanie problemu komiwojażera, przy użyciu algorytmu mrówkowego")
        .subcommand(SubCommand::with_name("mrowkowy")
            .about("Uruchamia algorytm mrówkowy")
            .arg(Arg::with_name("graf")
                .short("n")
                .required(true)
                .help("Nazwa grafu")
                .value_name("nazwa"))
            .arg(Arg::with_name("liczba_iteracji")
                .short("i")
                .required(true)
                .help("Liczba iteracji")
                .value_name("liczba"))
            .arg(Arg::with_name("waga_losowosci")
                .short("l")
                .required(true)
                .help("Waga losowości od 0.0 do 1.0")
                .value_name("liczba"))
            .arg(Arg::with_name("zostawiany_feromon")
                .short("z")
                .required(true)
                .help("Zostawiany feromon")
                .value_name("liczba"))
            .arg(Arg::with_name("ulatnianie_feromonu")
                .short("u")
                .required(true)
                .help("Ulatnianie feromonu")
                .value_name("liczba")))
        .subcommand(SubCommand::with_name("zachlanny")
            .about("Uruchamia algorytm zachłanny")
            .arg(Arg::with_name("graf")
                .required(true)
                .help("Nazwa grafu")
                .value_name("nazwa")))
        .subcommand(SubCommand::with_name("wylosuj")
            .about("Program losuje graf, zapisuje go w katalogu grafy")
            .arg(Arg::with_name("liczba_miast")
                .required(true)
                .help("Liczba miast grafu")
                .value_name("liczba"))
            .arg(Arg::with_name("minimalna_liczba")
                .required(true)
                .help("Minimalna wartość w grafie")
                .value_name("min"))
            .arg(Arg::with_name("maksymalna_liczba")
                .required(true)
                .help("Maksymalna wartość w grafie")
                .value_name("max")))
        .get_matches();

    // Uruchom algorytm mrowkowy:
    if let Some(matches) = matches.subcommand_matches("mrowkowy") {
        println!("Algorytm mrówkowy!");
        // Odczytaj wartości:
        let nazwa_grafu = matches.values_of("graf").unwrap().next().unwrap().parse::<String>().unwrap();
        let liczba_iteracji = matches.values_of("liczba_iteracji").unwrap().next().unwrap().parse::<usize>().unwrap();
        let waga_losowosci = matches.values_of("waga_losowosci").unwrap().next().unwrap().parse::<f32>().unwrap();
        let zostawiany_feromon = matches.values_of("zostawiany_feromon").unwrap().next().unwrap().parse::<f32>().unwrap();
        let ulatnianie_feromonu = matches.values_of("ulatnianie_feromonu").unwrap().next().unwrap().parse::<f32>().unwrap();

        // Wykonaj algorytm mrówkowy:
        //algorytm_mrowkowy(&graf1, liczba_iteracji, waga_losowosci, zostawiany_feromon, ulatnianie_feromonu);
        algorytm_mrowkowy_file(nazwa_grafu, liczba_iteracji, waga_losowosci, zostawiany_feromon, ulatnianie_feromonu);
    }

    // Uruchom algorytm zachłanny:
    if let Some(matches) = matches.subcommand_matches("zachlanny") {
        println!("Algorytm zachłanny!");
        let nazwa_grafu = matches.values_of("graf").unwrap().next().unwrap().parse::<String>().unwrap();
        algorytm_zachlanny_file(nazwa_grafu);
    }

    // Uruchom losowanie grafu:
    if let Some(matches) = matches.subcommand_matches("wylosuj") {
        println!("Losowanie grafu!");
        // Odczytaj wartości:
        let liczba_miast = matches.values_of("liczba_miast").unwrap().next().unwrap().parse::<usize>().unwrap();
        let min = matches.values_of("minimalna_liczba").unwrap().next().unwrap().parse::<usize>().unwrap();
        let max = matches.values_of("maksymalna_liczba").unwrap().next().unwrap().parse::<usize>().unwrap();

        // Generuj graf i zapisz go do pliku:
        generuj_graf(liczba_miast, min, max);
    }
}