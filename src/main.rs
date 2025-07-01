#![allow(unused_imports)]
use chrono::{Local, Timelike};
use dotenv::dotenv;
use futures::stream::StreamExt;
use mongodb::bson::doc;
use mongodb::bson::oid::ObjectId;
use mongodb::{Client, Collection, options::ClientOptions};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::Rect,
    style::{Color, Style},
    widgets::{BarChart, Block, Borders, Paragraph},
};
use serde::{Deserialize, Serialize};
use std::env;
use std::{collections::HashMap, io};
use std::io::{Stdout, Write};

#[derive(Deserialize, Debug)]
struct Prices  {
    #[serde(rename = "_id")]
    _id: Option<ObjectId>, 
    
   #[serde(flatten)]
    hours: HashMap<String, KloValue>,
}


#[derive(Deserialize, Debug)]
#[serde(untagged)]
#[allow(dead_code)]
enum KloValue {
    Map(HashMap<String, f64>),
    Int(i32), 
}

fn get_time_data(prices: &Prices, hour: u8) -> Option<&HashMap<String, f64>> {
    let key = format!("klo {:02}", hour);
    match prices.hours.get(&key) {
        Some(KloValue::Map(map)) => Some(map),
        _ => None,
    }
}

#[allow(dead_code)]
async fn fetch_data_from_api() -> Result<Vec<(String, u64)>, Box<dyn std::error::Error>> {
    dotenv().ok();
    let api_key = env::var("api_key").expect("Api key not found");

    let url = format!(
        "mongodb+srv://dutchystuff:{}@data.pgkic.mongodb.net/electricity_data?retryWrites=true&w=majority&appName=data",
        api_key
    );

    let client_options = ClientOptions::parse(&url).await?;
    let client = Client::with_options(client_options)?;

    let db = client.database("electricity_data");
    let collection: Collection<Prices> = db.collection("prices");

    let mut prices: Vec<Prices> = Vec::new();

    let mut cursor = collection.find(doc! {}).await?;
    while let Some(result) = cursor.next().await {
        match result {
            Ok(price) => prices.push(price),
            Err(e) => println!("Error retrieving document: {:?}", e),
        }
    }

    prices.remove(0);

    if prices.len() > 1 {
        prices.remove(0);
    }

    let mut final_form: Vec<(String, u64)> = Vec::new();

    for single_data_entry in &prices {
        for index in 0..24 {
            if let Some(klo_data) = get_time_data(single_data_entry, index) {
                if let Some(price_2025) = klo_data.get("2025") {
                    final_form.push((format!("{:02}", index), (price_2025 * 100.0) as u64));
                }
            }
        }
    }

    Ok(final_form)
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(&stdout);
    let backend2 = CrosstermBackend::new(&stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut terminal2 = Terminal::new(backend2)?;

    let name =    r#"
         _    _ _       _                      _   
        | |  | (_)     | |                    | |  
        | |__| |_ _ __ | |_ __ _   _ __  _   _| |_ 
        |  __  | | '_ \| __/ _` | | '_ \| | | | __|
        | |  | | | | | | || (_| | | | | | |_| | |_ 
        |_|  |_|_|_| |_|\__\__,_| |_| |_|\__, |\__|
                                        __/ |    
                                        |___/ 
                        Loading...   
        "#;

    terminal2.draw(|f| {
        let size = f.size();
        let header_rect = Rect::new(0, 0, size.width, 12);
        let header_block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Black))
                .title("");

        let paragraph = Paragraph::new(name)
                .block(header_block) 
                .alignment(ratatui::layout::Alignment::Center)
                .style(Style::default().fg(Color::White));

        f.render_widget(paragraph, header_rect);
    })?;

    let raw_data: Vec<(String, u64)> = fetch_data_from_api().await?;

    let data: Vec<(&str, u64)> = raw_data
        .iter()
        .map(|(hour, price)| (hour.as_str(), *price))
        .collect();

    terminal.draw(|f| {
        let size = f.size();
        let margin_top = 10;
        let chart_height = 18;
        let chart_area = Rect::new(0, margin_top, size.width, chart_height);
        let block = Block::default()
            .title("Sähkön-tuntihinta")
            .border_style(Style::default().fg(Color::Black))
            .borders(Borders::ALL);

        let barchart = BarChart::default()
            .block(block)
            .data(&data)
            .bar_width(4)
            .bar_gap(0)
            .bar_style(Style::default().fg(Color::Green))
            .value_style(Style::default().fg(Color::White));

        f.render_widget(barchart, chart_area);
    })?;

    let prices: Vec<u32> = raw_data
        .into_iter()
        .map(|(_, price)| price as u32)
        .collect();

    let now = Local::now();
    let hour: usize = now.hour() as usize;

    println!("Hinta nyt: {:?} c/kWh", prices[hour]);

    if let Some((index, min_price)) = prices.iter().enumerate().min_by_key(|&(_, v)| v) {
        println!(
            "Halvin tunti {:}-{:}: {:} c/kWh",
            index,
            (index + 1),
            min_price
        );
    }

    if let Some((index, max_price)) = prices.iter().enumerate().max_by_key(|&(_, v)| v) {
        println!(
            "Kallein tunti {:}-{:}: {:} c/kWh",
            index,
            (index + 1),
            max_price
        );
    }

    Ok(())
}

/*   let data: [(&str, u64); 24] = [
       ("00", 089),
       ("01", 503),
       ("02", 314),
       ("03", 055),
       ("04", 027),
       ("05", 027),
       ("06", 026),
       ("07", 043),
       ("08", 06),
       ("09", 054),
       ("10", 012),
       ("11", 008),
       ("12", 05),
       ("13", 0),
       ("14", 0),
       ("15", 0),
       ("16", 0),
       ("17", 0),
       ("18", 038),
       ("19", 017),
       ("20", 0),
       ("21", 0),
       ("22", 0),
       ("23", 1),
   ];
*/


/* 
    println!(
        r#"
         _    _ _       _                      _   
        | |  | (_)     | |                    | |  
        | |__| |_ _ __ | |_ __ _   _ __  _   _| |_ 
        |  __  | | '_ \| __/ _` | | '_ \| | | | __|
        | |  | | | | | | || (_| | | | | | |_| | |_ 
        |_|  |_|_|_| |_|\__\__,_| |_| |_|\__, |\__|
                                        __/ |    
                                        |___/     
        "#
    ); */