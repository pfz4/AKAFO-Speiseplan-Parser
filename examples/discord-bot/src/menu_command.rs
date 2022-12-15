use chrono::{Days, NaiveDate, Utc};
use serenity::{
    builder::{CreateApplicationCommand, CreateApplicationCommandOption},
    json::Value,
    model::prelude::{
        command::CommandOptionType, interaction::application_command::CommandDataOption,
    },
    utils::MessageBuilder,
};

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("menu")
        .description("Get RUB Mensa Menu")
        .description_localized("de", "Gibt das RUB Mensa-Menü für einen Tag aus")
        .name_localized("de", "speisekarte")
        .create_option(|option| {
            option
                .name("today")
                .name_localized("de", "heute")
                .description("get the menu for today")
                .description_localized("de", "Gebe das Menü für Heute aus")
                .kind(CommandOptionType::SubCommand)
        })
        .create_option(|option| {
            option
                .name("tomorrow")
                .name_localized("de", "morgen")
                .description("get the menu for tomorrow")
                .description_localized("de", "Gebe das Menü für Morgen aus")
                .kind(CommandOptionType::SubCommand)
        })
        .create_option(|option| {
            let mut sub_opt = CreateApplicationCommandOption::default();
            sub_opt
                .name("day")
                .name_localized("de", "datum")
                .description("day in yyyy-mm-dd format")
                .description_localized("de", "Datum im yyyy-mm-dd Format")
                .required(true)
                .kind(CommandOptionType::String);
            option
                .name("date")
                .name_localized("de", "datum")
                .description("get the menu for a specific date")
                .description_localized("de", "Gebe das Menü für einen Tag aus")
                .kind(CommandOptionType::SubCommand)
                .add_sub_option(sub_opt)
        })
}

pub async fn run(options: &[CommandDataOption]) -> String {
    let mut message_builder = MessageBuilder::new();
    if options.len() == 1 {
        let option = &options[0];
        let date = match option.name.as_str() {
            "today" => Utc::now().naive_utc().date(),
            "tomorrow" => Utc::now()
                .naive_utc()
                .date()
                .checked_add_days(Days::new(1))
                .unwrap(),
            "date" => match &option.options[0].value {
                Some(Value::String(val)) => NaiveDate::parse_from_str(val, "%Y-%m-%d").unwrap(),
                _ => Utc::now().naive_utc().date(),
            },
            _ => todo!(),
        };

        let menu = akafo_menu_parser::parse_from_uri("https://www.akafoe.de/gastronomie/speiseplaene-der-mensen/ruhr-universitaet-bochum?mid=1&tx_akafoespeiseplan_mensadetails%5Baction%5D=feed&tx_akafoespeiseplan_mensadetails%5Bcontroller%5D=AtomFeed").await.unwrap();
        let menu = menu.day_menues.iter().find(|x| x.date == date).unwrap();

        message_builder.push_bold_line_safe(format!("Menü für den {}", date.format("%d.%m.%Y")));
        message_builder.push_bold_line_safe("-----------------------");
        for meal_group in menu.meal_groups.iter() {
            message_builder.push_bold_line_safe(&meal_group.title);
            for meal in meal_group.meals.iter() {
                message_builder.push_line_safe(format!(
                    "- {} ({}€ - {}€) {:?}{:?}",
                    meal.name, meal.price_student, meal.price, meal.information, meal.additives
                ));
            }
            message_builder.push_line("");
        }
    }
    message_builder.build()
}
