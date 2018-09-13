use chrono::{DateTime, NaiveDateTime, Local, Datelike, Timelike, Weekday};
use postgres::{Connection, TlsMode};
#[cfg(feature = "web")]
use reqwest::{get};

use super::{CONFIG, error::Error};
#[cfg(feature = "web")]
pub fn check_for_daily_info() -> Result<bool, Error> {
    let c = get_conn()?;
    let today = Local::today().naive_local().and_hms(0,0,0);
    debug!(target: "robohome->debug", "check_for_daily_info today");
    let count = c.query(r#"SELECT "Id" from "KeyTimes" WHERE "Date" = $1"#, &[&today])?.len();
    Ok(count != 4)
}
#[cfg(feature = "web")]
pub fn get_daily_info() -> Result<(Time, Time), Error> {
    debug!(target: "robohome:debug", "get_daily_info");
    let ret: WeatherResponse = request_weather()?;
    let sunrise = Time::from(ret.sun_phase.sunrise.into()?, TimeKind::Sunrise);
    let sunset = Time::from(ret.sun_phase.sunset.into()?, TimeKind::Sunset);
    Ok((sunrise, sunset))
}
#[cfg(feature = "web")]
pub fn request_weather() -> Result<WeatherResponse, Error> {
    debug!(target: "robohome:debug", "request_weather");
    let mut attempts = 0;
    while attempts < CONFIG.weather_attempts {
        ::std::thread::sleep(::std::time::Duration::from_millis((attempts * 1000 * 30) as u64));
        match get(&CONFIG.weather_uri) {
            Ok(mut res) => match res.json() {
                Ok(ret) => {
                    debug!(target: "robohome:debug", "Got weather info");
                    return Ok(ret)
                },
                Err(e) => error!("Error getting weather info, {}", e),
            },
            Err(e) => error!("Error getting weather info, {}", e),
        }
        attempts += 1;
    }
    Err(Error::Other(String::from("exceeded total request attempts")))
}

pub fn save_daily_info(sunrise: Time, sunset: Time) -> Result<i32, Error> {
    debug!(target: "robohome:debug", "save_daily_info");
    let c = get_conn()?;
    let local = Local::today();
    let today = local.naive_local().and_hms(0,0,0);
    let trans = c.transaction()?;
    let stmt = trans.prepare(r#"INSERT INTO public."KeyTimes" ("Date", "Time_Hour", "Time_Minute",
                                                "Time_TimeOfDay", "Time_TimeType",
                                                "Time_DayOfWeek")
                VALUES ($1, $2, $3, $4, $5, $6)"#)?;
    stmt.execute(&[&today, &(sunrise.hour - 1), &sunrise.minute, &sunrise.tod.for_db(), &TimeKind::Dawn.for_db(), &sunrise.day_of_week])?;
    stmt.execute(&[&today, &sunrise.hour, &sunrise.minute, &sunrise.tod.for_db(), &sunrise.kind.for_db(), &sunrise.day_of_week])?;
    stmt.execute(&[&today, &(sunset.hour - 1), &sunset.minute, &sunset.tod.for_db(), &TimeKind::Dusk.for_db(), &sunset.day_of_week])?;
    stmt.execute(&[&today, &sunset.hour, &sunset.minute, &sunset.tod.for_db(), &sunset.kind.for_db(), &sunset.day_of_week])?;
    let mut count = 0;
    for row in &trans.query("SELECT update_key_times()", &[])? {
        let row_count: i32 = row.get(0);
        debug!(target: "robohome:debug", "Updated {} flips", row_count);
        count += row_count;
    }
    trans.commit()?;
    debug!(target: "robohome:debug", "daily info saved");
    Ok(count)
}

pub fn get_flips() -> Result<Vec<Flip>, Error> {
    debug!(target: "robohome:debug", "get_flips");
    let c = get_conn()?;
    let dow = get_dow();
    let rows = c.query(r#"SELECT id, direction, hour, min, tod, kind, dow, switch_id, remote_id
                FROM PendingFlips
                WHERE dow & $1 > 0"#, &[&dow])?;
    let mut ret = Vec::with_capacity(rows.len());
    for r in &rows {
        let id = r.get(0);
        let direction = r.get(1);
        let hour = r.get(2);
        let min = r.get(3);
        let tod = r.get(4);
        let kind = r.get(5);
        let dow = r.get(6);
        let sw_id = r.get(7);
        let rm_id = r.get(8);
        ret.push(Flip::from_db(id, direction, hour, min, tod, kind, dow, sw_id, rm_id)?)
    }
    Ok(ret)
}

pub fn get_conn() -> Result<Connection, Error> {
    let c = Connection::connect(CONFIG.db_conn_str.as_str(), TlsMode::None)?;
    Ok(c)
}

fn get_dow() -> i32 {
    match Local::today().weekday() {
        Weekday::Sun => 1,
        Weekday::Mon => 2,
        Weekday::Tue => 4,
        Weekday::Wed => 8,
        Weekday::Thu => 16,
        Weekday::Fri => 32,
        Weekday::Sat => 64,
    }
}

#[derive(Deserialize)]
pub struct WeatherResponse {
    pub sun_phase: SunPhase,
}

#[derive(Deserialize)]
pub struct SunPhase {
    sunrise: WeatherTime,
    sunset: WeatherTime
}

#[derive(Deserialize, Debug)]
pub struct WeatherTime {
    hour: String,
    minute: String,
}

impl WeatherTime {
    fn into(self) -> Result<NaiveDateTime, Error> {
        let ret = Local::today().naive_local();
        if let Ok(hour) = self.hour.parse() {
            if let Ok(min) = self.minute.parse() {
                return Ok(ret.and_hms(hour, min, 0));
            }
        }
        Err(Error::Other(format!("Unable to parse WeatherTime into NaiveDateTime {:?}", self)))
    }
}

#[derive(Serialize, Debug)]
pub struct Time {
    pub hour: i32,
    pub minute: i32,
    pub tod: TimeOfDay,
    pub kind: TimeKind,
    pub day_of_week: i32,
}

impl Time {
    pub fn from_db(hour: i32, minute: i32, tod: i32, kind: i32, dow: i32) -> Result<Self, Error> {
        let tod = TimeOfDay::from_db(tod)?;
        let kind = TimeKind::from_db(kind)?;
        Ok(Self::new(hour, minute, tod, kind, dow))
    }
    fn new(hour: i32, minute: i32, tod: TimeOfDay, kind: TimeKind, dow: i32) -> Self {
        Self {
            hour,
            minute,
            tod,
            kind,
            day_of_week: dow,
        }
    }
    fn from(dt: NaiveDateTime, kind: TimeKind) -> Self {
        let (is_pm, hour) = dt.hour12();
        let min = dt.minute() as i32;
        let tod = if is_pm { TimeOfDay::Pm } else { TimeOfDay::Am };
        let dow = 1 << dt.weekday().number_from_monday() as i32;
        Self::new(hour as i32, min, tod, kind, dow)
    }

    pub fn lte(&self, other: &DateTime<Local>) -> bool {
        let hour = if self.tod == TimeOfDay::Pm {
            self.hour + 12
        } else {
            self.hour
        };
        let other_h = other.hour() as i32;
        let other_min = other.minute() as i32;
        hour <= other_h && self.minute <= other_min
    }
}
#[derive(Serialize, Debug)]
pub enum TimeKind {
    Custom,
    Dawn,
    Sunrise,
    Noon,
    Sunset,
    Dusk,
    Midnight
}

impl TimeKind {
    pub fn for_db(&self) -> i32 {
        match self {
            TimeKind::Custom => 0,
            TimeKind::Dawn => 1,
            TimeKind::Sunrise => 2,
            TimeKind::Noon => 3,
            TimeKind::Sunset => 4,
            TimeKind::Dusk => 5,
            TimeKind::Midnight => 6,
        }
    }

    pub fn from_db(i: i32) -> Result<Self, Error> {
        match i {
            0 => Ok(TimeKind::Custom),
            1 => Ok(TimeKind::Dawn),
            2 => Ok(TimeKind::Sunrise),
            3 => Ok(TimeKind::Noon),
            4 => Ok(TimeKind::Sunset),
            5 => Ok(TimeKind::Dusk),
            6 => Ok(TimeKind::Midnight),
            _ => Err(Error::Enum("TimeKind".to_owned(), i))
        }
    }
}
#[derive(Serialize, PartialEq, Debug)]
pub enum TimeOfDay {
    Am,
    Pm,
}

impl TimeOfDay {
    pub fn for_db(&self) -> i32 {
        match self {
            TimeOfDay::Am => 0,
            TimeOfDay::Pm => 1,
        }
    }

    pub fn from_db(i: i32) -> Result<Self, Error> {
        match i {
            0 => Ok(TimeOfDay::Am),
            1 => Ok(TimeOfDay::Pm),
            _ => Err(Error::Enum("TimeOfDay".to_owned(), i))
        }
    }
}

#[derive(Serialize)]
pub struct Flip {
    pub id: i32,
    pub direction: SwitchState,
    pub time: Time,
    pub switch_id: i32,
    pub remote_id: i32,
}

impl Flip {
    pub fn from_db(id: i32, direction: i32,
                hour: i32, min: i32, tod: i32,
                time_kind: i32, dow: i32 ,
                switch_id: i32, remote_id: i32) -> Result<Self, Error> {
        let time = Time::from_db(hour, min, tod, time_kind, dow)?;
        let direction = SwitchState::from_db(direction)?;
        Ok(Self {
            id,
            direction,
            time,
            switch_id,
            remote_id,
        })
    }
}

#[derive(Serialize)]
pub enum SwitchState {
    Off,
    On,
}

impl SwitchState {
    pub fn for_db(&self) -> i32 {
        match self {
            SwitchState::Off => 0,
            SwitchState::On => 1
        }
    }

    pub fn from_db(i: i32) -> Result<Self, Error> {
        match i {
            0 => Ok(SwitchState::Off),
            1 => Ok(SwitchState::On),
            _ => Err(Error::Enum("SwitchState".to_owned(), i))
        }
    }
}