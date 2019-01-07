use rand::Rng;
use std::rc::Rc;
use std::cell::RefCell;
use clap::{Arg, App, SubCommand};

#[derive(Debug)]
struct Graf<T> {
    liczba_wezlow: usize,
    macierz: Vec<Vec<T>>,
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
    // Wypisz najlepsze rozwiązanie:
    println!("Rozkład feromonów:");
    println!("{:?}", feromony.borrow().macierz);
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
        .get_matches();

    // Uruchom algorytm mrowkowy:
    if let Some(matches) = matches.subcommand_matches("mrowkowy") {
        // Odczytaj wartości:
        let liczba_iteracji = matches.values_of("liczba_iteracji").unwrap().next().unwrap().parse::<usize>().unwrap();
        let waga_losowosci = matches.values_of("waga_losowosci").unwrap().next().unwrap().parse::<f32>().unwrap();
        let zostawiany_feromon = matches.values_of("zostawiany_feromon").unwrap().next().unwrap().parse::<f32>().unwrap();
        let ulatnianie_feromonu = matches.values_of("ulatnianie_feromonu").unwrap().next().unwrap().parse::<f32>().unwrap();

        // Wykonaj algorytm mrówkowy:
        algorytm_mrowkowy(&graf1, liczba_iteracji, waga_losowosci, zostawiany_feromon, ulatnianie_feromonu);
    }

    // Uruchom algorytm zachłanny:
    if let Some(matches) = matches.subcommand_matches("zachlanny") {
        println!("Zachłanny!");
    }
}