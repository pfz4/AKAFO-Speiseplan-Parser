use std::fmt::Display;

use chrono::{NaiveDate, NaiveDateTime};
use feed_rs::model::Feed;
use regex::Regex;
use thiserror::Error;

#[derive(Debug)]
pub struct Menu {
    pub title: String,
    pub id: String,
    pub updated: NaiveDateTime,
    pub day_menues: Vec<MenuDay>,
}

#[derive(Debug)]
pub struct MenuDay {
    pub id: String,
    pub date: NaiveDate,
    pub updated: NaiveDateTime,
    pub title: String,
    pub meal_groups: Vec<MealGroup>,
}

#[derive(Debug, Default)]
pub struct MealGroup {
    pub title: String,
    pub meals: Vec<Meal>,
}

#[derive(Debug)]
pub struct Meal {
    pub name: String,
    pub information: Vec<MealInformation>,
    pub additives: Vec<MealAdditive>,
    pub price_student: f32,
    pub price: f32,
}

#[derive(Debug, PartialEq, Eq)]
pub enum MealInformation {
    MitAlkohol,
    MitFisch,
    MitGefluegel,
    Halal,
    MitLamm,
    MitRind,
    MitSchwein,
    Vegetarisch,
    Vegan,
    MitWild,
}

#[derive(Debug, PartialEq, Eq)]
pub enum MealAdditive {
    MitFarbstoff,
    MitKonservierungsstoff,
    MitAntioxidationsmittel,
    MitGeschmacksverstaerker,
    Geschwefelt,
    Geschwaerzt,
    Gewachst,
    MitPhosphat,
    MitSuessungsmitteln,
    EnthaeltEinePhenylalaninquelle,
    KannBeiUebermaessigemKonsumAbfuehrendWirken,
    Koffeinhaltig,
    Chininhaltig,
}

#[derive(Debug, Error)]
pub enum Error {
    #[cfg(feature = "uri-download")]
    FailedToLoadUri(reqwest::Error),
    #[cfg(feature = "uri-download")]
    FailedToReadResponse(reqwest::Error),
    CouldNotParseFeed(feed_rs::parser::ParseFeedError),
    CouldNotExtractDateFromId,
    CouldNotParseMenuDate(chrono::ParseError),
    CouldNotParseHtml(html_parser::Error),
    UnexpectedFeedFormat,
    UnexpectedHtmlFormat,
    CouldNotParsePrice,
}
impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        format!("{:?}", self).fmt(f)
    }
}

#[cfg(feature = "uri-download")]
pub async fn parse_from_uri(uri: &str) -> Result<Menu, Error> {
    let response_body = reqwest::get(uri)
        .await
        .map_err(Error::FailedToLoadUri)?
        .text()
        .await
        .map_err(Error::FailedToReadResponse)?;
    parse_from_str(&response_body)
}

pub fn parse_from_str(input: &str) -> Result<Menu, Error> {
    let feed = feed_rs::parser::parse(input.as_bytes()).map_err(Error::CouldNotParseFeed)?;
    parse_menu(feed)
}

fn parse_menu(value: Feed) -> Result<Menu, Error> {
    let day_menues = value
        .entries
        .into_iter()
        .map(parse_menu_day)
        .collect::<Result<Vec<MenuDay>, Error>>()?;
    Ok(Menu {
        title: value
            .title
            .map(|x| x.content)
            .ok_or(Error::UnexpectedFeedFormat)?,
        id: value.id,
        updated: value
            .updated
            .ok_or(Error::UnexpectedFeedFormat)?
            .naive_utc(),
        day_menues,
    })
}

fn parse_menu_day(value: feed_rs::model::Entry) -> Result<MenuDay, Error> {
    let date = NaiveDate::parse_from_str(
        &format!(
            "20{}",
            value
                .id
                .split('/')
                .last()
                .ok_or(Error::CouldNotExtractDateFromId)?
        ), // See you in 78y
        "%Y-%m-%d",
    )
    .map_err(Error::CouldNotParseMenuDate)?;
    Ok(MenuDay {
        date,
        id: value.id,
        title: value
            .title
            .map(|x| x.content)
            .ok_or(Error::UnexpectedFeedFormat)?,
        updated: value
            .updated
            .ok_or(Error::UnexpectedFeedFormat)?
            .naive_utc(),
        meal_groups: content_to_meal_groups(value.content.ok_or(Error::UnexpectedFeedFormat)?)?,
    })
}

fn content_to_meal_groups(content: feed_rs::model::Content) -> Result<Vec<MealGroup>, Error> {
    let mut output = vec![];

    let html = content.body.ok_or(Error::UnexpectedFeedFormat)?;
    let dom = html_parser::Dom::parse(&html).map_err(Error::CouldNotParseHtml)?;

    let mut meal_group = MealGroup::default();

    for child in dom.children[0]
        .element()
        .ok_or(Error::UnexpectedHtmlFormat)?
        .children
        .iter()
    {
        if let html_parser::Node::Element(element) = child {
            if element.name == "p" {
                output.push(meal_group);
                meal_group = MealGroup::default();
                meal_group.title = element.children[0]
                    .element()
                    .ok_or(Error::UnexpectedHtmlFormat)?
                    .children[0]
                    .text()
                    .ok_or(Error::UnexpectedHtmlFormat)?
                    .to_string();
            } else if element.name == "ul" {
                for item in element.children.iter() {
                    let item = item.element().ok_or(Error::UnexpectedHtmlFormat)?;
                    let mut name = item.children[0]
                        .text()
                        .ok_or(Error::UnexpectedHtmlFormat)?
                        .to_string();

                    let information_regex = Regex::new(r"\([A-Z]{1,2}(,[A-Z]{1,2})*\)")
                        .expect("valid information regex");
                    let mut information = vec![];
                    if let Some(info) = information_regex.find(&name) {
                        information = parse_information_tupel(info.as_str());
                        name = name.replace(info.as_str(), "");
                    }

                    let additive_regex =
                        Regex::new(r"\([0-9]{1,2}(,[0-9]{1,2})*\)").expect("valid additice regex");
                    let mut additives = vec![];
                    if let Some(add) = additive_regex.find(&name) {
                        additives = parse_additive_tupel(add.as_str());
                        name = name.replace(add.as_str(), "");
                    }

                    name = name.trim_end().to_string();

                    let (price_student, price) =
                        parse_price(item.children[2].text().ok_or(Error::UnexpectedHtmlFormat)?)?;

                    let meal = Meal {
                        name,
                        information,
                        additives,
                        price,
                        price_student,
                    };
                    meal_group.meals.push(meal);
                }
            }
        }
    }

    output.push(meal_group);
    output.remove(0);
    Ok(output)
}

fn parse_information_tupel(tupel_str: &str) -> Vec<MealInformation> {
    tupel_str
        .replace(['(', ')'], "")
        .split(',')
        .filter_map(|x| MealInformation::try_from(x).ok())
        .collect()
}

fn parse_additive_tupel(tupel_str: &str) -> Vec<MealAdditive> {
    tupel_str
        .replace(['(', ')'], "")
        .split(',')
        .filter_map(|x| MealAdditive::try_from(x).ok())
        .collect()
}

fn parse_price(input: &str) -> Result<(f32, f32), Error> {
    let price_str = input.replace("EUR", "").replace(',', ".").replace(' ', "");
    let mut price_str = price_str.split('-');
    let price_student = price_str
        .next()
        .ok_or(Error::CouldNotParsePrice)?
        .parse::<f32>()
        .map_err(|_| Error::CouldNotParsePrice)?;
    let price = price_str
        .next()
        .ok_or(Error::CouldNotParsePrice)?
        .parse::<f32>()
        .map_err(|_| Error::CouldNotParsePrice)?;
    Ok((price_student, price))
}

impl TryFrom<&str> for MealInformation {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "A" => Ok(Self::MitAlkohol),
            "F" => Ok(Self::MitFisch),
            "G" => Ok(Self::MitGefluegel),
            "H" => Ok(Self::Halal),
            "L" => Ok(Self::MitLamm),
            "R" => Ok(Self::MitRind),
            "S" => Ok(Self::MitSchwein),
            "V" => Ok(Self::Vegetarisch),
            "VG" => Ok(Self::Vegan),
            "W" => Ok(Self::MitWild),
            _ => Err(()),
        }
    }
}

impl TryFrom<&str> for MealAdditive {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "1" => Ok(Self::MitFarbstoff),
            "2" => Ok(Self::MitKonservierungsstoff),
            "3" => Ok(Self::MitAntioxidationsmittel),
            "4" => Ok(Self::MitGeschmacksverstaerker),
            "5" => Ok(Self::Geschwefelt),
            "6" => Ok(Self::Geschwaerzt),
            "7" => Ok(Self::Gewachst),
            "8" => Ok(Self::MitPhosphat),
            "9" => Ok(Self::MitSuessungsmitteln),
            "10" => Ok(Self::EnthaeltEinePhenylalaninquelle),
            "11" => Ok(Self::KannBeiUebermaessigemKonsumAbfuehrendWirken),
            "12" => Ok(Self::Koffeinhaltig),
            "13" => Ok(Self::Chininhaltig),

            _ => Err(()),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{
        parse_additive_tupel, parse_information_tupel, parse_price, MealAdditive, MealInformation,
    };

    #[test]
    fn information_tupel_parsing() {
        let cases = [
            (
                "(VG,V,G)",
                vec![
                    MealInformation::Vegan,
                    MealInformation::Vegetarisch,
                    MealInformation::MitGefluegel,
                ],
            ),
            ("(VG)", vec![MealInformation::Vegan]),
            ("", vec![]),
        ];

        for (tupel, output) in cases {
            assert_eq!(parse_information_tupel(tupel), output);
        }
    }

    #[test]
    fn additive_tupel_parsing() {
        let cases = [
            (
                "(1,3)",
                vec![
                    MealAdditive::MitFarbstoff,
                    MealAdditive::MitAntioxidationsmittel,
                ],
            ),
            ("(12)", vec![MealAdditive::Koffeinhaltig]),
            ("", vec![]),
        ];

        for (tupel, output) in cases {
            assert_eq!(parse_additive_tupel(tupel), output);
        }
    }

    #[test]
    fn price_parsing() {
        assert_eq!((1.2, 2.0), parse_price("1,20 EUR - 2,00 EUR").unwrap());
        assert_eq!((1.0, 5.0), parse_price("1,00 EUR - 5,00 EUR").unwrap());
        assert_eq!((5.0, 1.0), parse_price("5,00 EUR - 1,00 EUR").unwrap());
    }
}
