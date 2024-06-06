use std::fs::File;
use std::io;
use std::io::{BufReader, Error, Read, Write};
use std::process::exit;
use serde::{Serialize, Deserialize};
use colored::*;
use serde_json::{json, Value};
use std::path::Path;
use std::thread;
use std::time::Duration;


//Struct to for default parameters.
#[derive(Serialize, Deserialize, Debug)]
struct DefaultParameters{
    city: String,
    country: String,
}

//Unwrap the DefaultParameters further
#[derive(Deserialize, Debug)]
struct Config {
    default_parameters: Vec<DefaultParameters>,
}



//Struct to deseieralize JSON from Weathermap API
#[derive(Deserialize, Debug)]
struct WeatherResponse {
    weather: Vec<Weather>, // Contains weather information
    main: Main, // Contains main weather parameters
    wind: Wind, // Contains wind information
    name: String, // Contains the name of the queried location
}

//Struct to represent weather description

#[derive(Deserialize, Debug)]
struct Weather{
    description: String,
}

//Struct to represent main weather parameters

#[derive(Deserialize, Debug)]
struct Main{
    temp: f64,
    humidity: f64,
    pressure: f64,
    feels_like: f64,

}

//Struct for wind info
#[derive(Deserialize, Debug)]
struct Wind{
    speed: f64,
}



//FN to get weather information from the APi

fn get_weather_info(city: &str, country_code: &str, api_key: &str) -> Result<WeatherResponse, reqwest::Error> {
    let url: String = format!("https://api.openweathermap.org/data/2.5/weather?q={},{}&units=metric&appid={}", city, country_code, api_key);
    let response = reqwest::blocking::get(&url)?;
    let response_json = response.json::<WeatherResponse>()?;
    Ok(response_json)
}

fn get_default_parameters(filename: &str) -> Result<(String, String), Error>{

    let file = File::open("data.json")?;
    let reader = BufReader::new(file);

    // Deserialize the JSON file into the Config struct
    let config: Config = serde_json::from_reader(reader)?;

    // Extract the values of city and country from the first element in the default_parameters vector
    if let Some(parameter) = config.default_parameters.get(0) {
        Ok((parameter.city.clone(), parameter.country.clone()))
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "No default parameters found",
        ))
    }
}


//Extract information from the JSON file
fn edit_json_file(file_path: &str, city_par: &str, country_par: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Read the JSON file
    let mut file = File::open(file_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    // Parse the JSON data
    let mut json_data: Value = serde_json::from_str(&contents)?;

    // Get a mutable reference to the "default_parameters" array
    let default_params = json_data.get_mut("default_parameters").ok_or("Unable to find 'default_parameters' key")?;

    // Check if the "default_parameters" array is not empty
    if !default_params.as_array().unwrap().is_empty() {
        // Get a mutable reference to the first object in the array
        let first_obj = &mut default_params.as_array_mut().unwrap()[0];

        // Update the "city" and "country" fields
        first_obj["city"] = json!(city_par);
        first_obj["country"] = json!(country_par);
    }

    // Write the updated JSON data back to the file
    let new_contents = serde_json::to_string_pretty(&json_data)?;
    let mut file = File::create(file_path)?;
    file.write_all(new_contents.as_bytes())?;

    Ok(())
}



//Functing to display the weather information
fn display_weather_info(response: &WeatherResponse) {
    //Extract the weather information from the response
    let description = &response.weather[0].description;
    let temperature= response.main.temp;
    let humidity = response.main.humidity;
    let pressure = response.main.pressure;
    let wind_speed: f64 = response.wind.speed;
    let feels_like: f64 = response.main.feels_like;
    //Formating weather info
    let weather_text: String = format!(
        "Weather in {}: {} {} \n
        >Temperature: {:.1}C, {:.1}F, \n
        >Feels like: {:.1}C, {:.1}F, \n
        >Humidity: {:.1}%,\n
        >Pressure: {:.1} hPa,\n
        >Wind Speed: {:.1} m/s",
        response.name,
        description,
        get_temp_emoji(temperature),
        temperature,
        (temperature * 1.8) + 32.0,
        feels_like,
        (feels_like * 1.8) + 32.0,
        humidity,
        pressure,
        wind_speed,
    );
    // Coloring the weather text based on weather conditions
    let weather_text_colored: ColoredString = match description.as_str() {
        "clear sky" => weather_text.bright_yellow(),
        "few clouds" | "scattered clouds" | "broken clouds" => weather_text.bright_blue(),
        "overcast clouds" | "mist" | "haze" | "smoke" | "sand" | "dust" | "fog" | "squalls" => weather_text.dimmed(),
        "shower rain" | "rain" | "thunderstorm" | "snow" => weather_text.bright_cyan(),
        _ => weather_text.normal(),
    };
    // Print the colored weather infomation
    println!("{}", weather_text_colored);

}
    //Func to get emoji based on temperature
fn get_temp_emoji(temperature: f64) -> &'static str {
        if temperature < 0.0 {
            "❄️"
        } else if temperature >= 0.0 && temperature < 10.0 {
            "☁️"
        } else if temperature >= 10.0 && temperature < 20.0 {
            "⛅"
        } else if temperature >= 20.0 && temperature < 30.0 {
            "🌤️"
        } else {
            "🔥"
        }
    }

//Create data.json if it doesn't exist
fn create_data_json() -> std::io::Result<()> {
    let data = json!({
        r#"default_parameters"#: [
            {
                "city": "",
                "country": ""
            }
        ]
    });

    let json_string = data.to_string();
    let mut file = File::create("data.json")?;
    file.write_all(json_string.as_bytes())?;

    Ok(())
}


fn main() {
    match File::open(Path::new("data.json")) {
        Err(_) => if let Err(e) = create_data_json()
        {
            eprintln!("Error creating data.json, creating a new one: {}", e);
        },
        Ok(_) => (),
    }



    let file_path = "data.json";

    let Ok((mut def_city,mut def_country)) = get_default_parameters(file_path) else { todo!() };

    let api_key = "*";

    println!("{}", r#"

 _       __________    __________  __  _________   __________     _       ___________  ________  ____________     ______________  ______________  _   __
| |     / / ____/ /   / ____/ __ \/  |/  / ____/  /_  __/ __ \   | |     / / ____/   |/_  __/ / / / ____/ __ \   / ___/_  __/   |/_  __/  _/ __ \/ | / /
| | /| / / __/ / /   / /   / / / / /|_/ / __/      / / / / / /   | | /| / / __/ / /| | / / / /_/ / __/ / /_/ /   \__ \ / / / /| | / /  / // / / /  |/ /
| |/ |/ / /___/ /___/ /___/ /_/ / /  / / /___     / / / /_/ /    | |/ |/ / /___/ ___ |/ / / __  / /___/ _, _/   ___/ // / / ___ |/ / _/ // /_/ / /|  /
|__/|__/_____/_____/\____/\____/_/  /_/_____/    /_/  \____/     |__/|__/_____/_/  |_/_/ /_/ /_/_____/_/ |_|   /____//_/ /_/  |_/_/ /___/\____/_/ |_/


"#.bright_cyan());

    //Check if data.json has existing variables and display weather information if so.
    if (&def_city, &def_country) != (&String::from(""),&String::from("")) {
        match get_weather_info(&def_city, &def_country, api_key) {
            Ok(response) => {
                display_weather_info(&response); // Displaying weather infromation
            }

            Err(err) => {
                eprintln!("Error {}", err) // Printing error in case of failure.
            }
        }
    } else { // else statement that covers the new user needs.
        println!("{}", "New user detected! Please set new default cities for further use.".bright_cyan());

        println!("{}", "Please enter the city that you would like to make the new default.".bright_green());
        io::stdin().read_line(&mut def_city).expect("Failed to read city input!");
        let def_city = def_city.trim();

        println!("{}", "Please enter the country code (i.e. US for United States) that you would like to make the new default.".bright_green());
        io::stdin().read_line(&mut def_country).expect("Failed to read city input!");
        let def_country = def_country.trim();

        let _ = edit_json_file(file_path, &def_city, &def_country);

        //just show weather
        match get_weather_info(&def_city, &def_country, api_key) {
            Ok(response) => {
                display_weather_info(&response); // Displaying weather infromation
            }

            Err(err) => {
                eprintln!("Error {}", err) // Printing error in case of failure.
            }
        }

    }
    // looping for the user to search for future weather
    println!("{}", "Do you want to search for weather forecast in another city? (yes/no): ".bright_green()); // if a user wants to continue to search for weather
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read input"); // Reading user input for continuationa
    let input = input.trim();

    if input != "no" {
        loop {


            // Reading City
            println!("{}", "Please enter the name of the city: "
                .bright_green());
            let mut city = String::new();
            io::stdin().read_line(&mut city).expect("Failed to read city input!");
            let city: &str = city.trim();

            // Reading Country
            println!("{}", "Please enter the country code (e.g., US for United States): "
                .bright_green());
            let mut country_code = String::new();
            io::stdin().read_line(&mut country_code).expect("Failed to read city input!");
            let country_code: &str = country_code.trim();



            //Calling the function to fetch weather information
            match get_weather_info(&city, &country_code, api_key) {
                Ok(response) => {
                    display_weather_info(&response); // Displaying weather infromation
                }

                Err(err) => {
                    eprintln!("Error {}", err) // Printing error in case of failure.
                }
            }

            println!("{}", "Do you want to search for weather forecast in another city? (yes/no): ".bright_green()); // if a user wants to continue to search for weather
            let mut input = String::new();
            io::stdin().read_line(&mut input).expect("Failed to read input"); // Reading user input for continuationa
            let input = input.trim();

           if input != "yes" {
               println!("{}", "Goodbye goober >:3".bright_purple());
               thread::sleep(Duration::from_secs(1));
               break;

            }
        }
    } else {
        println!("{}", "Goodbye goober >:3".bright_purple());
        thread::sleep(Duration::from_secs(1));
        exit(0);
    }
}