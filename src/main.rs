use netcdf;
use json;
use chrono::Utc;
use chrono::TimeZone;
use chrono::Duration;
use chrono::Datelike;
use chrono::Timelike;
use std::env;
use std::f64::NAN;

fn tidylon(longitude: f64) -> f64{
    // map longitude on [0,360] to [-180,180], required for mongo indexing
    if longitude <= 180.0{
        return longitude;
    }
    else{
        return longitude-360.0;
    }
}

fn nowstring() -> String{
    // returns a String representing the current ISO8601 datetime

    let now = Utc::now();
    return format!("{}-{:02}-{:02}T{:02}:{:02}:{:02}Z", now.year(), now.month(), now.day(), now.hour(), now.minute(), now.second());
}

// impementing a foreign trait on a forein struct //////////
// per the advice in https://stackoverflow.com/questions/76277096/deconstructing-enums-in-rust/76277117#76277117

struct Wrapper{
    s: String
}

impl std::convert::TryFrom<netcdf::attribute::AttrValue> for Wrapper {
    type Error = &'static str;

    fn try_from(value: netcdf::attribute::AttrValue) -> Result<Self, Self::Error> {

        if let netcdf::attribute::AttrValue::Str(v) = value {
            Ok(Wrapper{s: String::from(v)} )
        } else {
            Err("nope")
        }
    }
}
////////////////////

fn find_basin(basins: &netcdf::Variable, longitude: f64, latitude: f64) -> i64 {    
    let lonplus = (longitude-0.5).ceil()+0.5;
    let lonminus = (longitude-0.5).floor()+0.5;
    let latplus = (latitude-0.5).ceil()+0.5;
    let latminus = (latitude-0.5).floor()+0.5;

    let lonplus_idx = (lonplus - -179.5) as usize;
    let lonminus_idx = (lonminus - -179.5) as usize;
    let latplus_idx = (latplus - -77.5) as usize;
    let latminus_idx = (latminus - -77.5) as usize;

    //println!("{} {} {} {} {}", longitude, lonminus, lonminus_idx, lonplus, lonplus_idx);

    let corners_idx = [
        // bottom left corner, clockwise
        [latminus_idx, lonminus_idx],
        [latplus_idx, lonminus_idx],
        [latplus_idx, lonplus_idx],
        [latminus_idx, lonplus_idx]
    ];

    let distances = [
        (f64::powi(longitude-lonminus, 2) + f64::powi(latitude-latminus, 2)).sqrt(),
        (f64::powi(longitude-lonminus, 2) + f64::powi(latitude-latplus, 2)).sqrt(),
        (f64::powi(longitude-lonplus, 2) + f64::powi(latitude-latplus, 2)).sqrt(),
        (f64::powi(longitude-lonplus, 2) + f64::powi(latitude-latminus, 2)).sqrt()
    ];

    let mut closecorner_idx = corners_idx[0];
    let mut closedist = distances[0];
    for i in 1..4 {
        if distances[i] < closedist{
            closecorner_idx = corners_idx[i];
            closedist = distances[i];
        }
    }

    match basins.value::<i64,_>(closecorner_idx){
        Ok(idx) => idx,
        Err(e) => panic!("basin problems: {:?} {:#?}", e, closecorner_idx)
    }   
}

fn eq_with_nan_eq(a: f64, b: f64) -> bool {
    (a.is_nan() && b.is_nan()) || (a == b)
}

fn main() -> Result<(), netcdf::error::Error> {
    // command line argument extraction
    let args: Vec<String> = env::args().collect();

    // file opening
    let file = netcdf::open(&args[1])?; // data/CCMP_Wind_Analysis_19930103_V03.0_L4.0.nc
    
    // variable extraction
    let latitude = &file.variable("latitude").expect("Could not find variable 'latitude'");
    let longitude = &file.variable("longitude").expect("Could not find variable 'longitude'");
    let time = &file.variable("time").expect("Could not find variable 'time'");
    let vwnd = &file.variable("vwnd").expect("Could not find variable 'vwnd'");
    let uwnd = &file.variable("uwnd").expect("Could not find variable 'uwnd'");
    let ws   = &file.variable("ws").expect("Could not find variable 'ws'");
    let nobs = &file.variable("nobs").expect("Could not find variable 'nobs'");
    
    // basin lookup
    let basinfile = netcdf::open("data/basinmask_01.nc")?;
    let basins = &basinfile.variable("BASIN_TAG").expect("Could not find variable 'BASIN_TAG'");

    // metadata construction

    // all times recorded as hours since Jan 1 1987
    let t0 = Utc.with_ymd_and_hms(1987, 1, 1, 0, 0, 0).unwrap();

    let metadata = json::object!{
        "_id": (t0 + Duration::hours(time.value::<i64, _>(0)?)).to_rfc3339()[..10].replace("-",""),
        "data_type": "ccmp-wind",
        "data_info": [
            ["uwnd", "vwnd", "ws", "nobs"],
            ["units", "long_name"],
            [
                [Wrapper::try_from(uwnd.attribute("units").unwrap().value().unwrap()).unwrap().s,Wrapper::try_from(uwnd.attribute("long_name").unwrap().value().unwrap()).unwrap().s],
                [Wrapper::try_from(vwnd.attribute("units").unwrap().value().unwrap()).unwrap().s,Wrapper::try_from(vwnd.attribute("long_name").unwrap().value().unwrap()).unwrap().s],
                [Wrapper::try_from(ws.attribute("units").unwrap().value().unwrap()).unwrap().s,Wrapper::try_from(ws.attribute("long_name").unwrap().value().unwrap()).unwrap().s],
                ["null",Wrapper::try_from(nobs.attribute("long_name").unwrap().value().unwrap()).unwrap().s]
            ]
        ],
        "date_updated_argovis": nowstring(),
        "source": [
            json::object!{
                "source": [String::from("CCMP Wind Vector Analysis Product")],
                "url": format!("https://data.remss.com/ccmp/v03.0/daily/y{}/m{}/{}", &args[1][24..28], &args[1][28..30], &args[1][5..])
            }
        ]
    };

    println!("{}", metadata);

    for latidx in 0..latitude.len() {
        let lat = latitude.value::<f64, _>(latidx)?;
        for lonidx in 0..longitude.len() {
            let lon = tidylon(longitude.value::<f64, _>(lonidx)?);
            let mut timeseries = Vec::new();
            let mut vwndseries = Vec::new();
            let mut uwndseries = Vec::new();
            let mut wsseries = Vec::new();
            let mut nobsseries = Vec::new();
            for timeidx in 0..time.len() {
                if eq_with_nan_eq(vwnd.value::<f64, _>([latidx, lonidx, timeidx])?, NAN) && eq_with_nan_eq(uwnd.value::<f64, _>([latidx, lonidx, timeidx])?, NAN) && eq_with_nan_eq(ws.value::<f64, _>([latidx, lonidx, timeidx])?, NAN) && eq_with_nan_eq(nobs.value::<f64, _>([latidx, lonidx, timeidx])?, NAN){
                    // if no measurement contains anything for this timestep, no need to write it to the db
                    continue;
                }
                timeseries.push((t0 + Duration::hours(time.value::<i64, _>(timeidx)?)).to_rfc3339().replace("+00:00", "Z") );
                vwndseries.push(vwnd.value::<f64, _>([latidx, lonidx, timeidx])?);
                uwndseries.push(uwnd.value::<f64, _>([latidx, lonidx, timeidx])?);
                wsseries.push(ws.value::<f64, _>([latidx, lonidx, timeidx])?);
                nobsseries.push(nobs.value::<f64, _>([latidx, lonidx, timeidx])?);
            }

            if timeseries.len() > 0 {
                let data = json::object!{
                    "_id": [lon.to_string(), lat.to_string()].join("_"),
                    "metadata": [(t0 + Duration::hours(time.value::<i64, _>(0)?)).to_rfc3339()[..10].replace("-","")],
                    "basin": find_basin(&basins, lon, lat),
                    "geolocation": {
                        "type": "Point",
                        "coordinates": [lon, lat]
                    },
                    "data": [vwndseries, uwndseries, wsseries, nobsseries],
                    "timeseries": timeseries
                };
                println!("{}", data);
            }
        }
    }

    return Ok(())

}
