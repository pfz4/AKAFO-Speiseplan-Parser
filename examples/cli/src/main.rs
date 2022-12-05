#[tokio::main]
async fn main() {
    let menu = akafo_menu_parser::parse_from_uri("https://www.akafoe.de/gastronomie/speiseplaene-der-mensen/ruhr-universitaet-bochum?mid=1&tx_akafoespeiseplan_mensadetails%5Baction%5D=feed&tx_akafoespeiseplan_mensadetails%5Bcontroller%5D=AtomFeed").await.unwrap();

    for entry in menu.day_menues {
        println!();
        println!("Menü für den {}", entry.date.format("%d.%m.%Y"));
        println!("-----------------------");
        for meal_group in entry.meal_groups {
            println!("    {}", meal_group.title);
            for meal in meal_group.meals {
                println!(
                    "    - {} ({}€ - {}€) [{:?}{:?}]",
                    meal.name, meal.price_student, meal.price, meal.information, meal.additives
                );
            }
            println!();
        }
    }
}
